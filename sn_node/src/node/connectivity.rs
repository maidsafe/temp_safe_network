// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{flow_ctrl::cmds::Cmd, MyNode, Result};
use sn_fault_detection::IssueType;
use std::{collections::BTreeSet, net::SocketAddr};
use xor_name::XorName;

impl MyNode {
    /// Track comms issue if this is a peer we know and care about
    pub(crate) fn handle_failed_send(&self, addr: &SocketAddr) {
        let name = if let Some(peer) = self.network_knowledge.find_member_by_addr(addr) {
            debug!("Lost known peer {}", peer);
            peer.name()
        } else {
            trace!("Lost unknown peer {}", addr);
            return;
        };

        self.track_node_issue(name, IssueType::Communication);
    }

    pub(crate) fn cast_offline_proposals(&mut self, names: &BTreeSet<XorName>) -> Result<Vec<Cmd>> {
        let mut result: Vec<Cmd> = Vec::new();
        for name in names.iter() {
            if let Some(info) = self.network_knowledge.get_section_member(name) {
                if let Some(cmd) = self.propose_membership_change(info.leave()?) {
                    result.push(cmd);
                }
            }
        }
        Ok(result)
    }
}
