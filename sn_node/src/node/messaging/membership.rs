// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd, messaging::Peers, relocation::ChurnId, Event, MembershipEvent, Node,
    Result,
};

use bls::Signature;
use sn_consensus::{Decision, SignedVote};
use sn_interface::{
    messaging::system::{
        JoinResponse, KeyedSig, MembershipState, NodeState, SectionAuth, SystemMsg,
    },
    types::{log_markers::LogMarker, Peer},
};

use std::{collections::BTreeSet, vec};

// Message handling
impl Node {
    pub(crate) async fn propose_membership_change(&mut self, node_state: NodeState) -> Vec<Cmd> {
        info!(
            "Proposing membership change: {} - {:?}",
            node_state.name, node_state.state
        );
        let prefix = self.network_knowledge.prefix();
        match self.membership_propose(node_state, &prefix).await {
            Ok(cmds) => cmds,
            Err(e) => {
                warn!("{:?}: {e:?}", LogMarker::MembershipFailedToProposeChange);
                vec![]
            }
        }
    }

    /// Get our latest vote if any at this generation, and get cmds to resend to all elders
    /// (which should in turn trigger them to resend their votes)
    #[instrument(skip_all)]
    pub(crate) async fn membership_gossip_votes(&self) -> Option<Cmd> {
        trace!("{}", LogMarker::GossippingMembershipVotes);
        if let Ok(ae_votes) = self.membership_anti_entropy() {
            let cmd = self.send_msg_to_our_elders(SystemMsg::MembershipVotes(ae_votes));
            Some(cmd)
        } else {
            None
        }
    }

    pub(crate) async fn handle_membership_votes(
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

        for vote in signed_votes {
            match self.membership_handle_signed_vote(vote, &prefix).await {
                Ok(membership_cmds) => cmds.extend(membership_cmds),
                Err(e) => {
                    error!("Membership - error while processing vote {e:?}, dropping this and all votes in this batch thereafter");
                    break;
                }
            };
        }

        Ok(cmds)
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

        let _ = self
            .network_knowledge
            .handle_membership_decision(decision.clone())?;

        for (new_info, _signature) in joining_nodes.iter().cloned() {
            self.handle_node_joined(new_info).await;
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

        cmds.extend(self.promote_and_demote_elders_except(&BTreeSet::default())?);
        cmds.extend(self.send_ae_update_to_our_section(Some(decision.generation()?)));

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

    async fn handle_node_joined(&mut self, node: NodeState) {
        self.add_new_adult_to_trackers(node.name);

        info!("Node came online: {}", node.peer());

        // still used for testing
        self.send_event(Event::Membership(MembershipEvent::MemberJoined {
            name: node.name,
            previous_name: node.previous_name,
            age: node.age(),
        }))
        .await;
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

        let msg = SystemMsg::JoinResponse(Box::new(JoinResponse::Approved {
            genesis_key: *self.network_knowledge.genesis_key(),
            section_auth: self
                .network_knowledge
                .section_signed_authority_provider()
                .into_authed_msg(),
            section_chain: self.network_knowledge.our_section_dag(),
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
        let sig = KeyedSig {
            public_key: self.network_knowledge.section_key(),
            signature,
        };

        let node_state = SectionAuth {
            value: node_state,
            sig,
        }
        .into_authed_state();

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
            let msg = SystemMsg::Relocate(node_state.into_authed_msg());
            Some(self.send_system_msg(msg, Peers::Single(peer)))
        } else {
            None
        }
    }
}
