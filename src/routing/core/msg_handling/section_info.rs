// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::messaging::{
    node::NodeMsg,
    section_info::{GetSectionResponse, SectionInfoMsg},
    DstLocation, SectionAuthorityProvider, WireMsg,
};
use crate::routing::{
    error::Result,
    network::NetworkUtils,
    peer::PeerUtils,
    routing_api::command::Command,
    section::{SectionAuthorityProviderUtils, SectionUtils},
};
use std::net::SocketAddr;
use xor_name::XorName;

// Message handling
impl Core {
    pub(crate) async fn handle_section_info_msg(
        &mut self,
        sender: SocketAddr,
        mut dst_location: DstLocation,
        message: SectionInfoMsg,
    ) -> Vec<Command> {
        match message {
            SectionInfoMsg::GetSectionQuery(public_key) => {
                let name = XorName::from(public_key);

                debug!("Received GetSectionQuery({}) from {}", name, sender);

                let response = if let (true, Ok(pk_set)) =
                    (self.section.prefix().matches(&name), self.public_key_set())
                {
                    GetSectionResponse::Success(SectionAuthorityProvider {
                        prefix: self.section.authority_provider().prefix(),
                        public_key_set: pk_set,
                        elders: self
                            .section
                            .authority_provider()
                            .peers()
                            .map(|peer| (*peer.name(), *peer.addr()))
                            .collect(),
                    })
                } else {
                    // If we are elder, we should know a section that is closer to `name` that us.
                    // Otherwise redirect to our elders.
                    let section_auth = self
                        .network
                        .closest(&name)
                        .unwrap_or_else(|| self.section.authority_provider());
                    GetSectionResponse::Redirect(section_auth.clone())
                };

                let response = SectionInfoMsg::GetSectionResponse(response);
                debug!("Sending {:?} to {}", response, sender);

                // Provide our PK as the dst PK, only redundant as the message
                // itself contains details regarding relocation/registration.
                dst_location.set_section_pk(*self.section().chain().last_key());

                match WireMsg::new_section_info_msg(&response, dst_location) {
                    Ok(wire_msg) => vec![Command::SendMessage {
                        recipients: vec![(name, sender)],
                        delivery_group_size: 1,
                        wire_msg,
                    }],
                    Err(err) => {
                        error!("Failed to build section info response msg: {:?}", err);
                        vec![]
                    }
                }
            }
            SectionInfoMsg::GetSectionResponse(response) => {
                error!("GetSectionResponse unexpectedly received: {:?}", response);
                vec![]
            }
        }
    }

    pub(crate) fn handle_section_knowledge_query(
        &self,
        given_key: Option<bls::PublicKey>,
        returned_msg: Box<NodeMsg>,
        sender: SocketAddr,
        src_name: XorName,
        dst_location: DstLocation,
    ) -> Result<Command> {
        let chain = self.section.chain();
        let given_key = if let Some(key) = given_key {
            key
        } else {
            *self.section_chain().root_key()
        };
        let truncated_chain = chain.get_proof_chain_to_current(&given_key)?;
        let section_auth = self.section.section_signed_authority_provider();

        let node_msg = NodeMsg::SectionKnowledge {
            src_info: (section_auth.clone(), truncated_chain),
            msg: Some(returned_msg),
        };
        let dst_section_key = self.section_key_by_name(&src_name);

        let cmd = self.send_direct_message((src_name, sender), node_msg, dst_section_key)?;

        Ok(cmd)
    }
}
