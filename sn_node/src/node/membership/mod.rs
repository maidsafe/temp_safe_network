use std::{
    cmp::Ordering,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
};

use bls_dkg::{PublicKeySet, SecretKeyShare};
use core::fmt::Debug;
use sn_interface::{
    messaging::system::{MembershipState, NodeState},
    network_knowledge::SectionAuthorityProvider,
};
use thiserror::Error;
use xor_name::{Prefix, XorName};

use sn_consensus::{
    Ballot, Consensus, Decision, Generation, NodeId, SignedVote, Vote, VoteResponse,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Consensus error while processing vote {0}")]
    Consensus(#[from] sn_consensus::Error),
    #[error("Vote from wrong generation {0}")]
    WrongGeneration(Generation),
    #[error("Invalid proposal")]
    InvalidProposal,
    #[error("Network Knowledge error {0:?}")]
    NetworkKnowledge(#[from] sn_interface::network_knowledge::Error),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) enum VotingState {
    Voting(VoteResponse<NodeState>),
    Decided(Decision<NodeState>, VoteResponse<NodeState>),
}

/// Returns the nodes that should be candidates to become the next elders, sorted by names.
pub(crate) fn elder_candidates(
    candidates: impl IntoIterator<Item = NodeState>,
    current_elders: &SectionAuthorityProvider,
) -> BTreeSet<NodeState> {
    use itertools::Itertools;

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

#[derive(Debug, Clone)]
pub(crate) struct Membership {
    consensus: Consensus<NodeState>,
    bootstrap_members: BTreeSet<NodeState>,
    gen: Generation,
    history: BTreeMap<Generation, Decision<NodeState>>,
}

impl Membership {
    pub(crate) fn from(
        secret_key: (NodeId, SecretKeyShare),
        elders: PublicKeySet,
        n_elders: usize,
        bootstrap_members: BTreeSet<NodeState>,
    ) -> Self {
        Membership {
            consensus: Consensus::from(secret_key, elders, n_elders),
            bootstrap_members,
            gen: 0,
            history: BTreeMap::default(),
        }
    }

    pub(crate) fn generation(&self) -> Generation {
        self.gen
    }

    pub(crate) fn vote_generation(&self) -> Generation {
        self.gen + 1
    }

    pub(crate) fn voters_public_key_set(&self) -> &PublicKeySet {
        &self.consensus.elders
    }

    #[cfg(test)]
    pub(crate) fn is_churn_in_progress(&self) -> bool {
        !self.consensus.votes.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn force_bootstrap(&mut self, state: NodeState) {
        let _ = self.bootstrap_members.insert(state);
    }

    pub(crate) fn current_section_members(&self) -> BTreeMap<XorName, NodeState> {
        self.section_members(self.gen).unwrap_or_default()
    }

    pub(crate) fn section_members(&self, gen: Generation) -> Result<BTreeMap<XorName, NodeState>> {
        let mut members =
            BTreeMap::from_iter(self.bootstrap_members.iter().cloned().map(|n| (n.name, n)));

        if gen == 0 {
            return Ok(members);
        }

        for (history_gen, decision) in self.history.iter() {
            for (node_state, _sig) in decision.proposals.iter() {
                match node_state.state {
                    MembershipState::Joined => {
                        let _ = members.insert(node_state.name, node_state.clone());
                    }
                    MembershipState::Left => {
                        let _ = members.remove(&node_state.name);
                    }
                    MembershipState::Relocated(_) => {
                        if let Entry::Vacant(e) = members.entry(node_state.name) {
                            let _ = e.insert(node_state.clone());
                        } else {
                            let _ = members.remove(&node_state.name);
                        }
                    }
                }
            }

            if history_gen == &gen {
                return Ok(members);
            }
        }

        Err(Error::Consensus(sn_consensus::Error::InvalidGeneration(
            gen,
        )))
    }

    pub(crate) fn propose(
        &mut self,
        node_state: NodeState,
        prefix: &Prefix,
    ) -> Result<SignedVote<NodeState>> {
        info!("[{}] proposing {:?}", self.id(), node_state);
        let vote = Vote {
            gen: self.vote_generation(),
            ballot: Ballot::Propose(node_state),
            faults: self.consensus.faults(),
        };
        let signed_vote = self.sign_vote(vote)?;

        self.validate_proposals(&signed_vote, prefix)?;
        if let Err(e) = signed_vote.detect_byzantine_faults(
            &self.consensus.elders,
            &self.consensus.votes,
            &self.consensus.processed_votes_cache,
        ) {
            error!("Attempted invalid proposal: {e:?}");
            return Err(Error::InvalidProposal);
        }

        self.cast_vote(signed_vote)
    }

    pub(crate) fn anti_entropy(&self, from_gen: Generation) -> Vec<Decision<NodeState>> {
        info!("Membership - AntiEntropy for {from_gen:?}");
        Vec::from_iter(
            self
                .history
                .iter() // history is a BTreeSet, .iter() is ordered by generation
                .filter(move |(gen, _)| **gen >= from_gen)
                .map(|(_, decision)| decision)
                .cloned(),
        )
    }

    pub(crate) fn id(&self) -> NodeId {
        self.consensus.id()
    }

    pub(crate) fn handle_decision(&mut self, decision: Decision<NodeState>) -> Result<()> {
        let decision_gen = decision.generation()?;
        let our_gen = self.vote_generation();
        info!("Membership - handling decision from generation {decision_gen} (our generation: {our_gen})");

        if decision_gen != our_gen {
            return Err(Error::WrongGeneration(decision_gen));
        }

        decision.validate(&self.consensus.elders)?;
        self.terminate_consensus(decision);

        Ok(())
    }

    pub(crate) fn handle_signed_vote(
        &mut self,
        signed_vote: SignedVote<NodeState>,
        prefix: &Prefix,
    ) -> Result<VotingState> {
        self.validate_proposals(&signed_vote, prefix)?;

        let vote_gen = signed_vote.vote.gen;

        if vote_gen != self.vote_generation() {
            return Err(Error::WrongGeneration(vote_gen));
        }

        info!(
            "Membership - accepted signed vote from voter {:?}",
            signed_vote.voter
        );
        let vote_response = self.consensus.handle_signed_vote(signed_vote)?;

        if let Some(decision) = self.consensus.decision.clone() {
            info!(
                "Membership - decided {:?}",
                BTreeSet::from_iter(decision.proposals.keys())
            );

            self.terminate_consensus(decision.clone());

            Ok(VotingState::Decided(decision, vote_response))
        } else {
            Ok(VotingState::Voting(vote_response))
        }
    }

    fn sign_vote(&self, vote: Vote<NodeState>) -> Result<SignedVote<NodeState>> {
        Ok(self.consensus.sign_vote(vote)?)
    }

    pub(crate) fn cast_vote(
        &mut self,
        signed_vote: SignedVote<NodeState>,
    ) -> Result<SignedVote<NodeState>> {
        Ok(self.consensus.cast_vote(signed_vote)?)
    }

    /// Returns true if the proposal is valid
    fn validate_proposals(
        &self,
        signed_vote: &SignedVote<NodeState>,
        prefix: &Prefix,
    ) -> Result<()> {
        // check we're section the vote is for our current membership state
        signed_vote.validate_signature(&self.consensus.elders)?;

        // ensure we have a consensus instance for this votes generations
        if signed_vote.vote.gen != self.vote_generation() {
            return Err(Error::WrongGeneration(signed_vote.vote.gen));
        }

        let members =
            BTreeSet::from_iter(self.section_members(signed_vote.vote.gen - 1)?.into_keys());

        for proposal in signed_vote.proposals() {
            proposal.into_state().validate(prefix, &members)?;
        }

        Ok(())
    }

    fn terminate_consensus(&mut self, decision: Decision<NodeState>) {
        info!("Membership - terminating consensus {decision:#?}");
        assert_eq!(self.vote_generation(), decision.generation().unwrap());
        let _ = self.history.insert(self.vote_generation(), decision);
        self.gen = self.vote_generation();
        self.consensus = Consensus::from(
            self.consensus.secret_key.clone(),
            self.consensus.elders.clone(),
            self.consensus.n_elders,
        );
    }
}
