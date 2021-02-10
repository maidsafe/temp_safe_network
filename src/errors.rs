// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::client;
use std::result;
use thiserror::Error;

/// A specialised `Result` type for safecoin.
pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Client(#[from] client::Error),
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
