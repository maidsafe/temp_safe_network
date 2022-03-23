use std::collections::BTreeMap;

use bls::{PublicKeySet, SecretKeyShare, Signature};
use core::fmt::Debug;
use tracing::info;
use xor_name::Prefix;

use sn_consensus::consensus::{Consensus, VoteResponse};
use sn_consensus::vote::{Ballot, SignedVote, Vote};
use sn_consensus::{Error as ConsensusError, NodeId};

use crate::node::SectionAuthorityProvider;
use super::errors::{Error, Result};

pub type UniqueSectionId = u64;

pub enum SapCandidates {
    ElderHandover(SectionAuthorityProvider),
    SectionSplit(SectionAuthorityProvider, SectionAuthorityProvider),
}

#[derive(Debug)]
pub struct Handover {
    pub consensus: Consensus<SapCandidates>,
    pub gen: UniqueSectionId,
    pub section_prefix: Prefix,
}

impl Handover {
    pub fn from(
        secret_key: (NodeId, SecretKeyShare),
        elders: PublicKeySet,
        n_elders: usize,
        gen: UniqueSectionId,
        section_prefix: Prefix,
    ) -> Self {
        Handover {
            consensus: Consensus::<SapCandidates>::from(secret_key, elders, n_elders),
            gen,
            section_prefix,
        }
    }

    pub fn propose(&mut self, proposal: SapCandidates) -> Result<SignedVote<SapCandidates>> {
        let vote = Vote {
            gen: self.gen,
            ballot: Ballot::Propose(proposal),
            faults: self.consensus.faults(),
        };
        let signed_vote = self.sign_vote(vote)?;
        self.validate_proposals(&signed_vote)?;
        self.consensus
            .detect_byzantine_voters(&signed_vote)
            .map_err(|_| ConsensusError::AttemptedFaultyProposal)?;
        self.cast_vote(signed_vote)
    }

    // Get someone up to speed on our view of the current votes
    pub fn anti_entropy(&self) -> Result<Vec<SignedVote<SapCandidates>>> {
        info!("[HDVR] anti-entropy from {:?}", self.id());

        if let Some(decision) = self.consensus.decision.as_ref() {
            let vote = self.consensus.build_super_majority_vote(
                decision.votes.clone(),
                decision.faults.clone(),
                self.gen,
            )?;
            Ok(vec![vote])
        } else {
            Ok(self.consensus.votes.values().cloned().collect())
        }
    }

    pub fn resolve_votes<'a>(&self, proposals: &'a BTreeMap<SapCandidates, Signature>) -> Option<&'a SapCandidates> {
        // we need to choose one deterministically
        // proposals are comparable because they impl Ord so we arbitrarily pick the max
        proposals.keys().max()
    }

    pub fn id(&self) -> NodeId {
        self.consensus.id()
    }

    pub fn handle_signed_vote(&mut self, signed_vote: SignedVote<SapCandidates>) -> Result<VoteResponse<SapCandidates>> {
        self.validate_proposals(&signed_vote)?;

        self.consensus.handle_signed_vote(signed_vote)
    }

    pub fn sign_vote(&self, vote: Vote<SapCandidates>) -> Result<SignedVote<SapCandidates>> {
        self.consensus.sign_vote(vote)
    }

    pub fn cast_vote(&mut self, signed_vote: SignedVote<SapCandidates>) -> Result<SignedVote<SapCandidates>> {
        self.consensus.cast_vote(signed_vote)
    }

    pub fn consensus_value(&self) -> Option<SapCandidates> {
        if let Some(decision) = self.consensus.decision {
            // deterministically choose a single set of sap_candidates
            let sap_candidates = decision.proposals.keys().min();
            Some(sap_candidates)
        } else {
            None
        }
    }

    pub fn validate_proposals(&self, signed_vote: &SignedVote<SapCandidates>) -> Result<()> {
        if signed_vote.vote.gen != self.gen {
            return Err(ConsensusError::BadGeneration {
                requested_gen: signed_vote.vote.gen,
                gen: self.gen,
            });
        }

        signed_vote
            .proposals()
            .into_iter()
            .try_for_each(|prop| self.validate_proposal(prop))
    }

    pub fn validate_proposal(&self, proposal: SapCandidates) -> Result<()> {
        match proposal.as_slice() {
            // single handover, must be same prefix
            [single_sap] => {
                if single_sap.prefix() == self.section_prefix {
                    Ok(())
                } else {
                    Err(Error::InvalidSectionPrefixForCandidate)
                }
            },
            // section split, must be 2 distinct children prefixes
            [first_sap, second_sap] => {
                let our_p = &self.section_prefix;
                let p1 = first_sap.prefix();
                let p2 = second_sap.prefix();
                if p1.is_extension_of(our_p)
                && p2.is_extension_of(our_p)
                && p1.bit_count() == our_p.bit_count() + 1
                && p2.bit_count() == our_p.bit_count() + 1
                && p1 != p2 {
                    Ok(())
                } else {
                    Err(Error::InvalidSectionPrefixForSplitCandidates)
                }
            },
            // any other is invalid
            _ => Err(Error::InvalidAmountOfSectionCandidates(proposal.len())),
        }
    }
}
