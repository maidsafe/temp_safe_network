// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use tracing::warn;
use sn_consensus::SignedVote;

use crate::messaging::system::{NodeState as NodeStateMsg, SystemMsg};
use crate::node::{
    api::cmds::Cmd, core::Node, handover::SapCandidate, network_knowledge::NodeState, Result,
};

impl Node {
    /// Make a handover consensus proposal vote for a sap candidate
    pub(crate) async fn propose_handover_consensus(&self, sap_candidates: SapCandidate) -> Result<Vec<Cmd>> {
        let mut wlock = self.handover_voting.write().await;
        match &*wlock {
            Some(handover_voting_state) => {
                let mut vs = handover_voting_state.clone();
                let vote = vs.propose(sap_candidates)?;
                *wlock = Some(vs);
                Ok(self.broadcast_handover_vote_msg(vote).await)
            },
            None => {
                warn!("Failed to make handover consensus proposal because node is not an Elder");
                Ok(vec![])
            }
        }
    }

    /// Broadcast handover Vote message to Elders
    pub(crate) async fn broadcast_handover_vote_msg(
        &self,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Vec<Cmd> {
        // Deliver each SignedVote to all current Elders
        trace!(">>> Broadcasting Vote msg: {:?}", signed_vote);
        let node_msg = SystemMsg::HandoverVote(signed_vote);
        match self.send_msg_to_our_elders(node_msg).await {
            Ok(cmd) => vec![cmd],
            Err(err) => {
                error!(">>> Failed to send SystemMsg::Handover message: {:?}", err);
                vec![]
            }
        }
    }
}
