// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{flow_ctrl::cmds::Cmd, Error, MyNode, NodeContext, Result};

use sn_dbc::{
    get_blinded_amounts_from_transaction, BlindedAmount, DbcTransaction, PublicKey, SpentProof,
    SpentProofShare, Token,
};
use sn_interface::{
    dbcs::DbcReason,
    messaging::{
        data::{ClientMsg, DataCmd, DataQuery, DataResponse, SpendQuery, SpentbookCmd},
        system::NodeQueryResponse,
        AuthorityProof, ClientAuth, MsgId,
    },
    network_knowledge::{
        section_keys::build_spent_proof_share, NetworkKnowledge, SectionTreeUpdate,
    },
    types::{
        fees::{FeeCiphers, RequiredFee},
        log_markers::LogMarker,
        register::User,
        ClientId, ReplicatedData,
    },
};

use qp2p::SendStream;
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

impl MyNode {
    /// Forms a `CmdError` msg to send back to the client over the response stream
    pub(crate) fn send_cmd_error_response_over_stream(
        msg: DataResponse,
        correlation_id: MsgId,
        send_stream: SendStream,
        client_id: ClientId,
    ) -> Cmd {
        debug!("{correlation_id:?} sending cmd response error back to client");
        Cmd::send_data_response(msg, correlation_id, client_id, send_stream)
    }

    /// Handle data query
    pub(crate) async fn handle_data_query_where_stored(
        msg_id: MsgId,
        query: &DataQuery,
        auth: ClientAuth,
        client_id: ClientId,
        send_stream: SendStream,
        context: NodeContext,
    ) -> Vec<Cmd> {
        let response = if let DataQuery::Spentbook(SpendQuery::GetFees { dbc_id, priority }) = query
        {
            // We receive this directly from client, as an Elder, since `is_spend` is set to true (that is a very
            // messy/confusing pattern, to be fixed).

            // The client is asking for the fee to spend a specific dbc, and including the id of that dbc.
            // The required fee content is encrypted to that dbc id, and so only the holder of the dbc secret
            // key can unlock the contents.
            let amount = context.current_fee(priority).as_nano() as f64 * 1.1;
            let required_fee = RequiredFee::new(
                Token::from_nano(amount as u64),
                dbc_id,
                &context.reward_secret_key,
            );

            NodeQueryResponse::GetFees(Ok(required_fee))
        } else {
            context
                .data_storage
                .query(query, User::Key(auth.public_key))
                .await
        };

        trace!("{msg_id:?} data query response at node is: {response:?}");

        let msg = DataResponse::QueryResponse {
            response,
            correlation_id: msg_id,
        };

        vec![Cmd::send_data_response(msg, msg_id, client_id, send_stream)]
    }

    /// Handle incoming client msgs.
    /// If this is a store request, and we are an Elder and one of
    /// the `data_copy_count()` nodes, then we will send a wiremsg
    /// to ourselves, among the msgs sent to the other holders.
    pub(crate) async fn handle_client_msg_for_us(
        context: NodeContext,
        msg_id: MsgId,
        msg: ClientMsg,
        auth: AuthorityProof<ClientAuth>,
        client_id: ClientId,
        send_stream: SendStream,
    ) -> Result<Vec<Cmd>> {
        trace!("{:?}: {msg_id:?} {msg:?}", LogMarker::ClientMsgToBeHandled);

        match msg {
            ClientMsg::Cmd(cmd) => {
                MyNode::handle_data_cmd(cmd, msg_id, client_id, auth, send_stream, context).await
            }
            ClientMsg::Query(query) => Ok(MyNode::handle_data_query_where_stored(
                msg_id,
                &query,
                auth.into_inner(),
                client_id,
                send_stream,
                context,
            )
            .await),
        }
    }

    /// Handle the DataCmd variant
    async fn handle_data_cmd(
        data_cmd: DataCmd,
        msg_id: MsgId,
        client_id: ClientId,
        auth: AuthorityProof<ClientAuth>,
        send_stream: SendStream,
        mut context: NodeContext,
    ) -> Result<Vec<Cmd>> {
        // extract the data from the request
        let data_result: Result<ReplicatedData> = match data_cmd.clone() {
            DataCmd::StoreChunk(chunk) => Ok(ReplicatedData::Chunk(chunk)),
            DataCmd::Register(cmd) => Ok(ReplicatedData::RegisterWrite(cmd)),
            DataCmd::Spentbook(cmd) => {
                let SpentbookCmd::Spend {
                    network_knowledge,
                    public_key,
                    tx,
                    reason,
                    spent_proofs,
                    spent_transactions,
                    #[cfg(not(feature = "data-network"))]
                    fee_ciphers,
                } = cmd.clone();
                if let Some((proof_chain, signed_sap)) = network_knowledge {
                    info!(
                        "Received updated network knowledge with the request. Will return new command \
                        to update the node network knowledge before processing the spend."
                    );
                    let name = context.name;
                    let there_was_an_update =
                        context.network_knowledge.update_sap_knowledge_if_valid(
                            SectionTreeUpdate::new(signed_sap.clone(), proof_chain.clone()),
                            &name,
                        )?;

                    if there_was_an_update {
                        // To avoid a loop, recompose the message without the updated proof_chain.
                        let updated_client_msg =
                            ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                                public_key,
                                tx,
                                fee_ciphers,
                                reason,
                                spent_proofs,
                                spent_transactions,
                                network_knowledge: None,
                            }));
                        let update_command = Cmd::UpdateNetworkAndHandleValidClientMsg {
                            proof_chain,
                            signed_sap,
                            msg_id,
                            msg: updated_client_msg,
                            client_id,
                            send_stream,
                            auth,
                        };
                        return Ok(vec![update_command]);
                    }
                }

                // First we validate it here at the Elder.
                let (fee_paid, spent_share) = match MyNode::validate_spentbook_cmd(cmd, &context) {
                    Ok((fee_paid, share)) => (fee_paid, share),
                    Err(e) => {
                        return MyNode::send_error(msg_id, data_cmd, e, send_stream, client_id)
                    }
                };

                // Then, if payment-network, we enqueue the spend.
                #[cfg(not(feature = "data-network"))]
                return Ok(vec![Cmd::EnqueueSpend {
                    fee_paid,
                    spent_share,
                    send_stream,
                    client_id,
                    correlation_id: msg_id,
                }]);

                // Else if data-network, then we forward it to data holders.
                #[cfg(feature = "data-network")]
                return MyNode::forward_spent_share(
                    msg_id,
                    spent_share,
                    public_key,
                    client_id,
                    send_stream,
                    context,
                );
            }
        };

        match data_result {
            Ok(data) => {
                MyNode::store_data_and_respond(&context, data, send_stream, client_id, msg_id).await
            }
            Err(error) => MyNode::send_error(msg_id, data_cmd, error, send_stream, client_id),
        }
    }

    fn send_error(
        msg_id: MsgId,
        cmd: DataCmd,
        error: Error,
        send_stream: SendStream,
        client_id: ClientId,
    ) -> Result<Vec<Cmd>> {
        let data_response = DataResponse::CmdResponse {
            response: cmd.to_error_response(error.into()),
            correlation_id: msg_id,
        };
        let cmd = MyNode::send_cmd_error_response_over_stream(
            data_response,
            msg_id,
            send_stream,
            client_id,
        );
        Ok(vec![cmd])
    }

    fn validate_spentbook_cmd(
        cmd: SpentbookCmd,
        context: &NodeContext,
    ) -> Result<(Token, SpentProofShare)> {
        let SpentbookCmd::Spend {
            public_key,
            tx,
            reason,
            spent_proofs,
            spent_transactions,
            #[cfg(not(feature = "data-network"))]
            fee_ciphers,
            ..
        } = cmd;

        info!("Processing spend request for dbc key: {public_key:?}");

        // verify that fee is paid to us
        #[cfg(not(feature = "data-network"))]
        let fee_paid = MyNode::validate_fee(
            context,
            context.reward_secret_key.as_ref(),
            &tx,
            context.name,
            fee_ciphers,
        )?;
        #[cfg(feature = "data-network")]
        let fee_paid = Token::zero();

        let spent_proof_share = MyNode::gen_spent_proof_share(
            &public_key,
            &tx,
            reason,
            &spent_proofs,
            &spent_transactions,
            context,
        )?;

        Ok((fee_paid, spent_proof_share))
    }

    /// Generate a spent proof share from the information provided by the client.
    fn gen_spent_proof_share(
        public_key: &PublicKey,
        tx: &DbcTransaction,
        reason: DbcReason,
        spent_proofs: &BTreeSet<SpentProof>,
        spent_transactions: &BTreeSet<DbcTransaction>,
        context: &NodeContext,
    ) -> Result<SpentProofShare> {
        // verify the spent proofs
        MyNode::verify_spent_proofs(spent_proofs, &context.network_knowledge)?;

        let blinded_amounts_info =
            get_blinded_amounts_from_transaction(tx, spent_proofs, spent_transactions)?;

        // Do not sign invalid TX.
        let tx_blinded_amounts: Vec<BlindedAmount> = blinded_amounts_info
            .clone()
            .into_iter()
            .map(|(_, v)| v)
            .collect();

        if let Err(err) = tx.verify(&tx_blinded_amounts) {
            warn!("Dropping spend request: {:?}", err.to_string());
            return Err(Error::SpentbookError(err.to_string()));
        }

        // TODO:
        // Check the public_key wasn't already spent with a different TX (i.e. double spent)

        // Grab the amount specific to the spent public key.
        let blinded_amount: BlindedAmount = blinded_amounts_info
            .into_iter()
            .find(|(k, _c)| k == public_key)
            .map(|(_k, c)| c)
            .ok_or_else(|| {
                let msg = format!("There are no amounts for the given public key {public_key:?}",);
                warn!("Dropping spend request: {msg}");
                Error::SpentbookError(msg)
            })?;

        let spent_proof_share = build_spent_proof_share(
            public_key,
            tx,
            reason,
            &context.network_knowledge.section_auth(),
            &context.section_keys_provider,
            blinded_amount,
        )?;

        Ok(spent_proof_share)
    }

    #[cfg(not(feature = "data-network"))]
    fn validate_fee(
        context: &NodeContext,
        reward_secret_key: &bls::SecretKey,
        tx: &DbcTransaction,
        our_name: XorName,
        fee_ciphers: BTreeMap<XorName, FeeCiphers>,
    ) -> Result<Token> {
        // find the ciphers for us
        let fee_ciphers = fee_ciphers.get(&our_name).ok_or(Error::MissingFee)?;
        // decrypt the ciphers
        let (derived_key, revealed_amount) = fee_ciphers.decrypt(reward_secret_key)?;

        // find the output for the derived key
        let output_proof = match tx
            .outputs
            .iter()
            .find(|proof| proof.public_key() == &derived_key)
        {
            Some(proof) => proof,
            None => return Err(Error::MissingFee),
        };

        // blind the amount
        let blinded_amount = revealed_amount.blinded_amount(&sn_dbc::PedersenGens::default());
        // Since the output proof contains blinded amounts, we can only verify
        // that the amount is what we expect by..
        // 1. ..comparing equality to the blinded amount we build from the decrypted revealed amount (i.e. amount + blinding factor)..
        if blinded_amount != output_proof.blinded_amount() {
            return Err(Error::InvalidFeeBlindedAmount);
        }
        // .. and 2. checking that the revealed amount we have, (that we now know is what the output blinded amount contains, since the above check 1. passed),
        // also is what we expect the amount to be.
        let paid = Token::from_nano(revealed_amount.value());
        let (valid, required) = context.validate_fee(paid);

        if !valid {
            return Err(Error::FeeTooLow { paid, required });
        }

        Ok(paid)
    }

    // Verify spent proof signatures are valid, and each spent proof is signed by a known section key.
    fn verify_spent_proofs(
        spent_proofs: &BTreeSet<SpentProof>,
        network_knowledge: &NetworkKnowledge,
    ) -> Result<()> {
        let mut spent_proofs_keys = BTreeSet::new();

        // Verify each spent proof signature is valid.
        for proof in spent_proofs.iter() {
            if !proof
                .spentbook_pub_key
                .verify(&proof.spentbook_sig, proof.content.hash().as_ref())
            {
                let msg = format!(
                    "Spent proof signature {:?} is invalid",
                    proof.spentbook_pub_key
                );
                warn!("Dropping spend request: {msg}");
                return Err(Error::SpentbookError(msg));
            }
            let _ = spent_proofs_keys.insert(proof.spentbook_pub_key);
        }

        // Verify each spent proof is signed by a known section key.
        for pk in &spent_proofs_keys {
            if !network_knowledge.verify_section_key_is_known(pk) {
                warn!(
                    "Dropping spend request: spent proof is signed by unknown section with public \
                    key {:?}",
                    pk
                );
                return Err(Error::SpentProofUnknownSectionKey(*pk));
            }
        }

        Ok(())
    }
}
