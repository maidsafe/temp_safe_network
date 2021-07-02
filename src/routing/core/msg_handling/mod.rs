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
mod end_user;
mod join;
mod proposals;
mod relocation;
mod resource_proof;
mod section_info;
mod sync;

use super::Core;
use crate::messaging::{
    node::{Error as AggregatorError, NodeMsg, Proposal},
    DstLocation, EndUser, MessageId, MessageType, NodeMsgAuthority, WireMsg,
};
use crate::routing::{
    error::{Error, Result},
    messages::{NodeMsgAuthorityUtils, WireMsgUtils},
    network::NetworkUtils,
    relocation::RelocateState,
    routing_api::command::Command,
    section::{SectionAuthorityProviderUtils, SectionUtils},
    Event,
};
use bls::PublicKey as BlsPublicKey;
use std::{iter, net::SocketAddr};

// Message handling
impl Core {
    pub(crate) async fn handle_message(
        &mut self,
        sender: SocketAddr,
        wire_msg: WireMsg,
    ) -> Result<Vec<Command>> {
        // Check if the message is for us
        let dst_location = wire_msg.dst_location();
        let msg_id = wire_msg.msg_id();
        if !dst_location.contains(&self.node.name(), self.section.prefix()) {
            // Message is not for us
            info!("Relay message {} closer to the destination", msg_id);
            if let Some(cmds) = self.relay_message(wire_msg).await? {
                return Ok(vec![cmds]);
            } else {
                return Ok(vec![]);
            }
        }

        // Now check the signature of the msg authority is valid
        if let Err(err) = wire_msg.check_signature() {
            error!(
                "Discarding message received ({:?}) due to invalid signature: {:?}",
                msg_id, err
            );
            return Ok(vec![]);
        }

        // We can now deserialize the payload of the incoming message
        let message_type = match wire_msg.to_message() {
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
                self.handle_node_message(sender, msg_id, msg_authority, dst_location, msg)
                    .await
            }
            MessageType::Client {
                client_signed,
                msg,
                dst_location,
                ..
            } => {
                let user = match self.get_enduser_by_addr(&sender) {
                    Some(end_user) => {
                        debug!(
                            "Message from client {}, socket id already exists: {:?}",
                            sender, end_user
                        );
                        *end_user
                    }
                    None => {
                        // This is the first time we receive a message from this client
                        debug!("First message from client {}, creating a socket id", sender);

                        // TODO: remove the enduser registry and simply encrypt socket
                        // addr with this node's keypair and use that as the socket id
                        match self.try_add_enduser(sender) {
                            Ok(end_user) => end_user,
                            Err(err) => {
                                error!(
                                    "Failed to cache client socket address for message {:?}: {:?}",
                                    msg, err
                                );
                                return Ok(vec![]);
                            }
                        }
                    }
                };

                let is_in_destination = match dst_location.name() {
                    Some(dst_name) => self.section().prefix().matches(&dst_name),
                    None => true, // it's a DirectAndUnrouted dst
                };

                if is_in_destination {
                    // We send this message to be handled by the upper Node layer
                    // through the public event stream API
                    let event = Event::ClientMsgReceived {
                        msg: Box::new(msg),
                        client_signed,
                        user,
                    };
                    self.send_event(event).await;
                } else {
                    let node_msg = NodeMsg::ForwardClientMsg {
                        msg,
                        user,
                        client_signed,
                    };
                    let to_relay = match WireMsg::single_src(
                        &self.node,
                        dst_location,
                        node_msg,
                        self.section.authority_provider().section_key(),
                    ) {
                        Ok(msg) => msg,
                        Err(error) => {
                            error!("Failed create routing msg {:?}", error);
                            return Ok(vec![]);
                        }
                    };
                    match self.relay_message(to_relay).await {
                        Ok(Some(cmd)) => return Ok(vec![cmd]),
                        Ok(None) => {
                            error!("Failed to relay msg, no cmd returned.");
                        }
                        Err(error) => {
                            error!("Failed to relay msg {:?}", error);
                        }
                    }
                }

                Ok(vec![])
            }
        }
    }

    // Handler for all node messages
    async fn handle_node_message(
        &mut self,
        sender: SocketAddr,
        msg_id: MessageId,
        msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        node_msg: NodeMsg,
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

        // Let's check for entropy before we proceed to finally process the node message
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
        /*
        TODO: aggregation needs to be hanlded at each particular message
        that needs aggregation. This logic shall be moved to those msg handlers
        let node_msg = if let Some(msg) = self.aggregate_message(node_msg)? {
            msg
        } else {
            return Ok(vec![]);
        };
        */

        let src_name = msg_authority.name();

        match node_msg {
            NodeMsg::ForwardClientMsg {
                msg,
                user,
                client_signed,
            } => {
                // If elder, always handle Forward
                if self.is_not_elder() {
                    return Ok(vec![]);
                }

                self.handle_forwarded_message(msg, user, client_signed)
                    .await
            }
            NodeMsg::SectionKnowledge {
                src_info: (src_signed_sap, src_chain),
                msg,
            } => {
                if self.is_not_elder() {
                    return Ok(vec![]);
                }

                if !src_chain.check_trust(known_keys.iter()) {
                    return Ok(vec![]);
                }

                self.update_section_knowledge(src_signed_sap, src_chain);
                if let Some(node_msg) = msg {
                    // This included message shall have been sent from us originally.
                    // Now re-send it with the latest knowledge of the destination section.
                    let dst_section_pk = self
                        .network
                        .key_by_name(&src_name)
                        .map_err(|_| Error::NoMatchingSection)?;

                    Ok(vec![self.send_direct_message(
                        (src_name, sender),
                        *node_msg,
                        dst_section_pk,
                    )?])
                } else {
                    Ok(vec![])
                }
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
                dst_info,
            } => Ok(vec![self.handle_bounced_untrusted_message(
                msg_authority.peer(sender)?,
                dst_info.dst_section_pk,
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
                msg_authority.src_location().to_dst(),
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
            node_msg_types => {
                if let DstLocation::EndUser(EndUser { xorname, socket_id }) = dst_location {
                    self.handle_end_user_message(xorname, socket_id).await
                } else {
                    // We send them to sn_node layer as an event
                    unimplemented!();
                    /*self.send_event(Event::MessageReceived {
                        content,
                        src,
                        dst,
                        sig,
                        section_pk,
                    })
                    .await;

                    Ok(vec![])*/
                }
            }
        }
    }

    fn aggregate_message(&mut self, msg: NodeMsg) -> Result<Option<NodeMsg>> {
        unimplemented!();
        /*
        let sig_share = if let NodeMsgAuthority::BlsShare { sig_share, .. } = &msg.src {
            sig_share
        } else {
            // Not an aggregating message, return unchanged.
            return Ok(Some(msg));
        };

        let signed_bytes =
            bincode::serialize(&msg.signable_view()).map_err(|_| Error::InvalidMessage)?;
        match self
            .message_aggregator
            .add(&signed_bytes, sig_share.clone())
        {
            Ok(sig) => {
                trace!("Successfully accumulated signatures for message: {:?}", msg);
                Ok(Some(msg.into_dst_accumulated(sig)?))
            }
            Err(AggregatorError::NotEnoughShares) => Ok(None),
            Err(err) => {
                error!("Error accumulating message at dst: {:?}", err);
                Err(Error::InvalidSignatureShare)
            }
        }
        */
    }
}
