// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd, membership, messaging::Peers, relocation::ChurnId, Event,
    MembershipEvent, MyNode, Result,
};

use bls::Signature;
use sn_consensus::{Decision, Generation, SignedVote, VoteResponse};
use sn_interface::{
    messaging::{
        system::{JoinResponse, MembershipState, NodeMsg, NodeState, SectionSig, SectionSigned},
        SectionTreeUpdate,
    },
    types::{log_markers::LogMarker, Peer},
};

use std::{collections::BTreeSet, vec};

// Message handling
impl MyNode {
    pub(crate) fn propose_membership_change(&mut self, node_state: NodeState) -> Option<Cmd> {
        info!(
            "Proposing membership change: {} - {:?}",
            node_state.name, node_state.state
        );
        let prefix = self.network_knowledge.prefix();
        if let Some(membership) = self.membership.as_mut() {
            let membership_vote = match membership.propose(node_state, &prefix) {
                Ok(vote) => vote,
                Err(e) => {
                    warn!("Membership - failed to propose change: {e:?}");
                    return None;
                }
            };
            Some(self.send_msg_to_our_elders(NodeMsg::MembershipVotes(vec![membership_vote])))
        } else {
            error!("Membership - Failed to propose membership change, no membership instance");
            None
        }
    }

    /// Get our latest vote if any at this generation, and get cmds to resend to all elders
    /// (which should in turn trigger them to resend their votes)
    #[instrument(skip_all)]
    pub(crate) async fn membership_gossip_votes(&self) -> Option<Cmd> {
        let membership = self.membership.clone();

        if let Some(membership) = membership {
            trace!("{}", LogMarker::GossippingMembershipVotes);
            if let Ok(ae_votes) = membership.anti_entropy(membership.generation()) {
                let cmd = self.send_msg_to_our_elders(NodeMsg::MembershipVotes(ae_votes));
                return Some(cmd);
            }
        }

        None
    }

    pub(crate) fn handle_membership_votes(
        &mut self,
        peer: Peer,
        signed_votes: Vec<SignedVote<NodeState>>,
    ) -> Result<Vec<Cmd>> {
        debug!(
            "{:?} {signed_votes:?} from {peer}",
            LogMarker::MembershipVotesBeingHandled
        );
        let prefix = self.network_knowledge.prefix();

        let mut cmds = vec![];

        for signed_vote in signed_votes {
            let mut vote_broadcast = None;
            if let Some(membership) = self.membership.as_mut() {
                let (vote_response, decision) = match membership
                    .handle_signed_vote(signed_vote, &prefix)
                {
                    Ok(result) => result,
                    Err(membership::Error::RequestAntiEntropy) => {
                        debug!("Membership - We are behind the voter, requesting AE");
                        // We hit an error while processing this vote, perhaps we are missing information.
                        // We'll send a membership AE request to see if they can help us catch up.
                        debug!("{:?}", LogMarker::MembershipSendingAeUpdateRequest);
                        let msg = NodeMsg::MembershipAE(membership.generation());
                        cmds.push(self.send_system_msg(msg, Peers::Single(peer)));
                        // return the vec w/ the AE cmd there so as not to loop and generate AE for
                        // any subsequent commands
                        return Ok(cmds);
                    }
                    Err(e) => {
                        error!("Membership - error while processing vote {e:?}, dropping this and all votes in this batch thereafter");
                        break;
                    }
                };

                match vote_response {
                    VoteResponse::Broadcast(response_vote) => {
                        vote_broadcast = Some(NodeMsg::MembershipVotes(vec![response_vote]));
                    }
                    VoteResponse::WaitingForMoreVotes => {
                        // do nothing
                    }
                };

                if let Some(decision) = decision {
                    cmds.push(Cmd::HandleMembershipDecision(decision));
                }
            } else {
                error!(
                    "Attempted to handle membership vote when we don't yet have a membership instance"
                );
            };

            if let Some(vote_msg) = vote_broadcast {
                cmds.push(self.send_msg_to_our_elders(vote_msg));
            }
        }

        Ok(cmds)
    }

    pub(crate) fn handle_membership_anti_entropy(
        &self,
        peer: Peer,
        gen: Generation,
    ) -> Option<Cmd> {
        debug!(
            "{:?} membership anti-entropy request for gen {:?} from {}",
            LogMarker::MembershipAeRequestReceived,
            gen,
            peer
        );

        if let Some(membership) = self.membership.as_ref() {
            match membership.anti_entropy(gen) {
                Ok(catchup_votes) => {
                    Some(self.send_system_msg(
                        NodeMsg::MembershipVotes(catchup_votes),
                        Peers::Single(peer),
                    ))
                }
                Err(e) => {
                    error!("Membership - Error while processing anti-entropy {:?}", e);
                    None
                }
            }
        } else {
            error!(
                "Attempted to handle membership anti-entropy when we don't yet have a membership instance"
            );
            None
        }
    }

    pub(crate) async fn handle_membership_decision(
        &mut self,
        decision: Decision<NodeState>,
    ) -> Result<Vec<Cmd>> {
        debug!("{}", LogMarker::AgreementOfOnline);
        let mut cmds = vec![];

        let (joining_nodes, leaving_nodes): (Vec<_>, Vec<_>) = decision
            .proposals
            .clone()
            .into_iter()
            .partition(|(n, _)| n.state == MembershipState::Joined);

        info!(
            "Handling membership decision: joining = {:?}, leaving = {:?}",
            Vec::from_iter(joining_nodes.iter().map(|(n, _)| n.name)),
            Vec::from_iter(leaving_nodes.iter().map(|(n, _)| n.name))
        );

        for (new_info, signature) in joining_nodes.iter().cloned() {
            cmds.extend(self.handle_node_joined(new_info, signature).await);
        }

        for (new_info, signature) in leaving_nodes.iter().cloned() {
            cmds.extend(self.handle_node_left(new_info, signature).into_iter());
        }

        cmds.push(self.send_node_approvals(decision.clone()));

        // Do not disable node joins in first section.
        let our_prefix = self.network_knowledge.prefix();
        if !our_prefix.is_empty() {
            // ..otherwise, switch off joins_allowed on a node joining.
            // TODO: fix racing issues here? https://github.com/maidsafe/safe_network/issues/890
            self.joins_allowed = false;
        }

        if let Some((_, sig)) = decision.proposals.iter().max_by_key(|(_, sig)| *sig) {
            let churn_id = ChurnId(sig.to_bytes());
            let excluded_from_relocation =
                BTreeSet::from_iter(joining_nodes.iter().map(|(n, _)| n.name));

            cmds.extend(self.relocate_peers(churn_id, excluded_from_relocation)?);
        }

        cmds.extend(self.trigger_dkg()?);
        cmds.extend(self.send_ae_update_to_our_section()?);

        self.liveness_retain_only(
            self.network_knowledge
                .adults()
                .iter()
                .map(|peer| peer.name())
                .collect(),
        )?;

        if !leaving_nodes.is_empty() {
            self.joins_allowed = true;
        }

        self.log_section_stats();
        self.log_network_stats();

        Ok(cmds)
    }

    async fn handle_node_joined(&mut self, new_info: NodeState, signature: Signature) -> Vec<Cmd> {
        let sig = SectionSig {
            public_key: self.network_knowledge.section_key(),
            signature,
        };

        let new_info = SectionSigned {
            value: new_info.into_state(),
            sig,
        };

        if !self.network_knowledge.update_member(new_info.clone()) {
            info!("ignore Online: {}", new_info.peer());
            return vec![];
        }

        self.add_new_adult_to_trackers(new_info.name());

        info!("handle Online: {}", new_info.peer());

        // still used for testing
        self.send_event(Event::Membership(MembershipEvent::MemberJoined {
            name: new_info.name(),
            previous_name: new_info.previous_name(),
            age: new_info.age(),
        }))
        .await;

        vec![]
    }

    // Send `NodeApproval` to a joining node which makes it a section member
    pub(crate) fn send_node_approvals(&self, decision: Decision<NodeState>) -> Cmd {
        let peers: BTreeSet<_> = decision
            .proposals
            .keys()
            .filter(|n| n.state == MembershipState::Joined)
            .map(|n| n.peer())
            .collect();
        let prefix = self.network_knowledge.prefix();
        info!("Section {prefix:?} has approved new peers {peers:?}.");

        let msg = NodeMsg::JoinResponse(Box::new(JoinResponse::Approved {
            section_tree_update: SectionTreeUpdate::new(
                self.network_knowledge.signed_sap(),
                self.network_knowledge.section_chain(),
            ),
            decision,
        }));

        trace!("{}", LogMarker::SendNodeApproval);
        self.send_system_msg(msg, Peers::Multiple(peers))
    }

    pub(crate) fn handle_node_left(
        &mut self,
        node_state: NodeState,
        signature: Signature,
    ) -> Option<Cmd> {
        let sig = SectionSig {
            public_key: self.network_knowledge.section_key(),
            signature,
        };

        let node_state = SectionSigned {
            value: node_state,
            sig,
        }
        .into_signed_state();

        let _ = self.network_knowledge.update_member(node_state.clone());

        info!(
            "{}: {}",
            LogMarker::AcceptedNodeAsOffline,
            node_state.peer()
        );

        // If this is an Offline agreement where the new node state is Relocated,
        // we then need to send the Relocate msg to the peer attaching the signed NodeState
        // containing the relocation details.
        if node_state.is_relocated() {
            let peer = *node_state.peer();
            let msg = NodeMsg::Relocate(node_state.into_signed_msg());
            Some(self.send_system_msg(msg, Peers::Single(peer)))
        } else {
            None
        }
    }
}
