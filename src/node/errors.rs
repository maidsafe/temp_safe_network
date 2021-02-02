// Copyright 2021MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.
use std::{
    fmt::{self, Debug, Formatter},
    result,
};
use thiserror::Error;

/// A specialised `Result` type.
pub type Result<T, E = Error> = result::Result<T, E>;

/// Error debug struct
pub struct ErrorDebug<'a, T>(pub &'a Result<T>);

impl<'a, T> Debug for ErrorDebug<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Err(error) = self.0 {
            write!(f, "{:?}", error)
        } else {
            write!(f, "Success")
        }
    }
}

/// Main error type for the crate.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed signature check.")]
    FailedSignature,
    #[error("Cannot route.")]
    CannotRoute,
    #[error("Network layer error: {}", .0)]
    Network(String),
    #[error("The node is not in a state to handle the action.")]
    InvalidState,
    #[error("Bincode error: {}", .0)]
    Bincode(#[from] bincode::Error),
    #[error("Invalid source location.")]
    InvalidSrcLocation,
    #[error("Invalid destination location.")]
    InvalidDstLocation,
    #[error("Content of a received message is inconsistent.")]
    InvalidMessage,
    #[error("A signature share is invalid.")]
    InvalidSignatureShare,
    #[error("The secret key share is missing.")]
    MissingSecretKeyShare,
    #[error("Failed to send a message.")]
    FailedSend,
    #[error("Invalid vote.")]
    InvalidVote,
}
