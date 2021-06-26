// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    node::{
        JoinAsRelocatedRequest, JoinAsRelocatedResponse, NodeMsg, RelocatePayload, Section,
        SignedRelocateDetails, Variant,
    },
    DstInfo, DstLocation, MessageType, SectionAuthorityProvider,
};
use crate::routing::{
    dkg::SectionSignedUtils,
    ed25519,
    error::{Error, Result},
    messages::WireMsgUtils,
    node::Node,
    peer::PeerUtils,
    relocation::{RelocatePayloadUtils, SignedRelocateDetailsUtils},
    routing_api::command::Command,
    section::{SectionAuthorityProviderUtils, SectionUtils},
};
use crate::types::PublicKey;
use bls::PublicKey as BlsPublicKey;
use std::{collections::HashSet, net::SocketAddr};
use xor_name::{Prefix, XorName};

/// Re-join as a relocated node.
pub(crate) struct JoiningAsRelocated {
    node: Node,
    genesis_key: BlsPublicKey,
    section_key: BlsPublicKey,
    relocate_payload: Option<RelocatePayload>,
    relocate_details: SignedRelocateDetails,
    // Avoid sending more than one request to the same peer.
    used_recipients: HashSet<SocketAddr>,
}

impl JoiningAsRelocated {
    pub fn new(
        node: Node,
        genesis_key: BlsPublicKey,
        relocate_details: SignedRelocateDetails,
    ) -> Result<Self> {
        let section_key = relocate_details.dst_key()?;

        Ok(Self {
            node,
            genesis_key,
            section_key,
            relocate_payload: None,
            relocate_details,
            used_recipients: HashSet::<SocketAddr>::new(),
        })
    }

    // Generates the first command to send a `JoinAsRelocatedRequest`, responses
    // shall be fed back with `handle_join_response` function.
    pub fn start(&mut self, bootstrap_addrs: Vec<SocketAddr>) -> Result<Command> {
        let dst_xorname = *self.relocate_details.dst()?;
        let recipients: Vec<(XorName, SocketAddr)> = bootstrap_addrs
            .iter()
            .map(|addr| (dst_xorname, *addr))
            .collect();

        self.used_recipients.extend(bootstrap_addrs);

        // We send a first join request to obtain the section prefix, which
        // we will then use to generate the relocation payload and send the
        // `JoinAsRelocatedRequest` again with it.
        // TODO: include the section Prefix in the RelocationDetails so we save one request.
        self.build_join_request_cmd(&recipients)
    }

    // Handles a `JoinAsRelocatedResponse`, if it's a:
    // - `Retry`: repeat join request with the new info, which shall include the relocation payload.
    // - `Redirect`: repeat join request with the new set of addresses.
    // - `Approval`: returns the `Section` to use by this node, completing the relocation.
    // - `NodeNotReachable`: returns an error, completing the relocation attempt.
    pub async fn handle_join_response(
        &mut self,
        join_response: JoinAsRelocatedResponse,
        sender: SocketAddr,
    ) -> Result<Option<Command>> {
        match join_response {
            JoinAsRelocatedResponse::Approval {
                section_auth,
                section_chain,
                node_state,
            } => {
                if node_state.value.peer.name() != &self.node.name() {
                    trace!("Ignore NodeApproval not for us");
                    return Ok(None);
                }

                if self.genesis_key != *section_chain.root_key() {
                    trace!("Genesis key doesn't match");
                    return Ok(None);
                }

                if !section_auth.verify(&section_chain) || !node_state.verify(&section_chain) {
                    return Err(Error::InvalidMessage);
                }

                let trusted_key = if let Some(payload) = &self.relocate_payload {
                    Some(&payload.relocate_details()?.dst_key)
                } else {
                    // Note this should never be the case, and it will be fixed once we
                    // make the relocation payload nono-optional for JoinAsRelocatedRequest.
                    return Err(Error::InvalidState);
                };

                if !section_chain.check_trust(trusted_key) {
                    error!("Verification failed - untrusted Join approval message",);
                    return Ok(None);
                }

                trace!(
                    "This node has been approved to join the network at {:?}!",
                    section_auth.value.prefix,
                );

                Ok(Some(Command::HandleRelocationComplete {
                    node: self.node.clone(),
                    section: Section::new(self.genesis_key, section_chain, section_auth)?,
                }))
            }
            JoinAsRelocatedResponse::Retry(section_auth) => {
                let dst = match &self.relocate_payload {
                    Some(payload) => *payload.details.dst()?,
                    None => self.node.name(),
                };

                if !self.check_autority_provider(&section_auth, &dst) {
                    return Ok(None);
                }

                if section_auth.section_key() == self.section_key {
                    return Ok(None);
                }

                let new_recipients: Vec<(XorName, SocketAddr)> = section_auth
                    .elders
                    .iter()
                    .map(|(name, addr)| (*name, *addr))
                    .collect();

                // if we are relocating, and we didn't generate
                // the relocation payload yet, we do it now
                if self.relocate_payload.is_none() {
                    self.build_relocation_payload(&section_auth.prefix)?;
                }

                info!(
                    "Newer Join response for our prefix {:?} from {:?}",
                    section_auth, sender
                );
                self.section_key = section_auth.section_key();

                let cmd = self.build_join_request_cmd(&new_recipients)?;
                self.used_recipients
                    .extend(new_recipients.iter().map(|(_, addr)| addr));

                Ok(Some(cmd))
            }
            JoinAsRelocatedResponse::Redirect(section_auth) => {
                let dst = match &self.relocate_payload {
                    Some(payload) => *payload.details.dst()?,
                    None => self.node.name(),
                };

                if !self.check_autority_provider(&section_auth, &dst) {
                    return Ok(None);
                }

                if section_auth.section_key() == self.section_key {
                    return Ok(None);
                }

                // Ignore already used recipients
                let new_recipients: Vec<(XorName, SocketAddr)> = section_auth
                    .elders
                    .iter()
                    .filter(|(_, addr)| !self.used_recipients.contains(addr))
                    .map(|(name, addr)| (*name, *addr))
                    .collect();

                if new_recipients.is_empty() {
                    debug!("Joining redirected to the same set of peers we already contacted - ignoring response");
                    return Ok(None);
                } else {
                    info!(
                        "Joining redirected to another set of peers: {:?}",
                        new_recipients,
                    );
                }

                // if we are relocating, and we didn't generate
                // the relocation payload yet, we do it now
                if self.relocate_payload.is_none() {
                    self.build_relocation_payload(&section_auth.prefix)?;
                }

                info!(
                    "Newer Join response for our prefix {:?} from {:?}",
                    section_auth, sender
                );
                self.section_key = section_auth.section_key();

                let cmd = self.build_join_request_cmd(&new_recipients)?;
                self.used_recipients
                    .extend(new_recipients.iter().map(|(_, addr)| addr));

                Ok(Some(cmd))
            }
            JoinAsRelocatedResponse::NodeNotReachable(addr) => {
                error!(
                    "Node cannot join as relocated since it is not externally reachable: {}",
                    addr
                );
                Err(Error::NodeNotReachable(addr))
            }
        }
    }

    // Change our name to fit the destination section and apply the new age.
    fn build_relocation_payload(&mut self, prefix: &Prefix) -> Result<()> {
        // We are relocating so we need to change our name.
        // Use a name that will match the destination even after multiple splits
        let extra_split_count = 3;
        let name_prefix = Prefix::new(
            prefix.bit_count() + extra_split_count,
            *self.relocate_details.dst()?,
        );

        let age = self.relocate_details.relocate_details()?.age;
        let new_keypair = ed25519::gen_keypair(&name_prefix.range_inclusive(), age);
        let new_name = XorName::from(PublicKey::from(new_keypair.public));
        self.relocate_payload = Some(RelocatePayload::new(
            self.relocate_details.clone(),
            &new_name,
            &self.node.keypair,
        ));

        info!("Changing name to {}", new_name);
        self.node = Node::new(new_keypair, self.node.addr);

        Ok(())
    }

    fn build_join_request_cmd(&self, recipients: &[(XorName, SocketAddr)]) -> Result<Command> {
        let join_request = JoinAsRelocatedRequest {
            section_key: self.section_key,
            relocate_payload: self.relocate_payload.clone(),
        };

        info!("Sending {:?} to {:?}", join_request, recipients);

        let variant = Variant::JoinAsRelocatedRequest(Box::new(join_request));
        let routing_msg = NodeMsg::single_src(
            &self.node,
            DstLocation::DirectAndUnrouted,
            variant,
            self.section_key,
        )?;

        let message = MessageType::Routing {
            msg: routing_msg,
            dst_info: DstInfo {
                dst: recipients[0].0,
                dst_section_pk: self.section_key,
            },
        };

        let cmd = Command::SendMessage {
            recipients: recipients.to_vec(),
            delivery_group_size: recipients.len(),
            message,
        };

        Ok(cmd)
    }

    fn check_autority_provider(
        &self,
        section_auth: &SectionAuthorityProvider,
        dst: &XorName,
    ) -> bool {
        if !section_auth.prefix.matches(dst) {
            error!("Invalid JoinResponse bad prefix: {:?}", section_auth);
            false
        } else if section_auth.elders.is_empty() {
            error!(
                "Invalid JoinResponse, empty list of Elders: {:?}",
                section_auth
            );
            false
        } else {
            true
        }
    }
}
