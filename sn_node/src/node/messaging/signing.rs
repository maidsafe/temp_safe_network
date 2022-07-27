// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::OutgoingMsg;

use crate::node::{Node, Result};

#[cfg(feature = "traceroute")]
use sn_interface::messaging::Entity;
use sn_interface::{
    messaging::{
        data::ServiceMsg, system::SystemMsg, AuthKind, DstLocation, MsgId, NodeAuth, ServiceAuth,
        WireMsg,
    },
    types::{PublicKey, Signature},
};

use bytes::Bytes;
use signature::Signer;
use xor_name::XorName;

// Message handling
impl Node {
    /// Signing an outgoing msg.
    ///
    /// We don't need the destination,
    /// as that is always set on the WireMsg
    /// when handled in comms together with specified recipients.
    pub(crate) fn sign_msg(
        &self,
        msg: OutgoingMsg,
        #[cfg(feature = "traceroute")] traceroute: Vec<Entity>,
    ) -> Result<WireMsg> {
        let (auth, payload) = match msg {
            OutgoingMsg::System(msg) => self.sign_system_msg(msg)?,
            OutgoingMsg::Service(msg) => self.sign_service_msg(msg)?,
            OutgoingMsg::DstAggregated((auth, payload)) => (AuthKind::NodeBlsShare(auth), payload),
        };

        #[allow(unused_mut)]
        let mut wire_msg = WireMsg::new_msg(MsgId::new(), payload, auth, self.dst())?;

        #[cfg(feature = "traceroute")]
        {
            let mut trace = traceroute;
            trace.push(self.entity());
            wire_msg.add_trace(&mut trace);
        }

        #[cfg(feature = "test-utils")]
        let wire_msg = wire_msg.set_payload_debug(msg);

        Ok(wire_msg)
    }

    /// Currently using node's Ed key. May need to use bls key share for consensus purpose.
    fn sign_service_msg(&self, msg: ServiceMsg) -> Result<(AuthKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let signature = self.keypair.sign(&payload);

        let auth = AuthKind::Service(ServiceAuth {
            public_key: PublicKey::Ed25519(self.keypair.public),
            signature: Signature::Ed25519(signature),
        });

        Ok((auth, payload))
    }

    /// Currently using node's Ed key. May need to use bls key share for consensus purpose.
    fn sign_system_msg(&self, msg: SystemMsg) -> Result<(AuthKind, Bytes)> {
        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let src_section_pk = self.network_knowledge.section_key();
        let auth = AuthKind::Node(
            NodeAuth::authorize(src_section_pk, &self.keypair, &payload).into_inner(),
        );

        Ok((auth, payload))
    }

    fn dst(&self) -> DstLocation {
        let section_pk = self.network_knowledge.section_key();
        let name = XorName::from_content(&[]); // random name because it is overwritten in comms (which is somewhat iffy.. but tbd)
        DstLocation::Section { name, section_pk }
    }

    #[cfg(feature = "traceroute")]
    fn entity(&self) -> Entity {
        if self.is_elder() {
            Entity::Elder(PublicKey::Ed25519(self.info().keypair.public))
        } else {
            Entity::Adult(PublicKey::Ed25519(self.info().keypair.public))
        }
    }
}
