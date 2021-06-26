// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Messages to/from the client
pub mod client;
/// Node to node messages
pub mod node;
/// Queries and responses for section info
pub mod section_info;
/// Functionality for serialising and deserialising messages
pub mod serialisation;

// Error types definitions
mod errors;
// Source and destination structs for messages
mod location;
// Message ID definition
mod msg_id;
// Types of source authorities for message
mod msg_authority;
// SectionAuthorityProvider
mod sap;

pub use self::{
    errors::{Error, Result},
    location::{DstLocation, EndUser, Itinerary, SrcLocation},
    msg_authority::{BlsShareSigned, ClientSigned, MsgAuthority, NodeSigned, SectionSigned},
    msg_id::{MessageId, MESSAGE_ID_LEN},
    sap::SectionAuthorityProvider,
    serialisation::{MsgEnvelope, WireMsg},
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
    /// Message about infrastructure
    SectionInfo {
        /// message envelope
        msg_envelope: MsgEnvelope,
        /// the message
        msg: section_info::SectionInfoMsg,
    },
    /// Client message
    Client {
        /// message envelope
        msg_envelope: MsgEnvelope,
        /// the message
        msg: client::ClientMsg,
    },
    /// Node to node message
    Node {
        /// message envelope
        msg_envelope: MsgEnvelope,
        /// the message
        msg: node::NodeMsg,
    },
}
