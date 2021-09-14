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
use std::{io, net::SocketAddr};
use thiserror::Error;

/// Specialisation of `std::Result` for Client.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Client Errors
#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
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
    #[error("Client has failed to bootstrap to the network yet")]
    NotBootstrapped,
    /// Could not connect to sufficient elder to retrieve reliable responses.
    #[error(
        "Problem connecting to sufficient elders. A supermajority of responses is unobtainable. {0} were connected to"
    )]
    InsufficientElderConnections(usize),
    /// Could not query elder.
    #[error("Problem receiving query via qp2p")]
    ReceivingQuery,
    /// Could not send query to elder.
    #[error("Problem sending query via qp2p")]
    SendingQuery,
    /// Could not query elder.
    #[error("Problem receiving query internally in sn_client")]
    QueryReceiverError,
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
    QuicP2p(#[from] qp2p::SendError),
    /// Bincode error
    #[error(transparent)]
    Serialisation(#[from] Box<bincode::ErrorKind>),
    /// Sled error.
    #[error("Sled error:: {0}")]
    Sled(#[from] sled::Error),
    /// Database error.
    #[error("Database error:: {0}")]
    Database(#[from] crate::dbs::Error),
    /// Generic Error
    #[error("Generic error")]
    Generic(String),
    /// Could not bootstrap to an unresponsive peer
    #[error("Could not bootstrap to an unresponsive peer {0}")]
    BootstrapToPeerFailed(SocketAddr),
    /// Could not retrieve all chunks required to decrypt the data. (Expected, Actual)
    #[error("Not enough chunks! Required {}, but we have {}.)", _0, _1)]
    NotEnoughChunks(usize, usize),
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
