// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod agreement;
mod anti_entropy;
mod bad_msgs;
mod dkg;
mod join;
mod proposals;
mod relocation;
mod resource_proof;
mod section_info;
mod service_msgs;
//mod sync;

use super::Core;
use crate::messaging::{
    data::{DataCmd, DataQuery, ServiceMsg},
    node::{NodeCmd, NodeMsg, NodeQuery, Proposal},
    DstLocation, MessageId, MessageType, MsgKind, NodeMsgAuthority, SectionAuth, ServiceAuth,
    WireMsg,
};
use crate::routing::messages::WireMsgUtils;
use crate::routing::{
    core::AggregatorError,
    error::{Error, Result},
    messages::NodeMsgAuthorityUtils,
    relocation::RelocateState,
    routing_api::command::Command,
    section::SectionUtils,
    Event, MessageReceived, SectionAuthorityProviderUtils,
};
use crate::types::{Chunk, Keypair, PublicKey};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use rand::rngs::OsRng;
use std::{collections::BTreeSet, iter, net::SocketAddr};
use xor_name::XorName;
// Message handling
impl Core {
    pub(crate) async fn handle_message(
        &self,
        sender: SocketAddr,
        wire_msg: WireMsg,
    ) -> Result<Vec<Command>> {
        // Deserialize the payload of the incoming message
        let payload = wire_msg.payload.clone();
        let msg_id = wire_msg.msg_id();
        let message_type = match wire_msg.into_message() {
            Ok(message_type) => message_type,
            Err(error) => {
                error!(
                    "Failed to deserialize message payload ({:?}): {}",
                    msg_id, error
                );
                return Ok(vec![]);
            }
        };

        match message_type {
            MessageType::SectionInfo {
                dst_location, msg, ..
            } => Ok(self.handle_section_info_msg(sender, dst_location, msg)),
            MessageType::Node {
                msg_id,
                msg_authority,
                dst_location,
                msg,
            } => Ok(vec![Command::HandleNodeMessage {
                sender,
                msg_id,
                auth: msg_authority,
                dst_location,
                msg,
                payload,
            }]),
            MessageType::Service {
                msg_id,
                auth,
                msg,
                dst_location,
            } => {
                self.handle_service_message(sender, msg_id, auth, msg, dst_location)
                    .await
            }
        }
    }

    // Handler for all node messages
    pub(crate) async fn handle_node_message(
        &self,
        sender: SocketAddr,
        msg_id: MessageId,
        mut msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        node_msg: NodeMsg,
        payload: Bytes,
    ) -> Result<Vec<Command>> {
        // Let's now verify the section key in the msg authority is trusted
        // based on our current knowledge of the network and sections chains.
        let known_keys: Vec<BlsPublicKey> = self
            .section
            .chain()
            .keys()
            .copied()
            .chain(self.network.keys().map(|(_, key)| key))
            .chain(iter::once(*self.section.genesis_key()))
            .collect();

        if !msg_authority.verify_src_section_key(&known_keys) {
            debug!("Untrusted message from {:?}: {:?} ", sender, node_msg);
            let cmd = self.handle_untrusted_message(sender, node_msg, msg_authority)?;
            return Ok(vec![cmd]);
        }
        trace!(
            "Trusted msg authority in message from {:?}: {:?}",
            sender,
            node_msg
        );

        // Let's check for entropy before we proceed further
        if let Some(ae_command) = self
            .check_for_entropy(
                &node_msg,
                &msg_authority.src_location(),
                &dst_location,
                sender,
            )
            .await?
        {
            return Ok(vec![ae_command]);
        }

        trace!(
            "Entropy check passed. Handling verified node msg {}",
            msg_id
        );

        let mut commands = vec![];

        // We assume to be aggregated if it contains a BLS Share sig as authority.
        match self
            .aggregate_message_and_stop(&mut msg_authority, payload)
            .await
        {
            Ok(false) => match node_msg {
                NodeMsg::NodeCmd(_) | NodeMsg::NodeQuery(_) | NodeMsg::NodeQueryResponse { .. } => {
                    commands.push(Command::HandleVerifiedNodeDataMessage {
                        msg_id,
                        msg: node_msg,
                        auth: msg_authority,
                        dst_location,
                    });
                }
                _ => {
                    commands.push(Command::HandleVerifiedNodeNonDataMessage {
                        sender,
                        msg_id,
                        msg: node_msg,
                        auth: msg_authority,
                        dst_location,
                        known_keys,
                    });
                }
            },
            Err(Error::InvalidSignatureShare) => {
                let cmd = self.handle_untrusted_message(sender, node_msg, msg_authority)?;
                commands.push(cmd);
            }
            Ok(true) | Err(_) => {}
        }

        Ok(commands)
    }

    // Hanlder for node messages which have successfully
    // passed all signature checks and msg verifications
    pub(crate) async fn handle_verified_non_data_node_message(
        &mut self,
        sender: SocketAddr,
        msg_id: MessageId,
        msg_authority: NodeMsgAuthority,
        _dst_location: DstLocation,
        node_msg: NodeMsg,
        known_keys: &[BlsPublicKey],
    ) -> Result<Vec<Command>> {
        let src_name = msg_authority.name();

        match node_msg {
            NodeMsg::AntiEntropyRetry {
                section_auth,
                section_signed,
                proof_chain,
                bounced_msg,
            } => {
                self.handle_anti_entropy_retry_msg(
                    section_auth,
                    section_signed,
                    proof_chain,
                    bounced_msg,
                    sender,
                    src_name,
                )
                .await
            }
            NodeMsg::AntiEntropyRedirect {
                section_auth,
                section_signed,
                bounced_msg,
            } => {
                self.handle_anti_entropy_redirect_msg(
                    section_auth,
                    section_signed,
                    bounced_msg,
                    sender,
                )
                .await
            }
            NodeMsg::Relocate(ref details) => {
                if let NodeMsgAuthority::Section(section_signed) = msg_authority {
                    Ok(self
                        .handle_relocate(details.clone(), node_msg, section_signed)
                        .await?
                        .into_iter()
                        .collect())
                } else {
                    Err(Error::InvalidSrcLocation)
                }
            }
            NodeMsg::RelocatePromise(promise) => {
                self.handle_relocate_promise(promise, node_msg).await
            }
            NodeMsg::StartConnectivityTest(name) => {
                if self.is_not_elder() {
                    return Ok(vec![]);
                }

                Ok(vec![Command::TestConnectivity(name)])
            }
            NodeMsg::JoinRequest(join_request) => {
                self.handle_join_request(msg_authority.peer(sender)?, *join_request)
                    .await
            }
            NodeMsg::JoinAsRelocatedRequest(join_request) => {
                if self.is_not_elder()
                    && join_request.section_key == *self.section.chain().last_key()
                {
                    return Ok(vec![]);
                }

                self.handle_join_as_relocated_request(
                    msg_authority.peer(sender)?,
                    *join_request,
                    known_keys,
                )
                .await
            }
            NodeMsg::BouncedUntrustedMessage {
                msg: bounced_msg,
                dst_section_pk,
            } => Ok(vec![self.handle_bounced_untrusted_message(
                msg_authority.peer(sender)?,
                dst_section_pk,
                *bounced_msg,
            )?]),
            NodeMsg::DkgStart {
                dkg_key,
                elder_candidates,
            } => {
                if !elder_candidates.elders.contains_key(&self.node.name()) {
                    return Ok(vec![]);
                }

                self.handle_dkg_start(dkg_key, elder_candidates)
            }
            NodeMsg::DkgMessage { dkg_key, message } => {
                self.handle_dkg_message(dkg_key, message, src_name)
            }
            NodeMsg::DkgFailureObservation {
                dkg_key,
                sig,
                failed_participants,
            } => self.handle_dkg_failure_observation(dkg_key, &failed_participants, sig),
            NodeMsg::DkgFailureAgreement(sig_set) => {
                self.handle_dkg_failure_agreement(&src_name, &sig_set)
            }
            NodeMsg::Propose {
                ref content,
                ref sig_share,
            } => {
                // Any other proposal than SectionInfo needs to be signed by a known key.
                match content {
                    Proposal::SectionInfo(ref section_auth) => {
                        if section_auth.prefix == *self.section.prefix()
                            || section_auth.prefix.is_extension_of(self.section.prefix())
                        {
                            // This `SectionInfo` is proposed by the DKG participants and
                            // it's signed by the new key created by the DKG so we don't
                            // know it yet. We only require the src_name of the
                            // proposal to be one of the DKG participants.
                            if !section_auth.contains_elder(&src_name) {
                                return Ok(vec![]);
                            }
                        }
                    }
                    _ => {
                        if !self
                            .section
                            .chain()
                            .has_key(&sig_share.public_key_set.public_key())
                        {
                            let cmd =
                                self.handle_untrusted_message(sender, node_msg, msg_authority)?;
                            return Ok(vec![cmd]);
                        }
                    }
                }

                let mut commands = vec![];

                let result = self.handle_proposal(content.clone(), sig_share.clone())?;
                commands.extend(result);

                Ok(commands)
            }
            NodeMsg::JoinResponse(join_response) => {
                debug!("Ignoring unexpected message: {:?}", join_response);
                Ok(vec![])
            }
            NodeMsg::JoinAsRelocatedResponse(join_response) => {
                if let Some(RelocateState::InProgress(ref mut joining_as_relocated)) =
                    self.relocate_state.as_mut()
                {
                    if let Some(cmd) = joining_as_relocated
                        .handle_join_response(*join_response, sender)
                        .await?
                    {
                        return Ok(vec![cmd]);
                    }
                }

                Ok(vec![])
            }
            NodeMsg::NodeMsgError {
                error,
                correlation_id,
            } => {
                trace!(
                    "From {:?}({:?}), received error {:?} correlated to {:?}",
                    msg_authority.src_location(),
                    msg_id,
                    error,
                    correlation_id
                );
                Ok(vec![])
            }
            _ => {
                warn!(
                    "!!! Unexpected NodeMsg handled at verified NON DATA nodemsg handling: {:?}",
                    node_msg
                );
                Ok(vec![])
            }
        }
    }

    // Hanlder for node messages which have successfully
    // passed all signature checks and msg verifications
    pub(crate) async fn handle_verified_data_message(
        &self,
        msg_id: MessageId,
        msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        node_msg: NodeMsg,
    ) -> Result<Vec<Command>> {
        match node_msg {
            // The following type of messages are all handled by upper sn_node layer.
            // TODO: In the future the sn-node layer won't be receiving Events but just
            // plugging in msg handlers.
            NodeMsg::NodeCmd(node_cmd) => {
                match node_cmd {
                    NodeCmd::Chunks { cmd, auth, .. } => {
                        info!(
                            "Processing storing chunk command with MessageId: {:?}",
                            msg_id
                        );
                        let verified = WireMsg::verify_sig(
                            auth,
                            ServiceMsg::Cmd(DataCmd::Chunk(cmd.clone())),
                        )?;
                        self.chunk_storage.write(&cmd, verified.public_key).await?
                    }
                    NodeCmd::ReplicateChunk(chunk) => {
                        info!(
                            "Processing replicate chunk command with MessageId: {:?}",
                            msg_id
                        );

                        if self.is_elder() {
                            return self.republish_chunk(chunk).await;
                        } else {
                            // We are an adult here, so just store away!

                            // TODO: should this be a cmd returned for threading?
                            self.chunk_storage.store_for_replication(chunk).await?;
                        }
                    }
                    NodeCmd::RepublishChunk(chunk) => {
                        info!(
                            "Republishing chunk {:?} with MessageId {:?}",
                            chunk.address(),
                            msg_id
                        );

                        return self.republish_chunk(chunk).await;
                    }
                    _ => {
                        self.send_event(Event::MessageReceived {
                            msg_id,
                            src: msg_authority.src_location(),
                            dst: dst_location,
                            msg: Box::new(MessageReceived::NodeCmd(node_cmd)),
                        })
                        .await;
                    }
                }

                Ok(vec![])
            }
            NodeMsg::NodeQuery(node_query) => {
                match node_query {
                    // A request from EndUser - via elders - for locally stored chunk
                    NodeQuery::Chunks {
                        origin,
                        query,
                        auth,
                    } => {
                        let verified = WireMsg::verify_sig(
                            auth,
                            ServiceMsg::Query(DataQuery::Chunk(query.clone())),
                        )?;
                        // Send back response to our Elders
                        self.handle_chunk_query_at_adult(msg_id, query, verified.public_key, origin)
                            .await
                    }
                    _ => {
                        self.send_event(Event::MessageReceived {
                            msg_id,
                            src: msg_authority.src_location(),
                            dst: dst_location,
                            msg: Box::new(MessageReceived::NodeQuery(node_query)),
                        })
                        .await;
                        Ok(vec![])
                    }
                }
            }
            NodeMsg::NodeQueryResponse {
                response,
                correlation_id,
                user,
            } => {
                debug!("QueryResponse received from a node");
                let sending_nodes_pk = match msg_authority {
                    NodeMsgAuthority::Node(auth) => PublicKey::from(auth.into_inner().public_key),
                    _ => return Err(Error::InvalidQueryResponseAuthority),
                };

                self.handle_chunk_query_response_at_elder(
                    correlation_id,
                    response,
                    user,
                    sending_nodes_pk,
                )
                .await
            }
            NodeMsg::NodeMsgError {
                error,
                correlation_id,
            } => {
                trace!(
                    "From {:?}({:?}), received error {:?} correlated to {:?}, targeting {:?}",
                    msg_authority.src_location(),
                    msg_id,
                    error,
                    correlation_id,
                    dst_location
                );
                Ok(vec![])
            }
            _ => {
                warn!(
                    "Non data message provided to data message handler {:?}",
                    node_msg
                );
                // do nothing
                Ok(vec![])
            }
        }
    }

    // Locate ideal chunk holders for this chunk, line up wiremsgs for those to instruct them to store the chunk
    async fn republish_chunk(&self, chunk: Chunk) -> Result<Vec<Command>> {
        if self.is_elder() {
            let target_holders = self.get_chunk_holder_adults(chunk.name()).await;
            info!(
                "Republishing chunk {:?} to holders {:?}",
                chunk.address(),
                &target_holders,
            );

            let msg = NodeMsg::NodeCmd(NodeCmd::ReplicateChunk(chunk));
            let aggregation = false;

            self.send_node_msg_to_targets(msg, target_holders, aggregation)
        } else {
            error!("Received unexpected message while Adult");
            Ok(vec![])
        }
    }

    /// Takes a message and forms commands to send to specified targets
    pub(super) fn send_node_msg_to_targets(
        &self,
        msg: NodeMsg,
        targets: BTreeSet<XorName>,
        aggregation: bool,
    ) -> Result<Vec<Command>> {
        let msg_id = MessageId::new();

        let our_name = self.node().name();

        // we create a dummy/random dst location,
        // we will set it correctly for each msg and target
        // let name = network.our_name().await;
        let section_pk = *self.section_chain().last_key();

        let dummy_dst_location = DstLocation::Node {
            name: our_name,
            section_pk,
        };

        // separate this into form_wire_msg based on agg
        let mut wire_msg = if aggregation {
            let src = our_name;

            WireMsg::for_dst_accumulation(
                self.key_share().map_err(|err| err)?,
                src,
                dummy_dst_location,
                msg,
                section_pk,
            )
        } else {
            WireMsg::single_src(self.node(), dummy_dst_location, msg, section_pk)
        }?;

        wire_msg.set_msg_id(msg_id);

        let mut commands = vec![];

        for target in targets {
            debug!("sending {:?} to {:?}", wire_msg, target);
            let mut wire_msg = wire_msg.clone();
            let dst_section_pk = self.section_key_by_name(&target);
            wire_msg.set_dst_section_pk(dst_section_pk);
            wire_msg.set_dst_xorname(target);

            commands.push(Command::ParseAndSendWireMsg(wire_msg));
        }

        Ok(commands)
    }

    // Convert the provided NodeMsgAuthority to be a `Section` message
    // authority on successful accumulation. Also return 'true' if
    // current message shall not be processed any further.
    async fn aggregate_message_and_stop(
        &self,
        msg_authority: &mut NodeMsgAuthority,
        payload: Bytes,
    ) -> Result<bool> {
        let bls_share_auth = if let NodeMsgAuthority::BlsShare(bls_share_auth) = msg_authority {
            bls_share_auth
        } else {
            return Ok(false);
        };

        match SectionAuth::try_authorize(
            self.message_aggregator.clone(),
            bls_share_auth.clone().into_inner(),
            &payload,
        )
        .await
        {
            Ok(section_auth) => {
                *msg_authority = NodeMsgAuthority::Section(section_auth);
                Ok(false)
            }
            Err(AggregatorError::NotEnoughShares) => Ok(true),
            Err(err) => {
                error!("Error accumulating message at dst: {}", err);
                Err(Error::InvalidSignatureShare)
            }
        }
    }

    // TODO: Dedupe this w/ node
    fn random_client_signature(client_msg: &ServiceMsg) -> Result<(MsgKind, Bytes)> {
        let mut rng = OsRng;
        let keypair = Keypair::new_ed25519(&mut rng);
        let payload = WireMsg::serialize_msg_payload(client_msg)?;
        let signature = keypair.sign(&payload);

        let msg = MsgKind::ServiceMsg(ServiceAuth {
            public_key: keypair.public_key(),
            signature,
        });

        Ok((msg, payload))
    }
}
