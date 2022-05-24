use bls::{PublicKeySet, SecretKeyShare};
use core::fmt::Debug;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use tracing::info;
use xor_name::Prefix;

use sn_consensus::consensus::{Consensus, VoteResponse};
use sn_consensus::vote::{Ballot, SignedVote, Vote};
use sn_consensus::Generation;
use sn_consensus::NodeId;

use super::errors::{Error, Result};

use sn_interface::messaging::system::SectionAuth;
use sn_interface::network_knowledge::SapCandidate;
use sn_interface::network_knowledge::SectionAuthorityProvider;

#[derive(Debug, Clone)]
pub(crate) struct Handover {
    pub(crate) consensus: Consensus<SapCandidate>,
    pub(crate) section_prefix: Prefix,
    pub(crate) failed_consensus_rounds: BTreeMap<Generation, Consensus<SapCandidate>>,
    /// Handover gen starting at 0, and then +1 for each retries after failed consensus rounds
    pub(crate) gen: Generation,
}

impl Handover {
    pub(crate) fn from(
        secret_key: (NodeId, SecretKeyShare),
        elders: PublicKeySet,
        n_elders: usize,
        section_prefix: Prefix,
    ) -> Self {
        Handover {
            consensus: Consensus::<SapCandidate>::from(secret_key, elders, n_elders),
            section_prefix,
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
        self.validate_proposals(&signed_vote)?;
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
            .map(|(gen, consensus)| {
                if let Some(decision) = consensus.decision.clone() {
                    Ok(consensus.build_super_majority_vote(
                        decision.votes.clone(),
                        decision.faults,
                        *gen,
                    )?)
                } else {
                    Err(Error::CorruptedHandoverHistory(format!(
                        "missing decision for handover history at generation {}",
                        gen
                    )))
                }
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
                let old_consensus = std::mem::replace(&mut self.consensus, new_consensus);
                let _none = self.failed_consensus_rounds.insert(self.gen, old_consensus);
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
        if let Some(consensus) = self.failed_consensus_rounds.get_mut(&signed_vote.vote.gen) {
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
        self.validate_proposals(&signed_vote)?;

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

    pub(crate) fn validate_proposals(&self, signed_vote: &SignedVote<SapCandidate>) -> Result<()> {
        signed_vote
            .proposals()
            .into_iter()
            .try_for_each(|prop| self.validate_proposal(prop))
    }

    pub(crate) fn check_candidates_validity(
        &self,
        _sap: &SectionAuth<SectionAuthorityProvider>,
    ) -> Result<()> {
        // check that the candidates are the oldest in their membership gen
        // NB TODO check that the sap is valid (either latest candidates or in recent history)
        if true {
            Ok(())
        } else {
            Err(Error::InvalidSapCandidates)
        }
    }

    pub(crate) fn validate_proposal(&self, proposal: SapCandidate) -> Result<()> {
        match proposal {
            SapCandidate::ElderHandover(single_sap) => {
                self.check_candidates_validity(&single_sap)?;
                // single handover, must be same prefix
                if single_sap.prefix() == self.section_prefix {
                    Ok(())
                } else {
                    Err(Error::InvalidSectionPrefixForCandidate)
                }
            }
            SapCandidate::SectionSplit(sap1, sap2) => {
                self.check_candidates_validity(&sap1)?;
                self.check_candidates_validity(&sap2)?;
                // section split, must be 2 distinct children prefixes
                let our_p = &self.section_prefix;
                let p1 = sap1.prefix();
                let p2 = sap2.prefix();
                if p1.is_extension_of(our_p)
                    && p2.is_extension_of(our_p)
                    && p1.bit_count() == our_p.bit_count() + 1
                    && p2.bit_count() == our_p.bit_count() + 1
                    && p1 != p2
                {
                    Ok(())
                } else {
                    Err(Error::InvalidSectionPrefixForSplitCandidates)
                }
            }
        }
    }
}
