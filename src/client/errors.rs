// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub use crate::messaging::{client::Error as ErrorMessage, Error as MessagingError};
use crate::messaging::{
    client::{CmdError, QueryResponse},
    MessageId,
};
use crate::types::Error as DtError;
use qp2p::Error as QuicP2pError;
use std::io;

use thiserror::Error;

/// Client Errors
#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
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
    /// Client has not gone trhough qp2p bootstrap process yet
    #[error("Client has failed to bootstrap to a section yet")]
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
    /// Unexpected message type receivied while joining.
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
        "Error received from the network: {:?} MessageId: {:?}",
        source,
        msg_id
    )]
    ErrorMessage {
        /// The source of an error message
        source: ErrorMessage,
        /// Message ID that was used to send the query
        msg_id: MessageId,
    },
    /// Errors occurred when serialising or deserialising messages
    #[error(transparent)]
    MessagingProtocol(#[from] MessagingError),
    /// self_enryption errors
    #[error(transparent)]
    SelfEncryption(#[from] self_encryption::SelfEncryptionError),
    /// Other types errors
    #[error(transparent)]
    ConfigError(#[from] serde_json::Error),
    /// Io error.
    #[error(transparent)]
    IoError(#[from] io::Error),
    /// QuicP2p error.
    #[error(transparent)]
    QuicP2p(#[from] QuicP2pError),
    /// Bincode error
    #[error(transparent)]
    Serialisation(#[from] Box<bincode::ErrorKind>),
}

impl From<(CmdError, MessageId)> for Error {
    fn from((error, msg_id): (CmdError, MessageId)) -> Self {
        let CmdError::Data(source) = error;
        Error::ErrorMessage { source, msg_id }
    }
}

impl From<(ErrorMessage, MessageId)> for Error {
    fn from((source, msg_id): (ErrorMessage, MessageId)) -> Self {
        Self::ErrorMessage { source, msg_id }
    }
}
