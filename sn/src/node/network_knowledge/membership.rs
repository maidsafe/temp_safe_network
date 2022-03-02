use std::collections::{BTreeMap, BTreeSet};

use bls_dkg::{PublicKeySet, SecretKeyShare};
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use xor_name::XorName;

use sn_membership::consensus::{Consensus, VoteResponse};
use sn_membership::vote::{Ballot, SignedVote, Vote};
use sn_membership::{Decision, Error, Fault, NodeId, Result};

use crate::messaging::system::MembershipState;

use super::NodeState;

type Generation = u64;

#[derive(Debug)]
struct HistoryEntry {
    votes: BTreeSet<SignedVote<NodeState>>,
    proposals: BTreeMap<NodeState, bls::Signature>,
    faults: BTreeSet<Fault<NodeState>>,
}

#[derive(Debug)]
pub(crate) struct Membership {
    consensus: Consensus<NodeState>,
    gen: Generation,
    pending_gen: Generation,
    capacity: usize,
    bootstrap_nodes: BTreeSet<NodeState>,
    history: BTreeMap<Generation, HistoryEntry>, // for onboarding new procs, the vote proving super majority
}

impl Membership {
    pub(crate) fn new(
        secret_key: (NodeId, SecretKeyShare),
        elders: PublicKeySet,
        n_elders: usize,
        section_capacity: usize,
        bootstrap_nodes: BTreeSet<NodeState>,
    ) -> Self {
        Membership {
            consensus: Consensus::from(secret_key, elders, n_elders),
            gen: 0,
            pending_gen: 0,
            capacity: section_capacity,
            bootstrap_nodes,
            history: BTreeMap::new(),
        }
    }

    pub(crate) fn members(&self, gen: Generation) -> Result<BTreeMap<XorName, &NodeState>> {
        let mut members =
            BTreeMap::from_iter(self.bootstrap_nodes.iter().map(|node| (node.name(), node)));

        for (history_gen, history_entry) in self.history.iter() {
            if history_gen > &gen {
                return Ok(members);
            }
            for (node, _sig) in history_entry.proposals.iter() {
                match node.state() {
                    MembershipState::Joined => {
                        let _ = members.insert(node.name(), node);
                    }
                    MembershipState::Left | MembershipState::Relocated(_) => {
                        let _ = members.remove(&node.name());
                    }
                }
            }
        }

        Err(Error::InvalidGeneration(gen))
    }

    pub(crate) fn propose(&mut self, node_state: NodeState) -> Result<SignedVote<NodeState>> {
        let vote = Vote {
            gen: self.gen + 1,
            ballot: Ballot::Propose(node_state),
            faults: self.consensus.faults(),
        };
        let signed_vote = self.sign_vote(vote)?;
        self.validate_signed_vote(&signed_vote)?;
        self.consensus
            .detect_byzantine_voters(&signed_vote)
            .map_err(|_| Error::AttemptedFaultyProposal)?;
        Ok(self.cast_vote(signed_vote))
    }

    pub(crate) fn anti_entropy(&self, from_gen: Generation) -> Result<Vec<SignedVote<NodeState>>> {
        info!("[MBR] anti-entropy from gen {}", from_gen);

        let mut msgs = self
            .history
            .iter() // history is a BTreeSet, .iter() is ordered by generation
            .filter(|(gen, _)| **gen > from_gen)
            .map(|(gen, history_entry)| {
                self.consensus
                    .build_super_majority_vote(history_entry.votes.clone(), *gen)
            })
            .collect::<Result<Vec<_>>>()?;

        // include the current in-progres votes as well.
        msgs.extend(self.consensus.votes.values().cloned());

        Ok(msgs)
    }

    pub(crate) fn id(&self) -> NodeId {
        self.consensus.id()
    }

    pub(crate) fn handle_signed_vote(
        &mut self,
        signed_vote: SignedVote<NodeState>,
    ) -> Result<VoteResponse<NodeState>> {
        self.validate_signed_vote(&signed_vote)?;
        self.log_signed_vote(&signed_vote);

        let vote_response = self
            .consensus
            .handle_signed_vote(signed_vote, self.pending_gen)?;

        match &vote_response {
            VoteResponse::Broadcast(vote) => {
                self.pending_gen = vote.vote.gen;
            }
            VoteResponse::Decided(Decision {
                votes,
                proposals,
                faults,
            }) => {
                let _ = self.history.insert(
                    self.pending_gen,
                    HistoryEntry {
                        votes: votes.clone(),
                        proposals: proposals.clone(),
                        faults: faults.clone(),
                    },
                );
                self.gen = self.pending_gen;
                // clear our pending votes
                self.consensus.votes = Default::default();
            }
            VoteResponse::WaitingForMoreVotes => {}
        }

        Ok(vote_response)
    }

    pub(crate) fn sign_vote(&self, vote: Vote<NodeState>) -> Result<SignedVote<NodeState>> {
        self.consensus.sign_vote(vote)
    }

    pub(crate) fn cast_vote(
        &mut self,
        signed_vote: SignedVote<NodeState>,
    ) -> SignedVote<NodeState> {
        self.log_signed_vote(&signed_vote);
        signed_vote
    }

    fn log_signed_vote(&mut self, signed_vote: &SignedVote<NodeState>) {
        self.pending_gen = signed_vote.vote.gen;
        self.consensus.log_signed_vote(signed_vote);
    }

    pub(crate) fn count_votes(
        &self,
        votes: &BTreeSet<SignedVote<NodeState>>,
    ) -> BTreeMap<BTreeSet<NodeState>, usize> {
        self.consensus.count_votes(votes)
    }

    pub(crate) fn validate_signed_vote(&self, signed_vote: &SignedVote<NodeState>) -> Result<()> {
        if signed_vote.vote.gen != self.gen + 1 {
            return Err(Error::VoteNotForNextGeneration {
                vote_gen: signed_vote.vote.gen,
                gen: self.gen,
                pending_gen: self.pending_gen,
            });
        }

        signed_vote
            .proposals()
            .into_iter()
            .try_for_each(|node_state| self.validate_node_state(node_state))?;

        self.consensus.validate_signed_vote(signed_vote)
    }

    fn validate_node_state(&self, node: NodeState) -> Result<()> {
        let members = self.members(self.gen)?;
        match node.state() {
            MembershipState::Joined => {
                if members.contains_key(&node.name()) {
                    Err(Error::JoinRequestForExistingMember)
                } else if members.len() >= self.capacity {
                    Err(Error::MembersAtCapacity)
                } else {
                    Ok(())
                }
            }
            MembershipState::Left | MembershipState::Relocated(_) => {
                if !members.contains_key(&node.name()) {
                    Err(Error::LeaveRequestForNonMember)
                } else {
                    Ok(())
                }
            }
        }
    }
}
