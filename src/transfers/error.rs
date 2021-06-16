// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crdts::Dot;
use sn_data_types::{Credit, Debit, Error as DtError, PublicKey, Token};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
#[non_exhaustive]
/// SafeNetwork Transfers error type
pub enum Error {
    /// Missing sender actor
    #[error("Sender missing from actors")]
    MissingSender,
    /// Missing recipient
    #[error("Recipient missing from actors")]
    MissingRecipient,
    /// Sender and receiver are the same
    #[error("Sender and recipient are the same")]
    SameSenderAndRecipient,
    /// A debit is awaiting completion. A new debit cannot be started.
    #[error("Current pending debit has not been completed")]
    DebitPending,
    /// The provided PublicKey does not correlate with any key in the section chain history.
    #[error("PublicKey provided by the transfer was never a part of the section chain.")]
    SectionKeyNeverExisted,
    /// The proposed debit has already been seen, or is not the next expected debit
    #[error("Debit already proposed or out of order")]
    DebitProposed,
    /// Credit Id and debit Id do not match
    #[error("Credit and debit ids do not match")]
    CreditDebitIdMismatch,
    /// Credit and debits do not have the same value
    #[error("Credit and debit value do not match")]
    CreditDebitValueMismatch,
    /// This is not the correct actor to validate
    #[error("Validation not intended for this actor")]
    WrongValidationActor,
    /// No pending transfer could be found awaiting accumulation
    #[error("Could not find the expected transfer id among accumulating validations")]
    PendingTransferNotFound,
    /// Validation is not for this actor
    #[error("Validation not expected at this actor {0:?}")]
    NoSetForDebitId(Dot<PublicKey>),
    /// Transer is not for this actor
    #[error("Transfer not expected for this actor {0:?}")]
    NoSetForTransferId(Dot<PublicKey>),
    /// Proposed operation is not the next in sequence. The debit op should be current actor count + 1
    #[error(
        "Operation out of order: debit's counter is '{0}', actor next expected op count is '{1}'"
    )]
    OperationOutOfOrder(u64, u64),
    /// This account has not seen any debits yet. Sent debit should be 0 but was not.
    #[error("Operation out of order debit counter should be 0")]
    ShouldBeInitialOperation,
    /// No credits or debits were found to sync
    #[error("There was no valid history found for provided actor.")]
    NoActorHistory,
    /// 0-value transfers are invalid
    #[error("Transfer amount must be greater than zero")]
    ZeroValueTransfer,
    /// The validation has already been received
    #[error("Validation already received")]
    ValidatedAlready,
    /// Debit is not from this wallet
    #[error("Debit is not from wallet {0}. Debit: {1:?}")]
    DebitDoesNotBelong(PublicKey, Debit),
    /// Credit is not to this wallet
    #[error("Credit is not to wallet {0}. Credit: {1:?}")]
    CreditDoesNotBelong(PublicKey, Credit),
    /// Subtracting this transfer would cause an overlow
    #[error("Overflow when subtracting {0} from balance of: {1}")]
    SubtractionOverflow(Token, Token),
    /// Adding this transfer would cause an overflow
    #[error("Overflow when adding balance {0} and credit of: {1}")]
    AdditionOverflow(Token, Token),
    /// Receive validation failed..
    #[error("Receive validation failed")]
    ReceiveValidationFailed,
    /// TransferCreationFailed..
    #[error("TransferCreationFailed")]
    TransferCreationFailed,
    /// SenderValidationFailed..
    #[error("SenderValidationFailed")]
    SenderValidationFailed,
    /// RegisterProofFailed..
    #[error("RegisterProofFailed")]
    RegisterProofFailed,
    /// InvalidCreditOrDebit..
    #[error("InvalidCreditOrDebit")]
    InvalidCreditOrDebit,
    /// CouldNotGetWalletForReplica..
    #[error("CouldNotGetWalletForReplica")]
    CouldNotGetWalletForReplica,
    /// CouldNotFindGroup..
    #[error("CouldNotFindGroup")]
    CouldNotFindGroup,
    /// ReceivePropagationFailed..
    #[error("ReceivePropagationFailed")]
    ReceivePropagationFailed,
    /// SyncFailed..
    #[error("SyncFailed")]
    SyncFailed,
    /// ValidationFailed..
    #[error("ValidationFailed")]
    ValidationFailed,
    /// GetGenesisFailed..
    #[error("GenesisFailed")]
    GenesisFailed,
    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
    /// Wallet not found
    #[error("{0} Wallet not found locally.")]
    WalletNotFound(PublicKey),
    /// Signature shares are insufficient for BLS aggregation
    #[error("Could not aggregate with given signature shares")]
    CannotAggregate,
    /// Signature is not valid
    #[error("Signature is not valid")]
    InvalidSignature,
    /// Operation is not valid
    #[error("Operation is not valid")]
    InvalidOperation,
    /// Owner is not valid
    #[error("Owner is not valid")]
    InvalidOwner,
    /// Insufficient coins.
    #[error("Insufficient balance to complete this operation")]
    InsufficientBalance,
    /// Inexistent sender balance.
    #[error("No such sender key balance")]
    NoSuchSender,
    /// Inexistent recipient balance. Currently only thrown during network genesis
    #[error("No such recipient key balance")]
    NoSuchRecipient,
    /// Balance key already exists.
    #[error("Key already exists")]
    KeyExists,
    /// Other sn_data_types errors
    #[error(transparent)]
    NetworkDataError(#[from] DtError),
    /// Serialisation
    #[error("Serialisation error. {0}")]
    Serialisation(String),
}
