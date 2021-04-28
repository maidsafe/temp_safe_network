// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[cfg(not(feature = "client-only"))]
use crate::node::NodeMsg;
use crate::{client::ClientMsg, MessageType, Result, WireMsg};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use threshold_crypto::PublicKey as BlsPublicKey;
use xor_name::XorName;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum Msg {
    #[cfg(not(feature = "client-only"))]
    Node(NodeMsg),
    Client(ClientMsg),
}

impl Msg {
    /// Serializes the msg
    pub fn serialize(
        &self,
        dest: XorName,
        dest_section_pk: BlsPublicKey,
        #[cfg(not(feature = "client-only"))] src_section_pk: Option<BlsPublicKey>,
    ) -> Result<Bytes> {
        match self {
            #[cfg(not(feature = "client-only"))]
            Msg::Node(msg) => msg.serialize(dest, dest_section_pk, src_section_pk),
            Msg::Client(msg) => msg.serialize(dest, dest_section_pk),
        }
    }

    /// Convenience function to deserialize a 'Msg' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a node or client message.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        match deserialized {
            MessageType::Client { msg, .. } => Ok(Msg::Client(msg)),
            #[cfg(not(feature = "client-only"))]
            MessageType::Node { msg, .. } => Ok(Msg::Node(msg)),
            _ => Err(crate::Error::FailedToParse("bytes as a msg".to_string())),
        }
    }
}
