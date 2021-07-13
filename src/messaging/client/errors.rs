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
    /// Message read was built with an unsupported version.
    // FIXME: unused
    #[error("Unsupported messaging protocol version: {0}")]
    UnsupportedVersion(u16),
    /// Message read contains a payload with an unsupported serialization type.
    // FIXME: unused
    #[error("Unsupported payload serialization: {0}")]
    UnsupportedSerialization(u16),
    /// Access denied for supplied PublicKey
    #[error("Access denied for PublicKey: {0}")]
    AccessDenied(PublicKey),
    /// Error occurred when atempting to verify signature
    // FIXME: unused
    #[error("Signature verification error: {0}")]
    SignatureVerification(String),
    /// Serialization error
    // FIXME: unused
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// Requested data not found
    #[error("Requested data not found: {0:?}")]
    DataNotFound(DataAddress),
    /// No history found for PublicKey
    // FIXME: unused
    #[error("No history found for PublicKey: {0}")]
    NoHistoryForPublicKey(PublicKey),
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
    // FIXME: unused
    #[error("Exceeded a limit on a number of entries")]
    TooManyEntries,
    /// Key does not exist
    #[error("Key does not exist")]
    NoSuchKey,
    /// Duplicate Entries in this push
    // FIXME: unused
    #[error("Duplicate entries provided")]
    DuplicateEntryKeys,
    /// The list of owner keys is invalid
    #[error("Invalid owner key: {0}")]
    InvalidOwners(PublicKey),
    /// No Policy has been set to the data
    // FIXME: unused
    #[error("No policy has been set for this data")]
    PolicyNotSet,
    /// Invalid version for performing a given mutating operation. Contains the
    /// current data version.
    // FIXME: unused
    #[error("Invalid version provided: {0}")]
    InvalidSuccessor(u64),
    /// Invalid version for performing a given mutating operation. Contains the
    /// current owners version.
    // FIXME: unused
    #[error("Invalid owners version provided: {0}")]
    InvalidOwnersSuccessor(u64),
    /// Invalid mutating operation as it causality dependency is currently not satisfied
    // FIXME: unused
    #[error("Operation is not causally ready. Ensure you have the full history of operations.")]
    OpNotCausallyReady,
    /// Invalid version for performing a given mutating operation. Contains the
    /// current permissions version.
    // FIXME: unused
    #[error("Invalid permission version provided: {0}")]
    InvalidPermissionsSuccessor(u64),
    /// Invalid Operation such as a POST on ImmutableData
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    /// Mismatch between key type and signature type.
    // FIXME: unused
    #[error("Sign key and signature type do not match")]
    SigningKeyTypeMismatch,
    /// Failed signature validation.
    // FIXME: unused
    #[error("Invalid signature")]
    InvalidSignature,
    /// Received a request with a duplicate MessageId
    // FIXME: unused
    #[error("Duplicate message id received")]
    DuplicateMessageId,
    // /// Network error occurring at Node level which has no bearing on clients, e.g. serialisation
    // /// failure or database failure
    // #[error("Network error: {0}")]
    // NetworkOther(String),
    /// While parsing, precision would be lost.
    // FIXME: unused
    #[error("Lost precision on the number of coins during parsing")]
    LossOfPrecision,
    /// The amount would exceed the maximum value for `Token` (u64::MAX).
    // FIXME: unused
    #[error("The token amount would exceed the maximum value (u64::MAX)")]
    ExcessiveValue,
    /// Transaction ID already exists.
    // FIXME: unused
    #[error("Transaction Id already exists")]
    TransactionIdExists,
    /// Transfer hsitory missing, operation out of order. <received, expected>
    // FIXME: unused
    #[error("Transfer received out of order. Requested transfer operation index was {0}, but {1} was expected" )]
    MissingTransferHistory(u64, u64),
    /// Insufficient tokens provided to pay for this operation.
    // FIXME: unused
    #[error("Insufficient payment provided to complete this operation")]
    InsufficientPayment,
    /// Inexistent balance.
    // TODO: key/wallet/balance, what's our vocab here?
    // FIXME: unused
    #[error("No such key exists")]
    NoSuchBalance,
    /// Inexistent sender balance.
    // FIXME: unused
    #[error("No such sender key balance")]
    NoSuchSender,
    /// Inexistent recipient balance.
    // TODO: this should not be possible
    // FIXME: unused
    #[error("No such recipient key balance")]
    NoSuchRecipient,
    /// Coin balance already exists.
    // FIXME: unused
    #[error("Key already exists")]
    BalanceExists,
    /// Expected data size exceeded.
    // FIXME: unused
    #[error("Size of the structure exceeds the limit")]
    ExceededSize,
    /// The operation has not been signed by an actor PK and so cannot be validated.
    // FIXME: unused
    #[error("CRDT operation missing actor signature")]
    CrdtMissingOpSignature,
    /// The data for a given policy could not be located, so CRDT operations cannot be applied.
    // FIXME: unused
    #[error("CRDT data is in an unexpected and/or inconsistent state. No data found for current policy.")]
    CrdtUnexpectedState,
    /// Entry already exists. Contains the current entry Key.
    // FIXME: unused
    #[error("Entry already exists {0}")]
    EntryExists(u8),
    /// Problem registering the payment at a node
    // FIXME: unused
    #[error("Payment registration failed")]
    PaymentFailed,
    /// Node failed to delete the requested data for some reason.
    // FIXME: unused
    #[error("Failed to delete requested data")]
    FailedToDelete,
    /// Node does not manage any section funds.
    // FIXME: unused
    #[error("Node does not currently manage any section funds")]
    NoSectionFunds,
    /// Node does not manage any metadata, so is likely not a fully prepared elder yet.
    // FIXME: unused
    #[error("Node does not currently manage any section metadata")]
    NoSectionMetaData,
    /// Node does not manage any immutable chunks.
    // FIXME: unused
    #[error("Node does not currently manage any immutable chunks")]
    NoImmutableChunks,
    /// Node is currently churning so cannot perform the request.
    // FIXME: unused
    #[error("Cannot complete request due to churning of funds")]
    NodeChurningFunds,
    /// The node hasn't left the section, and was not marked for relocation during reward operations
    // FIXME: unused
    #[error("Node is not being relocated")]
    NodeWasNotRelocated,
}
