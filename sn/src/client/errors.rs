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
use bls::PublicKey;
use std::io;
use std::net::SocketAddr;
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
    /// Received unexpected event.
    #[error("Unexpected event received")]
    ReceivedUnexpectedEvent,
    /// Client has not gone through qp2p bootstrap process yet
    #[error("Client has not yet acquired any network knowledge, so anything sent is guaranteed to have a lengthy AE process")]
    NoNetworkKnowledge,
    /// qp2p's IncomingMessages errores
    #[error("An error was returned from IncomingMessages on one of our connections")]
    IncomingMessages,
    /// Could not connect to sufficient elder to retrieve reliable responses.
    #[error(
        "Problem connecting to sufficient elders. A supermajority of responses is unobtainable. {connections} were connected to, {required} needed."
    )]
    InsufficientElderConnections {
        /// Number of existing connections to Elders
        connections: usize,
        /// Minimum number of connections to Elders required for the operation
        required: usize,
    },
    /// Did not know of sufficient elders in the desired section to get supermajority of repsonses.
    #[error(
        "Problem finding sufficient elders. A supermajority of responses is unobtainable. {connections} were known in this section, {required} needed. Section pk: {section_pk:?}"
    )]
    InsufficientElderKnowledge {
        /// Number of existing connections to Elders
        connections: usize,
        /// Minimum number of connections to Elders required for the operation
        required: usize,
        /// Public key of the target section
        section_pk: PublicKey,
    },
    /// Peer connection retrieval failed
    #[error("Error with Peer's connection: {0:?}")]
    PeerConnection(SocketAddr),
    /// Cannot store empty file..
    #[error("Cannot store empty file.")]
    EmptyFileProvided,
    /// Not enough bytes for self-encryption.
    #[error("Not enough bytes for self-encryption. Try storing it as a SmallFile.")]
    TooSmallForSelfEncryption,
    /// Encryption oversized the SmallFile, so it cannot be stored as a SmallFile and be encrypted
    #[error("You might need to pad the `SmallFile` contents and then store it as a `LargeFile`, as the encryption has made it slightly too big")]
    SmallFilePaddingNeeded,
    /// The provided bytes is too large to store as a `SmallFile`.
    #[error(
        "The provided bytes is too large to store as a `SmallFile`. Store as a LargeFile instead."
    )]
    TooLargeAsSmallFile,
    /// No query response before timeout
    #[error("Query timed out")]
    QueryTimedOut,
    /// Could not get an encryption object.
    #[error("Could not get an encryption object.")]
    NoEncryptionObject,
    /// Could not query elder.
    #[error("Failed to obtain any response")]
    NoResponse,
    /// No operation Id could be found
    #[error("Could not retrieve the operation id of a query response")]
    UnknownOperationId,
    /// Unexpected response received
    #[error("Unexpected response received when querying {0:?}")]
    UnexpectedQueryResponse(QueryResponse),
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
    /// QuicP2p Connection error.
    #[error(transparent)]
    QuicP2pConnection(#[from] qp2p::ConnectionError),
    /// Bincode error
    #[error(transparent)]
    Serialisation(#[from] Box<bincode::ErrorKind>),
    /// Could not retrieve all chunks required to decrypt the data. (expected, error)
    #[error("Not all chunks were retrieved, expected {expected}, retrieved {retrieved}.")]
    NotEnoughChunksRetrieved {
        /// Number of Chunks expected to be retrieved
        expected: usize,
        /// Number of Chunks retrieved
        retrieved: usize,
    },
    /// Could not chunk all the data required to encrypt the data. (Expected, Actual)
    #[error("Not all data was chunked, expected {expected}, but we have {chunked}.)")]
    NotAllDataWasChunked {
        /// Number of Chunks expected to be generated
        expected: usize,
        /// Number of Chunks generated
        chunked: usize,
    },
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
