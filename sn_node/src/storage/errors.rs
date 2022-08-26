// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_interface::{
    messaging::data::Error as ErrorMsg,
    types::{convert_dt_error_to_error_msg, PublicKey, ReplicatedDataAddress},
};

use std::{io, path::PathBuf};
use thiserror::Error;
use xor_name::XorName;

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
    /// Register not found in local storage.
    #[error("Register not found in local storage: {0}")]
    RegisterNotFound(PathBuf),
    /// Data not found.
    #[error("No such data: {0:?}")]
    NoSuchData(ReplicatedDataAddress),
    /// Chunk not found.
    #[error("Chunk not found: {0:?}")]
    ChunkNotFound(XorName),
    /// Data already exists for this node
    #[error("Data already exists at this node")]
    DataExists,
    /// Register op already exists for this node. Pass in the RegCmdId
    #[error("RegCmd Operation already exists at this node: {0}")]
    RegCmdOperationExists(String),
    /// Data owner provided is invalid.
    #[error("Provided PublicKey is not a valid owner. Provided PublicKey: {0}")]
    InvalidOwner(PublicKey),
    /// Invalid store found
    #[error("A store was loaded, but found to be invalid")]
    InvalidStore,
    /// Storage not supported for type of data address
    #[error("Storage not supported for type of data address: {0:?}")]
    UnsupportedDataType(XorName),
    /// Data owner provided is invalid.
    #[error("Provided PublicKey could not validate signature {0:?}")]
    InvalidSignature(PublicKey),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// Bincode error.
    #[error("Bincode error:: {0}")]
    Bincode(#[from] bincode::Error),
    /// Hex decoding error.
    #[error("Hex decoding error:: {0}")]
    HexDecoding(#[from] hex::FromHexError),
    /// Invalid filename
    #[error("Invalid chunk filename")]
    InvalidFilename,
    /// NetworkData error.
    #[error("Network data error:: {0}")]
    NetworkData(#[from] sn_interface::types::Error),
    /// Messaging error.
    #[error("Messaging error:: {0}")]
    Messaging(#[from] sn_interface::messaging::data::Error),
    /// No filename found
    #[error("Path contains no file name")]
    NoFilename,
}

/// Convert db error to messaging error message for sending over the network.
pub(crate) fn convert_to_error_msg(error: Error) -> ErrorMsg {
    match error {
        Error::NotEnoughSpace => ErrorMsg::FailedToWriteFile,
        Error::NoSuchData(address) => ErrorMsg::DataNotFound(address),
        Error::ChunkNotFound(xorname) => ErrorMsg::ChunkNotFound(xorname),
        Error::DataExists => ErrorMsg::DataExists,
        Error::NetworkData(error) => convert_dt_error_to_error_msg(error),
        other => ErrorMsg::InvalidOperation(format!("Failed to perform operation: {:?}", other)),
    }
}
