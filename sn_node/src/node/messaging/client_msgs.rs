// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{core::NodeContext, flow_ctrl::cmds::Cmd, Error, MyNode, Result};

use bytes::BufMut;

use qp2p::SendStream;
use sn_dbc::{
    get_public_commitments_from_transaction, Commitment, KeyImage, RingCtTransaction, SpentProof,
    SpentProofShare,
};
use sn_interface::{
    messaging::{
        data::{
            ClientDataResponse, ClientMsg, DataCmd, DataQueryVariant, EditRegister,
            SignedRegisterEdit, SpentbookCmd,
        },
        system::{NodeDataResponse, OperationId},
        AuthorityProof, ClientAuth, MsgId,
    },
    network_knowledge::{section_keys::build_spent_proof_share, SectionTreeUpdate},
    types::{
        log_markers::LogMarker,
        register::{Permissions, Policy, Register, User},
        Keypair, Peer, RegisterCmd, ReplicatedData, SPENTBOOK_TYPE_TAG,
    },
};

use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

impl MyNode {
    /// Forms a `QueryError` msg to send back to the client on a stream
    pub(crate) fn send_query_error_response_over_stream(
        context: NodeContext,
        error: Error,
        query: &DataQueryVariant,
        source_client: Peer,
        correlation_id: MsgId,
        send_stream: SendStream,
    ) -> Cmd {
        let msg = ClientDataResponse::QueryResponse {
            response: query.to_error_response(error.into()),
            correlation_id,
        };

        Cmd::SendClientResponse {
            msg,
            correlation_id,
            send_stream,
            context,
            source_client,
        }
    }

    /// Forms a `CmdError` msg to send back to the client over the response stream
    pub(crate) fn send_cmd_error_response_over_stream(
        context: NodeContext,
        cmd: DataCmd,
        error: Error,
        correlation_id: MsgId,
        send_stream: SendStream,
        source_client: Peer,
    ) -> Cmd {
        let msg = ClientDataResponse::CmdResponse {
            response: cmd.to_error_response(error.into()),
            correlation_id,
        };

        debug!("{correlation_id:?} sending cmd response error back to client");
        Cmd::SendClientResponse {
            msg,
            correlation_id,
            send_stream,
            context,
            source_client,
        }
    }

    /// Handle data query
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn handle_data_query_where_stored(
        context: NodeContext,
        operation_id: OperationId,
        query: &DataQueryVariant,
        auth: ClientAuth,
        requesting_peer: Peer,
        msg_id: MsgId,
        send_stream: Option<SendStream>,
    ) -> Vec<Cmd> {
        let response = context
            .data_storage
            .query(query, User::Key(auth.public_key))
            .await;

        if let Some(send_stream) = send_stream {
            trace!("{msg_id:?} data query response at node is: {response:?}");
            let msg = NodeDataResponse::QueryResponse {
                response,
                operation_id,
            };

            vec![Cmd::SendNodeDataResponse {
                msg,
                correlation_id: msg_id,
                send_stream,
                context,
                requesting_peer,
            }]
        } else {
            error!("Send stream missing from {requesting_peer:?}, data request response was not sent out.");
            vec![]
        }
    }

    /// Handle incoming client msgs.
    /// If this is a store request, and we are an Elder and one of
    /// the `data_copy_count()` nodes, then we will send a wiremsg
    /// to ourselves, among the msgs sent to the other holders.
    pub(crate) fn handle_valid_client_msg(
        mut context: NodeContext,
        msg_id: MsgId,
        msg: ClientMsg,
        auth: AuthorityProof<ClientAuth>,
        origin: Peer,
        send_stream: SendStream,
    ) -> Result<Vec<Cmd>> {
        debug!("Handling client {msg_id:?}");

        trace!("{:?}: {msg:?} ", LogMarker::ClientMsgToBeHandled);

        let cmd = match msg {
            ClientMsg::Cmd(cmd) => cmd,
            ClientMsg::Query(query) => {
                return MyNode::read_data_and_respond_to_client(
                    context,
                    query,
                    msg_id,
                    auth,
                    origin,
                    send_stream,
                )
            }
        };

        // extract the data from the request
        let data_result = match cmd.clone() {
            DataCmd::StoreChunk(chunk) => Ok(ReplicatedData::Chunk(chunk)),
            DataCmd::Register(cmd) => Ok(ReplicatedData::RegisterWrite(cmd)),
            DataCmd::Spentbook(cmd) => {
                let SpentbookCmd::Spend {
                    network_knowledge,
                    key_image,
                    tx,
                    spent_proofs,
                    spent_transactions,
                } = cmd.clone();
                if let Some((proof_chain, signed_sap)) = network_knowledge {
                    debug!(
                        "Received updated network knowledge with the request. Will return new command \
                        to update the node network knowledge before processing the spend."
                    );
                    let name = context.name;
                    let there_was_an_update = context.network_knowledge.update_knowledge_if_valid(
                        SectionTreeUpdate::new(signed_sap.clone(), proof_chain.clone()),
                        None,
                        &name,
                    )?;

                    if there_was_an_update {
                        // To avoid a loop, recompose the message without the updated proof_chain.
                        let updated_client_msg =
                            ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                                key_image,
                                tx,
                                spent_proofs,
                                spent_transactions,
                                network_knowledge: None,
                            }));
                        let update_command = Cmd::UpdateNetworkAndHandleValidClientMsg {
                            proof_chain,
                            signed_sap,
                            msg_id,
                            msg: updated_client_msg,
                            origin,
                            send_stream,
                            auth,
                            context,
                        };
                        return Ok(vec![update_command]);
                    }
                }
                MyNode::extract_contents_as_replicated_data(&context, cmd)
            }
        };

        let data = match data_result {
            Ok(data) => data,
            Err(error) => {
                debug!("Will send error response back to client");
                let cmd = MyNode::send_cmd_error_response_over_stream(
                    context,
                    cmd,
                    error,
                    msg_id,
                    send_stream,
                    origin,
                );
                return Ok(vec![cmd]);
            }
        };

        trace!("{:?}: {:?}", LogMarker::DataStoreReceivedAtElder, data);

        // the store msg sent to nodes
        // cmds here may be fault tracking.
        // CmdAcks are sent over the send stream herein
        MyNode::store_data_at_nodes_and_ack_to_client(
            context,
            cmd,
            data,
            msg_id,
            send_stream,
            origin,
        )
    }

    // helper to extract the contents of the cmd as ReplicatedData
    fn extract_contents_as_replicated_data(
        context: &NodeContext,
        cmd: SpentbookCmd,
    ) -> Result<ReplicatedData> {
        let SpentbookCmd::Spend {
            key_image,
            tx,
            spent_proofs,
            spent_transactions,
            ..
        } = cmd;

        info!("Processing spend request for key image: {:?}", key_image);

        let spent_proof_share = MyNode::gen_spent_proof_share(
            context,
            &key_image,
            &tx,
            &spent_proofs,
            &spent_transactions,
        )?;
        let reg_cmd = MyNode::gen_register_cmd(context, &key_image, &spent_proof_share)?;
        Ok(ReplicatedData::SpentbookWrite(reg_cmd))
    }

    /// Generate a spent proof share from the information provided by the client.
    fn gen_spent_proof_share(
        context: &NodeContext,
        key_image: &KeyImage,
        tx: &RingCtTransaction,
        spent_proofs: &BTreeSet<SpentProof>,
        spent_transactions: &BTreeSet<RingCtTransaction>,
    ) -> Result<SpentProofShare> {
        // Verify spent proof signatures are valid.
        let mut spent_proofs_keys = BTreeSet::new();
        for proof in spent_proofs.iter() {
            if !proof
                .spentbook_pub_key
                .verify(&proof.spentbook_sig, proof.content.hash().as_ref())
            {
                let msg = format!(
                    "Spent proof signature {:?} is invalid",
                    proof.spentbook_pub_key
                );
                debug!("Dropping spend request: {msg}");
                return Err(Error::SpentbookError(msg));
            }
            let _ = spent_proofs_keys.insert(proof.spentbook_pub_key);
        }

        // Verify each spent proof is signed by a known section key (or the genesis key).
        for pk in &spent_proofs_keys {
            if !context.network_knowledge.verify_section_key_is_known(pk) {
                warn!(
                    "Dropping spend request: spent proof is signed by unknown section with public \
                    key {:?}",
                    pk
                );
                return Err(Error::SpentProofUnknownSectionKey(*pk));
            }
        }

        let public_commitments_info =
            get_public_commitments_from_transaction(tx, spent_proofs, spent_transactions)?;

        // Do not sign invalid TX.
        let tx_public_commitments: Vec<Vec<Commitment>> = public_commitments_info
            .clone()
            .into_iter()
            .map(|(_, v)| v)
            .collect();

        if let Err(err) = tx.verify(&tx_public_commitments) {
            debug!("Dropping spend request: {:?}", err.to_string());
            return Err(Error::SpentbookError(err.to_string()));
        }

        // TODO:
        // Check the key_image wasn't already spent with a different TX (i.e. double spent)

        // Grab the commitments specific to the spent key image.
        let public_commitments: Vec<Commitment> = public_commitments_info
            .into_iter()
            .flat_map(|(k, v)| if &k == key_image { v } else { vec![] })
            .collect();
        if public_commitments.is_empty() {
            let msg = format!("There are no commitments for the given key image {key_image:?}",);
            debug!("Dropping spend request: {msg}");
            return Err(Error::SpentbookError(msg));
        }
        let spent_proof_share = build_spent_proof_share(
            key_image,
            tx,
            &context.network_knowledge.section_auth(),
            &context.section_keys_provider,
            public_commitments,
        )?;
        Ok(spent_proof_share)
    }

    /// Generate the RegisterCmd to write the SpentProofShare as an entry in the Spentbook
    /// (Register).
    fn gen_register_cmd(
        context: &NodeContext,
        key_image: &KeyImage,
        spent_proof_share: &SpentProofShare,
    ) -> Result<RegisterCmd> {
        let mut permissions = BTreeMap::new();
        let _ = permissions.insert(User::Anyone, Permissions::new(true));

        // use our own keypair for generating the register command
        let own_keypair = Keypair::Ed25519(context.keypair.clone());
        let owner = User::Key(own_keypair.public_key());
        let policy = Policy { owner, permissions };

        let mut register = Register::new(
            owner,
            XorName::from_content(&key_image.to_bytes()),
            SPENTBOOK_TYPE_TAG,
            policy,
        );
        let mut entry = vec![].writer();
        rmp_serde::encode::write(&mut entry, spent_proof_share).map_err(|err| {
            Error::SpentbookError(format!(
                "Failed to serialise SpentProofShare to insert it into the spentbook (Register): {err:?}",
            ))
        })?;

        let (_, op) = register.write(entry.into_inner(), BTreeSet::default())?;
        let op = EditRegister {
            address: *register.address(),
            edit: op,
        };

        let signature = own_keypair.sign(&bincode::serialize(&op)?);
        let signed_edit = SignedRegisterEdit {
            op,
            auth: ClientAuth {
                public_key: own_keypair.public_key(),
                signature,
            },
        };

        debug!("Successfully generated spent proof share for spend request");
        Ok(RegisterCmd::Edit(signed_edit))
    }
}
