// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{NodeState as NodeStateMsg, SystemMsg};
use crate::node::{api::cmds::Cmd, core::Node, network_knowledge::NodeState};
use sn_membership::{Reconfig, SignedVote};

impl Node {
    /// Broadcast proposal to Elders to accept a new peer to join the section membership
    pub(crate) async fn propose_join_membership(&self, node_state: NodeState) -> Vec<Cmd> {
        self.broadcast_membership_proposal(Reconfig::Join(node_state.to_msg()))
            .await
    }

    /// Broadcast proposal to Elders to remove a node from section membership
    pub(crate) async fn propose_remove_from_membership(&self, node_state: NodeState) -> Vec<Cmd> {
        self.broadcast_membership_proposal(Reconfig::Leave(node_state.to_msg()))
            .await
    }

    // Broadcast a section membership proposal to Elders
    async fn broadcast_membership_proposal(&self, reconfig: Reconfig<NodeStateMsg>) -> Vec<Cmd> {
        match *self.network_knowledge.membership_voting.write().await {
            None => {
                error!(">>> Failed to broadcast membership proposal since we don't hold a membership voting state");
                return vec![];
            }
            Some(ref mut state) => match state.propose(reconfig.clone()) {
                Ok(signed_vote) => {
                    trace!(">>> Membership proposal {:?}", reconfig);
                    self.broadcast_membership_vote_msg(signed_vote).await
                }
                Err(err) => {
                    error!(
                        ">>> Failed to generate membership proposal {:?}: {:?}",
                        reconfig, err
                    );
                    vec![]
                }
            },
        }
    }

    /// Broadcast BRB membership Vote message to Elders
    pub(crate) async fn broadcast_membership_vote_msg(
        &self,
        signed_vote: SignedVote<Reconfig<NodeStateMsg>>,
    ) -> Vec<Cmd> {
        // Deliver each SignedVote to all current Elders
        trace!(">>> Broadcasting Vote msg: {:?}", signed_vote);
        let node_msg = SystemMsg::Membership(signed_vote);
        match self.send_msg_to_our_elders(node_msg).await {
            Ok(cmd) => vec![cmd],
            Err(err) => {
                error!(
                    ">>> Failed to send SystemMsg::Membership message: {:?}",
                    err
                );
                vec![]
            }
        }
    }
}
