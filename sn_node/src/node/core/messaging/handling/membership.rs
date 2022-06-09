// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::collections::BTreeSet;
use std::vec;

use sn_consensus::{Ballot, SignedVote, Vote, VoteResponse};
use sn_interface::messaging::system::{
    KeyedSig, MembershipState, NodeState, SectionAuth, SystemMsg,
};
use sn_interface::types::{log_markers::LogMarker, Peer};

use crate::node::api::cmds::Cmd;
use crate::node::core::{Node, Result};
use crate::node::membership::{self, VotingState};
use crate::node::Error;

// Message handling
impl Node {
    /// Returns Ok(()) if the proposal is valid
    async fn validate_proposals(&self, signed_vote: &SignedVote<NodeState>) -> Result<()> {
        // signature validation serves two purposes:
        // 1. detecting fraudulent votes.
        // 2. ensuring this vote was meant for our current section key.
        signed_vote.validate_signature(&self.network_knowledge.elders_public_key_set().await)?;

        // Next, validate that we are in the correct membership generation to validate this vote.
        self.network_knowledge
            .verify_membership_vote_generation(signed_vote.vote.gen)
            .await?;

        // Finally, validate each proposal w.r.t. the network state.
        let prefix = self.network_knowledge.prefix().await;
        let member_names = BTreeSet::from_iter(
            self.network_knowledge
                .current_section_members()
                .await
                .into_keys(),
        );

        for proposal in signed_vote.proposals() {
            proposal.into_state().validate(&prefix, &member_names)?;
        }

        Ok(())
    }

    pub async fn propose_membership_change(&self, node_state: NodeState) -> Result<Vec<Cmd>> {
        info!("Proposing membership change {:?}", node_state);
        if let Some(membership) = self.membership.write().await.as_mut() {
            let signed_vote = membership.sign_vote(Vote {
                gen: self.network_knowledge.membership_vote_generation().await,
                ballot: Ballot::Propose(node_state),
                faults: membership.faults(),
            })?;

            self.validate_proposals(&signed_vote).await?;

            if let Err(e) = signed_vote.detect_byzantine_faults(
                &membership.elders,
                &membership.votes,
                &membership.processed_votes_cache,
            ) {
                error!("Attempted invalid proposal: {e:?}");
                return Err(Error::InvalidMembershipProposal);
            }

            let vote = membership.cast_vote(signed_vote)?;

            Ok(vec![
                self.send_msg_to_our_elders(SystemMsg::MembershipVote(vote))
                    .await?,
            ])
        } else {
            Err(Error::NotAnElder)
        }
    }

    pub async fn handle_signed_vote(
        &self,
        peer: Peer,
        signed_vote: SignedVote<NodeState>,
    ) -> Result<Vec<Cmd>> {
        self.validate_proposals(&signed_vote).await?;

        let vote_gen = signed_vote.vote.gen;

        info!(
            "Membership - accepted signed vote from voter {:?}",
            signed_vote.voter
        );

        let section_key = self.network_knowledge.section_key().await;

        if let Some(membership) = self.membership.write().await.as_mut() {
            assert_eq!(membership.elders.public_key(), section_key);

            let vote_response = match membership.handle_signed_vote(signed_vote) {
                Err(sn_interface::network_knowledge::Error::InvalidMembershipGeneration {
                    request_gen,
                    ..
                }) => {
                    info!("Membership - Vote from wrong generation, sending AE");
                    return Ok(vec![Cmd::SendAntiEntropyToNodes {
                        recipients: vec![peer],
                        recipient_prefix: self.network_knowledge.prefix().await,
                        recipient_public_key: section_key,
                        recipient_generation: request_gen,
                    }]);
                }
                resp => resp?,
            };

            let mut cmds = vec![];

            if let Some(decision) = membership.decision.clone() {
                info!(
                    "Membership - decided {:?}",
                    BTreeSet::from_iter(decision.proposals.keys())
                );

                self.terminate_consensus(decision.clone());

                info!(
                    "Handling Membership Decision {:?}",
                    BTreeSet::from_iter(decision.proposals.keys())
                );

                self.network_knowledge
                    .handle_membership_decision(decision)
                    .await?;
                for (state, signature) in &decision.proposals {
                    let sig = KeyedSig {
                        public_key: section_key,
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
            };

            match vote_response {
                VoteResponse::Broadcast(response_vote) => {
                    cmds.push(
                        self.send_msg_to_our_elders(SystemMsg::MembershipVote(response_vote))
                            .await?,
                    );
                }
                VoteResponse::WaitingForMoreVotes => (), // do nothing
            };

            Ok(cmds)
        } else {
            Err(Error::NotAnElder)
        }
    }
}
