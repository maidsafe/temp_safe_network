// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::data::Error as ErrorMessage;
use crate::types::{convert_dt_error_to_error_message, DataAddress, PublicKey};
use std::io;
use thiserror::Error;

/// Specialisation of `std::Result` for dbs.
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
#[non_exhaustive]
/// Node error variants.
pub enum Error {
    /// Not enough space to store the value.
    #[error("Not enough space")]
    NotEnoughSpace,
    /// Key not found.
    #[error("Key not found: {0:?}")]
    KeyNotFound(String),
    /// Key, Value pair not found.
    #[error("No value found for key: {0:?}")]
    NoSuchValue(String),
    /// Data id not found.
    #[error("Data id not found: {0:?}")]
    DataIdNotFound(DataAddress),
    /// Data not found.
    #[error("No such data: {0:?}")]
    NoSuchData(DataAddress),
    /// Chunk already exists for this node
    #[error("Data already exists at this node")]
    DataExists,
    /// Data owner provided is invalid.
    #[error("Provided PublicKey is not a valid owner. Provided PublicKey: {0}")]
    InvalidOwner(PublicKey),
    /// Logic error.
    #[error("Logic error: {0}")]
    Logic(String),
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

/// Convert db error to messaging error message for sending over the network.
pub(crate) fn convert_to_error_message(error: Error) -> ErrorMessage {
    match error {
        Error::NotEnoughSpace => ErrorMessage::FailedToWriteFile,
        Error::DataIdNotFound(address) => ErrorMessage::DataNotFound(address),
        Error::NoSuchData(address) => ErrorMessage::DataNotFound(address),
        Error::TempDirCreationFailed(_) => ErrorMessage::FailedToWriteFile,
        Error::DataExists => ErrorMessage::DataExists,
        Error::NetworkData(error) => convert_dt_error_to_error_message(error),
        other => {
            ErrorMessage::InvalidOperation(format!("Failed to perform operation: {:?}", other))
        }
    }
}
