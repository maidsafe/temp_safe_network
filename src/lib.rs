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
pub mod section_info;
mod serialisation;

pub use self::{
    errors::{Error, Result},
    location::{Aggregation, DstLocation, EndUser, Itinerary, SrcLocation},
    msg_id::MessageId,
    serialisation::WireMsg,
};
use bytes::Bytes;
use threshold_crypto::PublicKey;
use xor_name::XorName;

/// Type of message.
/// Note this is part of this crate's public API but this enum is
/// never serialised or even part of the message that is sent over the wire.
#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    Ping(DestInfo),
    SectionInfo {
        msg: section_info::Message,
        dest_info: DestInfo,
    },
    ClientMessage {
        msg: client::Message,
        dest_info: DestInfo,
    },
    #[cfg(not(feature = "client-only"))]
    NodeMessage {
        msg: node::NodeMessage,
        dest_info: DestInfo,
    },
    #[cfg(not(feature = "client-only"))]
    NodeCmdMessage {
        msg: node::NodeCmdMessage,
        dest_info: DestInfo,
        src_section_pk: Option<PublicKey>,
    },
}

/// This is information kept by 'MessageType' so it can be properly
/// serialised with a valid 'WireMsgHeader'
#[derive(PartialEq, Debug, Clone)]
pub struct DestInfo {
    pub dest: XorName,
    pub dest_section_pk: PublicKey,
}

impl MessageType {
    /// serialize the message type into bytes ready to be sent over the wire.
    pub fn serialize(&self) -> Result<Bytes> {
        match self {
            Self::Ping(dest_info) => {
                WireMsg::new_ping_msg(dest_info.dest, dest_info.dest_section_pk).serialize()
            }
            Self::SectionInfo { msg, dest_info } => {
                WireMsg::serialize_sectioninfo_msg(msg, dest_info.dest, dest_info.dest_section_pk)
            }
            Self::ClientMessage { msg, dest_info } => {
                WireMsg::serialize_client_msg(msg, dest_info.dest, dest_info.dest_section_pk)
            }
            #[cfg(not(feature = "client-only"))]
            Self::NodeMessage { msg, dest_info } => {
                WireMsg::serialize_node_msg(msg, dest_info.dest, dest_info.dest_section_pk)
            }
            #[cfg(not(feature = "client-only"))]
            Self::NodeCmdMessage {
                msg,
                dest_info,
                src_section_pk,
            } => WireMsg::serialize_node_cmd_msg(
                msg,
                dest_info.dest,
                dest_info.dest_section_pk,
                *src_section_pk,
            ),
        }
    }

    pub fn update_dest_info(&mut self, dest_pk: Option<PublicKey>, dest: Option<XorName>) {
        #[cfg(not(feature = "client-only"))]
        match self {
            Self::Ping(dest_info)
            | Self::ClientMessage { dest_info, .. }
            | Self::SectionInfo { dest_info, .. } => {
                if let Some(dest) = dest {
                    dest_info.dest = dest
                }
                if let Some(dest_pk) = dest_pk {
                    dest_info.dest_section_pk = dest_pk
                }
            }
            #[cfg(not(feature = "client-only"))]
            Self::NodeMessage { dest_info, .. } => {
                if let Some(dest) = dest {
                    dest_info.dest = dest
                }
                if let Some(dest_pk) = dest_pk {
                    dest_info.dest_section_pk = dest_pk
                }
            }
            #[cfg(not(feature = "client-only"))]
            Self::NodeCmdMessage { dest_info, .. } => {
                if let Some(dest) = dest {
                    dest_info.dest = dest
                }
                if let Some(dest_pk) = dest_pk {
                    dest_info.dest_section_pk = dest_pk
                }
            }
        }

        #[cfg(feature = "client-only")]
        match self {
            Self::Ping(dest_info)
            | Self::ClientMessage { dest_info, .. }
            | Self::SectionInfo { dest_info, .. } => {
                if let Some(dest) = dest {
                    dest_info.dest = dest
                }
                if let Some(dest_pk) = dest_pk {
                    dest_info.dest_section_pk = dest_pk
                }
            }
        }
    }
}
