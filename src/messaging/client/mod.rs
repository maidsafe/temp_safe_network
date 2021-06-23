// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Client messaging.

mod chunk;
mod cmd;
mod data;
mod data_exchange;
mod duty;
mod errors;
mod map;
mod query;
mod register;
mod sequence;

pub use self::{
    chunk::{ChunkRead, ChunkWrite},
    cmd::Cmd,
    data::{DataCmd, DataQuery},
    data_exchange::{
        ChunkDataExchange, ChunkMetadata, DataExchange, HolderMetadata, MapDataExchange,
        RegisterDataExchange, SequenceDataExchange,
    },
    duty::{AdultDuties, Duty, NodeDuties},
    errors::{Error, Result},
    map::{MapCmd, MapRead, MapWrite},
    query::Query,
    register::{RegisterCmd, RegisterRead, RegisterWrite},
    sequence::{SequenceCmd, SequenceRead, SequenceWrite},
};

use crate::messaging::{MessageId, MessageType, WireMsg};
use crate::types::{
    register::{Entry, EntryHash, Permissions, Policy, Register},
    Chunk, Map, MapEntries, MapPermissionSet, MapValue, MapValues, PublicKey, Sequence,
    SequenceEntries, SequenceEntry, SequencePermissions, SequencePrivatePolicy,
    SequencePublicPolicy, Signature,
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
};
use xor_name::XorName;

/// Message envelope containing a Safe message payload,
/// This struct also provides utilities to obtain the serialized bytes
/// ready to send them over the wire.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    /// Process message.
    Process(ProcessMsg),

    /// Processing error.
    ProcessingError(ProcessingError),

    /// Supporting information.
    SupportingInfo(SupportingInfo),
}

/// Our response to a processing error. Anti entropy in that it updates the erroring node
/// with any relevant information, and includes the original message, which should hereafter
/// be actionable
// #[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct SupportingInfo {
    /// Supporting information for the source_message process
    pub info: SupportingInfoFor,
    /// The original message that triggered the error this update should be correcting
    pub source_message: ProcessMsg,
    /// MessageId
    pub id: MessageId,
    /// Correlates to a ProcessingError
    pub correlation_id: MessageId,
}

/// Various types of supporting information that can be received and acted upon by a node.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum SupportingInfoFor {}

impl SupportingInfo {
    /// Construct a wrapper for the given information and metadata.
    pub fn new(
        info: SupportingInfoFor,
        source_message: ProcessMsg,
        correlation_id: MessageId,
        id: MessageId,
    ) -> Self {
        Self {
            info,
            source_message,
            id,
            correlation_id,
        }
    }

    /// Get msg id
    pub fn id(&self) -> MessageId {
        self.id
    }

    /// Get source message that originally triggered a ProcessingError. This should usually be replayed at source after applying supporting information
    pub fn source_message(&self) -> &ProcessMsg {
        &self.source_message
    }

    /// Get the supporting information of this message
    pub fn info(&self) -> &SupportingInfoFor {
        &self.info
    }

    /// MessageId of the ProcessingError that triggered this InformationUpdate
    pub fn correlation_id(&self) -> MessageId {
        self.correlation_id
    }
}

/// Our LazyMesssage error. Recipient was unable to process this message for some reason.
/// The original message should be returned in full, and context can optionally be added via
/// reason.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct ProcessingError {
    /// Optional reason for the error. This should help the receiving node handle the error
    reason: Option<Error>,
    /// Message that triggered this error
    source_message: Option<ProcessMsg>,
    /// MessageId
    id: MessageId,
}

impl ProcessingError {
    /// Construct a new error with the given reason and metadata.
    pub fn new(reason: Option<Error>, source_message: Option<ProcessMsg>, id: MessageId) -> Self {
        Self {
            reason,
            source_message,
            id,
        }
    }

    /// Get the message ID.
    pub fn id(&self) -> MessageId {
        self.id
    }

    /// Get the [`ProcessMsg`] that caused the error.
    pub fn source_message(&self) -> &Option<ProcessMsg> {
        &self.source_message
    }

    /// The reason the error occurred (supplied by the recipient).
    pub fn reason(&self) -> &Option<Error> {
        &self.reason
    }
}

impl ClientMsg {
    /// Convenience function to deserialize a 'Message' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a client message.
    pub fn from(bytes: Bytes) -> crate::messaging::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::Client { msg, .. } = deserialized {
            Ok(msg)
        } else {
            Err(crate::messaging::Error::FailedToParse(
                "bytes as a client message".to_string(),
            ))
        }
    }

    /// Serialize this Message into bytes ready to be sent over the wire.
    pub fn serialize(
        &self,
        dst: XorName,
        dst_section_pk: BlsPublicKey,
    ) -> crate::messaging::Result<Bytes> {
        unimplemented!();
        //WireMsg::serialize_client_msg(self, dst, dst_section_pk)
    }

    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        match self {
            Self::Process(ProcessMsg::Cmd { id, .. })
            | Self::Process(ProcessMsg::Query { id, .. })
            | Self::Process(ProcessMsg::Event { id, .. })
            | Self::Process(ProcessMsg::QueryResponse { id, .. })
            | Self::Process(ProcessMsg::CmdError { id, .. })
            | Self::ProcessingError(ProcessingError { id, .. }) => *id,
            Self::SupportingInfo(SupportingInfo { id, .. }) => *id,
        }
    }

    /// return ProcessMessage if any
    pub fn get_process(&self) -> Option<&ProcessMsg> {
        match self {
            Self::Process(msg) => Some(msg),
            Self::ProcessingError(_) => None,
            Self::SupportingInfo(msg) => Some(&msg.source_message()),
        }
    }

    /// return ProcessMessage if any
    pub fn get_processing_error(&self) -> Option<&ProcessingError> {
        match self {
            Self::Process(_) => None,
            Self::SupportingInfo(_) => None,
            Self::ProcessingError(error) => Some(error),
        }
    }
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum ProcessMsg {
    /// A Cmd is leads to a write / change of state.
    /// We expect them to be successful, and only return a msg
    /// if something went wrong.
    Cmd {
        /// Message ID.
        id: MessageId,
        /// Cmd.
        cmd: Cmd,
    },
    /// Queries is a read-only operation.
    Query {
        /// Message ID.
        id: MessageId,
        /// Query.
        query: Query,
    },
    /// An Event is a fact about something that happened.
    Event {
        /// Message ID.
        id: MessageId,
        /// Request.
        event: Event,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// The response to a query, containing the query result.
    QueryResponse {
        /// Message ID.
        id: MessageId,
        /// QueryResponse.
        response: QueryResponse,
        /// ID of causing query.
        correlation_id: MessageId,
    },
    /// Cmd error.
    CmdError {
        /// Message ID.
        id: MessageId,
        /// The error.
        error: CmdError,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
}

impl ProcessMsg {
    /// Create a [`ProcessingError`] to indicate that this message could not be processed.
    ///
    /// Context for the error can optionally be supplied in `reason`.
    pub fn create_processing_error(&self, reason: Option<Error>) -> ProcessingError {
        ProcessingError {
            source_message: Some(self.clone()),
            id: MessageId::new(),
            reason,
        }
    }

    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        match self {
            Self::Cmd { id, .. }
            | Self::Query { id, .. }
            | Self::Event { id, .. }
            | Self::QueryResponse { id, .. }
            | Self::CmdError { id, .. } => *id,
        }
    }
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum CmdError {
    ///
    Data(Error), // DataError enum for better differentiation?
}

/// Events from the network that
/// are pushed to the client.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Event {}

/// Query responses from the network.
#[allow(clippy::large_enum_variant, clippy::type_complexity)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum QueryResponse {
    //
    // ===== Chunk =====
    //
    /// Get Chunk.
    GetChunk(Result<Chunk>),
    //
    // ===== Map =====
    //
    /// Get Map.
    GetMap(Result<Map>),
    /// Get Map shell.
    GetMapShell(Result<Map>),
    /// Get Map version.
    GetMapVersion(Result<u64>),
    /// List all Map entries (key-value pairs).
    ListMapEntries(Result<MapEntries>),
    /// List all Map keys.
    ListMapKeys(Result<BTreeSet<Vec<u8>>>),
    /// List all Map values.
    ListMapValues(Result<MapValues>),
    /// Get Map permissions for a user.
    ListMapUserPermissions(Result<MapPermissionSet>),
    /// List all Map permissions.
    ListMapPermissions(Result<BTreeMap<PublicKey, MapPermissionSet>>),
    /// Get Map value.
    GetMapValue(Result<MapValue>),
    //
    // ===== Sequence Data =====
    //
    /// Get Sequence.
    GetSequence(Result<Sequence>),
    /// Get Sequence entries from a range.
    GetSequenceRange(Result<SequenceEntries>),
    /// Get Sequence last entry.
    GetSequenceLastEntry(Result<(u64, SequenceEntry)>),
    /// Get public Sequence permissions for a user.
    GetSequencePublicPolicy(Result<SequencePublicPolicy>),
    /// Get private Sequence permissions for a user.
    GetSequencePrivatePolicy(Result<SequencePrivatePolicy>),
    /// Get Sequence permissions for a user.
    GetSequenceUserPermissions(Result<SequencePermissions>),
    //
    // ===== Register Data =====
    //
    /// Get Register.
    GetRegister(Result<Register>),
    /// Get Register owners.
    GetRegisterOwner(Result<PublicKey>),
    /// Read Register.
    ReadRegister(Result<BTreeSet<(EntryHash, Entry)>>),
    /// Get public Register permissions for a user.
    GetRegisterPolicy(Result<Policy>),
    /// Get Register permissions for a user.
    GetRegisterUserPermissions(Result<Permissions>),
}

impl QueryResponse {
    /// Returns true if the result returned is a success or not
    pub fn is_success(&self) -> bool {
        use QueryResponse::*;
        match self {
            GetChunk(result) => result.is_ok(),
            GetMap(result) => result.is_ok(),
            GetMapShell(result) => result.is_ok(),
            GetMapVersion(result) => result.is_ok(),
            ListMapEntries(result) => result.is_ok(),
            ListMapKeys(result) => result.is_ok(),
            ListMapValues(result) => result.is_ok(),
            ListMapUserPermissions(result) => result.is_ok(),
            ListMapPermissions(result) => result.is_ok(),
            GetMapValue(result) => result.is_ok(),
            GetSequence(result) => result.is_ok(),
            GetSequenceRange(result) => result.is_ok(),
            GetSequenceLastEntry(result) => result.is_ok(),
            GetSequencePublicPolicy(result) => result.is_ok(),
            GetSequencePrivatePolicy(result) => result.is_ok(),
            GetSequenceUserPermissions(result) => result.is_ok(),
            GetRegister(result) => result.is_ok(),
            GetRegisterOwner(result) => result.is_ok(),
            ReadRegister(result) => result.is_ok(),
            GetRegisterPolicy(result) => result.is_ok(),
            GetRegisterUserPermissions(result) => result.is_ok(),
        }
    }
}

/// Error type for an attempted conversion from `QueryResponse` to a type implementing
/// `TryFrom<Response>`.
#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum TryFromError {
    /// Wrong variant found in `QueryResponse`.
    WrongType,
    /// The `QueryResponse` contained an error.
    Response(Error),
}

macro_rules! try_from {
    ($ok_type:ty, $($variant:ident),*) => {
        impl TryFrom<QueryResponse> for $ok_type {
            type Error = TryFromError;
            fn try_from(response: QueryResponse) -> std::result::Result<Self, Self::Error> {
                match response {
                    $(
                        QueryResponse::$variant(Ok(data)) => Ok(data),
                        QueryResponse::$variant(Err(error)) => Err(TryFromError::Response(error)),
                    )*
                    _ => Err(TryFromError::WrongType),
                }
            }
        }
    };
}

try_from!(Chunk, GetChunk);
try_from!(Map, GetMap, GetMapShell);
try_from!(u64, GetMapVersion);
try_from!(MapEntries, ListMapEntries);
try_from!(BTreeSet<Vec<u8>>, ListMapKeys);
try_from!(MapValues, ListMapValues);
try_from!(MapPermissionSet, ListMapUserPermissions);
try_from!(BTreeMap<PublicKey, MapPermissionSet>, ListMapPermissions);
try_from!(MapValue, GetMapValue);
try_from!(Sequence, GetSequence);
try_from!(SequenceEntries, GetSequenceRange);
try_from!((u64, SequenceEntry), GetSequenceLastEntry);
try_from!(SequencePublicPolicy, GetSequencePublicPolicy);
try_from!(SequencePrivatePolicy, GetSequencePrivatePolicy);
try_from!(SequencePermissions, GetSequenceUserPermissions);
try_from!(Register, GetRegister);
try_from!(PublicKey, GetRegisterOwner);
try_from!(BTreeSet<(EntryHash, Entry)>, ReadRegister);
try_from!(Policy, GetRegisterPolicy);
try_from!(Permissions, GetRegisterUserPermissions);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChunkAddress, DataAddress, Keypair, MapKind, MapValue, PrivateChunk};
    use anyhow::{anyhow, Result};
    use std::convert::{TryFrom, TryInto};

    fn gen_keypairs() -> Vec<Keypair> {
        let mut rng = rand::thread_rng();
        let bls_secret_key = bls::SecretKeySet::random(1, &mut rng);
        vec![
            Keypair::new_ed25519(&mut rng),
            Keypair::new_bls_share(
                0,
                bls_secret_key.secret_key_share(0),
                bls_secret_key.public_keys(),
            ),
        ]
    }

    pub fn gen_keys() -> Vec<PublicKey> {
        gen_keypairs().iter().map(PublicKey::from).collect()
    }

    #[test]
    fn debug_format_functional() -> Result<()> {
        if let Some(key) = gen_keys().first() {
            let errored_response = QueryResponse::GetSequence(Err(Error::AccessDenied(*key)));
            assert!(format!("{:?}", errored_response)
                .contains("GetSequence(Err(AccessDenied(PublicKey::"));
            Ok(())
        } else {
            Err(anyhow!("Could not generate public key"))
        }
    }

    #[test]
    fn generate_processing_error() -> Result<()> {
        if let Some(keypair) = gen_keypairs().first() {
            let public_key = keypair.public_key();

            let msg = ProcessMsg::Query {
                id: MessageId::new(),
                query: Query::Data(DataQuery::Blob(ChunkRead::Get(ChunkAddress::Private(
                    XorName::random(),
                )))),
            };
            let random_addr = DataAddress::Chunk(ChunkAddress::Public(XorName::random()));
            let lazy_error =
                msg.create_processing_error(Some(Error::DataNotFound(random_addr.clone())));

            assert!(format!("{:?}", lazy_error).contains("Data(Blob(Get(Private"));
            assert!(format!("{:?}", lazy_error).contains("ProcessingError"));
            assert!(
                format!("{:?}", lazy_error).contains(&format!("DataNotFound({:?})", random_addr))
            );

            Ok(())
        } else {
            Err(anyhow!("Could not generate public key"))
        }
    }

    #[test]
    fn debug_format_processing_error() -> Result<()> {
        if let Some(keypair) = gen_keypairs().first() {
            let public_key = keypair.public_key();

            let chunk_addr = ChunkAddress::Public(XorName::random());
            let random_addr = DataAddress::Chunk(chunk_addr);
            let errored_response = ProcessingError {
                reason: Some(Error::DataNotFound(random_addr.clone())),
                source_message: Some(ProcessMsg::Query {
                    id: MessageId::new(),
                    query: Query::Data(DataQuery::Blob(ChunkRead::Get(chunk_addr))),
                }),
                id: MessageId::new(),
            };

            assert!(format!("{:?}", errored_response).contains("Data(Blob(Get(Public"));
            assert!(format!("{:?}", errored_response).contains("ProcessingError"));
            assert!(format!("{:?}", errored_response)
                .contains(&format!("DataNotFound({:?})", random_addr)));
            Ok(())
        } else {
            Err(anyhow!("Could not generate public key"))
        }
    }

    #[test]
    fn try_from() -> Result<()> {
        use QueryResponse::*;
        let key = match gen_keys().first() {
            Some(key) => *key,
            None => return Err(anyhow!("Could not generate public key")),
        };

        let owner = PublicKey::Bls(bls::SecretKey::random().public_key());

        let i_data = Chunk::Private(PrivateChunk::new(vec![1, 3, 1, 4], owner));
        let e = Error::AccessDenied(key);
        assert_eq!(
            i_data,
            GetChunk(Ok(i_data.clone()))
                .try_into()
                .map_err(|_| anyhow!("Mismatched types".to_string()))?
        );
        assert_eq!(
            Err(TryFromError::Response(e.clone())),
            Chunk::try_from(GetChunk(Err(e.clone())))
        );

        let mut data = BTreeMap::new();
        let _ = data.insert(
            vec![1],
            MapValue {
                pointer: XorName::random(),
                version: 0,
            },
        );

        let m_data = Map::new_with_data(
            *i_data.name(),
            1,
            data,
            BTreeMap::new(),
            owner,
            MapKind::Private,
        );
        assert_eq!(
            m_data,
            GetMap(Ok(m_data.clone()))
                .try_into()
                .map_err(|_| anyhow!("Mismatched types".to_string()))?
        );
        assert_eq!(
            Err(TryFromError::Response(e.clone())),
            Map::try_from(GetMap(Err(e)))
        );
        Ok(())
    }

    #[test]
    fn serialization() -> Result<()> {
        let keypair = &gen_keypairs()[0];
        let public_key = keypair.public_key();

        let id = MessageId::new();
        let message = ClientMsg::Process(ProcessMsg::Query {
            id,
            query: Query::Data(DataQuery::Blob(ChunkRead::Get(ChunkAddress::Private(
                XorName::random(),
            )))),
        });

        // test msgpack serialization
        let dst = XorName::random();
        let dst_section_pk = bls::SecretKey::random().public_key();
        let serialized = message.serialize(dst, dst_section_pk)?;
        let deserialized = ClientMsg::from(serialized)?;
        assert_eq!(deserialized, message);

        Ok(())
    }
}
