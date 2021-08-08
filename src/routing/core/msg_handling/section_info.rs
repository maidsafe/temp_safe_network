// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::messaging::{
    node::{NodeMsg, SectionAuth},
    section_info::{GetSectionResponse, SectionInfoMsg},
    DstLocation, SectionAuthorityProvider, WireMsg,
};
use crate::routing::{
    error::{Error, Result},
    network::NetworkLogic,
    peer::PeerUtils,
    routing_api::command::Command,
    section::SectionLogic,
    SectionAuthorityProviderUtils,
};
use secured_linked_list::SecuredLinkedList;
use std::net::SocketAddr;
use xor_name::XorName;

// Message handling
impl Core {
    pub(crate) async fn handle_section_info_msg(
        &self,
        sender: SocketAddr,
        mut dst_location: DstLocation,
        message: SectionInfoMsg,
    ) -> Vec<Command> {
        match message {
            SectionInfoMsg::GetSectionQuery(public_key) => {
                let name = XorName::from(public_key);

                debug!("Received GetSectionQuery({}) from {}", name, sender);

                let response = if let (true, Ok(pk_set)) = (
                    self.section.prefix().await.matches(&name),
                    self.public_key_set().await,
                ) {
                    GetSectionResponse::Success(SectionAuthorityProvider {
                        prefix: self.section.authority_provider().await.prefix(),
                        public_key_set: pk_set,
                        elders: self
                            .section
                            .authority_provider()
                            .await
                            .peers()
                            .map(|peer| (*peer.name(), *peer.addr()))
                            .collect(),
                    })
                } else {
                    // If we are elder, we should know a section that is closer to `name` that us.
                    // Otherwise redirect to our elders.
                    let network = self.network.get().await;
                    let section_auth = match network.closest(&name).await {
                        Some(section_auth) => section_auth.clone(),
                        None => self.section.authority_provider().await,
                    };
                    GetSectionResponse::Redirect(section_auth)
                };

                let response = SectionInfoMsg::GetSectionResponse(response);
                debug!("Sending {:?} to {}", response, sender);

                // Provide our PK as the dst PK, only redundant as the message
                // itself contains details regarding relocation/registration.
                dst_location.set_section_pk(self.section().last_key().await);

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

    pub(crate) async fn handle_section_knowledge_query(
        &self,
        given_key: Option<bls::PublicKey>,
        returned_msg: Box<NodeMsg>,
        sender: SocketAddr,
        src_name: XorName,
    ) -> Result<Command> {
        let chain = self.section.chain_clone().await;
        let given_key = if let Some(key) = given_key {
            key
        } else {
            *self.section_chain().await.root_key()
        };
        let truncated_chain = chain.get_proof_chain_to_current(&given_key)?;
        let section_auth = self.section.section_signed_authority_provider().await;

        let node_msg = NodeMsg::SectionKnowledge {
            src_info: (section_auth.clone(), truncated_chain),
            msg: Some(returned_msg),
        };
        let dst_section_key = self.section_key_by_name(&src_name).await;

        let cmd = self
            .send_direct_message((src_name, sender), node_msg, dst_section_key)
            .await?;

        Ok(cmd)
    }

    pub(crate) async fn handle_section_knowledge_msg(
        &self,
        signed_section_auth: SectionAuth<SectionAuthorityProvider>,
        proof_chain: SecuredLinkedList,
        msg: Option<Box<NodeMsg>>,
        src_name: XorName,
        sender: SocketAddr,
    ) -> Result<Vec<Command>> {
        // TODO: if the update fails due to not trusted SAP/prof-chain,
        // we may not need to resend the message as it's probably a sybil peer.
        let section_chain = self.section.chain_clone().await;
        let _ = self
            .network
            .get()
            .await
            .update_remote_section_sap(signed_section_auth, &proof_chain, &section_chain)
            .await;

        if let Some(node_msg) = msg {
            // This included message shall have been sent from us originally.
            // Now re-send it with the latest knowledge of the destination section.
            let dst_section_pk = self
                .network
                .get()
                .await
                .key_by_name(&src_name)
                .await
                .map_err(|_| Error::NoMatchingSection)?;

            Ok(vec![
                self.send_direct_message((src_name, sender), *node_msg, dst_section_pk)
                    .await?,
            ])
        } else {
            Ok(vec![])
        }
    }
}
