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
pub mod node;
mod serialisation;

use bytes::Bytes;
pub use errors::{Error, Result};
pub use serialisation::WireMsg;

/// Type of message
#[derive(PartialEq, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    Ping,
    InfrastructureQuery(infrastructure::Query),
    ClientMessage(client::MsgEnvelope),
    NodeMessage(node::NodeMessage),
}

impl MessageType {
    /// Serialise the message type into bytes ready to be sent over the wire.
    pub fn serialise(&self) -> Result<Bytes> {
        match self {
            Self::Ping => WireMsg::new_ping_msg().serialise(),
            Self::InfrastructureQuery(query) => WireMsg::serialise_infrastructure_query(query),
            Self::ClientMessage(msg) => WireMsg::serialise_client_msg(msg),
            Self::NodeMessage(msg) => WireMsg::serialise_node_msg(msg),
        }
    }
}
