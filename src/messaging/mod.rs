// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

pub mod client;
mod errors;
pub mod location;
mod msg_id;
#[cfg(not(feature = "client-only"))]
pub mod node;
pub mod sap;
pub mod section_info;
pub mod serialisation;

pub use self::{
    errors::{Error, Result},
    location::{Aggregation, DstLocation, EndUser, Itinerary, SrcLocation},
    msg_id::{MessageId, MESSAGE_ID_LEN},
    sap::SectionAuthorityProvider,
    serialisation::WireMsg,
};
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
    SectionInfo {
        msg: section_info::SectionInfoMsg,
        dst_info: DstInfo,
    },
    Client {
        msg: client::ClientMsg,
        dst_info: DstInfo,
    },
    #[cfg(not(feature = "client-only"))]
    Routing {
        msg: node::RoutingMsg,
        dst_info: DstInfo,
    },
    #[cfg(not(feature = "client-only"))]
    Node {
        msg: node::NodeMsg,
        dst_info: DstInfo,
        src_section_pk: Option<PublicKey>,
    },
}

/// This is information kept by 'MessageType' so it can be properly
/// serialised with a valid 'WireMsgHeader'
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq)]
pub struct DstInfo {
    pub dst: XorName,
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

    pub fn to_wire_msg(&self) -> Result<WireMsg> {
        match self {
            Self::SectionInfo { msg, dst_info } => {
                WireMsg::new_section_info_msg(msg, dst_info.dst, dst_info.dst_section_pk)
            }
            Self::Client { msg, dst_info } => {
                WireMsg::new_client_msg(msg, dst_info.dst, dst_info.dst_section_pk)
            }
            #[cfg(not(feature = "client-only"))]
            Self::Routing { msg, dst_info } => {
                WireMsg::new_routing_msg(msg, dst_info.dst, dst_info.dst_section_pk)
            }
            #[cfg(not(feature = "client-only"))]
            Self::Node {
                msg,
                dst_info,
                src_section_pk,
            } => WireMsg::new_node_msg(msg, dst_info.dst, dst_info.dst_section_pk, *src_section_pk),
        }
    }

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
