use bls::{PublicKeySet, SecretKeyShare};
use core::fmt::Debug;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use tracing::info;

use sn_consensus::consensus::{Consensus, VoteResponse};
use sn_consensus::vote::{Ballot, SignedVote, Vote};
use sn_consensus::Decision;
use sn_consensus::Generation;
use sn_consensus::NodeId;

use super::errors::{Error, Result};

use sn_interface::network_knowledge::SapCandidate;

#[derive(Debug, Clone)]
pub(crate) struct Handover {
    pub(crate) consensus: Consensus<SapCandidate>,
    pub(crate) failed_consensus_rounds:
        BTreeMap<Generation, (Consensus<SapCandidate>, Decision<SapCandidate>)>,
    /// Handover gen starting at 0, and then +1 for each retries after failed consensus rounds
    pub(crate) gen: Generation,
}

impl Handover {
    pub(crate) fn from(
        secret_key: (NodeId, SecretKeyShare),
        elders: PublicKeySet,
        n_elders: usize,
    ) -> Self {
        Handover {
            consensus: Consensus::<SapCandidate>::from(secret_key, elders, n_elders),
            failed_consensus_rounds: BTreeMap::new(),
            gen: 0,
        }
    }

    pub(crate) fn propose(&mut self, proposal: SapCandidate) -> Result<SignedVote<SapCandidate>> {
        let vote = Vote {
            gen: self.gen,
            ballot: Ballot::Propose(proposal),
            faults: self.consensus.faults(),
        };
        let signed_vote = self.sign_vote(vote)?;
        signed_vote
            .detect_byzantine_faults(
                &self.consensus.elders,
                &self.consensus.votes,
                &self.consensus.processed_votes_cache,
            )
            .map_err(|_| Error::FaultyProposal)?;
        self.cast_vote(signed_vote)
    }

    // Get someone up to speed on our view of the current votes when receiving votes from an older gen
    pub(crate) fn anti_entropy(
        &self,
        from_gen: Generation,
    ) -> Result<Vec<SignedVote<SapCandidate>>> {
        let mut proof_votes = self
            .failed_consensus_rounds
            .iter()
            .filter(|(gen, _)| **gen >= from_gen)
            .map(|(gen, (consensus, decision))| {
                Ok(consensus.build_super_majority_vote(
                    decision.votes.clone(),
                    decision.faults.clone(),
                    *gen,
                )?)
            })
            .collect::<Result<Vec<_>>>()?;

        // include the current in-progres votes as well.
        proof_votes.extend(self.consensus.votes.values().cloned());

        info!(
            "Handover - anti-entropy from gen {}..{} id {:?}",
            from_gen,
            self.gen,
            self.id()
        );
        Ok(proof_votes)
    }

    pub(crate) fn handle_empty_set_decision(&mut self) {
        if let Some(decision) = &self.consensus.decision {
            if decision.proposals.is_empty() {
                let new_consensus = Consensus::<SapCandidate>::from(
                    self.consensus.secret_key.clone(),
                    self.consensus.elders.clone(),
                    self.consensus.n_elders,
                );
                let old_decision = decision.clone();
                let old_consensus = std::mem::replace(&mut self.consensus, new_consensus);
                let _none = self
                    .failed_consensus_rounds
                    .insert(self.gen, (old_consensus, old_decision));
                self.gen += 1;
                info!(
                    "Handover - noticed consensus on empty set, updading to gen {} id {:?}",
                    self.gen,
                    self.id()
                );
            }
        }
    }

    pub(crate) fn id(&self) -> NodeId {
        self.consensus.id()
    }

    pub(crate) fn generation(&self) -> Generation {
        self.gen
    }

    fn handle_outdated_signed_vote(
        &mut self,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Result<VoteResponse<SapCandidate>> {
        if let Some((consensus, _decision)) =
            self.failed_consensus_rounds.get_mut(&signed_vote.vote.gen)
        {
            Ok(consensus.handle_signed_vote(signed_vote)?)
        } else {
            Err(Error::CorruptedHandoverHistory(format!(
                "could not find history for gen {} when we're at {}",
                signed_vote.vote.gen, self.gen
            )))
        }
    }

    pub(crate) fn handle_signed_vote(
        &mut self,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Result<VoteResponse<SapCandidate>> {
        info!(
            "Handover - {:?} handling vote: {:?}",
            self.id(),
            signed_vote
        );

        match signed_vote.vote.gen.cmp(&self.gen) {
            Ordering::Less => self.handle_outdated_signed_vote(signed_vote),
            Ordering::Greater => Err(Error::RequestAntiEntropy),
            Ordering::Equal => Ok(self.consensus.handle_signed_vote(signed_vote)?),
        }
    }

    pub(crate) fn sign_vote(&self, vote: Vote<SapCandidate>) -> Result<SignedVote<SapCandidate>> {
        Ok(self.consensus.sign_vote(vote)?)
    }

    pub(crate) fn cast_vote(
        &mut self,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Result<SignedVote<SapCandidate>> {
        Ok(self.consensus.cast_vote(signed_vote)?)
    }

    pub(crate) fn consensus_value(&self) -> Option<SapCandidate> {
        if let Some(decision) = &self.consensus.decision {
            // deterministically choose a single sap_candidate
            // sn_consensus decides on a set, we deterministically pick the min as the handover winner
            decision.proposals.keys().min().map(|s| s.to_owned())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bls_dkg::blsttc::SecretKeySet;
    use rand::{prelude::StdRng, SeedableRng};
    use std::collections::BTreeSet;

    #[test]
    fn test_handle_empty_set_decision() {
        let mut rng = StdRng::from_seed([0u8; 32]);

        // init dummy section
        let elders_sk = SecretKeySet::random(7, &mut rng);
        let mut nodes_handover_state = Vec::from_iter((1..=7).into_iter().map(|id| {
            Handover::from(
                (id, elders_sk.secret_key_share(id as usize)),
                elders_sk.public_keys(),
                7,
            )
        }));

        // section agrees on empty set
        let _ = nodes_handover_state.iter_mut().map(|state| {
            state.consensus.decision = Some(Decision {
                votes: BTreeSet::new(),
                proposals: BTreeMap::new(),
                faults: BTreeSet::new(),
            });
        });

        // make sure gen was updated after empty set check
        let _ = nodes_handover_state.iter_mut().map(|state| {
            state.handle_empty_set_decision();
        });
        let _ = nodes_handover_state.iter().map(|state| {
            assert_eq!(state.gen, 1);
            assert_eq!(state.consensus.decision, None);
        });
    }
}
