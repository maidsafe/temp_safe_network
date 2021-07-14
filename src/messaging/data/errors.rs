// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::DataAddress;
use crate::types::PublicKey;
use serde::{Deserialize, Serialize};
use std::result;
use thiserror::Error;

/// A specialised `Result` type.
pub type Result<T, E = Error> = result::Result<T, E>;

/// Errors that can occur when interactive with client messaging APIs.
#[derive(Error, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    /// Access denied for supplied PublicKey
    #[error("Access denied for PublicKey: {0}")]
    AccessDenied(PublicKey),
    /// Requested data not found
    #[error("Requested data not found: {0:?}")]
    DataNotFound(DataAddress),
    /// Failed to write file, likely due to a system Io error
    #[error("Failed to write file")]
    FailedToWriteFile,
    /// Provided data already exists on the network
    #[error("Data provided already exists")]
    DataExists,
    /// Entry could not be found on the data
    #[error("Requested entry not found")]
    NoSuchEntry,
    /// Key does not exist
    #[error("Key does not exist")]
    NoSuchKey,
    /// The list of owner keys is invalid
    #[error("Invalid owner key: {0}")]
    InvalidOwners(PublicKey),
    /// Invalid Operation such as a POST on ImmutableData
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    /// Node failed to delete the requested data for some reason.
    #[error("Failed to delete requested data")]
    FailedToDelete,
}
