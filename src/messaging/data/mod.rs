// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Messages that clients can send to the network, and their possible responses.
//!
//! See [`ClientMsg`] for message definitions.

mod chunk;
mod cmd;
mod data_exchange;
mod errors;
mod map;
mod query;
mod register;
mod sequence;

pub use self::{
    chunk::{ChunkRead, ChunkWrite},
    cmd::DataCmd,
    data_exchange::{
        ChunkDataExchange, ChunkMetadata, DataExchange, HolderMetadata, MapDataExchange,
        RegisterDataExchange, SequenceDataExchange,
    },
    errors::{Error, Result},
    map::{MapCmd, MapRead, MapWrite},
    query::DataQuery,
    register::{RegisterCmd, RegisterRead, RegisterWrite},
    sequence::{SequenceCmd, SequenceRead, SequenceWrite},
};

use crate::messaging::MessageId;
use crate::types::{
    register::{Entry, EntryHash, Permissions, Policy, Register},
    Chunk, Map, MapEntries, MapPermissionSet, MapValue, MapValues, PublicKey, Sequence,
    SequenceEntries, SequenceEntry, SequencePermissions, SequencePrivatePolicy,
    SequencePublicPolicy,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
};

/// Messages that a client can send to the network, and their possible responses.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum DataMsg {
    /// Data messages that a client or nodes can send, and their possible responses.
    Process(ProcessMsg),

    /// A response indicating that the recipient was unable to process a client's message.
    ProcessingError(ProcessingError),
}

/// A response indicating that the recipient was unable to process a client's message.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct ProcessingError {
    /// Optional reason for the error.
    ///
    /// This can be used to handle the error.
    pub reason: Option<Error>,
    /// Message that triggered this error.
    ///
    /// This could be used to retry the message if the error could be handled.
    pub source_message: Option<ProcessMsg>,
}

/// Messages that a client can send to the network, and their possible responses.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum ProcessMsg {
    /// Messages that lead to mutation.
    ///
    /// There will be no response to these messages on success, only if something went wrong. Due to
    /// the eventually consistent nature of the network, it may be necessary to continually retry
    /// operations that depend on the effects of mutations.
    Cmd(DataCmd),
    /// A read-only operation.
    ///
    /// Senders should eventually receive either a corresponding [`QueryResponse`] or an error in
    /// reply.
    /// [`QueryResponse`]: Self::QueryResponse
    Query(DataQuery),
    /// An Event is a fact about something that happened.
    // FIXME: this is never constructed, so could perhaps be removed.
    Event {
        /// Request.
        event: Event,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// The response to a query, containing the query result.
    QueryResponse {
        /// The result of the query.
        response: QueryResponse,
        /// ID of the query message.
        correlation_id: MessageId,
    },
    /// An error response to a [`Cmd`].
    ///
    /// [`Cmd`]: Self::Cmd
    CmdError {
        /// The error.
        error: CmdError,
        /// ID of causing [`Cmd`] message.
        ///
        /// [`Cmd`]: Self::Cmd
        correlation_id: MessageId,
    },
}

/// An error response to a [`Cmd`].
///
/// [`Cmd`]: ProcessMsg::Cmd
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum CmdError {
    /// An error response to a [`DataCmd`].
    // FIXME: `Cmd` is not an enum, so should this be?
    Data(Error), // DataError enum for better differentiation?
}

/// Events from the network that are pushed to the client.
// FIXME: this cannot be constructed as it is empty.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Event {}

/// The response to a query, containing the query result.
#[allow(clippy::large_enum_variant, clippy::type_complexity)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum QueryResponse {
    //
    // ===== Chunk =====
    //
    /// Response to [`ChunkRead::Get`].
    GetChunk(Result<Chunk>),
    //
    // ===== Map =====
    //
    /// Response to [`MapRead::Get`].
    GetMap(Result<Map>),
    /// Response to [`MapRead::GetShell`].
    ///
    /// Note that the contained [`Map`] if `Ok` will have all its metadata fields set, but not the
    /// data itself (e.g. the contained map will be empty, even if the actual map on the network is
    /// not). See [`MapRead::Get`], [`MapRead::GetValue`], [`MapRead::ListEntries`],
    /// [`MapRead::ListKeys`], or [`MapRead::ListValues`] for queries that return data in the map.
    GetMapShell(Result<Map>),
    /// Response to [`MapRead::GetVersion`].
    GetMapVersion(Result<u64>),
    /// Response to [`MapRead::ListEntries`].
    ListMapEntries(Result<MapEntries>),
    /// Response to [`MapRead::ListKeys`].
    ListMapKeys(Result<BTreeSet<Vec<u8>>>),
    /// Response to [`MapRead::ListValues`].
    ListMapValues(Result<MapValues>),
    /// Response to [`MapRead::ListUserPermissions`]
    ListMapUserPermissions(Result<MapPermissionSet>),
    /// Response to [`MapRead::ListPermissions`].
    ListMapPermissions(Result<BTreeMap<PublicKey, MapPermissionSet>>),
    /// Response to [`MapRead::GetValue`].
    GetMapValue(Result<MapValue>),
    //
    // ===== Sequence Data =====
    //
    /// Response to [`SequenceRead::Get`].
    GetSequence(Result<Sequence>),
    /// Response to [`SequenceRead::GetRange`].
    GetSequenceRange(Result<SequenceEntries>),
    /// Response to [`SequenceRead::GetLastEntry`].
    GetSequenceLastEntry(Result<(u64, SequenceEntry)>),
    /// Response to [`SequenceRead::GetPublicPolicy`].
    GetSequencePublicPolicy(Result<SequencePublicPolicy>),
    /// Response to [`SequenceRead::GetPrivatePolicy`].
    GetSequencePrivatePolicy(Result<SequencePrivatePolicy>),
    /// Response to [`SequenceRead::GetUserPermissions`].
    GetSequenceUserPermissions(Result<SequencePermissions>),
    //
    // ===== Register Data =====
    //
    /// Response to [`RegisterRead::Get`].
    GetRegister(Result<Register>),
    /// Response to [`RegisterRead::GetOwner`].
    GetRegisterOwner(Result<PublicKey>),
    /// Response to [`RegisterRead::Read`].
    ReadRegister(Result<BTreeSet<(EntryHash, Entry)>>),
    /// Response to [`RegisterRead::GetPolicy`].
    GetRegisterPolicy(Result<Policy>),
    /// Response to [`RegisterRead::GetUserPermissions`].
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

/// Error type for an attempted conversion from a [`QueryResponse`] variant to an expected wrapped
/// value.
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
    use xor_name::XorName;

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

    pub(crate) fn gen_keys() -> Vec<PublicKey> {
        gen_keypairs().iter().map(PublicKey::from).collect()
    }

    #[test]
    fn debug_format_functional() -> Result<()> {
        if let Some(key) = gen_keys().first() {
            let errored_response = QueryResponse::GetSequence(Err(Error::AccessDenied(*key)));
            assert!(format!("{:?}", errored_response).contains("GetSequence(Err(AccessDenied("));
            Ok(())
        } else {
            Err(anyhow!("Could not generate public key"))
        }
    }

    #[test]
    fn generate_processing_error() {
        let msg = ProcessMsg::Query(DataQuery::Blob(ChunkRead::Get(ChunkAddress::Private(
            XorName::random(),
        ))));
        let random_addr = DataAddress::Chunk(ChunkAddress::Public(XorName::random()));
        let lazy_error = ProcessingError {
            reason: Some(Error::DataNotFound(random_addr.clone())),
            source_message: Some(msg),
        };

        assert!(format!("{:?}", lazy_error).contains("Blob(Get(Private"));
        assert!(format!("{:?}", lazy_error).contains("ProcessingError"));
        assert!(format!("{:?}", lazy_error).contains(&format!("DataNotFound({:?})", random_addr)));
    }

    #[test]
    fn debug_format_processing_error() {
        let chunk_addr = ChunkAddress::Public(XorName::random());
        let random_addr = DataAddress::Chunk(chunk_addr);
        let errored_response = ProcessingError {
            reason: Some(Error::DataNotFound(random_addr.clone())),
            source_message: Some(ProcessMsg::Query(DataQuery::Blob(ChunkRead::Get(
                chunk_addr,
            )))),
        };

        assert!(format!("{:?}", errored_response).contains("Blob(Get(Public"));
        assert!(format!("{:?}", errored_response).contains("ProcessingError"));
        assert!(
            format!("{:?}", errored_response).contains(&format!("DataNotFound({:?})", random_addr))
        );
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
}
