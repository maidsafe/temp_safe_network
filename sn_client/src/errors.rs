// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::LinkError;

use sn_interface::{
    messaging::{
        data::{DataQuery, Error as ErrorMsg, QueryResponse},
        system::NodeMsg,
        Error as MessagingError, MsgId, NetworkMsg,
    },
    types::{Error as DtError, NodeId},
};

use bls::PublicKey;
use sn_dbc::PublicKey as DbcPublicKey;
use std::{io, time::Duration};
use thiserror::Error;
use xor_name::XorName;

/// Specialisation of `std::Result` for Client.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Client Errors
#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No section was found to be closest for given dataname/
    /// This is indicative of some larger problem
    #[error("No SAP for data name: {0}")]
    NoCloseSapFound(XorName),
    /// No elder was found to be closest from provided SAP. (The sap must therefore be empty)
    #[error("No elders found in AntiEntropy msg SAP")]
    AntiEntropyNoSapElders,
    /// Maximum number of retries upon AntiEntropy responses was reached
    #[error(
        "Maximum number of retries upon AntiEntropy responses was reached. \
        Message {msg_id:?} was re-sent {retries} times due to AE responses."
    )]
    AntiEntropyMaxRetries {
        /// Id of the cmd message sent
        msg_id: MsgId,
        /// Number of times the msg was re-sent
        retries: u8,
    },
    /// Failed to obtain network contacts to bootstrap to
    #[error("Failed to obtain network contacts to bootstrap to: {0}")]
    NetworkContacts(String),
    /// InsufficientAcksReceived
    #[error(
        "Did not receive sufficient ACK messages from Elders to be sure this cmd ({msg_id:?}) \
        passed, expected: {expected}, received {received}."
    )]
    InsufficientAcksReceived {
        /// Id of the cmd message sent
        msg_id: MsgId,
        /// Number of expected ACKs
        expected: usize,
        /// Number of received ACKs
        received: usize,
    },
    /// Initial network contact failed
    #[error("Initial network contact probe failed. Attempted contacts: {0:?}")]
    NetworkContact(Vec<NodeId>),
    /// Client has not gone through qp2p bootstrap process yet
    #[error(
        "Client has not yet acquired enough/any network knowledge for destination \
        xorname {0}, so anything sent is guaranteed to have a lengthy AE process"
    )]
    NoNetworkKnowledge(XorName),
    /// Could not connect to sufficient elder to retrieve reliable responses.
    #[error(
        "Problem connecting to sufficient elders. A supermajority of responses is unobtainable. \
        {connections} were connected to, {required} needed."
    )]
    InsufficientElderConnections {
        /// Number of existing connections to Elders
        connections: usize,
        /// Minimum number of connections to Elders required for the operation
        required: usize,
    },
    /// Did not know of sufficient elders in the desired section to get supermajority of responses.
    #[error(
        "Problem finding sufficient elders. A supermajority of responses is unobtainable. \
        {connections} were known in this section, {required} needed. Section pk: {section_pk:?}"
    )]
    InsufficientElderKnowledge {
        /// Number of existing connections to Elders
        connections: usize,
        /// Minimum number of connections to Elders required for the operation
        required: usize,
        /// Public key of the target section
        section_pk: PublicKey,
    },
    /// Cannot store empty file..
    #[error("Cannot store empty file.")]
    EmptyFileProvided,
    /// Not enough bytes for self-encryption.
    #[error(
        "Not enough bytes ({size}) for self-encryption, at least {minimum} bytes needed. \
        Try storing it as a SmallFile."
    )]
    TooSmallForSelfEncryption {
        /// Number of bytes
        size: usize,
        /// Minimum number of bytes for self-encryption
        minimum: usize,
    },
    #[cfg(feature = "limit-client-upload-size")]
    /// Upload size exceeded current file size upload limit.
    #[error(
        "Too large file upload attempted ({size} bytes), at most {limit} bytes allowed currently. \
        Try storing a smaller file."
    )]
    UploadSizeLimitExceeded {
        /// Number of bytes attempted.
        size: usize,
        /// Size limit, number of bytes.
        limit: usize,
    },
    /// Encryption oversized the SmallFile, so it cannot be stored as a SmallFile and be encrypted
    #[error(
        "You might need to pad the `SmallFile` contents and then store it as a `LargeFile`, \
        as the encryption has made it slightly too big ({0} bytes)"
    )]
    SmallFilePaddingNeeded(usize),
    /// The provided bytes is too large to store as a `SmallFile`.
    #[error(
        "The provided bytes ({size}) is too large to store as a `SmallFile` which maximum can be \
        {maximum}. Store as a LargeFile instead."
    )]
    TooLargeAsSmallFile {
        /// Number of bytes
        size: usize,
        /// Maximum number of bytes for a `SmallFile`
        maximum: usize,
    },
    /// Timeout occurred when trying to verify chunk was uploaded
    #[error("Timeout occurred after {elapsed:?} when trying to verify chunk at xorname address {address} was uploaded")]
    ChunkUploadValidationTimeout {
        /// Time elapsed before timing out
        elapsed: Duration,
        /// Address name of the chunk
        address: XorName,
    },
    /// Node closed the bi-stream we expected a response on
    #[error("The bi-stream we expected a msg response on, for {msg_id:?}, was closed by remote node: {node_id:?}")]
    ResponseStreamClosed {
        /// MsgId of the msg sent
        msg_id: MsgId,
        /// Node the msg was sent to
        node_id: NodeId,
    },
    /// Failed to obtain a response from Elders.
    #[error("Failed to obtain any response for {msg_id:?} from: {nodes:?}")]
    NoResponse {
        /// MsgId of the msg sent
        msg_id: MsgId,
        /// Nodes the msg was sent to
        nodes: Vec<NodeId>,
    },
    /// Timeout when awaiting command ACK from Elders.
    #[error("Timeout after {elapsed:?} when awaiting command ACK from Elders for {msg_id:?}, data address {dst_address}")]
    CmdAckValidationTimeout {
        /// MsgId of the msg sent
        msg_id: MsgId,
        /// Time elapsed before timing out
        elapsed: Duration,
        /// Address name of the data the ACK was expected for
        dst_address: XorName,
    },
    /// Unexpected query response received
    #[error("Unexpected response received for {query:?}. Received: {response:?}")]
    UnexpectedQueryResponse {
        /// Query sent to Elders
        query: DataQuery,
        /// Unexpected response received
        response: QueryResponse,
    },
    /// Unexpected NodeMsg received
    #[error("Unexpected type of NodeMsg received from {node_id} in response to {correlation_id:?}. Received: {msg:?}")]
    UnexpectedNodeMsg {
        /// MsgId of the msg sent
        correlation_id: MsgId,
        /// The node that the unexpected msg was received from
        node_id: NodeId,
        /// Unexpected msg received
        msg: NodeMsg,
    },
    /// Unexpected msg type received
    #[error("Unexpected type of message received from {node_id} in response to {correlation_id:?}. Received: {msg:?}")]
    UnexpectedNetworkMsg {
        /// MsgId of the msg sent
        correlation_id: MsgId,
        /// The node that the unexpected msg was received from
        node_id: NodeId,
        /// Unexpected msg received
        msg: NetworkMsg,
    },
    /// Other types errors
    #[error(transparent)]
    NetworkDataError(#[from] DtError),
    /// Errors received from the network via sn_messaging
    #[error("Error received from the network: {source:?}")]
    ErrorMsg {
        /// The source of an error msg
        source: ErrorMsg,
    },
    /// Error response received for a client cmd sent to the network
    #[error("Error received from the network: {source:?} for cmd: {msg_id:?}")]
    CmdError {
        /// The source of an error msg
        source: ErrorMsg,
        /// MsgId of the cmd sent
        msg_id: MsgId,
    },
    /// Errors occurred when serialising or deserialising msgs
    #[error(transparent)]
    MessagingProtocol(#[from] MessagingError),
    /// Self-Enryption errors
    #[error(transparent)]
    SelfEncryption(#[from] self_encryption::Error),
    /// Io error.
    #[error(transparent)]
    IoError(#[from] io::Error),
    /// Endpoint setup error.
    #[error(transparent)]
    EndpointSetup(#[from] qp2p::EndpointError),
    /// QuicP2p Recv error.
    #[error(transparent)]
    QuicP2p(#[from] qp2p::RecvError),
    /// QuicP2p Connection error.
    #[error("Failed to stablish a connection with node {node_id:?}: {error}.")]
    QuicP2pConnection {
        /// Node the connection was attempted to be stablished with
        node_id: NodeId,
        /// The error encountered when attempting to stablish the connection
        error: qp2p::ConnectionError,
        /// MsgId of the msg that was going to be sent
        msg_id: MsgId,
    },
    /// QuicP2p Send error.
    #[error("Failed to send a message to node {node_id:?}: {error}.")]
    QuicP2pSend {
        /// Node the message was attempted to be sent to
        node_id: NodeId,
        /// The error encountered when attempting to send the message
        error: qp2p::SendError,
        /// MsgId of the msg attempted to send
        msg_id: MsgId,
    },
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
    /// All attempts to initiate a bi-stream failed
    #[error("Could no initiate bi-stream for {msg_id:?}: {error:?}")]
    FailedToInitateBiDiStream {
        /// Id of the message to be sent
        msg_id: MsgId,
        /// The error encountered when trying to initiate a bi-stream
        error: LinkError,
    },
    /// Could not chunk all the data required to encrypt the data. (Expected, Actual)
    #[error("Not all data was chunked, expected {expected}, but we have {chunked}.)")]
    NotAllDataWasChunked {
        /// Number of Chunks expected to be generated
        expected: usize,
        /// Number of Chunks generated
        chunked: usize,
    },
    /// Occurs if a signed SAP cannot be obtained for a section key.
    #[error("A signed section authority provider was not found for section key {0:?}")]
    SignedSapNotFound(PublicKey),
    /// Occurs if a DBC spend command eventually fails after a number of retry attempts.
    #[error(
        "The DBC spend request failed after {attempts} attempts for public_key: {public_key:?}"
    )]
    DbcSpendRetryAttemptsExceeded {
        /// Number of attemtps made
        attempts: u8,
        /// The public_key that was attempted to spend
        public_key: DbcPublicKey,
    },
    /// Occurs if a section key is not found when searching the sections DAG.
    #[error("Section key {0:?} was not found in the sections DAG")]
    SectionsDagKeyNotFound(PublicKey),
    /// Data replicas check errors
    #[cfg(feature = "check-replicas")]
    #[error(transparent)]
    DataReplicasCheck(#[from] DataReplicasCheckError),
}

#[cfg(feature = "check-replicas")]
#[derive(Error, Debug)]
#[non_exhaustive]
/// Data replicas check errors
pub enum DataReplicasCheckError {
    /// No response or error received when sending query to data replicas
    #[error("No response or error obtained when sending query to all replicas: {0:?}")]
    NoResponse(DataQuery),
    /// Errors received when checking data replicas
    #[error("Errors occurred when sending the query to {}/{replicas} of the replicas: {query:?}. \
        Errors received: {errors:?}", errors.len()
    )]
    ReceivedErrors {
        /// Number of replicas queried
        replicas: usize,
        /// Query sent to data replicas
        query: DataQuery,
        /// List of errors received with their corresponding replica/Adult index
        errors: Vec<(Error, usize)>,
    },
    /// Not all responses received from data replicas are the same
    #[error(
        "Not all responses received are the same when sending query to {replicas} \
        replicas: {query:?}. Responses received: {responses:?}"
    )]
    DifferentResponses {
        /// Number of replicas queried
        replicas: usize,
        /// Query sent to data replicas
        query: DataQuery,
        /// List of responses received with their corresponding replica/Adult index
        responses: Vec<(QueryResponse, usize)>,
    },
}
