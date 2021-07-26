// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::io;
use thiserror::Error;

/// Specialisation of `std::Result` for dbs.
pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
#[non_exhaustive]
/// Node error variants.
pub enum Error {
    /// Not enough space in `DataStore` to perform `put`.
    #[error("Not enough space")]
    NotEnoughSpace,
    /// Key not found.
    #[error("Key not found")]
    KeyNotFound(String),
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialize(String),
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialize(String),
    /// Creating temp directory failed.
    #[error("Could not create temp store: {0}")]
    TempDirCreationFailed(String),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// Bincode error.
    #[error("Bincode error:: {0}")]
    Bincode(#[from] bincode::Error),
    ///Db key parse error.
    #[error("Could not parse key:: {0:?}")]
    CouldNotParseDbKey(Vec<u8>),
    /// Sled error.
    #[error("Sled error:: {0}")]
    Sled(#[from] sled::Error),
    /// NetworkData error.
    #[error("Network data error:: {0}")]
    NetworkData(#[from] crate::types::Error),
    /// Operation is invalid, eg signing validation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
