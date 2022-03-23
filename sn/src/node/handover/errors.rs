// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::result;
use thiserror::Error;

use sn_consensus::Error as ConsensusError;

/// A specialised `Result` type for Network Knowledge's handover consensus APIs.
pub type Result<T, E = Error> = result::Result<T, E>;

/// Errors that can occur when interacting with Network Knowledge's handover consensus
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Received an invalid section prefix when checking handover candidate's section prefix
    /// The candidate's section prefix is supposed to be identical to ours for a handover
    #[error("Invalid Section Prefix For Handover Candidate")]
    InvalidSectionPrefixForCandidate,

    /// Received invalid split sections candidates
    /// The candidates are supposed to be 2 distinct children prefixes for our section:
    /// Example: for section 10, the candidates are supposed to be 101, 100
    #[error("Invalid Section Prefix For Handover Split Candidate")]
    InvalidSectionPrefixForSplitCandidates,

    /// Received an invalid amount of candidates, only
    /// - 1 for section handover
    /// - 2 for section split handover
    /// are valid
    #[error("Unexpectedly got {0} candidates sections, where only 1 (handover) or 2 (split handover) are valid values")]
    InvalidAmountOfSectionCandidates(usize),

    /// Consensus related errors and faults
    #[error("Consensus Error: {0}")]
    ConsensusError(#[from] ConsensusError),
}
