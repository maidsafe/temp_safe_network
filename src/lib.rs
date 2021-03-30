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
    Ping(HeaderInfo),
    SectionInfo {
        msg: section_info::Message,
        hdr_info: HeaderInfo,
    },
    ClientMessage {
        msg: client::Message,
        hdr_info: HeaderInfo,
    },
    #[cfg(not(feature = "client-only"))]
    NodeMessage {
        msg: node::NodeMessage,
        hdr_info: HeaderInfo,
    },
}

/// This is information kept by 'MessageType' so it can be properly
/// serialised with a valid 'WireMsgHeader'
#[derive(PartialEq, Debug, Clone)]
pub struct HeaderInfo {
    pub dest: XorName,
    pub dest_section_pk: PublicKey,
}

impl MessageType {
    /// serialize the message type into bytes ready to be sent over the wire.
    pub fn serialize(&self) -> Result<Bytes> {
        match self {
            Self::Ping(hdr_info) => {
                WireMsg::new_ping_msg(hdr_info.dest, hdr_info.dest_section_pk).serialize()
            }
            Self::SectionInfo { msg, hdr_info } => {
                WireMsg::serialize_sectioninfo_msg(msg, hdr_info.dest, hdr_info.dest_section_pk)
            }
            Self::ClientMessage { msg, hdr_info } => {
                WireMsg::serialize_client_msg(msg, hdr_info.dest, hdr_info.dest_section_pk)
            }
            #[cfg(not(feature = "client-only"))]
            Self::NodeMessage { msg, hdr_info } => {
                WireMsg::serialize_node_msg(msg, hdr_info.dest, hdr_info.dest_section_pk)
            }
        }
    }

    pub fn update_header(&mut self, dest_pk: Option<PublicKey>, dest: Option<XorName>) {
        #[cfg(not(feature = "client-only"))]
        match self {
            Self::Ping(hdr_info)
            | Self::ClientMessage { hdr_info, .. }
            | Self::SectionInfo { hdr_info, .. } => {
                if let Some(dest) = dest {
                    hdr_info.dest = dest
                }
                if let Some(dest_pk) = dest_pk {
                    hdr_info.dest_section_pk = dest_pk
                }
            }
            #[cfg(not(feature = "client-only"))]
            Self::NodeMessage { hdr_info, .. } => {
                if let Some(dest) = dest {
                    hdr_info.dest = dest
                }
                if let Some(dest_pk) = dest_pk {
                    hdr_info.dest_section_pk = dest_pk
                }
            }
        }

        #[cfg(feature = "client-only")]
        match self {
            Self::Ping(hdr_info)
            | Self::ClientMessage { hdr_info, .. }
            | Self::SectionInfo { hdr_info, .. } => {
                if let Some(dest) = dest {
                    hdr_info.dest = dest
                }
                if let Some(dest_pk) = dest_pk {
                    hdr_info.dest_section_pk = dest_pk
                }
            }
        }
    }
}
