// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::UsedRecipientSaps;
use crate::messaging::{
    system::{
        JoinAsRelocatedRequest, JoinAsRelocatedResponse, RelocateDetails, RelocatePayload,
        SystemMsg,
    },
    AuthorityProof, DstLocation, SectionAuth, WireMsg,
};
use crate::node::{
    error::{Error, Result},
    routing::{
        api::command::Command, ed25519, messages::WireMsgUtils,
        network_knowledge::SectionAuthorityProvider, node::Node, relocation::RelocatePayloadUtils,
    },
};
use crate::peer::Peer;

use crate::types::PublicKey;
use bls::PublicKey as BlsPublicKey;
use std::net::SocketAddr;
use xor_name::{Prefix, XorName};

/// Re-join as a relocated node.
pub(crate) struct JoiningAsRelocated {
    pub(crate) node: Node,
    genesis_key: BlsPublicKey,
    dst_section_key: BlsPublicKey,
    relocate_details: RelocateDetails,
    node_msg: SystemMsg,
    node_msg_auth: AuthorityProof<SectionAuth>,
    // Avoid sending more than one duplicated request (with same SectionKey) to the same peer.
    used_recipient_saps: UsedRecipientSaps,
    relocate_payload: Option<RelocatePayload>,
}

impl JoiningAsRelocated {
    pub(crate) fn new(
        node: Node,
        genesis_key: BlsPublicKey,
        relocate_details: RelocateDetails,
        node_msg: SystemMsg,
        section_auth: AuthorityProof<SectionAuth>,
    ) -> Result<Self> {
        // First JoinAsRelocatedRequest doesn't contain RelocatePayload,
        // which triggers a Response being sent back anyway.
        // Setting dst_section_key to correct one could cause the check to fail
        // when handling such response.
        Ok(Self {
            node,
            genesis_key,
            dst_section_key: genesis_key,
            relocate_details,
            node_msg,
            node_msg_auth: section_auth,
            used_recipient_saps: UsedRecipientSaps::new(),
            relocate_payload: None,
        })
    }

    // Generates the first command to send a `JoinAsRelocatedRequest`, responses
    // shall be fed back with `handle_join_response` function.
    pub(crate) fn start(&mut self, bootstrap_addrs: Vec<SocketAddr>) -> Result<Command> {
        let dst_xorname = self.relocate_details.dst;
        let recipients: Vec<_> = bootstrap_addrs
            .iter()
            .map(|addr| Peer::new(dst_xorname, *addr))
            .collect();

        self.used_recipient_saps = bootstrap_addrs
            .iter()
            .map(|addr| (*addr, self.dst_section_key))
            .collect();

        // We send a first join request to obtain the section prefix, which
        // we will then use to generate the relocation payload and send the
        // `JoinAsRelocatedRequest` again with it.
        // TODO: include the section Prefix in the RelocationDetails so we save one request.
        self.build_join_request_cmd(&recipients, dst_xorname)
    }

    // Handles a `JoinAsRelocatedResponse`, if it's a:
    // - `Retry`: repeat join request with the new info, which shall include the relocation payload.
    // - `Redirect`: repeat join request with the new set of addresses.
    // - `Approval`: returns the `Section` to use by this node, completing the relocation.
    // - `NodeNotReachable`: returns an error, completing the relocation attempt.
    pub(crate) async fn handle_join_response(
        &mut self,
        join_response: JoinAsRelocatedResponse,
        sender: SocketAddr,
    ) -> Result<Option<Command>> {
        trace!("Hanlde JoinResponse {:?}", join_response);
        match join_response {
            JoinAsRelocatedResponse::Retry(section_auth) => {
                let section_auth = section_auth.into_state();
                if !self.check_autority_provider(&section_auth, &self.relocate_details.dst) {
                    trace!("failed to check authority");
                    return Ok(None);
                }

                if section_auth.section_key() == self.dst_section_key {
                    trace!("equal destination section key");
                    return Ok(None);
                }

                let new_section_key = section_auth.section_key();
                let new_recipients: Vec<_> = section_auth
                    .elders()
                    .filter(|peer| {
                        self.used_recipient_saps
                            .insert((peer.addr(), new_section_key))
                    })
                    .cloned()
                    .collect();

                if new_recipients.is_empty() {
                    debug!(
                        "Ignore JoinAsRelocatedResponse::Retry with old SAP that has been sent to: {:?}",
                        section_auth
                    );
                    return Ok(None);
                }

                // if we are relocating, and we didn't generate
                // the relocation payload yet, we do it now
                if self.relocate_payload.is_none() {
                    trace!("builing relocate payload");
                    self.build_relocation_payload(&section_auth.prefix())?;
                }

                info!(
                    "Newer Join response for our prefix {:?} from {:?}",
                    section_auth, sender
                );
                self.dst_section_key = section_auth.section_key();

                let cmd =
                    self.build_join_request_cmd(&new_recipients, section_auth.prefix().name())?;

                Ok(Some(cmd))
            }
            JoinAsRelocatedResponse::Redirect(section_auth) => {
                let section_auth = section_auth.into_state();

                if !self.check_autority_provider(&section_auth, &self.relocate_details.dst) {
                    return Ok(None);
                }

                if section_auth.section_key() == self.dst_section_key {
                    return Ok(None);
                }

                let new_section_key = section_auth.section_key();
                let new_recipients: Vec<_> = section_auth
                    .elders()
                    .filter(|peer| {
                        self.used_recipient_saps
                            .insert((peer.addr(), new_section_key))
                    })
                    .cloned()
                    .collect();

                if new_recipients.is_empty() {
                    debug!(
                        "Ignore JoinAsRelocatedResponse::Redirect with old SAP that has been sent to: {:?}",
                        section_auth
                    );
                    return Ok(None);
                }

                // if we are relocating, and we didn't generate
                // the relocation payload yet, we do it now
                if self.relocate_payload.is_none() {
                    self.build_relocation_payload(&section_auth.prefix())?;
                }

                info!(
                    "Newer Join response for our prefix {:?} from {:?}",
                    section_auth, sender
                );
                self.dst_section_key = section_auth.section_key();

                let cmd =
                    self.build_join_request_cmd(&new_recipients, section_auth.prefix().name())?;

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
            self.relocate_details.dst,
        );

        let age = self.relocate_details.age;
        let new_keypair = ed25519::gen_keypair(&name_prefix.range_inclusive(), age);
        let new_name = XorName::from(PublicKey::from(new_keypair.public));
        self.relocate_payload = Some(RelocatePayload::new(
            self.node_msg.clone(),
            self.node_msg_auth.clone(),
            &new_name,
            &self.node.keypair,
        ));

        info!("Changing name to {}", new_name);
        self.node = Node::new(new_keypair, self.node.addr);

        Ok(())
    }

    fn build_join_request_cmd(&self, recipients: &[Peer], dst_name: XorName) -> Result<Command> {
        let join_request = JoinAsRelocatedRequest {
            section_key: self.dst_section_key,
            relocate_payload: self.relocate_payload.clone(),
        };

        info!("Sending {:?} to {:?}", join_request, recipients);

        let node_msg = SystemMsg::JoinAsRelocatedRequest(Box::new(join_request));
        let wire_msg = WireMsg::single_src(
            &self.node,
            DstLocation::Section {
                name: dst_name,
                section_pk: self.dst_section_key,
            },
            node_msg,
            self.genesis_key,
        )?;

        let cmd = Command::SendMessage {
            recipients: recipients.to_vec(),
            wire_msg,
        };

        Ok(cmd)
    }

    fn check_autority_provider(
        &self,
        section_auth: &SectionAuthorityProvider,
        dst: &XorName,
    ) -> bool {
        if !section_auth.prefix().matches(dst) {
            error!("Invalid JoinResponse bad prefix: {:?}", section_auth);
            false
        } else if section_auth.elder_count() == 0 {
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
