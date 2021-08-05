// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::messaging::{
    section_info::{GetSectionResponse, SectionInfoMsg},
    DstLocation, SectionAuthorityProvider, WireMsg,
};
use crate::routing::{
    network::NetworkUtils, peer::PeerUtils, routing_api::command::Command, section::SectionUtils,
    SectionAuthorityProviderUtils,
};
use std::net::SocketAddr;
use xor_name::XorName;

// Message handling
impl Core {
    pub(crate) fn handle_section_info_msg(
        &self,
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
                    let section_auth = match self.network.closest(&name) {
                        Some(section_auth) => section_auth.value.clone(),
                        None => self.section.authority_provider().clone(),
                    };

                    GetSectionResponse::Redirect(section_auth)
                };

                let response = SectionInfoMsg::GetSectionResponse(response);
                debug!("Sending {:?} to {}", response, sender);

                // Provide our PK as the dst PK, only redundant as the message
                // itself contains details regarding relocation/registration.
                dst_location.set_section_pk(*self.section().chain().last_key());

                match WireMsg::new_section_info_msg(&response, dst_location) {
                    Ok(wire_msg) => vec![Command::SendMessage {
                        recipients: vec![(name, sender)],
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
}
