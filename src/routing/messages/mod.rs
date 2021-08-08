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
    section::{SectionKeyShare, Signer},
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use xor_name::XorName;

// Utilities for WireMsg.
pub(crate) trait WireMsgUtils {
    /// Return 'true' if the message kind is MsgKind::ServiceMsg or MsgKind::SectionInfoMsg
    fn is_client_msg_kind(&self) -> bool;

    /// Creates a signed message where signature is assumed valid.
    fn new_signed(
        payload: Bytes,
        node_msg_authority: NodeMsgAuthority,
        dst: DstLocation,
    ) -> Result<WireMsg, Error>;

    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: SectionKeyShare<impl Signer>,
        src_name: XorName,
        dst: DstLocation,
        node_msg: NodeMsg,
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
    /// Return 'true' if the message kind is MsgKind::ServiceMsg or MsgKind::SectionInfoMsg
    fn is_client_msg_kind(&self) -> bool {
        matches!(
            self.msg_kind(),
            MsgKind::ServiceMsg(_) | MsgKind::SectionInfoMsg
        )
    }

    /// Creates a signed message where signature is known to be valid.
    fn new_signed(
        payload: Bytes,
        node_msg_authority: NodeMsgAuthority,
        dst: DstLocation,
    ) -> Result<WireMsg, Error> {
        // Create message id from msg authority signature
        let (id, msg_kind) = match node_msg_authority {
            NodeMsgAuthority::Node(node_auth) => (
                MessageId::from_content(&node_auth.signature).unwrap_or_default(),
                MsgKind::NodeAuthMsg(node_auth.into_inner()),
            ),
            NodeMsgAuthority::BlsShare(bls_share_auth) => (
                MessageId::from_content(&bls_share_auth.sig_share.signature_share.0)
                    .unwrap_or_default(),
                MsgKind::NodeBlsShareAuthMsg(bls_share_auth.into_inner()),
            ),
            NodeMsgAuthority::Section(section_auth) => (
                MessageId::from_content(&section_auth.sig.signature).unwrap_or_default(),
                MsgKind::SectionAuthMsg(section_auth.into_inner()),
            ),
        };

        let msg = WireMsg::new_msg(id, payload, msg_kind, dst)?;

        Ok(msg)
    }

    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: SectionKeyShare<impl Signer>,
        src_name: XorName,
        dst: DstLocation,
        node_msg: NodeMsg,
    ) -> Result<WireMsg, Error> {
        let msg_payload =
            WireMsg::serialize_msg_payload(&node_msg).map_err(|_| Error::InvalidMessage)?;

        let msg_authority =
            NodeMsgAuthority::BlsShare(BlsShareAuth::authorize(src_name, key_share, &msg_payload));

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
