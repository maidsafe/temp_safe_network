// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    register::{EntryHash, User},
    RegisterAddress,
};

use crate::messaging::data::Error as ErrorMsg;

use bls::Error as BlsError;
use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
    result,
};
use thiserror::Error;

/// A specialised `Result` type for types crate.
pub type Result<T> = result::Result<T, Error>;

/// Error debug struct
struct ErrorDebug<'a, T>(&'a Result<T>);

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
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    /// Access denied for user
    #[error("Access denied for user: {0:?}")]
    AccessDenied(User),
    /// Serialization error
    #[error("Serialisation error: {0}")]
    Serialisation(String),
    /// Entry already exists. Contains the current entry Key.
    #[error("Entry already exists {0}")]
    EntryExists(u8),
    /// Entry is too big to fit inside a register
    #[error("Entry is too big to fit inside a register: {0}, max: {1}")]
    EntryTooBig(usize, usize),
    /// Cannot add another entry since the register entry cap has been reached.
    #[error("Cannot add another entry since the register entry cap has been reached: {0}")]
    TooManyEntries(usize),
    /// Supplied actions are not valid
    #[error("Some entry actions are not valid")]
    InvalidEntryActions(BTreeMap<Vec<u8>, Error>),
    /// Entry could not be found on the data
    #[error("Requested entry not found {0}")]
    NoSuchEntry(EntryHash),
    /// User entry could not be found on the data
    #[error("Requested user not found {0:?}")]
    NoSuchUser(User),
    /// Owner is not valid
    #[error("Owner is not a PublicKeySet")]
    InvalidOwnerNotPublicKeySet,
    /// No Policy has been set to the data
    #[error("No policy has been set for this data")]
    PolicyNotSet,
    /// Invalid Operation such as a POST on ImmutableData
    #[error("Invalid operation")]
    InvalidOperation,
    /// Mismatch between key type and signature type.
    #[error("Sign key and signature type do not match")]
    SigningKeyTypeMismatch,
    /// Failed signature validation.
    #[error("Invalid signature")]
    InvalidSignature,
    /// Failed to parse a string.
    #[error("Failed to parse: {0}")]
    FailedToParse(String),
    /// Number out of expected range.
    #[error("The provided number is out of the expected range")]
    OutOfRange,
    /// The CRDT operation cannot be applied as it targets a different content address.
    #[error("The CRDT operation cannot be applied as it targets a different content address.")]
    CrdtWrongAddress(RegisterAddress),
    #[error("BlsError: {0}")]
    BlsError(#[from] BlsError),
}

pub(crate) fn convert_bincode_error(err: bincode::Error) -> Error {
    Error::Serialisation(err.as_ref().to_string())
}

/// Convert type errors to `messaging::Errors` for sending scross the network
pub fn convert_dt_error_to_error_msg(error: Error) -> ErrorMsg {
    match error {
        Error::InvalidOperation => {
            ErrorMsg::InvalidOperation("DtError::InvalidOperation".to_string())
        }
        Error::NoSuchEntry(hash) => ErrorMsg::NoSuchEntry(hash),
        Error::NoSuchUser(user) => ErrorMsg::NoSuchUser(user),
        Error::AccessDenied(pk) => ErrorMsg::AccessDenied(pk),
        other => ErrorMsg::InvalidOperation(format!("DtError: {:?}", other)),
    }
}
