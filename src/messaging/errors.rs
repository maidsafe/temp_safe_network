// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::result;
use thiserror::Error;

/// A specialised `Result` type for safecoin.
pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Error, Debug)]
#[non_exhaustive]
/// Error messages across the network
pub enum Error {
    /// Error message sent from a client
    #[error("Error received from a client")]
    Client(String),
    /// Serialisation error
    #[error("Failed to serialize message: {0}")]
    Serialisation(String),
    /// Failed to parse a string.
    #[error("Failed to parse: {0}")]
    FailedToParse(String),
    /// Message read was built with an unsupported version.
    #[error("Unsupported messaging protocol version: {0}")]
    UnsupportedVersion(u16),
    /// Message is of an unsupported kind.
    #[error("Unsupported messaging kind: {0}")]
    UnsupportedMessageKind(u8),
}
