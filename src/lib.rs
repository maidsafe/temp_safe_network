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
pub mod infrastructure;
pub mod location;
mod msg_id;
#[cfg(not(feature = "client-only"))]
pub mod node;
mod serialisation;

pub use self::{
    errors::{Error, Result},
    location::{DstLocation, SrcLocation, User},
    msg_id::MessageId,
    serialisation::WireMsg,
};
use bytes::Bytes;

/// Type of message.
/// Note this is part of this crate's public API but this enum is
/// never serialised or even part of the message that is sent over the wire.
#[derive(PartialEq, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    Ping,
    InfrastructureMessage(infrastructure::Message),
    ClientMessage(client::MsgEnvelope),
    #[cfg(not(feature = "client-only"))]
    NodeMessage(node::NodeMessage),
}

impl MessageType {
    /// serialize the message type into bytes ready to be sent over the wire.
    pub fn serialize(&self) -> Result<Bytes> {
        match self {
            Self::Ping => WireMsg::new_ping_msg().serialize(),
            Self::InfrastructureMessage(query) => WireMsg::serialize_infrastructure_msg(query),
            Self::ClientMessage(msg) => WireMsg::serialize_client_msg(msg),
            #[cfg(not(feature = "client-only"))]
            Self::NodeMessage(msg) => WireMsg::serialize_node_msg(msg),
        }
    }
}
