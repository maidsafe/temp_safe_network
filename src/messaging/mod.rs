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
// Types of messages and corresponding source authorities
mod msg_kind;
// SectionAuthorityProvider
mod sap;

pub use self::{
    errors::{Error, Result},
    location::{DstLocation, EndUser, Itinerary, SrcLocation},
    msg_id::{MessageId, MESSAGE_ID_LEN},
    msg_kind::{BlsShareSigned, ClientSigned, MsgKind, NodeSigned, SectionSigned},
    sap::SectionAuthorityProvider,
    serialisation::{MessageType, NodeMsgAuthority, WireMsg},
};
