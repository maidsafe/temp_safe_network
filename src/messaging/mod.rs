// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Messages to/from the client
pub mod client;
mod errors;
/// Source and destination structs for messages
pub mod location;
mod msg_id;
#[cfg(not(feature = "client-only"))]
/// Node to node messages
pub mod node;
/// SectionAuthorityProvider
pub mod sap;
/// Queries and responses for section info
pub mod section_info;

/// Functionality for serialising and deserialising messages
pub mod serialisation;

pub use self::{
    errors::{Error, Result},
    location::{Aggregation, DstLocation, EndUser, Itinerary, SrcLocation},
    msg_id::{MessageId, MESSAGE_ID_LEN},
    sap::SectionAuthorityProvider,
    serialisation::WireMsg,
};
use crate::messaging::node::Variant;
use bls::PublicKey;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Type of message.
/// Note this is part of this crate's public API but this enum is
/// never serialised or even part of the message that is sent over the wire.
#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    /// Message about infrastructure (may be directed at nodes or clients)
    SectionInfo {
        /// the message
        msg: section_info::SectionInfoMsg,
        /// destination info
        dst_info: DstInfo,
    },
    /// Client message
    Client {
        /// the message
        msg: client::ClientMsg,
        /// destination info
        dst_info: DstInfo,
    },
    #[cfg(not(feature = "client-only"))]
    /// Routing layer messages
    Routing {
        /// the message
        msg: node::RoutingMsg,
        /// destination info
        dst_info: DstInfo,
    },
    #[cfg(not(feature = "client-only"))]
    /// Node to node message
    Node {
        /// the message
        msg: node::NodeMsg,
        /// destination info
        dst_info: DstInfo,
        /// source section pk
        src_section_pk: Option<PublicKey>,
    },
}

/// This is information kept by 'MessageType' so it can be properly
/// serialised with a valid 'WireMsgHeader'
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq)]
pub struct DstInfo {
    /// destination xorname
    pub dst: XorName,
    /// Destination section pk
    /// This is used to check we are communicating with the correct section.
    /// An out of date key here will result in Anti-Entropy updates being received.
    pub dst_section_pk: PublicKey,
}

impl MessageType {
    /// serialize the message type into bytes ready to be sent over the wire.
    pub fn serialize(&self) -> Result<Bytes> {
        match self {
            Self::SectionInfo { msg, dst_info } => {
                WireMsg::serialize_section_info_msg(msg, dst_info.dst, dst_info.dst_section_pk)
            }
            Self::Client { msg, dst_info } => {
                WireMsg::serialize_client_msg(msg, dst_info.dst, dst_info.dst_section_pk)
            }
            #[cfg(not(feature = "client-only"))]
            Self::Routing { msg, dst_info } => {
                WireMsg::serialize_routing_msg(msg, dst_info.dst, dst_info.dst_section_pk)
            }
            #[cfg(not(feature = "client-only"))]
            Self::Node {
                msg,
                dst_info,
                src_section_pk,
            } => WireMsg::serialize_node_msg(
                msg,
                dst_info.dst,
                dst_info.dst_section_pk,
                *src_section_pk,
            ),
        }
    }

    /// Returns a WireMsg built from this MessageType
    pub fn to_wire_msg(&self) -> Result<WireMsg> {
        match self {
            Self::SectionInfo { msg, dst_info } => {
                WireMsg::new_section_info_msg(msg, dst_info.dst, dst_info.dst_section_pk)
            }
            Self::Client { msg, dst_info } => {
                WireMsg::new_client_msg(msg, dst_info.dst, dst_info.dst_section_pk)
            }
            #[cfg(not(feature = "client-only"))]
            Self::Routing { msg, dst_info } => WireMsg::new_routing_msg(
                msg,
                dst_info.dst,
                dst_info.dst_section_pk,
                matches!(msg.variant, Variant::JoinRequest(_)),
            ),
            #[cfg(not(feature = "client-only"))]
            Self::Node {
                msg,
                dst_info,
                src_section_pk,
            } => WireMsg::new_node_msg(msg, dst_info.dst, dst_info.dst_section_pk, *src_section_pk),
        }
    }

    /// Update the destination info on the contained message
    pub fn update_dst_info(&mut self, dst_pk: Option<PublicKey>, dst: Option<XorName>) {
        #[cfg(not(feature = "client-only"))]
        match self {
            Self::Client { dst_info, .. } | Self::SectionInfo { dst_info, .. } => {
                if let Some(dst) = dst {
                    dst_info.dst = dst
                }
                if let Some(dst_pk) = dst_pk {
                    dst_info.dst_section_pk = dst_pk
                }
            }
            #[cfg(not(feature = "client-only"))]
            Self::Routing { dst_info, .. } => {
                if let Some(dst) = dst {
                    dst_info.dst = dst
                }
                if let Some(dst_pk) = dst_pk {
                    dst_info.dst_section_pk = dst_pk
                }
            }
            #[cfg(not(feature = "client-only"))]
            Self::Node { dst_info, .. } => {
                if let Some(dst) = dst {
                    dst_info.dst = dst
                }
                if let Some(dst_pk) = dst_pk {
                    dst_info.dst_section_pk = dst_pk
                }
            }
        }

        #[cfg(feature = "client-only")]
        match self {
            Self::Client { dst_info, .. } | Self::SectionInfo { dst_info, .. } => {
                if let Some(dst) = dst {
                    dst_info.dst = dst
                }
                if let Some(dst_pk) = dst_pk {
                    dst_info.dst_section_pk = dst_pk
                }
            }
        }
    }
}
