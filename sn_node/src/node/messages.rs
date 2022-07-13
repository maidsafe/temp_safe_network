// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{Error, Result};

use sn_interface::{
    messaging::{
        system::{SigShare, SystemMsg},
        AuthKind, AuthorityProof, BlsShareAuth, DstLocation, MsgId, NodeAuth, WireMsg,
    },
    network_knowledge::{NodeInfo, SectionKeyShare},
};

use bls::PublicKey as BlsPublicKey;
use xor_name::XorName;

// Utilities for WireMsg.
pub(crate) trait WireMsgUtils {
    /// Creates a message signed using a BLS `KeyShare` for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        node_msg: SystemMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg, Error>;

    /// Creates a signed message from single node.
    fn single_src(
        node: &NodeInfo,
        dst: DstLocation,
        node_msg: SystemMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg>;
}

impl WireMsgUtils for WireMsg {
    /// Creates a message signed using a BLS `KeyShare` for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        msg: SystemMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg, Error> {
        let msg_payload =
            WireMsg::serialize_msg_payload(&msg).map_err(|_| Error::InvalidMessage)?;

        let auth_kind = AuthKind::NodeBlsShare(
            bls_share_authorize(src_section_pk, src_name, key_share, &msg_payload).into_inner(),
        );

        let wire_msg = WireMsg::new_msg(MsgId::new(), msg_payload, auth_kind, dst)?;

        #[cfg(feature = "test-utils")]
        let wire_msg = wire_msg.set_payload_debug(msg);

        Ok(wire_msg)
    }

    /// Creates a signed message from single node.
    fn single_src(
        node: &NodeInfo,
        dst: DstLocation,
        msg: SystemMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg> {
        let msg_payload =
            WireMsg::serialize_msg_payload(&msg).map_err(|_| Error::InvalidMessage)?;

        let auth_kind = AuthKind::Node(
            NodeAuth::authorize(src_section_pk, &node.keypair, &msg_payload).into_inner(),
        );

        let wire_msg = WireMsg::new_msg(MsgId::new(), msg_payload, auth_kind, dst)?;

        #[cfg(feature = "test-utils")]
        let wire_msg = wire_msg.set_payload_debug(msg);

        Ok(wire_msg)
    }
}

// Construct verified authority of a single node's share of section authority.
fn bls_share_authorize(
    section_pk: BlsPublicKey,
    src_name: XorName,
    key_share: &SectionKeyShare,
    payload: impl AsRef<[u8]>,
) -> AuthorityProof<BlsShareAuth> {
    AuthorityProof(BlsShareAuth {
        section_pk,
        src_name,
        sig_share: SigShare {
            public_key_set: key_share.public_key_set.clone(),
            index: key_share.index,
            signature_share: key_share.secret_key_share.sign(payload),
        },
    })
}
