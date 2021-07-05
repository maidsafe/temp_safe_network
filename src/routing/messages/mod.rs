// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod msg_authority;
// mod plain_message;

pub use self::msg_authority::NodeMsgAuthorityUtils;
use crate::messaging::{
    node::{KeyedSig, NodeMsg, SigShare},
    BlsShareSigned, ClientSigned, DstLocation, MessageId, MsgKind, NodeMsgAuthority, NodeSigned,
    SectionSigned, WireMsg,
};
use crate::routing::{
    ed25519::{self, Verifier},
    error::{Error, Result},
    node::Node,
    section::SectionKeyShare,
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use xor_name::XorName;

// Utilities for WireMsg.
pub trait WireMsgUtils {
    /// Verify this message is properly signed.
    fn check_signature(&self) -> Result<()>;

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

    /// Converts the message src authority from `BlsShare` to `Section` on successful accumulation.
    /// Returns errors if src is not `BlsShare` or if the signed is invalid.
    fn into_dst_accumulated(&mut self, sig: KeyedSig) -> Result<()>;

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        node_msg: NodeMsg,
        src_section_pk: BlsPublicKey,
    ) -> Result<WireMsg>;
}

impl WireMsgUtils for WireMsg {
    /// Verify this message is properly signed.
    fn check_signature(&self) -> Result<()> {
        match self.msg_kind() {
            MsgKind::SectionInfoMsg => {}
            MsgKind::ClientMsg(ClientSigned {
                public_key,
                signature,
            }) => {
                if public_key.verify(signature, &self.payload).is_err() {
                    error!("Failed signature: {:?}", self);
                    return Err(Error::FailedSignature);
                }
            }
            MsgKind::NodeSignedMsg(NodeSigned {
                public_key,
                signature,
                ..
            }) => {
                if public_key.verify(&self.payload, signature).is_err() {
                    error!("Failed signature: {:?}", self);
                    return Err(Error::FailedSignature);
                }
            }
            MsgKind::NodeBlsShareSignedMsg(BlsShareSigned {
                sig_share,
                section_pk,
                ..
            }) => {
                // Signed chain is required for accumulation at destination.
                if sig_share.public_key_set.public_key() != *section_pk {
                    error!(
                        "Signed share public key doesn't match signed chain last key: {:?}",
                        self
                    );
                    return Err(Error::InvalidMessage);
                }

                if !sig_share.verify(&self.payload) {
                    error!("Failed signature: {:?}", self);
                    return Err(Error::FailedSignature);
                }
            }
            MsgKind::SectionSignedMsg(SectionSigned {
                sig, section_pk, ..
            }) => {
                // Signed chain is required for section-src messages.
                if !section_pk.verify(&sig.signature, &self.payload) {
                    error!(
                        "Failed signature: {:?} (Section PK: {:?})",
                        self, section_pk
                    );
                    return Err(Error::FailedSignature);
                }
            }
        }

        Ok(())
    }

    /// Creates a signed message where signature is assumed valid.
    fn new_signed(
        payload: Bytes,
        node_msg_authority: NodeMsgAuthority,
        dst: DstLocation,
    ) -> Result<WireMsg, Error> {
        // Create message id from msg authority signature
        let (id, msg_kind) = match node_msg_authority {
            NodeMsgAuthority::Node(node_signed) => (
                MessageId::from_content(&node_signed.signature).unwrap_or_default(),
                MsgKind::NodeSignedMsg(node_signed),
            ),
            NodeMsgAuthority::BlsShare(bls_share_signed) => (
                MessageId::from_content(&bls_share_signed.sig_share.signature_share.0)
                    .unwrap_or_default(),
                MsgKind::NodeBlsShareSignedMsg(bls_share_signed),
            ),
            NodeMsgAuthority::Section(section_signed) => (
                MessageId::from_content(&section_signed.sig.signature).unwrap_or_default(),
                MsgKind::SectionSignedMsg(section_signed),
            ),
        };

        let msg = WireMsg::new_msg(id, payload, msg_kind, dst)?;

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

        let signature_share = key_share.secret_key_share.sign(&msg_payload);
        let sig_share = SigShare {
            public_key_set: key_share.public_key_set.clone(),
            index: key_share.index,
            signature_share,
        };

        let msg_authority = NodeMsgAuthority::BlsShare(BlsShareSigned {
            src_name,
            sig_share,
            section_pk: src_section_pk,
        });

        Self::new_signed(msg_payload, msg_authority, dst)
    }

    /// Converts the message src authority from `BlsShare` to `Section` on successful accumulation.
    /// Returns errors if authority  is not `BlsShare` or if the signed is invalid.
    fn into_dst_accumulated(&mut self, sig: KeyedSig) -> Result<()> {
        let (section_pk, src_name, sig_share) =
            if let MsgKind::NodeBlsShareSignedMsg(BlsShareSigned {
                section_pk,
                src_name,
                sig_share,
            }) = self.msg_kind()
            {
                (*section_pk, *src_name, sig_share)
            } else {
                error!("Not a message for dst accumulation: {:?}", self);
                return Err(Error::InvalidMessage);
            };

        if sig_share.public_key_set.public_key() != sig.public_key {
            error!(
                "Signed public key doesn't match signed share public key: {:?}",
                self
            );
            return Err(Error::InvalidMessage);
        }

        if sig.public_key != section_pk {
            error!("Signed public key doesn't match the section PK: {:?}", self);
            return Err(Error::InvalidMessage);
        }

        if !sig.verify(&self.payload) {
            return Err(Error::FailedSignature);
        }

        self.header.msg_envelope.msg_kind = MsgKind::SectionSignedMsg(SectionSigned {
            section_pk,
            src_name,
            sig,
        });

        Ok(())
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

        let signature = ed25519::sign(&msg_payload, &node.keypair);
        let msg_authority = NodeMsgAuthority::Node(NodeSigned {
            public_key: node.keypair.public,
            section_pk: src_section_pk,
            signature,
        });

        WireMsg::new_signed(msg_payload, msg_authority, dst)
    }
}
