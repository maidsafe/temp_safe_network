use bls::{PublicKeySet, SecretKeyShare};
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use tracing::info;
use xor_name::Prefix;

use sn_consensus::consensus::{Consensus, VoteResponse};
use sn_consensus::vote::{Ballot, SignedVote, Vote};
use sn_consensus::NodeId;

use super::errors::{Error, Result};
use crate::messaging::system::SectionAuth;
use crate::node::SectionAuthorityProvider;

#[allow(clippy::large_enum_variant)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub enum SapCandidate {
    ElderHandover(SectionAuth<SectionAuthorityProvider>),
    SectionSplit(
        SectionAuth<SectionAuthorityProvider>,
        SectionAuth<SectionAuthorityProvider>,
    ),
}

#[derive(Debug, Clone)]
pub(crate) struct Handover {
    pub(crate) consensus: Consensus<SapCandidate>,
    pub(crate) section_prefix: Prefix,
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
        }
    }

    pub(crate) fn propose(&mut self, proposal: SapCandidate) -> Result<SignedVote<SapCandidate>> {
        let vote = Vote {
            gen: 0,
            ballot: Ballot::Propose(proposal),
            faults: self.consensus.faults(),
        };
        let signed_vote = self.sign_vote(vote)?;
        self.validate_proposals(&signed_vote)?;
        self.consensus
            .detect_byzantine_voters(&signed_vote)
            .map_err(|_| Error::FaultyProposal)?;
        self.cast_vote(signed_vote)
    }

    // NB TODO do we need anti-entropy for handover? How do we trigger it?
    // // Get someone up to speed on our view of the current votes
    // pub(crate) fn anti_entropy(&self) -> Result<Vec<SignedVote<SapCandidate>>> {
    //     info!("[HDVR] anti-entropy from {:?}", self.id());
    //
    //     if let Some(decision) = self.consensus.decision.as_ref() {
    //         let vote = self.consensus.build_super_majority_vote(
    //             decision.votes.clone(),
    //             decision.faults.clone(),
    //             0,
    //         )?;
    //         Ok(vec![vote])
    //     } else {
    //         Ok(self.consensus.votes.values().cloned().collect())
    //     }
    // }

    pub(crate) fn id(&self) -> NodeId {
        self.consensus.id()
    }

    pub(crate) fn handle_signed_vote(
        &mut self,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Result<VoteResponse<SapCandidate>> {
        info!("[HDVR] {:?} handling vote: {:?}", self.id(), signed_vote);
        self.validate_proposals(&signed_vote)?;

        Ok(self.consensus.handle_signed_vote(signed_vote)?)
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
