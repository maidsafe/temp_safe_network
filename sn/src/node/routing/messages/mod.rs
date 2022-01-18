// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod msg_authority;

pub(super) use self::msg_authority::NodeMsgAuthorityUtils;

use crate::messaging::{
    system::SystemMsg, BlsShareAuth, DstLocation, MessageId, MsgKind, NodeAuth, WireMsg,
};
use crate::node::{
    error::{Error, Result},
    routing::{network_knowledge::SectionKeyShare, node::Node},
};

use bls::PublicKey as BlsPublicKey;
use xor_name::XorName;

// Utilities for WireMsg.
pub(crate) trait WireMsgUtils {
    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        node_msg: SystemMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg, Error>;

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        node_msg: SystemMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg>;
}

impl WireMsgUtils for WireMsg {
    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        node_msg: SystemMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg, Error> {
        let msg_payload =
            WireMsg::serialize_msg_payload(&node_msg).map_err(|_| Error::InvalidMessage)?;

        let msg_kind = MsgKind::NodeBlsShareAuthMsg(
            BlsShareAuth::authorize(src_section_pk, src_name, key_share, &msg_payload).into_inner(),
        );

        let wire_msg = WireMsg::new_msg(MessageId::new(), msg_payload, msg_kind, dst)?;

        #[cfg(feature = "unstable-wiremsg-debuginfo")]
        let wire_msg = wire_msg.set_payload_debug(node_msg);

        Ok(wire_msg)
    }

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        node_msg: SystemMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg> {
        let msg_payload =
            WireMsg::serialize_msg_payload(&node_msg).map_err(|_| Error::InvalidMessage)?;

        let msg_kind = MsgKind::NodeAuthMsg(
            NodeAuth::authorize(src_section_pk, &node.keypair, &msg_payload).into_inner(),
        );

        let wire_msg = WireMsg::new_msg(MessageId::new(), msg_payload, msg_kind, dst)?;

        #[cfg(feature = "unstable-wiremsg-debuginfo")]
        let wire_msg = wire_msg.set_payload_debug(node_msg);

        Ok(wire_msg)
    }
}
