// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod plain_message;
mod src_authority;

pub use self::{plain_message::PlainMessageUtils, src_authority::SrcAuthorityUtils};
use crate::messaging::{
    node::{KeyedSig, PlainMessage, RoutingMsg, SigShare, SrcAuthority, Variant},
    Aggregation, DstLocation, MessageId,
};
use crate::routing::{
    ed25519::{self, Verifier},
    error::{Error, Result},
    node::Node,
    section::SectionKeyShare,
};
use bls::PublicKey as BlsPublicKey;
use secured_linked_list::{error::Error as SecuredLinkedListError, SecuredLinkedList};
use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;
use xor_name::XorName;

// View of a message that can be serialized for the purpose of signing.
#[derive(Serialize)]
pub struct SignableView<'a> {
    // TODO: why don't we include also `src`?
    pub dst: &'a DstLocation,
    pub variant: &'a Variant,
}

/// Message sent over the network.
pub(crate) trait RoutingMsgUtils {
    /// Verify this message is properly signed.
    fn check_signature(&self) -> Result<()>;

    // Verify if the section chain of the SrcAuthority can be trusted
    // based on a set of known keys.
    fn verify_src_section_chain(&self, known_keys: &[BlsPublicKey]) -> bool;

    /// Creates a signed message where signature is assumed valid.
    fn new_signed(
        src: SrcAuthority,
        dst: DstLocation,
        variant: Variant,
        section_key: bls::PublicKey,
    ) -> Result<RoutingMsg, Error>;

    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        variant: Variant,
        proof_chain: SecuredLinkedList,
    ) -> Result<RoutingMsg, Error>;

    /// Converts the message src authority from `BlsShare` to `Section` on successful accumulation.
    /// Returns errors if src is not `BlsShare` or if the signed is invalid.
    fn into_dst_accumulated(self, sig: KeyedSig) -> Result<RoutingMsg>;

    fn signable_view(&self) -> SignableView;

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        variant: Variant,
        section_key: bls::PublicKey,
    ) -> Result<RoutingMsg>;

    /// Creates a signed message from a section.
    /// Note: `signed` isn't verified and is assumed valid.
    fn section_src(
        plain: PlainMessage,
        sig: KeyedSig,
        section_chain: SecuredLinkedList,
    ) -> Result<RoutingMsg>;

    /// Getter
    fn keyed_sig(&self) -> Option<KeyedSig>;

    /// Returns an updated message with the provided Section key i.e. known to be latest.
    fn updated_with_latest_key(&mut self, section_pk: bls::PublicKey);
}

impl RoutingMsgUtils for RoutingMsg {
    /// Verify this message is properly signed.
    fn check_signature(&self) -> Result<()> {
        let bytes = bincode::serialize(&self.signable_view()).map_err(|_| Error::InvalidMessage)?;

        match &self.src {
            SrcAuthority::Node {
                public_key,
                signature,
                ..
            } => {
                if public_key.verify(&bytes, signature).is_err() {
                    error!("Failed signature: {:?}", self);
                    return Err(Error::FailedSignature);
                }
            }
            SrcAuthority::BlsShare { sig_share, .. } => {
                // Signed chain is required for accumulation at destination.
                if sig_share.public_key_set.public_key() != self.section_pk {
                    error!(
                        "Signed share public key doesn't match signed chain last key: {:?}",
                        self
                    );
                    return Err(Error::InvalidMessage);
                }

                if !sig_share.verify(&bytes) {
                    error!("Failed signature: {:?}", self);
                    return Err(Error::FailedSignature);
                }
            }
            SrcAuthority::Section { sig, .. } => {
                // Signed chain is required for section-src messages.
                if !self.section_pk.verify(&sig.signature, &bytes) {
                    error!(
                        "Failed signature: {:?} (Section PK: {:?})",
                        self, self.section_pk
                    );
                    return Err(Error::FailedSignature);
                }
            }
        }

        Ok(())
    }

    // Verify if the section chain of the SrcAuthority can be trusted
    // based on a set of known keys.
    fn verify_src_section_chain(&self, known_keys: &[BlsPublicKey]) -> bool {
        match &self.src {
            SrcAuthority::Node { .. } => true,
            SrcAuthority::BlsShare { section_chain, .. }
            | SrcAuthority::Section { section_chain, .. } => {
                section_chain.check_trust(known_keys.iter())
            }
        }
    }

    /// Creates a signed message where signature is assumed valid.
    fn new_signed(
        src: SrcAuthority,
        dst: DstLocation,
        variant: Variant,
        section_pk: bls::PublicKey,
    ) -> Result<RoutingMsg, Error> {
        // Create message id from src authority signature
        let id = match &src {
            SrcAuthority::Node { signature, .. } => MessageId::from_content(signature),
            SrcAuthority::BlsShare { sig_share, .. } => {
                MessageId::from_content(&sig_share.signature_share.0)
            }
            SrcAuthority::Section { sig, .. } => MessageId::from_content(&sig.signature),
        }
        .unwrap_or_default();

        let msg = RoutingMsg {
            id,
            src,
            dst,
            aggregation: Aggregation::None,
            variant,
            section_pk,
        };

        Ok(msg)
    }

    /// Creates a message signed using a BLS KeyShare for destination accumulation
    fn for_dst_accumulation(
        key_share: &SectionKeyShare,
        src_name: XorName,
        dst: DstLocation,
        variant: Variant,
        section_chain: SecuredLinkedList,
    ) -> Result<Self, Error> {
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

        let src = SrcAuthority::BlsShare {
            src_name,
            sig_share,
            section_chain: section_chain.clone(),
        };

        Self::new_signed(src, dst, variant, *section_chain.last_key())
    }

    /// Converts the message src authority from `BlsShare` to `Section` on successful accumulation.
    /// Returns errors if src is not `BlsShare` or if the signed is invalid.
    fn into_dst_accumulated(mut self, sig: KeyedSig) -> Result<Self> {
        let (sig_share, src_name, section_chain) = if let SrcAuthority::BlsShare {
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

        self.src = SrcAuthority::Section {
            sig,
            src_name,
            section_chain: section_chain.clone(),
        };

        Ok(self)
    }

    fn signable_view(&self) -> SignableView {
        SignableView {
            dst: &self.dst,
            variant: &self.variant,
        }
    }

    /// Creates a signed message from single node.
    fn single_src(
        node: &Node,
        dst: DstLocation,
        variant: Variant,
        section_pk: bls::PublicKey,
    ) -> Result<Self> {
        let serialized = bincode::serialize(&SignableView {
            dst: &dst,
            variant: &variant,
        })
        .map_err(|_| Error::InvalidMessage)?;

        let signature = ed25519::sign(&serialized, &node.keypair);
        let src = SrcAuthority::Node {
            public_key: node.keypair.public,
            signature,
        };

        RoutingMsg::new_signed(src, dst, variant, section_pk)
    }

    /// Creates a signed message from a section.
    /// Note: `signed` isn't verified and is assumed valid.
    fn section_src(
        plain: PlainMessage,
        sig: KeyedSig,
        section_chain: SecuredLinkedList,
    ) -> Result<Self> {
        Self::new_signed(
            SrcAuthority::Section {
                src_name: plain.src,
                sig,
                section_chain: section_chain.clone(),
            },
            plain.dst,
            plain.variant,
            *section_chain.last_key(),
        )
    }

    /// Getter
    fn keyed_sig(&self) -> Option<KeyedSig> {
        if let SrcAuthority::Section { sig, .. } = &self.src {
            Some(sig.clone())
        } else {
            None
        }
    }

    fn updated_with_latest_key(&mut self, section_pk: bls::PublicKey) {
        self.section_pk = section_pk
    }
}

/// Error returned from `RoutingMsg::extend_proof_chain`.
#[derive(Debug, Error)]
pub enum ExtendSignedChainError {
    #[error("message has no signed chain")]
    NoSignedChain,
    #[error("failed to extend signed chain: {}", .0)]
    Extend(#[from] SecuredLinkedListError),
}
