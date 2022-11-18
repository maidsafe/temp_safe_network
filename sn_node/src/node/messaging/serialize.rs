// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{MyNode, Result};

use sn_interface::messaging::{data::ClientMsgResponse, system::NodeMsg, MsgKind, WireMsg};

use bytes::Bytes;
use xor_name::XorName;

impl MyNode {
    /// Serialize a message for a Client
    pub(crate) fn serialize_client_msg_response(
        our_node_name: XorName,
        msg: ClientMsgResponse,
    ) -> Result<(MsgKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let kind = MsgKind::ClientMsgResponse(our_node_name);

        Ok((kind, payload))
    }

    /// Serialize a message for a Node
    pub(crate) fn serialize_node_msg(
        our_node_name: XorName,
        msg: NodeMsg,
    ) -> Result<(MsgKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let kind = MsgKind::Node(our_node_name);

        Ok((kind, payload))
    }
}
