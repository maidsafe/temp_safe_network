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
mod service_msgs;
mod update_section;

use super::Core;
use crate::messaging::{
    data::{ServiceMsg, StorageLevel},
    signature_aggregator::Error as AggregatorError,
    system::{NodeCmd, NodeQuery, Proposal, SystemMsg},
    DstLocation, EndUser, MessageId, MessageType, MsgKind, NodeMsgAuthority, SectionAuth,
    ServiceAuth, SrcLocation, WireMsg,
};
use crate::routing::{
    messages::{NodeMsgAuthorityUtils, WireMsgUtils},
    relocation::RelocateState,
    routing_api::command::Command,
    Error, Event, MessageReceived, Result, SectionAuthorityProviderUtils,
};
use crate::types::{Chunk, Keypair, PublicKey};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use rand::rngs::OsRng;
use std::{collections::BTreeSet, net::SocketAddr};
use xor_name::XorName;

// Message handling
impl Core {
    pub(crate) async fn handle_message(
        &self,
        sender: SocketAddr,
        wire_msg: WireMsg,
        original_bytes: Option<Bytes>,
    ) -> Result<Vec<Command>> {
        // Deserialize the payload of the incoming message
        let payload = wire_msg.payload.clone();
        let msg_id = wire_msg.msg_id();

        let message_type = match wire_msg.clone().into_message() {
            Ok(message_type) => message_type,
            Err(error) => {
                error!(
                    "Failed to deserialize message payload ({:?}): {:?}",
                    msg_id, error
                );
                return Ok(vec![]);
            }
        };

        match message_type {
            MessageType::System {
                msg_id,
                msg_authority,
                dst_location,
                msg,
            } => {
                trace!("System message received");
                // Let's now verify the section key in the msg authority is trusted
                // based on our current knowledge of the network and sections chains.
                let mut known_keys: Vec<BlsPublicKey> =
                    self.section.chain().keys().copied().collect();
                known_keys.extend(self.network.section_keys());
                known_keys.push(*self.section.genesis_key());

                if !msg_authority.verify_src_section_key_is_known(&known_keys) {
                    debug!("Untrusted message from {:?}: {:?} ", sender, msg);

                    return if !matches!(msg, SystemMsg::AntiEntropyUpdate { .. }) {
                        let cmd = self.handle_untrusted_message(sender, msg, msg_authority)?;
                        Ok(vec![cmd])
                    } else {
                        // otherwise we might loop
                        warn!("Dropping untrusted AE Update");
                        Ok(vec![])
                    };
                }
                trace!(
                    "Trusted msg authority in message from {:?}: {:?}",
                    sender,
                    msg
                );

                // Let's check for entropy before we proceed further
                // Adult nodes don't need to carry out entropy checking,
                // however the message shall always be handled.
                if self.is_elder() {
                    // For the case of receiving a join request not matching our prefix,
                    // we just let the join request handler to deal with it later on.
                    // We also skip AE check on Anti-Entropy messages
                    //
                    // TODO: consider changing the join and "join as relocated" flows to
                    // make use of AntiEntropy retry/redirect responses.
                    match msg {
                        SystemMsg::AntiEntropyRetry { .. }
                        | SystemMsg::AntiEntropyUpdate { .. }
                        | SystemMsg::AntiEntropyRedirect { .. }
                        | SystemMsg::JoinRequest(_)
                        | SystemMsg::JoinAsRelocatedRequest(_) => {}
                        _ => match dst_location.section_pk() {
                            None => {}
                            Some(dst_section_pk) => {
                                let msg_bytes = original_bytes.unwrap_or(wire_msg.serialize()?);

                                if let Some(ae_command) = self
                                    .check_for_entropy(
                                        // a cheap clone w/ Bytes
                                        msg_bytes,
                                        &msg_authority.src_location(),
                                        &dst_section_pk,
                                        dst_location.name(),
                                        sender,
                                    )
                                    .await?
                                {
                                    // short circuit and send those AE responses
                                    return Ok(vec![ae_command]);
                                }
                            }
                        },
                    }

                    info!("Entropy check passed. Handling verified msg {}", msg_id);
                }

                Ok(vec![Command::HandleSystemMessage {
                    sender,
                    msg_id,
                    msg_authority,
                    dst_location,
                    msg,
                    payload,
                    known_keys,
                }])
            }
            MessageType::Service {
                msg_id,
                auth,
                msg,
                dst_location,
            } => {
                let dst_name = match msg.dst_address() {
                    Some(name) => name,
                    None => {
                        error!(
                            "Service msg has been dropped since {:?} is not a valid msg to send from a client {}.",
                            msg, sender
                        );
                        return Ok(vec![]);
                    }
                };
                let user = match self.comm.get_connection_id(&sender).await {
                    Some(name) => EndUser(name),
                    None => {
                        error!(
                            "Service msg has been dropped since client connection id for {} was not found: {:?}",
                            sender, msg
                        );
                        return Ok(vec![]);
                    }
                };

                let src_location = SrcLocation::EndUser(user);

                if self.is_not_elder() {
                    trace!("Redirecting from adult to section elders");
                    return Ok(vec![self.ae_redirect(sender, &src_location, &wire_msg)?]);
                }

                // First we perform AE checks
                let received_section_pk = match dst_location.section_pk() {
                    Some(section_pk) => section_pk,
                    None => {
                        warn!("Dropping service message as there is no valid dst section_pk.");
                        return Ok(vec![]);
                    }
                };

                let msg_bytes = original_bytes.unwrap_or(wire_msg.serialize()?);
                if let Some(command) = self
                    .check_for_entropy(
                        // a cheap clone w/ Bytes
                        msg_bytes,
                        &src_location,
                        &received_section_pk,
                        dst_name,
                        sender,
                    )
                    .await?
                {
                    // short circuit and send those AE responses
                    return Ok(vec![command]);
                }

                self.handle_service_message(msg_id, auth, msg, dst_location, user)
                    .await
            }
        }
    }

    // Handler for all system messages
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn handle_system_message(
        &self,
        sender: SocketAddr,
        msg_id: MessageId,
        mut msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        msg: SystemMsg,
        payload: Bytes,
        known_keys: Vec<BlsPublicKey>,
    ) -> Result<Vec<Command>> {
        info!("Handling system message");

        // We assume to be aggregated if it contains a BLS Share sig as authority.
        match self
            .aggregate_message_and_stop(&mut msg_authority, payload)
            .await
        {
            Ok(false) => match msg {
                SystemMsg::NodeCmd(_)
                | SystemMsg::NodeQuery(_)
                | SystemMsg::NodeQueryResponse { .. } => {
                    let cmd = Command::HandleVerifiedNodeDataMessage {
                        msg_id,
                        msg,
                        msg_authority,
                        dst_location,
                    };

                    Ok(vec![cmd])
                }
                _ => {
                    let cmd = Command::HandleVerifiedNodeNonDataMessage {
                        sender,
                        msg_id,
                        msg,
                        msg_authority,
                        known_keys,
                    };

                    Ok(vec![cmd])
                }
            },
            Err(Error::InvalidSignatureShare) => {
                info!(
                    "Invalid signature on received system message. Handling as untrusted message."
                );
                let cmd = self.handle_untrusted_message(sender, msg, msg_authority)?;
                Ok(vec![cmd])
            }
            Ok(true) | Err(_) => Ok(vec![]),
        }
    }

    // Hanlder for node messages which have successfully
    // passed all signature checks and msg verifications
    pub(crate) async fn handle_verified_non_data_node_message(
        &mut self,
        sender: SocketAddr,
        msg_id: MessageId,
        msg_authority: NodeMsgAuthority,
        node_msg: SystemMsg,
        known_keys: Vec<BlsPublicKey>,
    ) -> Result<Vec<Command>> {
        let src_name = msg_authority.name();

        trace!("Handling verified Non-Data message");
        match node_msg {
            SystemMsg::AntiEntropyRetry {
                section_auth,
                section_signed,
                proof_chain,
                bounced_msg,
            } => {
                trace!("Handling msg: AE-Retry from {}", sender);
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
            SystemMsg::AntiEntropyRedirect {
                section_auth,
                section_signed,
                bounced_msg,
            } => {
                trace!("Handling msg: AE-Redirect from {}", sender);
                self.handle_anti_entropy_redirect_msg(
                    section_auth,
                    section_signed,
                    bounced_msg,
                    sender,
                )
                .await
            }
            SystemMsg::AntiEntropyUpdate {
                section_auth,
                section_signed,
                proof_chain,
                members,
            } => {
                trace!("Handling msg: AE-Update from {}", sender);
                self.handle_anti_entropy_update_msg(
                    section_auth,
                    section_signed,
                    proof_chain,
                    members,
                    sender,
                )
                .await
            }
            SystemMsg::Relocate(ref details) => {
                trace!("Handling msg: Relocate from {}", sender);
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
            SystemMsg::RelocatePromise(promise) => {
                trace!("Handling msg: RelocatePromise from {}", sender);
                self.handle_relocate_promise(promise, node_msg).await
            }
            SystemMsg::StartConnectivityTest(name) => {
                trace!("Handling msg: StartConnectivityTest from {}", sender);
                if self.is_not_elder() {
                    return Ok(vec![]);
                }

                Ok(vec![Command::TestConnectivity(name)])
            }
            SystemMsg::JoinRequest(join_request) => {
                trace!("Handling msg: JoinRequest from {}", sender);
                self.handle_join_request(msg_authority.peer(sender)?, *join_request)
                    .await
            }
            SystemMsg::JoinAsRelocatedRequest(join_request) => {
                trace!("Handling msg: JoinAsRelocatedRequest from {}", sender);
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
            SystemMsg::BouncedUntrustedMessage {
                msg: bounced_msg,
                dst_section_pk,
            } => {
                trace!("Handling msg: BouncedUntrustedMessage from {}", sender);

                Ok(self.handle_bounced_untrusted_message(
                    msg_authority.peer(sender)?,
                    dst_section_pk,
                    *bounced_msg,
                )?)
            }
            SystemMsg::DkgStart {
                dkg_key,
                elder_candidates,
            } => {
                trace!("Handling msg: Dkg-Start from {}", sender);
                if !elder_candidates.elders.contains_key(&self.node.name()) {
                    return Ok(vec![]);
                }

                self.handle_dkg_start(dkg_key, elder_candidates)
            }
            SystemMsg::DkgMessage { dkg_key, message } => {
                trace!(
                    "Handling msg: Dkg-Msg ({:?} - {:?}) from {}",
                    dkg_key,
                    message,
                    sender
                );
                self.handle_dkg_message(dkg_key, message, src_name)
            }
            SystemMsg::DkgFailureObservation {
                dkg_key,
                sig,
                failed_participants,
            } => {
                trace!("Handling msg: Dkg-FailureObservation from {}", sender);
                self.handle_dkg_failure_observation(dkg_key, &failed_participants, sig)
            }
            SystemMsg::DkgFailureAgreement(sig_set) => {
                trace!("Handling msg: Dkg-FailureAgreement from {}", sender);
                self.handle_dkg_failure_agreement(&src_name, &sig_set)
            }
            SystemMsg::Propose {
                ref content,
                ref sig_share,
            } => {
                trace!("Handling msg: Propose from {}", sender);
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

                commands.extend(self.check_lagging((src_name, sender), sig_share)?);

                let result = self.handle_proposal(content.clone(), sig_share.clone())?;
                commands.extend(result);

                Ok(commands)
            }
            SystemMsg::JoinResponse(join_response) => {
                debug!("Ignoring unexpected message: {:?}", join_response);
                Ok(vec![])
            }
            SystemMsg::JoinAsRelocatedResponse(join_response) => {
                trace!("Handling msg: JoinAsRelocatedResponse from {}", sender);
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
            SystemMsg::NodeMsgError {
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

    // Handler for data messages which have successfully
    // passed all signature checks and msg verifications
    pub(crate) async fn handle_verified_data_message(
        &self,
        msg_id: MessageId,
        msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        node_msg: SystemMsg,
    ) -> Result<Vec<Command>> {
        match node_msg {
            // The following type of messages are all handled by upper sn_node layer.
            // TODO: In the future the sn-node layer won't be receiving Events but just
            // plugging in msg handlers.
            SystemMsg::NodeCmd(node_cmd) => {
                match node_cmd {
                    NodeCmd::StoreChunk { chunk, .. } => {
                        info!("Processing chunk write with MessageId: {:?}", msg_id);
                        // There is no point in verifying a sig from a sender A or B here.
                        let level_report = self.chunk_storage.store(&chunk).await?;
                        return Ok(self.record_if_any(level_report).await);
                    }
                    NodeCmd::ReplicateChunk(chunk) => {
                        info!(
                            "Processing replicate chunk cmd with MessageId: {:?}",
                            msg_id
                        );

                        return if self.is_elder() {
                            self.republish_chunk(chunk).await
                        } else {
                            // We are an adult here, so just store away!

                            // TODO: should this be a cmd returned for threading?
                            let level_report =
                                self.chunk_storage.store_for_replication(chunk).await?;
                            Ok(self.record_if_any(level_report).await)
                        };
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
            SystemMsg::NodeQuery(node_query) => {
                match node_query {
                    // A request from EndUser - via elders - for locally stored chunk
                    NodeQuery::GetChunk { origin, address } => {
                        // There is no point in verifying a sig from a sender A or B here.
                        // Send back response to the sending elder

                        let sender_xorname = msg_authority.get_auth_xorname();
                        self.handle_get_chunk_at_adult(msg_id, address, origin, sender_xorname)
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
            SystemMsg::NodeQueryResponse {
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
            SystemMsg::NodeMsgError {
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

    async fn record_if_any(&self, level: Option<StorageLevel>) -> Vec<Command> {
        let mut cmds = vec![];
        if let Some(level) = level {
            info!("Storage has now passed {} % used.", 10 * level.value());
            let node_id = PublicKey::from(self.node().keypair.public);
            let node_xorname = XorName::from(node_id);

            // we ask the section to record the new level reached
            let msg = SystemMsg::NodeCmd(NodeCmd::RecordStorageLevel {
                section: node_xorname,
                node_id,
                level,
            });

            let dst = DstLocation::Section {
                name: node_xorname,
                section_pk: self.section.section_auth.value.section_key(),
            };

            cmds.push(Command::PrepareNodeMsgToSend { msg, dst });
        }
        cmds
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

            let msg = SystemMsg::NodeCmd(NodeCmd::ReplicateChunk(chunk));
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
        msg: SystemMsg,
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
                error!("Error accumulating message at dst: {:?}", err);
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
