// Copyright 2022 MaidSafe.net limited.
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

// highest priority, since we must sort out membership first of all
pub(crate) const DKG_MSG_PRIORITY: i32 = 8;
// very high prio, since we must have correct contact details to the network
pub(crate) const ANTIENTROPY_MSG_PRIORITY: i32 = 6;
// high prio as recipient can't do anything until they've joined. Needs to be lower than DKG (or else no split)
pub(crate) const JOIN_RESPONSE_PRIORITY: i32 = 4;
// Membership changes
pub(crate) const MEMBERSHIP_PRIORITY: i32 = 4;
// our joining to the network
pub(crate) const JOIN_RELOCATE_MSG_PRIORITY: i32 = 2;
// not maintaining network structure, so can wait
pub(crate) const DATA_REPLICATION_MSG_PRIORITY: i32 = 3;
// not maintaining network structure, so can wait
pub(crate) const NODE_DATA_MSG_PRIORITY: i32 = -6;
// has payment throttle, but is not critical for network function
pub(crate) const CLIENT_CMD_PRIORITY: i32 = -8;
// has no throttle and is sent by clients, lowest prio
pub(crate) const CLIENT_QUERY_PRIORITY: i32 = -10;

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
        // the sender name
        sender: xor_name::XorName,
    },
    /// The response to a NodeDataCmd or NodeDataQuery, containing the result.
    NodeDataResponse {
        /// Message ID
        msg_id: MsgId,
        /// The message
        msg: NodeDataResponse,
    },
}

impl MsgType {
    /// The priority of the message, when handled by lower level comms.
    pub fn priority(&self) -> i32 {
        match self {
            // node <-> node system comms
            Self::Node { msg, .. } => msg.priority(),
            // client <-> node service comms
            Self::Client { msg, .. } => msg.priority(),
            Self::ClientDataResponse { msg, .. } => msg.priority(),
            Self::NodeDataResponse { msg, .. } => msg.priority(),
        }
    }
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
