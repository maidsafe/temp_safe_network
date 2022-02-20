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
mod data_msgs;
mod dkg;
mod join;
mod proposals;
mod relocation;
mod resource_proof;
mod section_info;
mod sync;

use super::Core;
use crate::messaging::{
    data::ServiceMsg,
    node::{NodeMsg, Proposal},
    AuthorityProof, DstLocation, MessageId, MessageType, MsgKind, NodeMsgAuthority, SectionAuth,
    ServiceAuth, WireMsg,
};
use rand::rngs::OsRng;

use crate::routing::{
    core::AggregatorError,
    error::{Error, Result},
    messages::{NodeMsgAuthorityUtils, WireMsgUtils},
    network::NetworkUtils,
    relocation::RelocateState,
    routing_api::command::Command,
    section::SectionUtils,
    Event, MessageReceived, SectionAuthorityProviderUtils,
};
use crate::types::Keypair;
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use std::{iter, net::SocketAddr};

// Message handling
impl Core {
    pub(crate) async fn handle_message(
        &mut self,
        sender: SocketAddr,
        wire_msg: WireMsg,
    ) -> Result<Vec<Command>> {
        // Make sure the message is for us, unless it's a client msg
        // in which case we'll handle/forward it after signature check.
        let dst_location = wire_msg.dst_location();
        let msg_id = wire_msg.msg_id();
        if !wire_msg.is_client_msg_kind()
            && !dst_location.contains(&self.node.name(), self.section.prefix())
        {
            // Message is not for us
            info!("Relay message {} closer to the destination", msg_id);
            let cmd = self.relay_message(wire_msg).await?;
            return Ok(vec![cmd]);
        }

        // We can now deserialize the payload of the incoming message
        let payload = wire_msg.payload.clone();
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
            } => Ok(self
                .handle_section_info_msg(sender, dst_location, msg)
                .await),
            MessageType::Node {
                msg_id,
                msg_authority,
                dst_location,
                msg,
            } => {
                self.handle_node_message(sender, msg_id, msg_authority, dst_location, msg, payload)
                    .await
            }
            MessageType::Service {
                msg_id,
                auth,
                msg,
                dst_location,
            } => {
                self.handle_data_message(sender, msg_id, auth, msg, dst_location, payload)
                    .await
            }
        }
    }

    // Handler for all node messages
    async fn handle_node_message(
        &mut self,
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
        let (ae_command, shall_be_handled) = self
            .check_for_entropy(&node_msg, &msg_authority, &dst_location, sender)
            .await?;

        let mut commands = vec![];

        if let Some(cmd) = ae_command {
            commands.push(cmd);
        }

        if shall_be_handled {
            trace!(
                "Entropy check passed. Handling verified node msg {}",
                msg_id
            );

            // We assume to be aggregated if it contains a BLS Share sig as authority.
            match self.aggregate_message_and_stop(&mut msg_authority, payload) {
                Ok(false) => {
                    commands.extend(
                        self.handle_verified_node_message(
                            sender,
                            msg_id,
                            msg_authority,
                            dst_location,
                            node_msg,
                            &known_keys,
                        )
                        .await?,
                    );
                }
                Err(Error::InvalidSignatureShare) => {
                    let cmd = self.handle_untrusted_message(sender, node_msg, msg_authority)?;
                    commands.push(cmd);
                }
                Ok(true) | Err(_) => {}
            }
        }

        Ok(commands)
    }

    // Hanlder for node messages which have successfully
    // passed all signature checks and msg verifications
    async fn handle_verified_node_message(
        &mut self,
        sender: SocketAddr,
        msg_id: MessageId,
        msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        node_msg: NodeMsg,
        known_keys: &[BlsPublicKey],
    ) -> Result<Vec<Command>> {
        let src_name = msg_authority.name();

        match node_msg {
            NodeMsg::ForwardServiceMsg { msg, user, auth } => {
                // If elder, always handle Forward
                if self.is_not_elder() {
                    return Ok(vec![]);
                }

                // Check the client authority is valid
                // TODO: preserve the source bytes so we don't need to serialize again here, or else
                // verify earlier.
                let payload = WireMsg::serialize_msg_payload(&msg)?;
                let auth = AuthorityProof::verify(auth, &payload)?;

                Ok(vec![Command::HandleServiceMessage {
                    msg_id,
                    msg,
                    user,
                    auth,
                }])
            }
            NodeMsg::SectionKnowledge {
                src_info: (signed_section_auth, proof_chain),
                msg,
            } => {
                if self.is_not_elder() {
                    return Ok(vec![]);
                }

                self.handle_section_knowledge_msg(
                    signed_section_auth,
                    proof_chain,
                    msg,
                    src_name,
                    sender,
                )
            }
            NodeMsg::Sync {
                ref section,
                ref network,
            } => {
                // Ignore `Sync` not for our section.
                if !section.prefix().matches(&self.node.name()) {
                    return Ok(vec![]);
                }

                if section.chain().check_trust(known_keys.iter()) {
                    self.handle_sync(section, network).await
                } else {
                    debug!(
                        "Untrusted Sync message from {:?} and section: {:?} ",
                        sender, section
                    );
                    let cmd = self.handle_untrusted_message(sender, node_msg, msg_authority)?;
                    Ok(vec![cmd])
                }
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
            NodeMsg::SectionKnowledgeQuery {
                last_known_key,
                msg: returned_msg,
            } => Ok(vec![self.handle_section_knowledge_query(
                last_known_key,
                returned_msg,
                sender,
                src_name,
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
                            // This `SectionInfo` is proposed by the DKG participants and is signed by the new
                            // key created by the DKG so we don't know it yet. We only require the src_name of the
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
            // The following type of messages are all handled by upper sn_node layer.
            // TODO: In the future the sn-node layer won't be receiving Events but just
            // plugging in msg handlers.
            NodeMsg::NodeCmd(node_cmd) => {
                // Does this/can this /_should_ this ever happen via NodeMsg?
                self.send_event(Event::MessageReceived {
                    msg_id,
                    src: msg_authority.src_location(),
                    dst: dst_location,
                    msg: Box::new(MessageReceived::NodeCmd(node_cmd)),
                })
                .await;

                Ok(vec![])
            }
            NodeMsg::NodeQuery(node_query) => {
                self.send_event(Event::MessageReceived {
                    msg_id,
                    src: msg_authority.src_location(),
                    dst: dst_location,
                    msg: Box::new(MessageReceived::NodeQuery(node_query)),
                })
                .await;
                Ok(vec![])
            }
            NodeMsg::NodeQueryResponse {
                response,
                correlation_id,
            } => {
                self.send_event(Event::MessageReceived {
                    msg_id,
                    src: msg_authority.src_location(),
                    dst: dst_location,
                    msg: Box::new(MessageReceived::NodeQueryResponse {
                        response,
                        correlation_id,
                    }),
                })
                .await;
                Ok(vec![])
            }
            NodeMsg::NodeMsgError {
                error,
                correlation_id,
            } => {
                self.send_event(Event::MessageReceived {
                    msg_id,
                    src: msg_authority.src_location(),
                    dst: dst_location,
                    msg: Box::new(MessageReceived::NodeMsgError {
                        error,
                        correlation_id,
                    }),
                })
                .await;
                Ok(vec![])
            }
        }
    }

    // Convert the provided NodeMsgAuthority to be a `Section` message
    // authority on successful accumulation. Also return 'true' if
    // current message shall not be processed any further.
    fn aggregate_message_and_stop(
        &mut self,
        msg_authority: &mut NodeMsgAuthority,
        payload: Bytes,
    ) -> Result<bool> {
        let bls_share_auth = if let NodeMsgAuthority::BlsShare(bls_share_auth) = msg_authority {
            bls_share_auth
        } else {
            return Ok(false);
        };

        match SectionAuth::try_authorize(
            &mut self.message_aggregator,
            bls_share_auth.clone().into_inner(),
            &payload,
        ) {
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
            node_pk: keypair.public_key(),
            signature,
        });

        Ok((msg, payload))
    }
}
