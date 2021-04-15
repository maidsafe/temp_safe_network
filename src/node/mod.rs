// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod node_msg;

use crate::{MessageType, WireMsg};
use bytes::Bytes;
pub use node_msg::{
    NodeCmd, NodeCmdError, NodeDataError, NodeDataQueryResponse, NodeEvent, NodeMsg, NodeQuery,
    NodeQueryResponse, NodeRewardQuery, NodeSystemCmd, NodeSystemQuery, NodeSystemQueryResponse,
    NodeTransferCmd, NodeTransferError, NodeTransferQuery, NodeTransferQueryResponse,
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Formatter};
use threshold_crypto::PublicKey as BlsPublicKey;
use xor_name::XorName;

/// Node message sent over the network.
// TODO: this is currently holding just bytes as a placeholder, next step
// is to move all actual node messages structs and definitions within it.
#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct RoutingMsg(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl RoutingMsg {
    /// Creates a new instance which wraps the provided node message bytes.
    pub fn new(bytes: Bytes) -> Self {
        Self(bytes.to_vec())
    }

    /// Convenience function to deserialize a 'RoutingMsg' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a node message.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::Routing { msg, .. } = deserialized {
            Ok(msg)
        } else {
            Err(crate::Error::FailedToParse(
                "bytes as a node message".to_string(),
            ))
        }
    }

    /// serialize this RoutingMsg into bytes ready to be sent over the wire.
    pub fn serialize(&self, dest: XorName, dest_section_pk: BlsPublicKey) -> crate::Result<Bytes> {
        WireMsg::serialize_routing_msg(self, dest, dest_section_pk)
    }
}

impl PartialEq for RoutingMsg {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Debug for RoutingMsg {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter
            .debug_tuple("RoutingMsg")
            .field(&format_args!("{:10}", hex_fmt::HexFmt(&self.0)))
            .finish()
    }
}
