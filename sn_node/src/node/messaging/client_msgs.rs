// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::messaging::{OutgoingMsg, Peers};

use crate::node::{flow_ctrl::cmds::Cmd, Error, Node, Result};
use bytes::Bytes;

use sn_dbc::{
    get_public_commitments_from_transaction, Commitment, KeyImage, RingCtTransaction, SpentProof,
    SpentProofShare,
};
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::network_knowledge::section_keys::build_spent_proof_share;
use sn_interface::{
    data_copy_count,
    messaging::{
        data::{
            ClientMsg, DataCmd, DataQueryVariant, EditRegister, OperationId, SignedRegisterEdit,
            SpentbookCmd,
        },
        system::{NodeMsg, NodeQueryResponse},
        AuthorityProof, ClientAuth, EndUser, MsgId,
    },
    types::{
        log_markers::LogMarker,
        register::{Permissions, Policy, Register, User},
        Keypair, Peer, PublicKey, RegisterCmd, ReplicatedData, SPENTBOOK_TYPE_TAG,
    },
};

use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

impl Node {
    /// Forms a `CmdAck` msg to send back to the client
    pub(crate) fn send_cmd_ack(
        &self,
        target: Peer,
        correlation_id: MsgId,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Cmd {
        let the_ack_msg = ClientMsg::CmdAck { correlation_id };
        self.send_service_msg(
            the_ack_msg,
            Peers::Single(target),
            #[cfg(feature = "traceroute")]
            traceroute,
        )
    }

    /// Forms a `CmdError` msg to send back to the client
    pub(crate) fn cmd_error_response(
        &self,
        error: Error,
        target: Peer,
        correlation_id: MsgId,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Cmd {
        let the_error_msg = ClientMsg::CmdError {
            error: error.into(),
            correlation_id,
        };

        self.send_service_msg(
            the_error_msg,
            Peers::Single(target),
            #[cfg(feature = "traceroute")]
            traceroute,
        )
    }

    /// Forms a cmd to send a cmd response error/ack to the client
    fn send_service_msg(
        &self,
        msg: ClientMsg,
        recipients: Peers,
        #[cfg(feature = "traceroute")] mut traceroute: Traceroute,
    ) -> Cmd {
        #[cfg(feature = "traceroute")]
        traceroute.0.push(self.identity());

        Cmd::SendMsg {
            msg: OutgoingMsg::Client(msg),
            msg_id: MsgId::new(),
            recipients,
            #[cfg(feature = "traceroute")]
            traceroute,
        }
    }

    /// Handle data query
    pub(crate) async fn handle_data_query_at_adult(
        &self,
        correlation_id: MsgId,
        query: &DataQueryVariant,
        auth: ClientAuth,
        user: EndUser,
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
            correlation_id,
            user,
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
        correlation_id: MsgId,
        response: NodeQueryResponse,
        user: EndUser,
        sending_node_pk: PublicKey,
        op_id: OperationId,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Option<Cmd> {
        debug!(
            "Handling data read @ elders, received from {:?}, op id: {:?}",
            sending_node_pk, op_id
        );

        let node_id = XorName::from(sending_node_pk);
        let query_peers = self.pending_data_queries.remove(&(op_id, node_id));
        // Clear expired queries from the cache.
        self.pending_data_queries.remove_expired();

        // First check for waiting peers. If no one is waiting, we drop the response
        let waiting_peers = if let Some(peers) = query_peers {
            if peers.is_empty() {
                // nothing to do
                return None;
            }
            peers
        } else {
            warn!(
                "Dropping chunk query response from Adult {}. We might have already forwarded this chunk to the requesting client or the client connection cache has expired: {}",
                sending_node_pk, user.0
            );
            return None;
        };

        let pending_removed = self
            .dysfunction_tracking
            .request_operation_fulfilled(&node_id, op_id);

        if !pending_removed {
            trace!("Ignoring un-expected response");
            return None;
        }

        let query_response = response.convert();

        let msg = ClientMsg::QueryResponse {
            response: query_response,
            correlation_id,
        };

        Some(self.send_service_msg(
            msg,
            Peers::Multiple(waiting_peers),
            #[cfg(feature = "traceroute")]
            traceroute,
        ))
    }

    /// Handle incoming service msgs. Though NOT queries, as this requires
    /// mutable access to the Node
    pub(crate) async fn handle_valid_service_msg(
        &self,
        msg_id: MsgId,
        msg: ClientMsg,
        auth: AuthorityProof<ClientAuth>,
        origin: Peer,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Result<Vec<Cmd>> {
        if !self.is_elder() {
            return Ok(vec![]);
        }

        trace!("{:?} {:?}", LogMarker::ServiceMsgToBeHandled, msg);

        // extract the data from the request
        let data = match msg {
            // These reads/writes are for adult nodes...
            ClientMsg::Cmd(DataCmd::Register(cmd)) => ReplicatedData::RegisterWrite(cmd),
            ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                key_image,
                tx,
                spent_proofs,
                spent_transactions,
                network_knowledge,
            })) => {
                info!("Processing spend request for key image: {:?}", key_image);
                if let Some((proof_chain, signed_sap)) = network_knowledge {
                    debug!(
                        "Received updated network knowledge with the request. Will return new command \
                        to update the node network knowledge before processing the spend."
                    );
                    // To avoid a loop, recompose the message without the updated proof_chain.
                    let updated_service_msg =
                        ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                            key_image,
                            tx,
                            spent_proofs,
                            spent_transactions,
                            network_knowledge: None,
                        }));
                    let update_command = Cmd::UpdateNetworkAndHandleValidServiceMsg {
                        proof_chain,
                        signed_sap,
                        msg_id,
                        msg: updated_service_msg,
                        origin,
                        auth,
                        #[cfg(feature = "traceroute")]
                        traceroute,
                    };
                    return Ok(vec![update_command]);
                }
                let spent_proof_share = self.gen_spent_proof_share(
                    &key_image,
                    &tx,
                    &spent_proofs,
                    &spent_transactions,
                )?;
                let reg_cmd = self.gen_register_cmd(&key_image, &spent_proof_share)?;
                ReplicatedData::SpentbookWrite(reg_cmd)
            }
            ClientMsg::Cmd(DataCmd::StoreChunk(chunk)) => ReplicatedData::Chunk(chunk),
            ClientMsg::Query(query) => {
                return self
                    .read_data_from_adults(
                        query,
                        msg_id,
                        auth,
                        origin,
                        #[cfg(feature = "traceroute")]
                        traceroute,
                    )
                    .await;
            }
            _ => {
                warn!(
                    "!!!! Unexpected ServiceMsg received, and it was not handled: {:?}",
                    msg
                );
                return Ok(vec![]);
            }
        };

        trace!("{:?}: {:?}", LogMarker::DataStoreReceivedAtElder, data);

        let mut cmds = vec![];
        let targets = self.target_data_holders(data.name());

        // make sure the expected replication factor is achieved
        if data_copy_count() > targets.len() {
            error!("InsufficientAdults for storing data reliably");
            return Err(Error::InsufficientAdults {
                prefix: self.network_knowledge().prefix(),
                expected: data_copy_count() as u8,
                found: targets.len() as u8,
            });
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
            #[cfg(feature = "traceroute")]
            traceroute,
        ));

        Ok(cmds)
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
