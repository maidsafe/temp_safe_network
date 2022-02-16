// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{JoinResponse, NodeState, SectionAuth, SystemMsg};
use crate::node::{api::cmds::Cmd, core::Node};
use crate::types::{log_markers::LogMarker, Peer};

impl Node {
    // Send `NodeApproval` to a joining node which makes it a section member
    pub(crate) async fn send_node_approval(&self, node_state: SectionAuth<NodeState>) -> Vec<Cmd> {
        let peer = Peer::new(node_state.value.name, node_state.value.addr);
        let section_chain = self.network_knowledge.section_chain().await;
        let genesis_key = *self.network_knowledge.genesis_key();
        let section_auth = self
            .network_knowledge
            .section_signed_authority_provider()
            .await
            .into_authed_msg();
        let section_peers = self
            .network_knowledge
            .section_signed_members()
            .await
            .iter()
            .map(|state| state.clone().into_authed_msg())
            .collect();

        info!(
            "Our section with {:?} has approved peer {}.",
            section_auth.prefix, peer,
        );

        let node_msg = SystemMsg::JoinResponse(Box::new(JoinResponse::Approval {
            genesis_key,
            section_auth,
            section_chain,
            section_peers,
        }));

        let dst_section_pk = self.network_knowledge.section_key().await;
        trace!("{}", LogMarker::SendNodeApproval);
        match self
            .send_direct_msg(peer.clone(), node_msg, dst_section_pk)
            .await
        {
            Ok(cmd) => vec![cmd],
            Err(err) => {
                error!("Failed to send join approval to node {}: {:?}", peer, err);
                vec![]
            }
        }
    }
}
