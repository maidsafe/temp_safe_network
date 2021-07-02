// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{
    node::{DstInfo, NodeMsg, Peer},
    NodeMsgAuthority,
};
use crate::routing::{
    messages::NodeMsgAuthorityUtils, peer::PeerUtils, routing_api::command::Command,
    section::SectionUtils, Error, Result,
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
        node_msg: NodeMsg,
        msg_authority: NodeMsgAuthority,
    ) -> Result<Command> {
        let src_name = msg_authority.name();

        let bounce_dst_section_pk = self.section_key_by_name(&src_name);
        let dst_info = DstInfo {
            dst: src_name,
            dst_section_pk: bounce_dst_section_pk,
        };

        let bounce_node_msg = NodeMsg::BouncedUntrustedMessage {
            msg: Box::new(node_msg),
            dst_info,
        };
        let cmd =
            self.send_direct_message((src_name, sender), bounce_node_msg, bounce_dst_section_pk)?;

        Ok(cmd)
    }

    pub(crate) fn handle_bounced_untrusted_message(
        &self,
        sender: Peer,
        dst_section_key: BlsPublicKey,
        bounced_msg: NodeMsg,
    ) -> Result<Command> {
        let span = trace_span!("Received BouncedUntrustedMessage", ?bounced_msg, %sender);
        let _span_guard = span.enter();

        let new_node_msg = match bounced_msg {
            NodeMsg::Sync { section, network } => {
                // `Sync` messages are handled specially, because they don't carry a signed chain.
                // Instead we use the section chain that's part of the included `Section` struct.
                // Problem is we can't extend that chain as it would invalidate the signature. We
                // must construct a new message instead.
                let section = section
                    .extend_chain(&dst_section_key, self.section.chain())
                    .map_err(|err| {
                        error!("extending section chain failed: {:?}", err);
                        Error::InvalidMessage // TODO: more specific error
                    })?;

                NodeMsg::Sync { section, network }
            }
            bounced_msg => bounced_msg,
        };

        let cmd = self.send_direct_message(
            (*sender.name(), *sender.addr()),
            new_node_msg,
            dst_section_key,
        )?;

        Ok(cmd)
    }
}
