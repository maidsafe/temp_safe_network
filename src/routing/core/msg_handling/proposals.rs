// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::node::{Proposal, SigShare};
use crate::routing::{
    core::AggregatorError, dkg::ProposalError, routing_api::command::Command, Error, Result,
};

// Decisions
impl Core {
    // Insert the proposal into the proposal aggregator and handle it if aggregated.
    pub(crate) fn handle_proposal(
        &mut self,
        proposal: Proposal,
        sig_share: SigShare,
    ) -> Result<Vec<Command>> {
        match self.proposal_aggregator.add(proposal, sig_share) {
            Ok((proposal, sig)) => Ok(vec![Command::HandleAgreement { proposal, sig }]),
            Err(ProposalError::Aggregation(AggregatorError::NotEnoughShares)) => Ok(vec![]),
            Err(error) => {
                error!("Failed to add proposal: {}", error);
                Err(Error::InvalidSignatureShare)
            }
        }
    }
}
