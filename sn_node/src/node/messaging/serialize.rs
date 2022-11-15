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

impl MyNode {
    /// Serialize a message for a Client
    pub(crate) fn serialize_client_msg_response(
        &self,
        msg: ClientMsgResponse,
    ) -> Result<(MsgKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let kind = MsgKind::ClientMsgResponse(self.name());

        Ok((kind, payload))
    }

    /// Serialize a message for a Node
    pub(crate) fn serialize_node_msg(&self, msg: NodeMsg) -> Result<(MsgKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let kind = MsgKind::Node(self.name());

        Ok((kind, payload))
    }
}
