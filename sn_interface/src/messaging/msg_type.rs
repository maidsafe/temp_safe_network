// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::Dst;

use super::{
    data::{ClientDataResponse, ClientMsg},
    system::{NodeDataResponse, NodeMsg},
    AuthorityProof, ClientAuth, MsgId,
};
use std::fmt::{Display, Formatter};

/// Type of message.
/// Note this is part of this crate's public API but this enum is
/// never serialised or even part of the message that is sent over the wire.
#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MsgType {
    /// Message from client to nodes.
    Client {
        /// Message ID
        msg_id: MsgId,
        /// Requester's authority over this message
        auth: AuthorityProof<ClientAuth>,
        /// Message dst
        dst: Dst,
        /// the message
        msg: ClientMsg,
    },
    /// Message response for clients sent by nodes.
    ClientDataResponse {
        /// Message ID
        msg_id: MsgId,
        /// the message
        msg: ClientDataResponse,
    },
    /// System message for node<->node comms.
    Node {
        /// Message ID
        msg_id: MsgId,
        /// Message dst
        dst: Dst,
        /// the message
        msg: NodeMsg,
    },
    /// The response to a NodeDataCmd or NodeDataQuery, containing the result.
    NodeDataResponse {
        /// Message ID
        msg_id: MsgId,
        /// The message
        msg: NodeDataResponse,
    },
}

impl Display for MsgType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node { msg, .. } => write!(f, "MsgType::Node({})", msg),
            Self::Client { msg, .. } => write!(f, "MsgType::Client({})", msg),
            Self::ClientDataResponse { msg, .. } => {
                write!(f, "MsgType::ClientDataResponse({})", msg)
            }
            Self::NodeDataResponse { msg, .. } => write!(f, "MsgType::NodeDataResponse({})", msg),
        }
    }
}
