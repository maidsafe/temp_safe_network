// Copyright 2021MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Formatter},
    result,
};

use sn_data_types::PublicKey;
use thiserror::Error;

/// A specialised `Result` type for safecoin.
pub type Result<T> = result::Result<T, Error>;

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
#[derive(Error, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    /// Access denied for supplied PublicKey
    #[error("Access denied for PublicKey: {0}")]
    AccessDenied(PublicKey),
    /// Serialization error
    #[error("Serialisation error: {0}")]
    Serialisation(String),
    /// Requested data not found
    #[error("Requested data not found")]
    NoSuchData,

    /// No history found for PublicKey
    #[error("No history found for PublicKey: {0}")]
    NoHistoryForPublicKey(sn_data_types::PublicKey),

    /// Failed to write file, likely due to a system Io error
    #[error("Failed to write file")]
    FailedToWriteFile,

    /// Provided data already exists on the network
    #[error("Data provided already exists")]
    DataExists,
    /// Entry could not be found on the data
    #[error("Requested entry not found")]
    NoSuchEntry,
    /// Exceeds limit on entrites for the given data type
    #[error("Exceeded a limit on a number of entries")]
    TooManyEntries,
    /// Key does not exist
    #[error("Key does not exist")]
    NoSuchKey,
    /// Node NotEnoughSpace error
    #[error("Node does not have sufficient space to store chunk")]
    NotEnoughSpace,
    /// Duplicate Entries in this push
    #[error("Duplicate entries provided")]
    DuplicateEntryKeys,
    /// The list of owner keys is invalid
    #[error("Invalid owner key: {0}")]
    InvalidOwners(sn_data_types::PublicKey),
    /// No Policy has been set to the data
    #[error("No policy has been set for this data")]
    PolicyNotSet,
    /// Invalid version for performing a given mutating operation. Contains the
    /// current data version.
    #[error("Invalid version provided: {0}")]
    InvalidSuccessor(u64),
    /// Invalid version for performing a given mutating operation. Contains the
    /// current owners version.
    #[error("Invalid owners version provided: {0}")]
    InvalidOwnersSuccessor(u64),
    /// Invalid mutating operation as it causality dependency is currently not satisfied
    #[error("Operation is not causally ready. Ensure you have the full history of operations.")]
    OpNotCausallyReady,
    /// Invalid version for performing a given mutating operation. Contains the
    /// current permissions version.
    #[error("Invalid permission version provided: {0}")]
    InvalidPermissionsSuccessor(u64),
    /// Invalid Operation such as a POST on ImmutableData
    #[error("Invalid operation")]
    InvalidOperation,
    /// Mismatch between key type and signature type.
    #[error("Sign key and signature type do not match")]
    SigningKeyTypeMismatch,
    /// Failed signature validation.
    #[error("Invalid signature")]
    InvalidSignature,
    /// Received a request with a duplicate MessageId
    #[error("Duplicate message id received")]
    DuplicateMessageId,
    // /// Network error occurring at Node level which has no bearing on clients, e.g. serialisation
    // /// failure or database failure
    // #[error("Network error: {0}")]
    // NetworkOther(String),
    /// While parsing, precision would be lost.
    #[error("Lost precision on the number of coins during parsing")]
    LossOfPrecision,
    /// The amount would exceed the maximum value for `Money` (u64::MAX).
    #[error("The money amount would exceed the maximum value (u64::MAX)")]
    ExcessiveValue,
    /// Failed to parse a string.
    #[error("Failed to parse: {0}")]
    FailedToParse(String),
    /// Transaction ID already exists.
    #[error("Transaction Id already exists")]
    TransactionIdExists,
    /// Insufficient coins.
    #[error("Insufficient balance to complete this operation")]
    InsufficientBalance,
    /// Inexistent balance.
    // TODO: key/wallet/balance, what's our vocab here?
    #[error("No such key exists")]
    NoSuchBalance,
    /// Inexistent sender balance.
    #[error("No such sender key balance")]
    NoSuchSender,
    /// Inexistent recipient balance.
    // TODO: this should not be possible
    #[error("No such recipient key balance")]
    NoSuchRecipient,
    /// Coin balance already exists.
    #[error("Key already exists")]
    BalanceExists,
    /// Expected data size exceeded.
    #[error("Size of the structure exceeds the limit")]
    ExceededSize,
    /// The operation has not been signed by an actor PK and so cannot be validated.
    #[error("CRDT operation missing actor signature")]
    CrdtMissingOpSignature,
    /// The data for a given policy could not be located, so CRDT operations cannot be applied.
    #[error("CRDT data is in an unexpected and/or inconsistent state. No data found for current policy.")]
    CrdtUnexpectedState,
    /// Entry already exists. Contains the current entry Key.
    #[error("Entry already exists {0}")]
    EntryExists(u8),

    /// Problem registering the payment at a node
    #[error("Payment registration failed")]
    PaymentFailed,

    /// Node failed to delete the requested data for some reason.
    #[error("Failed to delete requested data")]
    FailedToDelete,

    /// Node error that was not expected for message response
    #[error("There was an unexpected error at the node: '{0}'")]
    UnexpectedNodeError(String), // /// NetworkData error.
                                 // #[error("Network data error:: {0}")]
                                 // NetworkData(#[from] sn_data_types::Error)
}
