// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::data::Error as ErrorMessage;
use crate::types::convert_dt_error_to_error_message;
use std::io;
use thiserror::Error;
use xor_name::XorName;

/// Specialisation of `std::Result` for dbs.
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
#[non_exhaustive]
/// Chunk Store error variants.
pub enum Error {
    /// Not enough space to store the Chunk.
    #[error("Not enough space")]
    NotEnoughSpace,
    /// Chunk not found.
    #[error("Chunk not found: {0:?}")]
    ChunkNotFound(XorName),
    /// Invalid filename
    #[error("Invalid chunk filename")]
    InvalidFilename,
    /// No filename found
    #[error("Path contains no file name")]
    NoFilename,
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// NetworkData error.
    #[error("Network data error:: {0}")]
    NetworkData(#[from] crate::types::Error),
}

/// Convert chunk errors to messaging error message for sending over the network.
pub(crate) fn convert_to_error_message(error: Error) -> ErrorMessage {
    match error {
        Error::NotEnoughSpace => ErrorMessage::FailedToWriteFile,
        Error::ChunkNotFound(xorname) => ErrorMessage::ChunkNotFound(xorname),
        Error::NetworkData(error) => convert_dt_error_to_error_message(error),
        other => {
            ErrorMessage::InvalidOperation(format!("Failed to perform operation: {:?}", other))
        }
    }
}
