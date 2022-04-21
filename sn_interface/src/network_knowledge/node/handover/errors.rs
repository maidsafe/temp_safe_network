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

/// A specialised `Result` type for the handover consensus APIs.
pub(crate) type Result<T, E = Error> = result::Result<T, E>;

/// Errors that can occur in handover consensus
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

    /// Received invalid SAP candidates
    /// The candidates are supposed to be the oldest nodes in their generation
    #[error("Invalid SAP Candidates")]
    InvalidSapCandidates,

    /// Proposal is faulty
    #[error("Faulty Proposal")]
    FaultyProposal,

    /// Consensus related errors and faults
    #[error("Consensus Error: {0}")]
    ConsensusFault(#[from] ConsensusError),
}
