// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{MyNode, Result};

use bytes::Bytes;
use ed25519_dalek::Keypair;
use ed25519_dalek::Signer;
use sn_interface::{
    messaging::{data::ClientMsg, system::NodeMsg, ClientAuth, MsgKind, WireMsg},
    types::{PublicKey, Signature},
};
use std::sync::Arc;
use xor_name::XorName;

impl MyNode {
    /// Serialize and sign a message for a Client with our ed key
    pub(crate) fn serialize_sign_client_msg(
        keypair: Arc<Keypair>,
        msg: ClientMsg,
    ) -> Result<(MsgKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let signature = keypair.sign(&payload);

        let kind = MsgKind::Client(ClientAuth {
            public_key: PublicKey::Ed25519(keypair.public),
            signature: Signature::Ed25519(signature),
        });

        Ok((kind, payload))
    }

    /// Serialize a message for a Node with our ed key
    pub(crate) fn serialize_node_msg(
        our_node_name: XorName,
        msg: NodeMsg,
    ) -> Result<(MsgKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let kind = MsgKind::Node(our_node_name);

        Ok((kind, payload))
    }
}
