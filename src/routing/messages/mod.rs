// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod msg_authority;

pub(super) use self::msg_authority::NodeMsgAuthorityUtils;
use crate::messaging::{
    node::NodeMsg, BlsShareAuth, DstLocation, MessageId, MsgKind, NodeAuth, NodeMsgAuthority,
    WireMsg,
};
use crate::routing::{
    error::{Error, Result},
    node::Node,
    section::SectionKeyShare,
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use xor_name::XorName;

// Utilities for WireMsg.
pub(crate) trait WireMsgUtils {
    /// Creates a signed message where signature is assumed valid.
    fn new_signed(
        payload: Bytes,
        node_msg_authority: NodeMsgAuthority,
        dst: DstLocation,
    ) -> Result<WireMsg, Error>;

    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        node_msg: NodeMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg, Error>;

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        node_msg: NodeMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg>;
}

impl WireMsgUtils for WireMsg {
    /// Creates a signed message where signature is known to be valid.
    fn new_signed(
        payload: Bytes,
        node_msg_authority: NodeMsgAuthority,
        dst: DstLocation,
    ) -> Result<WireMsg, Error> {
        // Create message id from msg authority signature
        let msg_kind = match node_msg_authority {
            NodeMsgAuthority::Node(node_auth) => MsgKind::NodeAuthMsg(node_auth.into_inner()),
            NodeMsgAuthority::BlsShare(bls_share_auth) => {
                MsgKind::NodeBlsShareAuthMsg(bls_share_auth.into_inner())
            }
            NodeMsgAuthority::Section(section_auth) => {
                MsgKind::SectionAuthMsg(section_auth.into_inner())
            }
        };

        let msg = WireMsg::new_msg(MessageId::new(), payload, msg_kind, dst)?;

        Ok(msg)
    }

    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        node_msg: NodeMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg, Error> {
        let msg_payload =
            WireMsg::serialize_msg_payload(&node_msg).map_err(|_| Error::InvalidMessage)?;

        let msg_authority = NodeMsgAuthority::BlsShare(BlsShareAuth::authorize(
            src_section_pk,
            src_name,
            key_share,
            &msg_payload,
        ));

        Self::new_signed(msg_payload, msg_authority, dst)
    }

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        node_msg: NodeMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg> {
        let msg_payload =
            WireMsg::serialize_msg_payload(&node_msg).map_err(|_| Error::InvalidMessage)?;

        let msg_authority = NodeMsgAuthority::Node(NodeAuth::authorize(
            src_section_pk,
            &node.keypair,
            &msg_payload,
        ));

        WireMsg::new_signed(msg_payload, msg_authority, dst)
    }
}
