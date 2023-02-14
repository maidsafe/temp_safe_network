// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{Error, MyNode, Result};

use sn_interface::messaging::{system::NodeMsg, MsgKind, MsgType, WireMsg};

use bytes::Bytes;
use xor_name::XorName;

impl MyNode {
    /// Serialize a message for a Node
    pub(crate) fn serialize_node_msg(our_name: XorName, msg: &NodeMsg) -> Result<(MsgKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(msg)?;
        let kind = MsgKind::Node {
            name: our_name,
            is_join: msg.is_join(),
        };
        Ok((kind, payload))
    }

    /// Serialize a network message
    pub(crate) fn serialize_msg(our_name: XorName, msg: &MsgType) -> Result<(MsgKind, Bytes)> {
        let (payload, kind) = match msg {
            MsgType::AntiEntropy(msg) => (
                WireMsg::serialize_msg_payload(msg)?,
                MsgKind::AntiEntropy(our_name),
            ),
            MsgType::Node(msg) => (
                WireMsg::serialize_msg_payload(msg)?,
                MsgKind::Node {
                    name: our_name,
                    is_join: msg.is_join(),
                },
            ),
            MsgType::ClientDataResponse(msg) => (
                WireMsg::serialize_msg_payload(msg)?,
                MsgKind::ClientDataResponse(our_name),
            ),
            MsgType::Client { .. } => return Err(Error::InvalidMessage),
        };

        Ok((kind, payload))
    }
}
