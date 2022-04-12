// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_consensus::{SignedVote, VoteResponse};

use crate::node::handover::{Handover, SapCandidate};
use crate::node::{api::cmds::Cmd, core::Node};
use crate::types::log_markers::LogMarker;

impl Node {
    /// helper to handle a handover vote
    #[instrument(skip(self), level = "trace")]
    async fn handle_vote(
        &self,
        handover_state: &mut Handover,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Vec<Cmd> {
        match handover_state.handle_signed_vote(signed_vote.clone()) {
            Ok(VoteResponse::Broadcast(signed_vote)) => {
                trace!(
                    ">>> Handover Vote msg successfully handled, broadcasting our vote: {:?}",
                    signed_vote
                );
                self.broadcast_handover_vote_msg(signed_vote).await
            }
            Ok(VoteResponse::WaitingForMoreVotes) => {
                trace!(
                    ">>> Handover Vote msg successfully handled, awaiting for more votes: {:?}",
                    signed_vote
                );
                vec![]
            }
            Err(err) => {
                error!(">>> Failed to handle handover Vote msg: {:?}", err);
                vec![]
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_handover_msg(
        &self,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Vec<Cmd> {
        debug!(
            ">>> {}: {:?}",
            LogMarker::HandoverMsgToBeHandled,
            signed_vote
        );

        let mut wlock = self.handover_voting.write().await;
        match &*wlock {
            Some(handover_state) => {
                let mut state = handover_state.clone();
                let mut cmds = self.handle_vote(&mut state, signed_vote).await;
                if let Some(candidates_sap) = state.consensus_value() {
                    debug!(
                        ">>> {}: {:?}",
                        LogMarker::HandoverConsensusTermination,
                        candidates_sap
                    );
                    // NB TOTO make sure error has to be swallowed
                    let bcast_cmds = self.broadcast_handover_decision(candidates_sap).await;
                    cmds.extend(bcast_cmds);
                }
                *wlock = Some(state);
                cmds
            }
            None => {
                trace!(">>> Non-elder node unexpectedly received handover Vote msg, ignoring...");
                vec![]
            }
        }
    }
}
