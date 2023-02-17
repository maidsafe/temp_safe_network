// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    data::{ClientDataResponse, ClientMsg},
    system::NodeMsg,
    AntiEntropyMsg, AuthorityProof, ClientAuth,
};
use std::fmt::{Display, Formatter};

/// Type of message.
/// Note this is part of this crate's public API but this enum is
/// never serialised or even part of the message that is sent over the wire.
#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MsgType {
    /// Msgs for synchronizing network membership state.
    AntiEntropy(AntiEntropyMsg),
    /// Msg from client to nodes.
    Client {
        /// Requester's authority over this msg.
        auth: AuthorityProof<ClientAuth>,
        /// The msg from requester.
        msg: ClientMsg,
    },
    /// Data response msg.
    ClientDataResponse(ClientDataResponse),
    /// Msg for node<->node comms.
    Node(NodeMsg),
}

impl Display for MsgType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AntiEntropy(msg) => write!(f, "MsgType::AntiEntropy({msg:?})"),
            Self::Client { msg, .. } => write!(f, "MsgType::Client({msg})"),
            Self::Node(msg) => write!(f, "MsgType::Node({msg})"),
            Self::ClientDataResponse(msg) => {
                write!(f, "MsgType::ClientDataResponse({msg:?})")
            }
        }
    }
}
