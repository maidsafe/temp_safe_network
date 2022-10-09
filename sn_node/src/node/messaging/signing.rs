// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::OutgoingMsg;

use crate::node::{MyNode, Result};

use sn_interface::{
    messaging::{data::ClientMsg, system::NodeMsg, AuthKind, ClientAuth, NodeSig, WireMsg},
    types::{PublicKey, Signature},
};

use bytes::Bytes;
use signature::Signer;

// Message handling
impl MyNode {
    /// Signing an outgoing msg.
    ///
    /// We don't need the destination,
    /// as that is always set on the WireMsg
    /// when handled in comms together with specified recipients.
    pub(crate) fn sign_msg(&self, msg: OutgoingMsg) -> Result<(AuthKind, Bytes)> {
        match msg {
            OutgoingMsg::Node(msg) => self.sign_system_msg(msg),
            OutgoingMsg::Client(msg) => self.sign_client_msg(msg),
            OutgoingMsg::SectionAuth((auth, payload)) => {
                Ok((AuthKind::SectionShare(auth), payload))
            }
        }
    }

    /// Currently using node's Ed key. May need to use bls key share for consensus purpose.
    fn sign_client_msg(&self, msg: ClientMsg) -> Result<(AuthKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let signature = self.keypair.sign(&payload);

        let auth = AuthKind::Client(ClientAuth {
            public_key: PublicKey::Ed25519(self.keypair.public),
            signature: Signature::Ed25519(signature),
        });

        Ok((auth, payload))
    }

    /// Currently using node's Ed key. May need to use bls key share for consensus purpose.
    fn sign_system_msg(&self, msg: NodeMsg) -> Result<(AuthKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let src_section_pk = self.network_knowledge.section_key();
        let auth = AuthKind::Node(
            NodeSig::authorize(src_section_pk, &self.keypair, &payload).into_inner(),
        );

        Ok((auth, payload))
    }
}
