// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod errors;

pub use self::errors::{Error, Result};
use crate::{MessageType, WireMsg};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Formatter};

/// Node message sent over the network.
// TODO: this is currently holding just bytes as a placeholder, next step
// is to move all actual node messages structs and definitions within it.
#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct NodeMessage(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl NodeMessage {
    /// Creates a new instance which wraps the provided node message bytes.
    pub fn new(bytes: Bytes) -> Self {
        Self(bytes.to_vec())
    }

    /// Convinience function to deserialise a 'NodeMessage' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a node message.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialised = WireMsg::deserialise(bytes)?;
        if let MessageType::NodeMessage(msg) = deserialised {
            Ok(msg)
        } else {
            Err(crate::Error::FailedToParse(
                "bytes as a node message".to_string(),
            ))
        }
    }

    /// Serialise this NodeMessage into bytes ready to be sent over the wire.
    pub fn serialise(&self) -> crate::Result<Bytes> {
        WireMsg::serialise_node_msg(self)
    }
}

impl PartialEq for NodeMessage {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Debug for NodeMessage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter
            .debug_struct("NodeMessage")
            .field("bytes", &self.0)
            .finish()
    }
}
