// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub use crate::messaging::data::Error as ErrorMessage;
use crate::messaging::{
    data::{CmdError, OperationId, QueryResponse},
    Error as MessagingError,
};
use crate::types::Error as DtError;
use std::io;
use thiserror::Error;

/// Specialisation of `std::Result` for Client.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Client Errors
#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Genesis Key from the config and the PrefixMap mismatch
    #[error("Genesis Key from the config and the PrefixMap mismatch. You may need to remove your prefixmap or update your config file.")]
    GenesisKeyMismatch,
    /// Error reading home dir for client
    #[error("Error reading home dir for client")]
    CouldNotReadHomeDir,
    /// Error creating root dir for client
    #[error("Error creating root dir for client")]
    CouldNotCreateRootDir,
    /// Signature Aggregation Error
    #[error("Error on aggregating signatures from network")]
    Aggregation(String),
    /// Asymmetric Key Decryption Failed.
    #[error("Asymmetric key decryption failure")]
    AsymmetricDecipherFailure,
    /// Symmetric Key Decryption Failed.
    #[error("Symmetric key decryption failure")]
    SymmetricDecipherFailure,
    /// Received unexpected data.
    #[error("Unexpected data received")]
    ReceivedUnexpectedData,
    /// Received unexpected event.
    #[error("Unexpected event received")]
    ReceivedUnexpectedEvent,
    /// Could not query elder.
    #[error("Problem querying elder")]
    ElderQuery,
    /// Could not connect to elder.
    #[error("Problem connecting to elder")]
    ElderConnection,
    /// Client has not gone through qp2p bootstrap process yet
    #[error("Client has not yet acquired any network knowledge, so anything sent is guaranteed to have a lengthy AE process")]
    NoNetworkKnowledge,
    /// qp2p's IncomingMessages errores
    #[error("An error was returned from IncomingMessages on one of our connections")]
    IncomingMessages,
    /// Could not connect to sufficient elder to retrieve reliable responses.
    #[error(
        "Problem connecting to sufficient elders. A supermajority of responses is unobtainable. {0} were connected to, {1} needed."
    )]
    InsufficientElderConnections(usize, usize),
    /// Could not query elder.
    #[error("Problem receiving query via qp2p")]
    ReceivingQuery,
    /// Cannot store empty bytes..
    #[error("Cannot store empty bytes.")]
    EmptyBytesProvided,
    /// The provided bytes is too small to be a `Blob`.
    #[error("The provided bytes is too small to be a `Blob`")]
    TooSmallToBeBlob,
    /// Encryption oversized the Spot, so it cannot be stored as a Spot and be encrypted
    #[error("You might need to pad the `Spot` contents and then store it as a `Blob`, as the encryption has made it slightly too big")]
    SpotPaddingNeeded,
    /// The provided bytes is too large to be a `Spot`.
    #[error("The provided bytes is too large to be a `Spot`")]
    TooLargeToBeSpot,
    /// Could not send query to elder.
    #[error("Problem sending query via qp2p")]
    SendingQuery,
    /// No query response before timeout
    #[error("Query timed out")]
    QueryTimedOut,
    /// Could not query elder.
    #[error("Problem receiving query internally in sn_client")]
    QueryReceiverError,
    /// Could not get an encryption object.
    #[error("Could not get an encryption object.")]
    NoEncryptionObject,
    /// Could not query elder.
    #[error("Failed to obtain any response")]
    NoResponse,
    /// No BLS section key known.
    #[error("No BLS Section Key available")]
    NoBlsSectionKey,
    /// No section prefix found for session
    #[error("We do not have a section prefix.")]
    NoSectionPrefixKnown,
    /// Unexpected message type received while joining.
    #[error("Unexpected message type receivied while joining: {0}")]
    UnexpectedMessageOnJoin(String),
    /// Permission set provided is not a PublicPermissionSet.
    #[error("Expected public permission set")]
    NotPublicPermissions,
    /// Permission set provided is not a PrivatePermissionSet.
    #[error("Expected private permission set")]
    NotPrivatePermissions,
    /// Did not receive an incoming connection listener from qp2p
    #[error("Could not listen on elder connection")]
    NoElderListenerEstablished,
    /// Incorrect user permissions were returned
    #[error("Incorrect user permissions were returned")]
    IncorrectPermissions,
    /// No operation Id could be found
    #[error("Could not retrieve the operation id of a query response")]
    UnknownOperationId,
    /// Unexpected response received
    #[error("Unexpected response received when querying {0:?}")]
    UnexpectedQueryResponse(QueryResponse),
    /// Not in testnet "simulated payout" mode
    #[error("Simulated payouts unavailable without 'simualted-payouts' feature flag at build")]
    NotBuiltWithSimulatedPayouts,
    /// Other types errors
    #[error(transparent)]
    NetworkDataError(#[from] DtError),
    /// Errors received from the network via sn_messaging
    #[error(
        "Error received from the network: {:?} Operationid: {:?}",
        source,
        op_id
    )]
    ErrorMessage {
        /// The source of an error message
        source: ErrorMessage,
        /// operation ID that was used to send the query
        op_id: OperationId,
    },
    /// Errors occurred when serialising or deserialising messages
    #[error(transparent)]
    MessagingProtocol(#[from] MessagingError),
    /// self_enryption errors
    #[error(transparent)]
    SelfEncryption(#[from] self_encryption::Error),
    /// Other types errors
    #[error(transparent)]
    ConfigError(#[from] serde_json::Error),
    /// Io error.
    #[error(transparent)]
    IoError(#[from] io::Error),
    /// Endpoint setup error.
    #[error(transparent)]
    EndpointSetup(#[from] qp2p::ClientEndpointError),
    /// QuicP2p error.
    #[error(transparent)]
    QuicP2p(#[from] qp2p::RpcError),
    /// Bincode error
    #[error(transparent)]
    Serialisation(#[from] Box<bincode::ErrorKind>),
    /// Sled error.
    #[error("Sled error:: {0}")]
    Sled(#[from] sled::Error),
    /// Database error.
    #[error("Database error:: {0}")]
    Database(#[from] crate::dbs::Error),
    /// Could not retrieve all chunks required to decrypt the data. (Expected, Actual)
    #[error("Not enough chunks! Required {}, but we have {}.)", _0, _1)]
    NotEnoughChunks(usize, usize),
    /// Could not chunk all the data required to encrypt the data. (Expected, Actual)
    #[error("Not all data was chunked! Required {}, but we have {}.)", _0, _1)]
    NotAllDataWasChunked(usize, usize),
}

impl From<(CmdError, OperationId)> for Error {
    fn from((error, op_id): (CmdError, OperationId)) -> Self {
        let CmdError::Data(source) = error;
        Error::ErrorMessage { source, op_id }
    }
}

impl From<(ErrorMessage, OperationId)> for Error {
    fn from((source, op_id): (ErrorMessage, OperationId)) -> Self {
        Self::ErrorMessage { source, op_id }
    }
}

impl From<qp2p::SendError> for Error {
    fn from(error: qp2p::SendError) -> Self {
        Self::QuicP2p(error.into())
    }
}

impl From<qp2p::RecvError> for Error {
    fn from(error: qp2p::RecvError) -> Self {
        Self::QuicP2p(error.into())
    }
}
