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
mod sync;
mod user_msg;

use super::Core;
use crate::messaging::{
    node::{DstInfo, Error as AggregatorError, NodeMsg, Proposal},
    DstLocation, MessageId, NodeMsgAuthority,
};
use crate::routing::{
    error::{Error, Result},
    messages::{NodeMsgAuthorityUtils, WireMsgUtils},
    network::NetworkUtils,
    relocation::RelocateState,
    routing_api::command::Command,
    section::{SectionAuthorityProviderUtils, SectionUtils},
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use std::{iter, net::SocketAddr};

// Message handling
impl Core {
    pub(crate) async fn handle_message(
        &mut self,
        sender: Option<SocketAddr>,
        msg_id: MessageId,
        msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        msg: NodeMsg,
    ) -> Result<Vec<Command>> {
        // Check if the message is for us.
        let in_dst_location = dst_location.contains(&self.node.name(), self.section.prefix());

        // TODO: Broadcast message to our section when src is a Node as nodes might not know
        // all the elders in our section and the msg needs to be propagated.
        if !in_dst_location {
            // RoutingMsg not for us.
            info!("Relay message {:?} closer to the destination", msg);
            unimplemented!();
            /*
            if let Some(cmd) = self.relay_message(&msg).await? {
                return Ok(vec![cmd]);
            } else {
                return Ok(vec![]);
            }
            */
        }

        // Let's first verify the section chain in the src authority is trusted
        // based on our current knowledge of the network and sections chains.
        let known_keys: Vec<BlsPublicKey> = self
            .section
            .chain()
            .keys()
            .copied()
            .chain(self.network.keys().map(|(_, key)| key))
            .chain(iter::once(*self.section.genesis_key()))
            .collect();

        if !msg_authority.verify_src_section_chain(&known_keys) {
            debug!("Untrusted message from {:?}: {:?} ", sender, msg);
            let cmd = self.handle_untrusted_message(sender, &msg, &msg_authority)?;
            return Ok(vec![cmd]);
        }

        trace!(
            "Trusted source authority in message from {:?}: {:?}",
            sender,
            msg
        );
        let (ae_command, shall_be_handled) = self
            .check_for_entropy(&msg, &msg_authority, &dst_location, sender)
            .await?;

        let mut commands = vec![];

        if let Some(cmd) = ae_command {
            commands.push(cmd);
        }

        if shall_be_handled {
            trace!("Entropy check passed. Handling useful msg {:?}!", msg);
            commands.extend(
                self.handle_useful_message(sender, msg, dst_location, msg_authority, &known_keys)
                    .await?,
            );
        }

        Ok(commands)
    }

    pub(crate) async fn handle_useful_message(
        &mut self,
        sender: Option<SocketAddr>,
        node_msg: NodeMsg,
        dst_location: DstLocation,
        msg_authority: NodeMsgAuthority,
        known_keys: &[BlsPublicKey],
    ) -> Result<Vec<Command>> {
        let node_msg = if let Some(msg) = self.aggregate_message(node_msg)? {
            msg
        } else {
            return Ok(vec![]);
        };
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
                unimplemented!()
                /*
                if let Some(message) = msg {
                    // This included message shall be sent from us originally.
                    // Now send it back with the latest knowledge of the destination section.
                    let addr = if let Some(addr) = sender {
                        addr
                    } else {
                        error!("SectionKnowledge bounced message {:?} doesn't have addr of sender {:?}", message, src_name);
                        return Ok(vec![]);
                    };
                    let dst_section_pk = self
                        .network
                        .key_by_name(&src_name)
                        .map_err(|_| Error::NoMatchingSection)?;
                    let cmd = Command::send_message_to_node(
                        (src_name, addr),
                        *message,
                        DstInfo {
                            dst: src_name,
                            dst_section_pk,
                        },
                    );
                    Ok(vec![cmd])
                } else {
                    Ok(vec![])
                }*/
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
                    let cmd = self.handle_untrusted_message(sender, &node_msg, &msg_authority)?;
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
                let sender = sender.ok_or(Error::InvalidSrcLocation)?;
                self.handle_join_request(msg_authority.peer(sender)?, *join_request)
            }
            NodeMsg::JoinAsRelocatedRequest(join_request) => {
                if self.is_not_elder()
                    && join_request.section_key == *self.section.chain().last_key()
                {
                    return Ok(vec![]);
                }

                let sender = sender.ok_or(Error::InvalidSrcLocation)?;
                self.handle_join_as_relocated_request(
                    msg_authority.peer(sender)?,
                    *join_request,
                    &known_keys,
                )
            }
            NodeMsg::UserMessage(ref content) => {
                // If elder, always handle UserMessage, otherwise
                // handle it only if addressed directly to us as a node.
                unimplemented!();
                /*
                let our_dst = DstLocation::Node {
                    name: self.node.name(),
                    section_pk: *self.section.chain().last_key(),
                };
                if self.is_not_elder() && node_msg.dst != our_dst {
                    return Ok(vec![]);
                }

                self.handle_user_message(
                    Bytes::from(content.clone()),
                    msg_authority.src_location(),
                    dst_location,
                    node_msg.section_pk,
                    node_msg.keyed_sig(),
                )
                .await*/
            }
            NodeMsg::BouncedUntrustedMessage {
                msg: bounced_msg,
                dst_info,
            } => {
                let sender = sender.ok_or(Error::InvalidSrcLocation)?;
                Ok(vec![self.handle_bounced_untrusted_message(
                    msg_authority.peer(sender)?,
                    dst_info.dst_section_pk,
                    *bounced_msg,
                )?])
            }
            NodeMsg::SectionKnowledgeQuery {
                last_known_key,
                msg: returned_msg,
            } => {
                let sender = sender.ok_or(Error::InvalidSrcLocation)?;
                Ok(vec![self.handle_section_knowledge_query(
                    last_known_key,
                    returned_msg,
                    sender,
                    src_name,
                    msg_authority.src_location().to_dst(),
                )?])
            }
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
                                self.handle_untrusted_message(sender, &node_msg, &msg_authority)?;
                            return Ok(vec![cmd]);
                        }
                    }
                }

                let mut commands = vec![];

                if let Some(addr) = sender {
                    commands.extend(self.check_lagging((src_name, addr), sig_share)?);
                }

                let result = self.handle_proposal(content.clone(), sig_share.clone())?;
                commands.extend(result);

                Ok(commands)
            }
            NodeMsg::JoinResponse(join_response) => {
                debug!("Ignoring unexpected message: {:?}", join_response);
                Ok(vec![])
            }
            NodeMsg::JoinAsRelocatedResponse(join_response) => {
                match (sender, self.relocate_state.as_mut()) {
                    (
                        Some(sender),
                        Some(RelocateState::InProgress(ref mut joining_as_relocated)),
                    ) => {
                        if let Some(cmd) = joining_as_relocated
                            .handle_join_response(*join_response, sender)
                            .await?
                        {
                            return Ok(vec![cmd]);
                        }
                    }
                    (Some(_), _) => {}
                    (None, _) => error!("Missing sender of {:?}", join_response),
                }

                Ok(vec![])
            }
            _node_msg_types => {
                // TODO: send them to sn_node layer as an event
                unimplemented!();
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
