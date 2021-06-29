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
    node::{KeyedSig, NodeMsg, PlainMessage, SigShare},
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
use secured_linked_list::{error::Error as SecuredLinkedListError, SecuredLinkedList};
use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;
use xor_name::XorName;

// Utilities for WireMsg.
pub(crate) trait WireMsgUtils {
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
        proof_chain: SecuredLinkedList,
    ) -> Result<WireMsg, Error>;

    /// Converts the message src authority from `BlsShare` to `Section` on successful accumulation.
    /// Returns errors if src is not `BlsShare` or if the signed is invalid.
    fn into_dst_accumulated(self, sig: KeyedSig) -> Result<WireMsg>;

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        node_msg: NodeMsg,
        section_key: BlsPublicKey,
    ) -> Result<WireMsg>;

    /// Creates a signed message from a section.
    /// Note: `signed` isn't verified and is assumed valid.
    fn section_src(
        plain: PlainMessage,
        sig: KeyedSig,
        section_chain: SecuredLinkedList,
    ) -> Result<WireMsg>;

    /// Getter
    fn keyed_sig(&self) -> Option<KeyedSig>;

    /// Returns an updated message with the provided Section key i.e. known to be latest.
    fn updated_with_latest_key(&mut self, section_pk: BlsPublicKey);
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
        let (id, msg_kind) = match &node_msg_authority {
            NodeMsgAuthority::Node(node_signed) => (
                MessageId::from_content(&node_signed.signature),
                MsgKind::NodeSignedMsg(*node_signed),
            ),
            NodeMsgAuthority::BlsShare(bls_share_signed) => (
                MessageId::from_content(&bls_share_signed.sig_share.signature_share.0),
                MsgKind::NodeBlsShareSignedMsg(*bls_share_signed),
            ),
            NodeMsgAuthority::Section(section_signed) => (
                MessageId::from_content(&section_signed.sig.signature),
                MsgKind::SectionSignedMsg(*section_signed),
            ),
        };

        let msg = WireMsg::new_msg(id.unwrap_or_default(), payload, msg_kind, dst)?;

        Ok(msg)
    }

    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        node_msg: NodeMsg,
        section_chain: SecuredLinkedList,
    ) -> Result<WireMsg, Error> {
        unimplemented!();
        /*
        let serialized = bincode::serialize(&SignableView {
            dst: &dst,
            variant: &variant,
        })
        .map_err(|_| Error::InvalidMessage)?;

        let signature_share = key_share.secret_key_share.sign(&serialized);
        let sig_share = SigShare {
            public_key_set: key_share.public_key_set.clone(),
            index: key_share.index,
            signature_share,
        };

        let src = MsgAuthority::BlsShare {
            src_name,
            sig_share,
            section_chain: section_chain.clone(),
        };

        Self::new_signed(src, dst, variant, *section_chain.last_key())
        */
    }

    /// Converts the message src authority from `BlsShare` to `Section` on successful accumulation.
    /// Returns errors if src is not `BlsShare` or if the signed is invalid.
    fn into_dst_accumulated(mut self, sig: KeyedSig) -> Result<WireMsg> {
        unimplemented!();
        /*let (sig_share, src_name, section_chain) = if let MsgAuthority::BlsShare {
            sig_share,
            src_name,
            section_chain,
        } = &self.src
        {
            (sig_share.clone(), *src_name, section_chain)
        } else {
            error!("not a message for dst accumulation");
            return Err(Error::InvalidMessage);
        };

        if sig_share.public_key_set.public_key() != sig.public_key {
            error!("signed public key doesn't match signed share public key");
            return Err(Error::InvalidMessage);
        }

        if sig.public_key != self.section_pk {
            error!("signed public key doesn't match the attached section PK");
            return Err(Error::InvalidMessage);
        }

        let bytes = bincode::serialize(&self.signable_view()).map_err(|_| Error::InvalidMessage)?;

        if !sig.verify(&bytes) {
            return Err(Error::FailedSignature);
        }

        self.src = MsgAuthority::Section {
            sig,
            src_name,
            section_chain: section_chain.clone(),
        };

        Ok(self)*/
    }

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        node_msg: NodeMsg,
        section_pk: BlsPublicKey,
    ) -> Result<WireMsg> {
        let msg_payload =
            WireMsg::serialize_msg_payload(&NodeMsg).map_err(|_| Error::InvalidMessage)?;

        let signature = ed25519::sign(&msg_payload, &node.keypair);
        let msg_authority = NodeMsgAuthority::Node(NodeSigned {
            public_key: node.keypair.public,
            section_pk,
            signature,
        });

        WireMsg::new_signed(msg_payload, msg_authority, dst)
    }

    /// Creates a signed message from a section.
    /// Note: `signed` isn't verified and is assumed valid.
    fn section_src(
        plain: PlainMessage,
        sig: KeyedSig,
        section_chain: SecuredLinkedList,
    ) -> Result<WireMsg> {
        unimplemented!();
        /*
                // TODO: ideally we should get rid of SignableView and sign just the message
                let msg_payload = WireMsg::serialize_msg_payload(&SignableView {
                    dst: &plain.dst,
                    variant: &plain.variant,
                })
                .map_err(|_| Error::InvalidMessage)?;

                let msg_authority = MsgAuthority::Section {
                    src_name: plain.src,
                    sig,
                    section_pk: section_chain.last_key(),
                };

                WireMsg::new_signed(msg_payload, msg_authority, plain.dst)
        */
    }

    /// Getter
    fn keyed_sig(&self) -> Option<KeyedSig> {
        unimplemented!();
        /*        if let MsgAuthority::Section { sig, .. } = &self.src {
            Some(sig.clone())
        } else {
            None
        }
        */
    }

    fn updated_with_latest_key(&mut self, section_pk: BlsPublicKey) {
        unimplemented!();
        //self.section_pk = section_pk
    }
}

/// Error returned from `NodeMsg::extend_proof_chain`.
#[derive(Debug, Error)]
pub enum ExtendSignedChainError {
    #[error("message has no signed chain")]
    NoSignedChain,
    #[error("failed to extend signed chain: {}", .0)]
    Extend(#[from] SecuredLinkedListError),
}
