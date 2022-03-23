// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{NodeState as NodeStateMsg, SystemMsg};
use crate::node::{api::cmds::Cmd, core::Node, network_knowledge::NodeState};
use sn_consensus::SignedVote;
use SapCandidates;

impl Node {
    /// Propose a consensus vote over a handover candidate SAP
    pub(crate) async fn propose_handover_sap(&self, sap_candidates: SapCandidates) -> Vec<Cmd> {
        let vote = self.handover.propose(sap_candidates);
        self.broadcast_handover_vote_msg(vote).await
    }

    /// Broadcast handover Vote message to Elders
    pub(crate) async fn broadcast_handover_vote_msg(
        &self,
        signed_vote: SignedVote<Vec<SectionAuthorityProvider>>,
    ) -> Vec<Cmd> {
        // Deliver each SignedVote to all current Elders
        trace!(">>> Broadcasting Vote msg: {:?}", signed_vote);
        let node_msg = SystemMsg::Handover(signed_vote);
        match self.send_msg_to_our_elders(node_msg).await {
            Ok(cmd) => vec![cmd],
            Err(err) => {
                error!(
                    ">>> Failed to send SystemMsg::Handover message: {:?}",
                    err
                );
                vec![]
            }
        }
    }
}
