// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{KeyedSig, NodeState, SectionAuth, SystemMsg};
use crate::node::membership::{Generation, SignedVote, VoteResponse};
use crate::node::{api::cmds::Cmd, core::Node, Result};
use crate::types::Peer;

use std::collections::BTreeSet;
use std::vec;

// Message handling
impl Node {
    pub(crate) async fn handle_membership_vote(
        &self,
        peer: Peer,
        signed_vote: SignedVote<NodeState>,
    ) -> Result<Vec<Cmd>> {
        debug!("Received membership vote {:?} from {}", signed_vote, peer);

        let cmds = if let Some(membership) = self.membership.write().await.as_mut() {
            let mut cmds = match membership
                .handle_signed_vote(signed_vote, &self.network_knowledge.prefix().await)
            {
                Ok(VoteResponse::Broadcast(response_vote)) => {
                    vec![
                        self.send_msg_to_our_elders(SystemMsg::MembershipVote(response_vote))
                            .await?,
                    ]
                }
                Ok(VoteResponse::WaitingForMoreVotes) => vec![],
                Err(e) => {
                    error!("Membership - Error while processing vote {:?}", e);
                    // We hit an error while processing this vote, perhaps we are missing information.
                    // We'll send a membership AE request to see if they can help us catch up.
                    let sap = self.network_knowledge.authority_provider().await;
                    let dst_section_pk = sap.section_key();
                    let section_name = sap.prefix().name();
                    let msg = SystemMsg::MembershipAE(membership.generation());
                    let cmd = self
                        .send_direct_msg_to_nodes(vec![peer], msg, section_name, dst_section_pk)
                        .await?;
                    vec![cmd]
                }
            };

            // TODO: We should be able to detect when a *new* decision is made
            //       As it stands, we will reprocess each decision for any new vote
            //       we receive, it should be safe to do as `HandleNewNodeOnline`
            //       should be idempotent.
            if let Some(decision) = membership.most_recent_decision() {
                // process the membership change
                info!(
                    "Handling Membership Decision {:?}",
                    BTreeSet::from_iter(decision.proposals.keys())
                );
                for (state, signature) in &decision.proposals {
                    let sig = KeyedSig {
                        public_key: membership.voters_public_key_set().public_key(),
                        signature: signature.clone(),
                    };
                    cmds.push(Cmd::HandleNewNodeOnline(SectionAuth {
                        value: state.clone(),
                        sig,
                    }));
                }
            }

            cmds
        } else {
            error!(
                "Attempted to handle membership vote when we don't yet have a membership instance"
            );
            vec![]
        };

        Ok(cmds)
    }

    pub(crate) async fn handle_membership_anti_entropy(
        &self,
        peer: Peer,
        gen: Generation,
    ) -> Result<Vec<Cmd>> {
        debug!(
            "Received membership anti-entropy for gen {:?} from {}",
            gen, peer
        );

        let cmds = if let Some(membership) = self.membership.read().await.as_ref() {
            match membership.anti_entropy(gen) {
                Ok(catchup_votes) => {
                    let mut catchup_cmds = Vec::new();
                    for vote in catchup_votes {
                        catchup_cmds.push(
                            self.send_msg_to_our_elders(SystemMsg::MembershipVote(vote))
                                .await?,
                        );
                    }
                    catchup_cmds
                }
                Err(e) => {
                    error!("Membership - Error while processing anti-entropy {:?}", e);
                    vec![]
                }
            }
        } else {
            error!(
                "Attempted to handle membership anti-entropy when we don't yet have a membership instance"
            );
            vec![]
        };

        Ok(cmds)
    }
}
