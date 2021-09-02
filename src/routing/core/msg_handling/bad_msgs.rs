// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{
    system::{Peer, SystemMsg},
    NodeMsgAuthority,
};
use crate::routing::{
    messages::NodeMsgAuthorityUtils, peer::PeerUtils, routing_api::command::Command,
    section::SectionUtils, Result,
};
use bls::PublicKey as BlsPublicKey;
use std::net::SocketAddr;

// Bad msgs
impl Core {
    // Handle message whose trust we can't establish because its signature
    // contains only keys we don't know.
    pub(crate) fn handle_untrusted_message(
        &self,
        sender: SocketAddr,
        system_msg: SystemMsg,
        msg_authority: NodeMsgAuthority,
    ) -> Result<Command> {
        let src_name = msg_authority.name();

        let bounce_dst_section_pk = self.section_key_by_name(&src_name);

        let bounce_system_msg = SystemMsg::BouncedUntrustedMessage {
            msg: Box::new(system_msg),
            dst_section_pk: bounce_dst_section_pk,
        };

        self.send_direct_message((src_name, sender), bounce_system_msg, bounce_dst_section_pk)
    }

    /// Generate command to update a peer with our current section chain
    pub(crate) fn send_ae_update_to_sender(
        &self,
        sender: Peer,
        dst_section_key: BlsPublicKey,
    ) -> Result<Command> {
        let section_signed_auth = self.section.section_signed_authority_provider().clone();
        let section_auth = section_signed_auth.value;
        let section_signed = section_signed_auth.sig;

        let proof_chain = self
            .section
            .chain()
            .get_proof_chain_to_current(&dst_section_key)?;

        let ae_msg = SystemMsg::AntiEntropyUpdate {
            section_auth,
            section_signed,
            proof_chain,
        };

        Ok(self.send_direct_message((*sender.name(), *sender.addr()), ae_msg, dst_section_key)?)
    }

    pub(crate) fn handle_bounced_untrusted_message(
        &self,
        sender: Peer,
        dst_section_key: BlsPublicKey,
        bounced_msg: SystemMsg,
    ) -> Result<Vec<Command>> {
        let span = trace_span!("Received BouncedUntrustedMessage", ?bounced_msg, %sender);
        let _span_guard = span.enter();
        let mut commands = vec![];

        // first lets update the sender with our section info, which they currently do not trust
        commands.push(self.send_ae_update_to_sender(sender, dst_section_key)?);

        let cmd = self.send_direct_message(
            (*sender.name(), *sender.addr()),
            bounced_msg,
            dst_section_key,
        )?;
        commands.push(cmd);

        Ok(commands)
    }
}
