use super::{
    node::{KeyedSig, SigShare},
    Error, Result,
};
use crate::{
    routing::{AggregatorError, SectionKeyShare, SignatureAggregator},
    types::{PublicKey, Signature},
};
use bls::PublicKey as BlsPublicKey;
use ed25519_dalek::{
    Keypair as EdKeypair, PublicKey as EdPublicKey, Signature as EdSignature, Signer as _,
    Verifier as _,
};
use xor_name::XorName;

/// Authority of a network peer.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ServiceAuth {
    /// Peer's public key.
    pub node_pk: PublicKey,
    /// Peer's signature.
    pub signature: Signature,
}

/// Authority of a single peer.
#[derive(Clone, Eq, PartialEq, custom_debug::Debug, serde::Deserialize, serde::Serialize)]
pub struct NodeAuth {
    /// Section key of the source.
    pub section_pk: BlsPublicKey,
    /// Public key of the source peer.
    #[debug(with = "PublicKey::fmt_ed25519")]
    pub node_pk: EdPublicKey,
    /// Ed25519 signature of the message corresponding to the public key of the source peer.
    #[debug(with = "Signature::fmt_ed25519")]
    #[serde(with = "serde_bytes")]
    pub signature: EdSignature,
}

impl NodeAuth {
    /// Construct verified node authority by signing a payload.
    pub(crate) fn authorize(
        section_pk: BlsPublicKey,
        keypair: &EdKeypair,
        payload: impl AsRef<[u8]>,
    ) -> AuthorityProof<Self> {
        AuthorityProof(NodeAuth {
            section_pk,
            node_pk: keypair.public,
            signature: keypair.sign(payload.as_ref()),
        })
    }
}

/// Authority of a single peer that uses it's BLS Keyshare to sign the message.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BlsShareAuth {
    /// Section key of the source.
    pub section_pk: BlsPublicKey,
    /// Name in the source section.
    pub src_name: XorName,
    /// Proof Share signed by the peer's BLS KeyShare.
    pub sig_share: SigShare,
}

impl BlsShareAuth {
    /// Construct verified authority of a single node's share of section authority.
    pub(crate) fn authorize(
        section_pk: BlsPublicKey,
        src_name: XorName,
        key_share: &SectionKeyShare,
        payload: impl AsRef<[u8]>,
    ) -> AuthorityProof<Self> {
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
}

/// Authority of a whole section.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SectionAuth {
    /// Section key of the source.
    pub section_pk: BlsPublicKey,
    /// Name in the source section.
    pub src_name: XorName,
    /// BLS proof of the message corresponding to the source section.
    pub sig: KeyedSig,
}

impl SectionAuth {
    /// Try to construct verified section authority by aggregating a new share.
    pub(crate) fn try_authorize(
        aggregator: &mut SignatureAggregator,
        share: BlsShareAuth,
        payload: impl AsRef<[u8]>,
    ) -> Result<AuthorityProof<Self>, AggregatorError> {
        let sig = aggregator.add(payload.as_ref(), share.sig_share.clone())?;

        if share.sig_share.public_key_set.public_key() != sig.public_key {
            return Err(AggregatorError::InvalidShare);
        }

        if sig.public_key != share.section_pk {
            return Err(AggregatorError::InvalidShare);
        }

        Ok(AuthorityProof(SectionAuth {
            section_pk: share.section_pk,
            src_name: share.src_name,
            sig,
        }))
    }
}

/// Verified authority.
///
/// Values of this type constitute a proof that the signature is valid for a particular payload.
/// This is made possible by performing verification in all possible constructors of the type.
///
/// Validation is defined by the [`VerifyAuthority`] impl for `T`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityProof<T>(T);

impl<T: VerifyAuthority> AuthorityProof<T> {
    /// Verify the authority of `inner`.
    ///
    /// This is the only way to construct an instance of [`Authority`] from a `T`. Since it's
    /// implemented to call [`VerifyAuthority::verify_authority`] an instance of `AuthorityProof<T>` is
    /// guaranteed to be valid with respect to that trait's impl.
    pub fn verify(inner: T, payload: impl AsRef<[u8]>) -> Result<Self> {
        inner.verify_authority(payload).map(Self)
    }

    /// Drop the proof of validity and return the wrapped value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> core::ops::Deref for AuthorityProof<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Verify authority.
///
/// This trait drives the verification logic used by [`Authority`].
///
/// **Note:** this trait is 'sealed', and as such cannot be implemented outside of this crate.
pub trait VerifyAuthority: Sized + sealed::Sealed {
    /// Verify that we represent authority for `payload`.
    fn verify_authority(self, payload: impl AsRef<[u8]>) -> Result<Self>;
}

impl VerifyAuthority for ServiceAuth {
    fn verify_authority(self, payload: impl AsRef<[u8]>) -> Result<Self> {
        self.node_pk
            .verify(&self.signature, payload)
            .map_err(|_| Error::InvalidSignature)?;
        Ok(self)
    }
}
impl sealed::Sealed for ServiceAuth {}

impl VerifyAuthority for NodeAuth {
    fn verify_authority(self, payload: impl AsRef<[u8]>) -> Result<Self> {
        self.node_pk
            .verify(payload.as_ref(), &self.signature)
            .map_err(|_| Error::InvalidSignature)?;
        Ok(self)
    }
}
impl sealed::Sealed for NodeAuth {}

impl VerifyAuthority for BlsShareAuth {
    fn verify_authority(self, payload: impl AsRef<[u8]>) -> Result<Self> {
        // Signed chain is required for accumulation at destination.
        if self.sig_share.public_key_set.public_key() != self.section_pk {
            return Err(Error::InvalidSignature);
        }

        if !self.sig_share.verify(payload.as_ref()) {
            return Err(Error::InvalidSignature);
        }

        Ok(self)
    }
}
impl sealed::Sealed for BlsShareAuth {}

impl VerifyAuthority for SectionAuth {
    fn verify_authority(self, payload: impl AsRef<[u8]>) -> Result<Self> {
        if !self.section_pk.verify(&self.sig.signature, payload) {
            return Err(Error::InvalidSignature);
        }

        Ok(self)
    }
}
impl sealed::Sealed for SectionAuth {}

mod sealed {
    #[allow(missing_docs, unreachable_pub)]
    pub trait Sealed {}
}
