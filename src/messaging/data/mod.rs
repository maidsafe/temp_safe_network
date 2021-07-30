// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Data messages and their possible responses.

mod chunk;
mod cmd;
mod data_exchange;
mod errors;
mod query;
mod register;

pub use self::{
    chunk::{ChunkRead, ChunkWrite},
    cmd::DataCmd,
    data_exchange::{
        ChunkDataExchange, ChunkMetadata, DataExchange, HolderMetadata, RegisterDataExchange,
    },
    errors::{Error, Result},
    query::DataQuery,
    register::{RegisterCmd, RegisterRead, RegisterWrite},
};

use crate::messaging::{data::Error as ErrorMessage, MessageId};
use crate::types::{
    register::{Entry, EntryHash, Permissions, Policy, Register},
    Chunk, PublicKey,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, convert::TryFrom};

/// A message indicating that an error occurred as a node was handling a client's message.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct ServiceError {
    /// Optional reason for the error.
    ///
    /// This can be used to handle the error.
    pub reason: Option<Error>,
    /// Message that triggered this error.
    ///
    /// This could be used to retry the message if the error could be handled.
    pub source_message: Option<Box<ServiceMsg>>,
}

/// Network service messages that clients or nodes send in order to use the services,
/// communicate and carry out the tasks.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum ServiceMsg {
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
    /// A message indicating that an error occurred as a node was handling a client's message.
    ServiceError(ServiceError),
}

/// An error response to a [`Cmd`].
///
/// [`Cmd`]: ServiceMsg::Cmd
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum CmdError {
    /// An error response to a [`DataCmd`].
    // FIXME: `Cmd` is not an enum, so should this be?
    Data(Error), // DataError enum for better differentiation?
}

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
            GetRegister(result) => result.is_ok(),
            GetRegisterOwner(result) => result.is_ok(),
            ReadRegister(result) => result.is_ok(),
            GetRegisterPolicy(result) => result.is_ok(),
            GetRegisterUserPermissions(result) => result.is_ok(),
        }
    }

    /// Returns true if data was not found
    pub fn failed_with_data_not_found(&self) -> bool {
        use QueryResponse::*;

        match self {
            GetChunk(result) => match result {
                Ok(_) => false,
                Err(error) => matches!(*error, ErrorMessage::DataNotFound(_)),
            },
            GetRegister(result) => match result {
                Ok(_) => false,
                Err(error) => matches!(*error, ErrorMessage::DataNotFound(_)),
            },
            GetRegisterOwner(result) => match result {
                Ok(_) => false,
                Err(error) => matches!(*error, ErrorMessage::DataNotFound(_)),
            },
            ReadRegister(result) => match result {
                Ok(_) => false,
                Err(error) => matches!(*error, ErrorMessage::DataNotFound(_)),
            },
            GetRegisterPolicy(result) => match result {
                Ok(_) => false,
                Err(error) => matches!(*error, ErrorMessage::DataNotFound(_)),
            },
            GetRegisterUserPermissions(result) => match result {
                Ok(_) => false,
                Err(error) => matches!(*error, ErrorMessage::DataNotFound(_)),
            },
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
try_from!(Register, GetRegister);
try_from!(PublicKey, GetRegisterOwner);
try_from!(BTreeSet<(EntryHash, Entry)>, ReadRegister);
try_from!(Policy, GetRegisterPolicy);
try_from!(Permissions, GetRegisterUserPermissions);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChunkAddress, DataAddress, Keypair, PrivateChunk};
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
            let errored_response = QueryResponse::GetRegister(Err(Error::AccessDenied(*key)));
            assert!(format!("{:?}", errored_response).contains("GetRegister(Err(AccessDenied("));
            Ok(())
        } else {
            Err(anyhow!("Could not generate public key"))
        }
    }

    #[test]
    fn generate_service_error() {
        let msg = ServiceMsg::Query(DataQuery::Blob(ChunkRead::Get(ChunkAddress::Private(
            XorName::random(),
        ))));
        let random_addr = DataAddress::Chunk(ChunkAddress::Public(XorName::random()));
        let lazy_error = ServiceError {
            reason: Some(Error::DataNotFound(random_addr.clone())),
            source_message: Some(Box::new(msg)),
        };

        assert!(format!("{:?}", lazy_error).contains("Blob(Get(Private"));
        assert!(format!("{:?}", lazy_error).contains("ServiceError"));
        assert!(format!("{:?}", lazy_error).contains(&format!("DataNotFound({:?})", random_addr)));
    }

    #[test]
    fn debug_format_service_error() {
        let chunk_addr = ChunkAddress::Public(XorName::random());
        let random_addr = DataAddress::Chunk(chunk_addr);
        let errored_response = ServiceError {
            reason: Some(Error::DataNotFound(random_addr.clone())),
            source_message: Some(Box::new(ServiceMsg::Query(DataQuery::Blob(
                ChunkRead::Get(chunk_addr),
            )))),
        };

        assert!(format!("{:?}", errored_response).contains("Blob(Get(Public"));
        assert!(format!("{:?}", errored_response).contains("ServiceError"));
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
            Chunk::try_from(GetChunk(Err(e)))
        );

        Ok(())
    }
}
