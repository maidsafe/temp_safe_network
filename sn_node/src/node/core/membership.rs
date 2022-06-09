use sn_consensus::{Ballot, Generation, SignedVote, Vote};
use sn_interface::messaging::system::NodeState;
use thiserror::Error;
use xor_name::Prefix;

use super::Node;
pub(crate) type Result<T> = std::result::Result<T, Error>;

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

impl Node {
    #[cfg(test)]
    pub async fn is_churn_in_progress(&self) -> bool {
        self.membership
            .read()
            .await
            .map(|m| !m.votes.is_empty())
            .unwrap_or(false)
    }
}

impl Membership {
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
        let _ = self.history.push(decision);
        self.consensus = Consensus::from(
            self.consensus.secret_key.clone(),
            self.consensus.elders.clone(),
            self.consensus.n_elders,
        );
    }
}
