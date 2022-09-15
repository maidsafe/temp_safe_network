// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.
use sn_consensus::{
    Ballot, Consensus, Decision, Fault, Generation, NodeId, SignedVote, Vote, VoteResponse,
};
use sn_interface::{
    messaging::system::{DkgSessionId, NodeState, SystemMsg},
    network_knowledge::{partition_by_prefix, recommended_section_size, SectionAuthorityProvider},
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;
use xor_name::{Prefix, XorName};

use super::{core::Node, flow_ctrl::cmds::Cmd, Error, Result};

fn get_split_info(
    prefix: Prefix,
    members: &BTreeMap<XorName, NodeState>,
) -> Option<(BTreeSet<NodeState>, BTreeSet<NodeState>)> {
    let (zero, one) = partition_by_prefix(&prefix, members.keys().copied())?;

    // make sure the sections contain enough entries
    let split_threshold = recommended_section_size();
    if zero.len() < split_threshold || one.len() < split_threshold {
        return None;
    }

    Some((
        BTreeSet::from_iter(zero.into_iter().map(|n| members[&n].clone())),
        BTreeSet::from_iter(one.into_iter().map(|n| members[&n].clone())),
    ))
}

/// Checks if we can split the section
/// If we have enough nodes for both subsections, returns the `DkgSessionId`'s
pub(crate) fn try_split_dkg(
    members: &BTreeMap<XorName, NodeState>,
    sap: &SectionAuthorityProvider,
    section_chain_len: u64,
    membership_gen: Generation,
) -> Option<(DkgSessionId, DkgSessionId)> {
    let prefix = sap.prefix();

    let (zero, one) = get_split_info(prefix, members)?;

    // get elders for section ...0
    let zero_prefix = prefix.pushed(false);
    let zero_elders = elder_candidates(zero.iter().cloned(), sap);

    // get elders for section ...1
    let one_prefix = prefix.pushed(true);
    let one_elders = elder_candidates(one.iter().cloned(), sap);

    // create the DKG session IDs
    let zero_id = DkgSessionId {
        prefix: zero_prefix,
        elders: BTreeMap::from_iter(zero_elders.iter().map(|node| (node.name, node.addr))),
        section_chain_len,
        bootstrap_members: zero,
        membership_gen,
    };
    let one_id = DkgSessionId {
        prefix: one_prefix,
        elders: BTreeMap::from_iter(one_elders.iter().map(|node| (node.name, node.addr))),
        section_chain_len,
        bootstrap_members: one,
        membership_gen,
    };

    Some((zero_id, one_id))
}

/// Returns the nodes that should be candidates to become the next elders, sorted by names.
pub(crate) fn elder_candidates(
    candidates: impl IntoIterator<Item = NodeState>,
    current_elders: &SectionAuthorityProvider,
) -> BTreeSet<NodeState> {
    use itertools::Itertools;
    use std::cmp::Ordering;

    // Compare candidates for the next elders. The one comparing `Less` wins.
    fn cmp_elder_candidates(
        lhs: &NodeState,
        rhs: &NodeState,
        current_elders: &SectionAuthorityProvider,
    ) -> Ordering {
        // Older nodes are preferred. In case of a tie, prefer current elders. If still a tie, break
        // it comparing by the signed signatures because it's impossible for a node to predict its
        // signature and therefore game its chances of promotion.
        rhs.age()
            .cmp(&lhs.age())
            .then_with(|| {
                let lhs_is_elder = current_elders.contains_elder(&lhs.name);
                let rhs_is_elder = current_elders.contains_elder(&rhs.name);

                match (lhs_is_elder, rhs_is_elder) {
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            })
            .then_with(|| lhs.name.cmp(&rhs.name))
        // TODO: replace name cmp above with sig cmp.
        // .then_with(|| lhs.sig.signature.cmp(&rhs.sig.signature))
    }

    candidates
        .into_iter()
        .sorted_by(|lhs, rhs| cmp_elder_candidates(lhs, rhs, current_elders))
        .take(sn_interface::elder_count())
        .collect()
}

impl Node {
    #[cfg(test)]
    pub(crate) fn is_churn_in_progress(&self) -> bool {
        self.membership
            .as_ref()
            .map(|m| !m.votes.is_empty())
            .unwrap_or(false)
    }

    fn membership(&self) -> Result<&Consensus<NodeState>> {
        self.membership.as_ref().ok_or(Error::InvalidState)
    }

    fn membership_mut(&mut self) -> Result<&mut Consensus<NodeState>> {
        self.membership.as_mut().ok_or(Error::InvalidState)
    }

    pub(crate) async fn membership_propose(
        &mut self,
        node_state: NodeState,
        prefix: &Prefix,
    ) -> Result<Vec<Cmd>> {
        info!("[{}] proposing {:?}", self.membership_id()?, node_state);
        let membership = self.membership()?;
        let signed_vote = membership.sign_vote(Vote {
            gen: self.network_knowledge.membership_gen() + 1,
            ballot: Ballot::Propose(node_state),
            faults: membership.faults(),
        })?;

        self.membership_validate_vote(&signed_vote, prefix)?;
        if let Err(faults) = signed_vote.detect_byzantine_faults(
            &membership.elders,
            &membership.votes,
            &membership.processed_votes_cache,
        ) {
            if let Some(Fault::ChangedVote { .. }) = faults.get(&self.membership_id()?) {
                // We've already voted.
                info!("Rejecting proposal since we've already voted this generation");
            } else {
                error!("Attempted invalid proposal: {faults:?}");
            }
            return Err(Error::InvalidMembershipProposal);
        }

        self.membership_cast_vote(signed_vote).await
    }

    pub(crate) fn membership_anti_entropy(&self) -> Result<Vec<SignedVote<NodeState>>> {
        let msgs = Vec::from_iter(self.membership()?.votes.values().cloned());

        info!("Membership - anti-entropy: {} msgs", msgs.len());

        Ok(msgs)
    }

    pub(crate) fn membership_id(&self) -> Result<NodeId> {
        Ok(self.membership()?.id())
    }

    pub(crate) async fn membership_handle_signed_vote(
        &mut self,
        signed_vote: SignedVote<NodeState>,
        prefix: &Prefix,
    ) -> Result<Vec<Cmd>> {
        self.membership_validate_vote(&signed_vote, prefix)?;

        if !self
            .membership()?
            .processed_votes_cache
            .contains(&signed_vote.sig)
        {
            info!(
                "Received new membership vote from {}, resetting membership timer",
                signed_vote.voter
            );
            self.membership_last_received_vote_time = Some(Instant::now());
        }

        let vote_response = self
            .membership_mut()?
            .handle_signed_vote(signed_vote.clone())?;

        info!(
            "Accepted membership vote from {:?} - {vote_response:?}",
            signed_vote.voter
        );

        self.membership_process_vote_response(vote_response).await
    }

    pub(crate) async fn membership_process_decision(
        &mut self,
        decision: Decision<NodeState>,
    ) -> Result<Vec<Cmd>> {
        let gen = decision.generation()?;
        let proposals = BTreeSet::from_iter(decision.proposals.keys());
        info!("Membership - decided gen={gen}, proposal={proposals:?}",);

        if let Ok(membership) = self.membership() {
            self.membership = Some(Consensus::from(
                membership.secret_key.clone(),
                membership.elders.clone(),
                membership.n_elders,
            ));
        }

        self.membership_last_received_vote_time = None;
        let cmds = self.handle_membership_decision(decision).await?;

        Ok(cmds)
    }

    async fn membership_process_vote_response(
        &mut self,
        vote_response: VoteResponse<NodeState>,
    ) -> Result<Vec<Cmd>> {
        let mut cmds = vec![];
        match vote_response {
            VoteResponse::WaitingForMoreVotes => (),
            VoteResponse::Broadcast(response_vote) => {
                cmds.push(
                    self.send_msg_to_our_elders(SystemMsg::MembershipVotes(vec![response_vote])),
                );
            }
        }

        if let Some(decision) = self.membership()?.decision.clone() {
            cmds.extend(self.membership_process_decision(decision).await?);
        };

        Ok(cmds)
    }

    async fn membership_cast_vote(
        &mut self,
        signed_vote: SignedVote<NodeState>,
    ) -> Result<Vec<Cmd>> {
        self.membership_last_received_vote_time = Some(Instant::now());
        let vote = self.membership_mut()?.cast_vote(signed_vote)?;
        self.membership_process_vote_response(VoteResponse::Broadcast(vote))
            .await
    }

    /// Returns true if the proposal is valid
    fn membership_validate_vote(
        &self,
        signed_vote: &SignedVote<NodeState>,
        prefix: &Prefix,
    ) -> Result<()> {
        // check we're section the vote is for our current membership state
        let membership = self.membership()?;
        signed_vote.validate_signature(&membership.elders)?;

        let current_gen = self.network_knowledge.membership_gen();
        // ensure we have a consensus instance for this votes generations
        if signed_vote.vote.gen != current_gen + 1 {
            info!(
                "Received membership vote for wrong generation, vote_gen:{} != our_gen:{} + 1",
                signed_vote.vote.gen, current_gen
            );
            return Err(Error::MembershipVoteForWrongGeneration);
        }

        let members = self
            .network_knowledge
            .section_members_upto_gen(signed_vote.vote.gen - 1);

        let archived_members = self.network_knowledge.archived_members();

        for proposal in signed_vote.proposals() {
            proposal
                .into_state()
                .validate(prefix, &members, &archived_members)?;
        }

        Ok(())
    }
}
