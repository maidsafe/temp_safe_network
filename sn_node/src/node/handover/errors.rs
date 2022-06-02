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
    /// Proposal is faulty
    #[error("Faulty Proposal")]
    FaultyProposal,

    /// Handover generation is behind received vote, we need anti-entropy
    #[error("We are behind the voter, caller should request anti-entropy")]
    RequestAntiEntropy,

    /// Handover history is corrupted, there is a mistake in the code or the node is byzantine
    #[error("Corrupted Handover History: {0}")]
    CorruptedHandoverHistory(String),

    /// Consensus related errors and faults
    #[error("Consensus Error: {0}")]
    ConsensusFault(#[from] ConsensusError),
}
