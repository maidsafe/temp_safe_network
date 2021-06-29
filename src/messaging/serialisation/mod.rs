// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod wire_msg;
mod wire_msg_header;

pub use self::wire_msg::WireMsg;
use super::{
    client::ClientMsg, node::NodeMsg, section_info::SectionInfoMsg, BlsShareSigned, ClientSigned,
    DstLocation, MessageId, NodeSigned, SectionSigned,
};
use std::fmt::Debug;

/// Type of message.
/// Note this is part of this crate's public API but this enum is
/// never serialised or even part of the message that is sent over the wire.
#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    /// Message about infrastructure
    SectionInfo {
        msg_id: MessageId,
        dst_location: DstLocation,
        /// the message
        msg: SectionInfoMsg,
    },
    /// Client message
    Client {
        msg_id: MessageId,
        client_signed: ClientSigned,
        dst_location: DstLocation,
        /// the message
        msg: ClientMsg,
    },
    /// Node to node message
    Node {
        msg_id: MessageId,
        msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        /// the message
        msg: NodeMsg,
    },
}

/// Authority of a NodeMsg.
/// Src of message and authority to send it. Authority is validated by the signature.
#[derive(PartialEq, Debug, Clone)]
pub enum NodeMsgAuthority {
    /// Authority of a single peer.
    Node(NodeSigned),
    /// Authority of a single peer that uses it's BLS Keyshare to sign the message.
    BlsShare(BlsShareSigned),
    /// Authority of a whole section.
    Section(SectionSigned),
}
