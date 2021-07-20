use super::node::{KeyedSig, SigShare};
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
    #[serde(deserialize_with = "Signature::deserialize_ed25519")]
    #[serde(serialize_with = "Signature::serialize_ed25519")]
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
