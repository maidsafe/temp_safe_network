// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::messaging::{OutgoingMsg, Peers};

use crate::node::{flow_ctrl::cmds::Cmd, Error, MyNode, Result};
use bytes::Bytes;

use qp2p::SendStream;
use sn_dbc::{
    get_public_commitments_from_transaction, Commitment, KeyImage, RingCtTransaction, SpentProof,
    SpentProofShare,
};
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::{
    data_copy_count,
    messaging::{
        data::{
            ClientMsg, CmdResponse, DataCmd, DataQueryVariant, EditRegister, SignedRegisterEdit,
            SpentbookCmd,
        },
        system::{NodeMsg, NodeQueryResponse, OperationId},
        AuthorityProof, ClientAuth, MsgId,
    },
    network_knowledge::section_keys::build_spent_proof_share,
    types::{
        log_markers::LogMarker,
        register::{Permissions, Policy, Register, User},
        Keypair, Peer, RegisterCmd, ReplicatedData, SPENTBOOK_TYPE_TAG,
    },
};
use tokio::sync::Mutex;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use xor_name::XorName;

impl MyNode {
    /// Forms a `ACK` msg to send back to the client
    pub(crate) fn send_cmd_ack(
        &self,
        target: Peer,
        correlation_id: MsgId,
        send_stream: Option<Arc<Mutex<SendStream>>>,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Cmd {
        // NB: temp use of `CmdResponse::StoreChunk(Ok(()))` before that is handled at Adult.
        let the_ack_msg = ClientMsg::CmdResponse {
            response: CmdResponse::StoreChunk(Ok(())),
            correlation_id,
        };
        self.send_client_msg(
            the_ack_msg,
            Peers::Single(target),
            send_stream,
            #[cfg(feature = "traceroute")]
            traceroute,
        )
    }

    /// Forms a `QueryError` msg to send back to the client
    pub(crate) fn query_error_response(
        &self,
        error: Error,
        query: &DataQueryVariant,
        target: Peer,
        correlation_id: MsgId,
        send_stream: Option<Arc<Mutex<SendStream>>>,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Cmd {
        let the_error_msg = ClientMsg::QueryResponse {
            response: query.to_error_response(error.into()),
            correlation_id,
        };

        self.send_client_msg(
            the_error_msg,
            Peers::Single(target),
            send_stream,
            #[cfg(feature = "traceroute")]
            traceroute,
        )
    }

    /// Forms a `CmdError` msg to send back to the client
    pub(crate) fn cmd_error_response(
        &self,
        cmd: DataCmd,
        error: Error,
        target: Peer,
        correlation_id: MsgId,
        send_stream: Option<Arc<Mutex<SendStream>>>,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Cmd {
        let the_error_msg = ClientMsg::CmdResponse {
            response: cmd.to_error_response(error.into()),
            correlation_id,
        };

        self.send_client_msg(
            the_error_msg,
            Peers::Single(target),
            send_stream,
            #[cfg(feature = "traceroute")]
            traceroute,
        )
    }

    /// Forms a cmd to send a cmd response error/ack to the client
    fn send_client_msg(
        &self,
        msg: ClientMsg,
        recipients: Peers,
        send_stream: Option<Arc<Mutex<SendStream>>>,
        #[cfg(feature = "traceroute")] mut traceroute: Traceroute,
    ) -> Cmd {
        #[cfg(feature = "traceroute")]
        traceroute.0.push(self.identity());

        let msg_id = MsgId::new();

        debug!("SendMSg formed for {:?}", msg_id);
        Cmd::SendMsg {
            msg: OutgoingMsg::Client(msg),
            msg_id,
            recipients,
            send_stream,
            #[cfg(feature = "traceroute")]
            traceroute,
        }
    }

    /// Handle data query
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn handle_data_query_at_adult(
        &self,
        operation_id: OperationId,
        query: &DataQueryVariant,
        auth: ClientAuth,
        requesting_elder: Peer,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Cmd {
        let response = self
            .data_storage
            .query(query, User::Key(auth.public_key))
            .await;

        trace!("data query response at adult is: {:?}", response);
        let msg = NodeMsg::NodeQueryResponse {
            response,
            operation_id,
        };

        self.trace_system_msg(
            msg,
            Peers::Single(requesting_elder),
            #[cfg(feature = "traceroute")]
            traceroute,
        )
    }

    /// Handle data read
    /// Records response in liveness tracking
    /// Forms a response to send to the requester
    pub(crate) async fn handle_data_query_response_at_elder(
        &mut self,
        op_id: OperationId,
        response: NodeQueryResponse,
        sender_name: XorName,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Vec<Cmd> {
        debug!(
            "Handling data read @ elders, received from {:?}, op id: {:?}",
            sender_name, op_id
        );

        let query_peers = self.pending_data_queries.remove(&(op_id, sender_name));

        // First check for waiting peers. If no one is waiting, we drop the response
        let waiting_peers = if let Some(peers) = query_peers {
            if peers.is_empty() {
                warn!("No waiting peers to send {op_id:?} to....");
                // nothing to do
                return vec![];
            }
            peers
        } else {
            warn!(
                "Dropping chunk query response from Adult {}. We might have already forwarded this chunk to the requesting client or the client connection cache has expired: {}",
                sender_name, op_id
            );
            return vec![];
        };

        let pending_removed = self
            .dysfunction_tracking
            .request_operation_fulfilled(&sender_name, op_id);

        if !pending_removed {
            trace!("Ignoring un-expected response");
            return vec![];
        }

        let mut cmds = vec![];
        for ((correlation_id, peer), send_stream) in waiting_peers.into_iter() {
            let msg = ClientMsg::QueryResponse {
                response: response.clone(),
                correlation_id,
            };

            cmds.push(self.send_client_msg(
                msg,
                Peers::Single(peer),
                send_stream,
                #[cfg(feature = "traceroute")]
                traceroute.clone(),
            ));
        }

        // Clear expired queries from the cache.
        self.pending_data_queries.remove_expired();

        cmds
    }

    /// Handle incoming client msgs. Though NOT queries, as this requires
    /// mutable access to the Node
    pub(crate) async fn handle_valid_client_msg(
        &self,
        msg_id: MsgId,
        msg: ClientMsg,
        auth: AuthorityProof<ClientAuth>,
        origin: Peer,
        send_stream: Option<Arc<Mutex<SendStream>>>,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Vec<Cmd> {
        if !self.is_elder() {
            return vec![];
        }

        trace!("{:?} {:?}", LogMarker::ClientMsgToBeHandled, msg);

        let cmd = match msg {
            ClientMsg::Cmd(cmd) => cmd,
            ClientMsg::Query(query) => {
                return self
                    .read_data_from_adults(
                        query,
                        msg_id,
                        auth,
                        origin,
                        send_stream,
                        #[cfg(feature = "traceroute")]
                        traceroute,
                    )
                    .await;
            }
            _ => {
                warn!(
                    "!!!! Unexpected ClientMsg received, and it was not handled: {:?}",
                    msg
                );
                return vec![];
            }
        };

        // extract the data from the request
        let data_result = match cmd.clone() {
            // These reads/writes are for adult nodes...
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
                        auth,
                        #[cfg(feature = "traceroute")]
                        traceroute,
                    };
                    return vec![update_command];
                }
                self.extract_contents_as_replicated_data(cmd)
            }
        };

        let data = match data_result {
            Ok(data) => data,
            Err(error) => {
                debug!("Will send error response back to client");
                let cmd = self.cmd_error_response(
                    cmd,
                    error,
                    origin,
                    msg_id,
                    send_stream,
                    #[cfg(feature = "traceroute")]
                    traceroute,
                );
                return vec![cmd];
            }
        };

        trace!("{:?}: {:?}", LogMarker::DataStoreReceivedAtElder, data);

        let mut cmds = vec![];
        let targets = self.target_data_holders(data.name());

        // make sure the expected replication factor is achieved
        if data_copy_count() > targets.len() {
            error!("InsufficientAdults for storing data reliably");
            let error = Error::InsufficientAdults {
                prefix: self.network_knowledge().prefix(),
                expected: data_copy_count() as u8,
                found: targets.len() as u8,
            };

            debug!("Will send error response back to client");
            let cmd = self.cmd_error_response(
                cmd,
                error,
                origin,
                msg_id,
                send_stream,
                #[cfg(feature = "traceroute")]
                traceroute,
            );
            return vec![cmd];
        }

        // the replication msg sent to adults
        cmds.push(self.replicate_data(
            data,
            targets,
            #[cfg(feature = "traceroute")]
            traceroute.clone(),
        ));

        // the ack sent to client
        cmds.push(self.send_cmd_ack(
            origin,
            msg_id,
            send_stream,
            #[cfg(feature = "traceroute")]
            traceroute,
        ));

        cmds
    }

    // helper to extract the contents of the cmd as ReplicatedData
    fn extract_contents_as_replicated_data(&self, cmd: SpentbookCmd) -> Result<ReplicatedData> {
        let SpentbookCmd::Spend {
            key_image,
            tx,
            spent_proofs,
            spent_transactions,
            ..
        } = cmd;

        info!("Processing spend request for key image: {:?}", key_image);

        let spent_proof_share =
            self.gen_spent_proof_share(&key_image, &tx, &spent_proofs, &spent_transactions)?;
        let reg_cmd = self.gen_register_cmd(&key_image, &spent_proof_share)?;
        Ok(ReplicatedData::SpentbookWrite(reg_cmd))
    }

    /// Generate a spent proof share from the information provided by the client.
    fn gen_spent_proof_share(
        &self,
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
            if !self.network_knowledge.verify_section_key_is_known(pk) {
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
            let msg = format!(
                "There are no commitments for the given key image {:?}",
                key_image
            );
            debug!("Dropping spend request: {msg}");
            return Err(Error::SpentbookError(msg));
        }
        let spent_proof_share = build_spent_proof_share(
            key_image,
            tx,
            &self.network_knowledge.section_auth(),
            &self.section_keys_provider,
            public_commitments,
        )?;
        Ok(spent_proof_share)
    }

    /// Generate the RegisterCmd to write the SpentProofShare as an entry in the Spentbook
    /// (Register).
    fn gen_register_cmd(
        &self,
        key_image: &KeyImage,
        spent_proof_share: &SpentProofShare,
    ) -> Result<RegisterCmd> {
        let mut permissions = BTreeMap::new();
        let _ = permissions.insert(User::Anyone, Permissions::new(true));

        // use our own keypair for generating the register command
        let own_keypair = Keypair::Ed25519(self.info().keypair);
        let owner = User::Key(own_keypair.public_key());
        let policy = Policy { owner, permissions };

        let mut register = Register::new(
            owner,
            XorName::from_content(&key_image.to_bytes()),
            SPENTBOOK_TYPE_TAG,
            policy,
        );

        let entry = Bytes::from(rmp_serde::to_vec_named(spent_proof_share).map_err(|err| {
            Error::SpentbookError(format!(
                "Failed to serialise SpentProofShare to insert it into the spentbook (Register): {:?}",
                err
            ))
        })?);

        let (_, op) = register.write(entry.to_vec(), BTreeSet::default())?;
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
