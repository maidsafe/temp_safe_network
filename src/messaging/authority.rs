use super::{
    node::{KeyedSig, SigShare},
    Error, Result,
};
use crate::types::{PublicKey, Signature};
use bls::PublicKey as BlsPublicKey;
use ed25519_dalek::{PublicKey as EdPublicKey, Signature as EdSignature};
use xor_name::XorName;

/// Authority of a client
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ClientSigned {
    /// Client public key.
    pub public_key: PublicKey,
    /// Client signature.
    pub signature: Signature,
}

impl ClientSigned {
    /// Verify that the pair of `public_key` created `signature` over `payload`.
    ///
    /// The returned `Ok` variant represents a proof that the owner of `public_key` indeed signed
    /// `payload`, and so bears their authority. Note however that it may still be necessary to
    /// confirm that `public_key` is who you expect!
    pub fn verify(self, payload: &impl AsRef<[u8]>) -> Result<ClientAuthority> {
        ClientAuthority::verify(self.public_key, self.signature, payload)
    }
}

/// A [`ClientAuthority`] can be converted back to a [`ClientSigned`], losing the 'proof' of validity.
impl From<ClientAuthority> for ClientSigned {
    fn from(signed: ClientAuthority) -> Self {
        Self {
            public_key: signed.public_key,
            signature: signed.signature,
        }
    }
}

/// Verified authority of a client.
///
/// Values of this type constitute a proof that the signature is valid for a particular payload.
/// This is made possible by performing verification in all possible constructors of the type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientAuthority {
    public_key: PublicKey,
    signature: Signature,
}

impl ClientAuthority {
    /// Verify that `payload` has client's authority.
    ///
    /// This verifies that the owner of `public_key` (e.g. the holder of the corresponding private
    /// key) created the `signature` by signing the `payload` with theor private key. When this is
    /// true, we say the payload has client authority.
    pub fn verify(
        public_key: PublicKey,
        signature: Signature,
        payload: &impl AsRef<[u8]>,
    ) -> Result<Self> {
        public_key
            .verify(&signature, payload)
            .map_err(|_| Error::InvalidSignature)?;
        Ok(Self {
            public_key,
            signature,
        })
    }

    /// Get the signer's public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Create a [`ClientSigned`] from this authority by cloning the fields.
    ///
    /// Since [`ClientAuthority`] cannot be serialized, it's sometimes necessary to convert back to
    /// an unverified signature. Prefer [`ClientSigned::from`][1] if you don't need to retain the
    /// `ClientAuthority`, as this won't clone the fields.
    ///
    /// [1]: ClientSigned#impl-From<ClientAuthority>
    pub fn to_signed(&self) -> ClientSigned {
        ClientSigned {
            public_key: self.public_key,
            signature: self.signature.clone(),
        }
    }
}

/// Authority of a single peer.
#[derive(Clone, Eq, PartialEq, custom_debug::Debug, serde::Deserialize, serde::Serialize)]
pub struct NodeSigned {
    /// Section key of the source.
    pub section_pk: BlsPublicKey,
    /// Public key of the source peer.
    #[debug(with = "PublicKey::fmt_ed25519")]
    pub public_key: EdPublicKey,
    /// Ed25519 signature of the message corresponding to the public key of the source peer.
    #[debug(with = "Signature::fmt_ed25519")]
    #[serde(with = "serde_bytes")]
    pub signature: EdSignature,
}

/// Authority of a single peer that uses it's BLS Keyshare to sign the message.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BlsShareSigned {
    /// Section key of the source.
    pub section_pk: BlsPublicKey,
    /// Name in the source section.
    pub src_name: XorName,
    /// Proof Share signed by the peer's BLS KeyShare.
    pub sig_share: SigShare,
}

/// Authority of a whole section.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SectionSigned {
    /// Section key of the source.
    pub section_pk: BlsPublicKey,
    /// Name in the source section.
    pub src_name: XorName,
    /// BLS proof of the message corresponding to the source section.
    pub sig: KeyedSig,
}
