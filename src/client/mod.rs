// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod blob;
mod cmd;
mod data;
mod duty;
mod errors;
mod map;
mod network;
mod query;
mod sender;
mod sequence;
mod transfer;

pub use self::{
    blob::{BlobRead, BlobWrite},
    cmd::Cmd,
    data::{DataCmd, DataQuery},
    duty::{AdultDuties, Duty, ElderDuties, NodeDuties},
    errors::{Error, ErrorDebug, Result},
    map::{MapRead, MapWrite},
    network::{
        NodeCmd, NodeCmdError, NodeDataError, NodeDataQueryResponse, NodeEvent, NodeQuery,
        NodeQueryResponse, NodeRewardError, NodeRewardQuery, NodeRewardQueryResponse,
        NodeSystemCmd, NodeSystemQuery, NodeTransferCmd, NodeTransferError, NodeTransferQuery,
        NodeTransferQueryResponse,
    },
    query::Query,
    sender::{Address, MsgSender, TransientElderKey, TransientSectionKey},
    sequence::{SequenceRead, SequenceWrite},
    transfer::{TransferCmd, TransferQuery},
};

use crate::{MessageId, MessageType, WireMsg};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sn_data_types::{
    ActorHistory, Blob, Map, MapEntries, MapPermissionSet, MapValue, MapValues, PublicKey,
    Sequence, SequenceEntries, SequenceEntry, SequencePermissions, SequencePrivatePolicy,
    SequencePublicPolicy, Token, TransferAgreementProof, TransferValidated,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    fmt,
};
use threshold_crypto::PublicKey as BlsPublicKey;
use xor_name::XorName;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Message {
    Functional(FunctionalMsg),
    ProcessingError(ProcessingError),
}

/// Our LazyMesssage error. Recipient was unable to process this message for some reason.
/// The original message should be returned in full, and context can optionally be added via
/// reason.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct ProcessingError {
    /// Optional reason for the error. This should help recveiving node handle the error
    reason: Option<ProcessingErrorReason>,
    /// Message that triggered this error
    source_message: FunctionalMsg,
    /// MessageId
    id: MessageId,
}

/// Error reasons for inability to handle a message at a client/node.
/// These should be recoverable with updated information from the sender
/// or other nodes in the network.
///
/// A lot of errors should be handled even before they reach the node via
/// Routing ensuring we're targetting the correct section key.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum ProcessingErrorReason {
    /// No section key known of at our node.
    NoSectionKey,
    /// No database set up to handle this function
    NoTransfersDatabase,
    /// No database set up to handle this function
    NoMetaDataDatabase,
    /// No database set up to handle this function
    NoSectionFundsDatabase,
    /// Node genesis_stage
    IncorrectGenesisStage,
    /// Could not validate signed data
    CouldNotValidate,
    /// Could not find `key`
    CouldNotFindKey,
}

/// Message envelope containing a Safe message payload,
/// This struct also provides utilities to obtain the serialized bytes
/// ready to send them over the wire.
impl Message {
    /// Convenience function to deserialize a 'Message' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a client message.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::ClientMessage { msg, .. } = deserialized {
            Ok(msg)
        } else {
            Err(crate::Error::FailedToParse(
                "bytes as a client message".to_string(),
            ))
        }
    }

    /// Serialize this Message into bytes ready to be sent over the wire.
    pub fn serialize(&self, dest: XorName, dest_section_pk: BlsPublicKey) -> crate::Result<Bytes> {
        WireMsg::serialize_client_msg(self, dest, dest_section_pk)
    }

    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        match self {
            Self::Functional(FunctionalMsg::Cmd { id, .. })
            | Self::Functional(FunctionalMsg::Query { id, .. })
            | Self::Functional(FunctionalMsg::Event { id, .. })
            | Self::Functional(FunctionalMsg::QueryResponse { id, .. })
            | Self::Functional(FunctionalMsg::CmdError { id, .. })
            | Self::Functional(FunctionalMsg::NodeCmd { id, .. })
            | Self::Functional(FunctionalMsg::NodeEvent { id, .. })
            | Self::Functional(FunctionalMsg::NodeQuery { id, .. })
            | Self::Functional(FunctionalMsg::NodeCmdError { id, .. })
            | Self::Functional(FunctionalMsg::NodeQueryResponse { id, .. })
            | Self::ProcessingError(ProcessingError { id, .. }) => *id,
        }
    }
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum FunctionalMsg {
    /// A Cmd is leads to a write / change of state.
    /// We expect them to be successful, and only return a msg
    /// if something went wrong.
    Cmd {
        /// Cmd.
        cmd: Cmd,
        /// Message ID.
        id: MessageId,
    },
    /// Queries is a read-only operation.
    Query {
        /// Query.
        query: Query,
        /// Message ID.
        id: MessageId,
    },
    /// An Event is a fact about something that happened.
    Event {
        /// Request.
        event: Event,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// The response to a query, containing the query result.
    QueryResponse {
        /// QueryResponse.
        response: QueryResponse,
        /// Message ID.
        id: MessageId,
        /// ID of causing query.
        correlation_id: MessageId,
    },
    /// Cmd error.
    CmdError {
        /// The error.
        error: CmdError,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// Cmds only sent internally in the network.
    NodeCmd {
        /// NodeCmd.
        cmd: NodeCmd,
        /// Message ID.
        id: MessageId,
    },
    /// An error of a NodeCmd.
    NodeCmdError {
        /// The error.
        error: NodeCmdError,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// Events only sent internally in the network.
    NodeEvent {
        /// Request.
        event: NodeEvent,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// Queries is a read-only operation.
    NodeQuery {
        /// Query.
        query: NodeQuery,
        /// Message ID.
        id: MessageId,
    },
    /// The response to a query, containing the query result.
    NodeQueryResponse {
        /// QueryResponse.
        response: NodeQueryResponse,
        /// Message ID.
        id: MessageId,
        /// ID of causing query.
        correlation_id: MessageId,
    },
}

impl FunctionalMsg {
    #[allow(dead_code)]
    fn create_processing_error_msg(&self, reason: Option<ProcessingErrorReason>) -> Message {
        Message::ProcessingError(ProcessingError {
            source_message: self.clone(),
            id: MessageId::new(),
            reason,
        })
    }
}

impl FunctionalMsg {
    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        match self {
            Self::Cmd { id, .. }
            | Self::Query { id, .. }
            | Self::Event { id, .. }
            | Self::QueryResponse { id, .. }
            | Self::CmdError { id, .. }
            | Self::NodeCmd { id, .. }
            | Self::NodeEvent { id, .. }
            | Self::NodeQuery { id, .. }
            | Self::NodeCmdError { id, .. }
            | Self::NodeQueryResponse { id, .. } => *id,
        }
    }
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum CmdError {
    ///
    Auth(Error), // temporary, while Authenticator is not handling this
    ///
    Data(Error), // DataError enum for better differentiation?
    ///
    Transfer(TransferError),
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum TransferError {
    /// The error of a ValidateTransfer cmd.
    TransferValidation(Error),
    /// The error of a RegisterTransfer cmd.
    TransferRegistration(Error),
}

/// Events from the network that
/// are pushed to the client.
#[allow(clippy::large_enum_variant, clippy::type_complexity)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Event {
    /// The transfer was validated by a Replica instance.
    TransferValidated {
        /// This is the validation of the transfer
        /// requested by the client for an account.
        event: TransferValidated,
    },
    /// An aggregate event created client side
    /// (for upper Client layers) out of a quorum of TransferValidated events.
    /// This is a temporary variant, until
    /// SignatureAccumulation has been broken out
    /// to its own crate, and can be used at client.
    TransferAgreementReached {
        /// The accumulated proof.
        proof: TransferAgreementProof,
    },
}

/// Query responses from the network.
#[allow(clippy::large_enum_variant, clippy::type_complexity)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum QueryResponse {
    //
    // ===== Blob =====
    //
    /// Get Blob.
    GetBlob(Result<Blob>),
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
    /// Get Sequence owners.
    GetSequenceOwner(Result<PublicKey>),
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
    // ===== Tokens =====
    //
    /// Get key balance.
    GetBalance(Result<Token>),
    /// Get key transfer history.
    GetHistory(Result<ActorHistory>),
    /// Get Store Cost.
    GetStoreCost(Result<Token>),
}

/// The kind of authorisation needed for a request.
pub enum AuthorisationKind {
    /// Authorisation for data requests.
    Data(DataAuthKind),
    /// Authorisation for token requests.
    Token(TokenAuthKind),
    /// Miscellaneous authorisation kinds.
    /// NB: Not very well categorized yet
    Misc(MiscAuthKind),
    /// When none required.
    None,
}

/// Authorisation for data requests.
pub enum DataAuthKind {
    /// Read of public data.
    PublicRead,
    /// Read of private data.
    PrivateRead,
    /// Write of data/metadata.
    Write,
}

/// Authorisation for token requests.
pub enum TokenAuthKind {
    /// Request to get key balance.
    ReadBalance,
    /// Request to get key transfer history.
    ReadHistory,
    /// Request to transfer tokens from key.
    Transfer,
}

/// Miscellaneous authorisation kinds.
/// NB: Not very well categorized yet
pub enum MiscAuthKind {
    /// Request to manage app keys.
    ManageAppKeys,
    /// Request to mutate and transfer tokens from key.
    WriteAndTransfer,
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

try_from!(Blob, GetBlob);
try_from!(Map, GetMap, GetMapShell);
try_from!(u64, GetMapVersion);
try_from!(MapEntries, ListMapEntries);
try_from!(BTreeSet<Vec<u8>>, ListMapKeys);
try_from!(MapValues, ListMapValues);
try_from!(MapPermissionSet, ListMapUserPermissions);
try_from!(BTreeMap<PublicKey, MapPermissionSet>, ListMapPermissions);
try_from!(MapValue, GetMapValue);
try_from!(Sequence, GetSequence);
try_from!(PublicKey, GetSequenceOwner);
try_from!(SequenceEntries, GetSequenceRange);
try_from!((u64, SequenceEntry), GetSequenceLastEntry);
try_from!(SequencePublicPolicy, GetSequencePublicPolicy);
try_from!(SequencePrivatePolicy, GetSequencePrivatePolicy);
try_from!(SequencePermissions, GetSequenceUserPermissions);
try_from!(Token, GetBalance);
try_from!(ActorHistory, GetHistory);

impl fmt::Debug for QueryResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use QueryResponse::*;

        match self {
            // Blob
            GetBlob(res) => write!(f, "QueryResponse::GetBlob({:?})", ErrorDebug(res)),
            // Map
            GetMap(res) => write!(f, "QueryResponse::GetMap({:?})", ErrorDebug(res)),
            GetMapShell(res) => write!(f, "QueryResponse::GetMapShell({:?})", ErrorDebug(res)),
            GetMapVersion(res) => write!(f, "QueryResponse::GetMapVersion({:?})", ErrorDebug(res)),
            ListMapEntries(res) => {
                write!(f, "QueryResponse::ListMapEntries({:?})", ErrorDebug(res))
            }
            ListMapKeys(res) => write!(f, "QueryResponse::ListMapKeys({:?})", ErrorDebug(res)),
            ListMapValues(res) => write!(f, "QueryResponse::ListMapValues({:?})", ErrorDebug(res)),
            ListMapPermissions(res) => write!(
                f,
                "QueryResponse::ListMapPermissions({:?})",
                ErrorDebug(res)
            ),
            ListMapUserPermissions(res) => write!(
                f,
                "QueryResponse::ListMapUserPermissions({:?})",
                ErrorDebug(res)
            ),
            GetMapValue(res) => write!(f, "QueryResponse::GetMapValue({:?})", ErrorDebug(res)),
            // Sequence
            GetSequence(res) => write!(f, "QueryResponse::GetSequence({:?})", ErrorDebug(res)),
            GetSequenceRange(res) => {
                write!(f, "QueryResponse::GetSequenceRange({:?})", ErrorDebug(res))
            }
            GetSequenceLastEntry(res) => write!(
                f,
                "QueryResponse::GetSequenceLastEntry({:?})",
                ErrorDebug(res)
            ),
            GetSequenceUserPermissions(res) => write!(
                f,
                "QueryResponse::GetSequenceUserPermissions({:?})",
                ErrorDebug(res)
            ),
            GetSequencePublicPolicy(res) => write!(
                f,
                "QueryResponse::GetSequencePublicPolicy({:?})",
                ErrorDebug(res)
            ),
            GetSequencePrivatePolicy(res) => write!(
                f,
                "QueryResponse::GetSequencePrivatePolicy({:?})",
                ErrorDebug(res)
            ),
            GetSequenceOwner(res) => {
                write!(f, "QueryResponse::GetSequenceOwner({:?})", ErrorDebug(res))
            }
            GetBalance(res) => write!(f, "QueryResponse::GetBalance({:?})", ErrorDebug(res)),
            GetHistory(res) => write!(f, "QueryResponse::GetHistory({:?})", ErrorDebug(res)),
            GetStoreCost(res) => write!(f, "QueryResponse::GetStoreCost({:?})", ErrorDebug(res)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};
    use sn_data_types::{Keypair, PublicBlob, UnseqMap};
    use std::convert::{TryFrom, TryInto};

    fn gen_keypairs() -> Vec<Keypair> {
        let mut rng = rand::thread_rng();
        let bls_secret_key = threshold_crypto::SecretKeySet::random(1, &mut rng);
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
                .contains("QueryResponse::GetSequence(AccessDenied(PublicKey::"));
            Ok(())
        } else {
            Err(anyhow!("Could not generate public key"))
        }
    }
    #[test]
    fn generate_processing_error() -> Result<()> {
        if let Some(key) = gen_keys().first() {
            let msg = FunctionalMsg::Query {
                query: Query::Transfer(TransferQuery::GetBalance(*key)),
                id: MessageId::new(),
            };
            let lazy_error =
                msg.create_processing_error_msg(Some(ProcessingErrorReason::CouldNotFindKey));

            assert!(format!("{:?}", lazy_error).contains("TransferQuery::GetBalance"));
            assert!(format!("{:?}", lazy_error).contains("ProcessingError"));
            assert!(format!("{:?}", lazy_error).contains("CouldNotFindKey"));

            Ok(())
        } else {
            Err(anyhow!("Could not generate public key"))
        }
    }

    #[test]
    fn debug_format_processing_error() -> Result<()> {
        if let Some(key) = gen_keys().first() {
            let errored_response = ProcessingError {
                reason: Some(ProcessingErrorReason::CouldNotFindKey),
                source_message: FunctionalMsg::Query {
                    id: MessageId::new(),
                    query: Query::Transfer(TransferQuery::GetBalance(*key)),
                },
                id: MessageId::new(),
            };

            assert!(format!("{:?}", errored_response).contains("TransferQuery::GetBalance"));
            assert!(format!("{:?}", errored_response).contains("ProcessingError"));
            assert!(format!("{:?}", errored_response).contains("CouldNotFindKey"));
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

        let i_data = Blob::Public(PublicBlob::new(vec![1, 3, 1, 4]));
        let e = Error::AccessDenied(key);
        assert_eq!(
            i_data,
            GetBlob(Ok(i_data.clone()))
                .try_into()
                .map_err(|_| anyhow!("Mismatched types".to_string()))?
        );
        assert_eq!(
            Err(TryFromError::Response(e.clone())),
            Blob::try_from(GetBlob(Err(e.clone())))
        );

        let mut data = BTreeMap::new();
        let _ = data.insert(vec![1], vec![10]);
        let owners = PublicKey::Bls(threshold_crypto::SecretKey::random().public_key());
        let m_data = Map::Unseq(UnseqMap::new_with_data(
            *i_data.name(),
            1,
            data,
            BTreeMap::new(),
            owners,
        ));
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
        let pk = keypair.public_key();

        let random_xor = xor_name::XorName::random();
        let id = MessageId(random_xor);
        let message = Message::Functional(FunctionalMsg::Query {
            query: Query::Transfer(TransferQuery::GetBalance(pk)),
            id,
        });

        // test msgpack serialization
        let dest = XorName::random();
        let dest_section_pk = threshold_crypto::SecretKey::random().public_key();
        let serialized = message.serialize(dest, dest_section_pk)?;
        let deserialized = Message::from(serialized)?;
        assert_eq!(deserialized, message);

        Ok(())
    }
}
