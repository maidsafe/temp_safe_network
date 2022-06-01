// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::collections::BTreeSet;
use std::vec;

use sn_consensus::{SignedVote, VoteResponse};
use sn_interface::messaging::system::{
    KeyedSig, MembershipState, NodeState, SectionAuth, SystemMsg,
};
use sn_interface::types::{log_markers::LogMarker, Peer};

use crate::node::api::cmds::Cmd;
use crate::node::core::{Node, Result};
use crate::node::membership::{self, VotingState};

// Message handling
impl Node {
    pub(crate) async fn propose_membership_change(
        &self,
        node_state: NodeState,
    ) -> Result<Vec<Cmd>> {
        info!(
            "Proposing membership change: {} - {:?}",
            node_state.name, node_state.state
        );
        let prefix = self.network_knowledge.prefix().await;
        if let Some(membership) = self.membership.write().await.as_mut() {
            let membership_vote = match membership.propose(node_state, &prefix) {
                Ok(vote) => vote,
                Err(e) => {
                    warn!("Membership - failed to propose change: {e:?}");
                    return Ok(vec![]);
                }
            };

            let cmds = self
                .send_msg_to_our_elders(SystemMsg::MembershipVote(membership_vote))
                .await?;
            Ok(vec![cmds])
        } else {
            error!("Membership - Failed to propose membership change, no membership instance");
            Ok(vec![])
        }
    }

    pub(crate) async fn handle_membership_vote(
        &self,
        peer: Peer,
        signed_vote: SignedVote<NodeState>,
    ) -> Result<Vec<Cmd>> {
        debug!(
            "{:?} {signed_vote:?} from {peer}",
            LogMarker::MembershipVotesBeingHandled
        );
        let prefix = self.network_knowledge.prefix().await;

        let mut cmds = vec![];

        let vote_response = if let Some(membership) = self.membership.write().await.as_mut() {
            let public_key = membership.voters_public_key_set().public_key();
            match membership.handle_signed_vote(signed_vote, &prefix) {
                Ok(VotingState::Voting(vote_response)) => vote_response,
                Ok(VotingState::Decided(decision, vote_response)) => {
                    // process the membership change
                    info!(
                        "Handling Membership Decision {:?}",
                        BTreeSet::from_iter(decision.proposals.keys())
                    );
                    for (state, signature) in &decision.proposals {
                        let sig = KeyedSig {
                            public_key,
                            signature: signature.clone(),
                        };
                        if state.state == MembershipState::Joined {
                            cmds.push(Cmd::HandleNewNodeOnline(SectionAuth {
                                value: state.clone(),
                                sig,
                            }));
                        } else {
                            cmds.push(Cmd::HandleNodeLeft(SectionAuth {
                                value: state.clone(),
                                sig,
                            }));
                        }
                    }

                    vote_response
                }
                Err(err) => match err {
                    membership::Error::WrongGeneration(peer_gen) => {
                        info!("Membership - Vote from wrong generation, sending AE");
                        cmds.push(Cmd::SendAntiEntropyToNodes {
                            recipients: vec![peer],
                            recipient_prefix: prefix,
                            recipient_public_key: public_key,
                            recipient_generation: peer_gen,
                        });
                        return Ok(cmds);
                    }
                    e => {
                        error!("Membership - error while processing vote {e:?}, dropping vote");
                        return Ok(cmds);
                    }
                },
            }
        } else {
            error!(
                "Attempted to handle membership vote when we don't yet have a membership instance"
            );
            return Ok(cmds);
        };

        match vote_response {
            VoteResponse::Broadcast(response_vote) => {
                cmds.push(
                    self.send_msg_to_our_elders(SystemMsg::MembershipVote(response_vote))
                        .await?,
                );
            }
            VoteResponse::WaitingForMoreVotes => (), //do nothing
        };

        Ok(cmds)
    }
}
