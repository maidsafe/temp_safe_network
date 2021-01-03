// Copyright 2021MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{AdultDuties, Duty, ElderDuties};
use ed25519_dalek::PublicKey as Ed25519PublicKey;
use ed25519_dalek::Signature as Ed25519Signature;
use hex_fmt::HexFmt;
use serde::{Deserialize, Serialize};
use signature::Verifier;
use sn_data_types::{PublicKey, Result, Signature, SignatureShare};
use std::{
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
};
use threshold_crypto::{
    PublicKey as BlsPublicKey, PublicKeySet as BlsPublicKeySet,
    PublicKeyShare as BlsPublicKeyShare, Signature as BlsSignature,
    SignatureShare as BlsSignatureShare,
};
pub use xor_name::Prefix;
use xor_name::{XorName, XOR_NAME_LEN};

/// A msg sender in the larger network (clients + nodes),
/// provides its identification by specifying type of entity,
/// and providing a signature over it.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct MsgSender {
    entity: Entity,
    sig: Option<EntitySignature>,
}

/// An identifier of a section, as
/// of a specific Elder constellation, thereby making it transient.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransientSectionKey {
    /// The group sig related id.
    pub bls_key: BlsPublicKey,
}

impl Debug for TransientSectionKey {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "TransientSectionKey::")?;
        write!(
            formatter,
            "Bls({:<8})",
            HexFmt(&self.bls_key.to_bytes()[..XOR_NAME_LEN])
        )
    }
}

impl Display for TransientSectionKey {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

/// An identifier of an Elder, as
/// of a specific Elder constellation, thereby making it transient.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransientElderKey {
    /// The xorspace related id for the node.
    pub node_id: Ed25519PublicKey,
    /// Part of the group sig related id.
    pub bls_key: BlsPublicKeyShare,
    /// Part of the group sig related id.
    pub bls_share_index: usize,
    /// Part of the group sig related id.
    pub bls_public_key_set: BlsPublicKeySet,
}

impl Debug for TransientElderKey {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "TransientElderKey::")?;
        write!(
            formatter,
            "Ed25519({:<8})",
            HexFmt(&self.node_id.to_bytes())
        )?;
        write!(
            formatter,
            "BlsShare({:<8})",
            HexFmt(&self.bls_key.to_bytes()[..XOR_NAME_LEN])
        )
    }
}

impl Display for TransientElderKey {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

/// An entity in the messaging ecosystem.
/// It has an address that can be used for messaging.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Entity {
    ///
    Client(PublicKey),
    /// Elder, Adult, or any other sort of node.
    AnyNode(Ed25519PublicKey, Duty),
    ///
    AdultNode(Ed25519PublicKey, AdultDuties),
    ///
    ElderNode(TransientElderKey, ElderDuties),
    ///
    Section(TransientSectionKey, ElderDuties),
}

impl Debug for Entity {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Entity::")?;
        match self {
            Self::Client(key) => Debug::fmt(key, formatter),
            Self::AnyNode(key, duty) => {
                write!(formatter, "Ed25519({:<8})", HexFmt(&key.to_bytes()))?;
                write!(formatter, "AnyNode({:?})", duty)
            }
            Self::AdultNode(key, duty) => {
                write!(formatter, "Ed25519({:<8})", HexFmt(&key.to_bytes()))?;
                write!(formatter, "AdultNode({:?})", duty)
            }
            Self::ElderNode(key, duty) => {
                Debug::fmt(key, formatter)?;
                write!(formatter, "ElderNode({:?})", duty)
            }
            Self::Section(key, duty) => {
                Debug::fmt(key, formatter)?;
                write!(formatter, "Section({:?})", duty)
            }
        }
    }
}

impl Display for Entity {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

///
pub enum EntityId {
    /// Not an xorspace id.
    Client(PublicKey),
    /// An xorspace id.
    Node(Ed25519PublicKey),
    /// Not an xorspace id.
    Section(BlsPublicKey),
}

impl Debug for EntityId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "EntityId::")?;
        match self {
            Self::Client(pub_key) => Debug::fmt(pub_key, formatter),
            Self::Node(pub_key) => write!(formatter, "Ed25519({:<8})", HexFmt(&pub_key.to_bytes())),
            Self::Section(pub_key) => write!(
                formatter,
                "Bls({:<8})",
                HexFmt(&pub_key.to_bytes()[..XOR_NAME_LEN])
            ),
        }
    }
}

impl Display for EntityId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum EntitySignature {
    /// Any constellation.
    Client(Signature),
    /// When acting individually.
    Node(Ed25519Signature),
    /// Elders acting in group.
    Elder(BlsSignatureShare),
    /// The group.
    Section(BlsSignature),
}

impl Debug for EntitySignature {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "EntitySignature::")?;
        match self {
            Self::Client(sig) => {
                write!(formatter, "Client::")?;
                Debug::fmt(sig, formatter)
            }
            Self::Node(_) => {
                write!(formatter, "Node::")?;
                write!(formatter, "Ed25519(..)")
            }
            Self::Elder(_) => {
                write!(formatter, "Elder::")?;
                write!(formatter, "BlsShare(..)")
            }
            Self::Section(_) => {
                write!(formatter, "Section::")?;
                write!(formatter, "Bls(..)")
            }
        }
    }
}

impl Display for EntitySignature {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Address {
    ///
    Client(XorName),
    ///
    Node(XorName),
    ///
    Section(XorName),
}

impl Address {
    /// Extracts the underlying XorName.
    pub fn xorname(&self) -> XorName {
        use Address::*;
        match self {
            Client(xorname) | Node(xorname) | Section(xorname) => *xorname,
        }
    }
}

impl EntityId {
    ///
    pub fn public_key(&self) -> PublicKey {
        match self {
            Self::Client(key) => *key,
            Self::Node(key) => PublicKey::Ed25519(*key),
            Self::Section(key) => PublicKey::Bls(*key),
        }
    }
}

impl MsgSender {
    ///
    pub fn client(key: PublicKey, sig: Signature) -> Result<Self> {
        Ok(Self {
            entity: Entity::Client(key),
            sig: Some(EntitySignature::Client(sig)),
        })
    }

    ///
    pub fn any_node(key: Ed25519PublicKey, duty: Duty, sig: Ed25519Signature) -> Result<Self> {
        Ok(Self {
            entity: Entity::AnyNode(key, duty),
            sig: Some(EntitySignature::Node(sig)),
        })
    }

    ///
    pub fn adult(key: Ed25519PublicKey, duty: AdultDuties, sig: Ed25519Signature) -> Result<Self> {
        Ok(Self {
            entity: Entity::AdultNode(key, duty),
            sig: Some(EntitySignature::Node(sig)),
        })
    }

    ///
    pub fn elder(
        key: TransientElderKey,
        duty: ElderDuties,
        sig: BlsSignatureShare,
    ) -> Result<Self> {
        Ok(Self {
            entity: Entity::ElderNode(key, duty),
            sig: Some(EntitySignature::Elder(sig)),
        })
    }

    ///
    pub fn section(key: TransientSectionKey, duty: ElderDuties) -> Result<Self> {
        Ok(Self {
            entity: Entity::Section(key, duty),
            sig: None,
        })
    }

    /// The id of the sender.
    pub fn id(&self) -> EntityId {
        self.entity.id()
    }

    /// The network address of the sender.
    pub fn address(&self) -> Address {
        self.entity.address()
    }

    /// The duty under which the sender operated.
    pub fn duty(&self) -> Option<Duty> {
        use Entity::*;
        match self.entity {
            Client(_) => None,
            AnyNode(_, duty) => Some(duty),
            AdultNode(_, duty) => Some(Duty::Adult(duty)),
            ElderNode(_, duty) | Section(_, duty) => Some(Duty::Elder(duty)),
        }
    }

    /// Verifies a payload as sent by this sender.
    pub fn verify(&self, payload: &[u8]) -> bool {
        self.entity.try_verify(self.sig.clone(), payload)
    }

    /// If sender is Elder
    pub fn group_key_set(&self) -> Option<BlsPublicKeySet> {
        if let Entity::ElderNode(key, _) = &self.entity {
            Some(key.bls_public_key_set.clone())
        } else {
            None
        }
    }

    /// If sender is Elder
    pub fn group_sig_share(&self) -> Option<SignatureShare> {
        if let Entity::ElderNode(key, _) = &self.entity {
            if let Some(EntitySignature::Elder(sig)) = &self.sig {
                return Some(SignatureShare {
                    index: key.bls_share_index,
                    share: sig.clone(),
                });
            } else {
                unreachable!("Should not be possible to instantiate such a combination.")
            }
        }
        None
    }

    ///
    pub fn is_client(&self) -> bool {
        matches!(self.entity, Entity::Client(_))
    }

    ///
    pub fn is_any_node(&self) -> bool {
        matches!(self.entity, Entity::AnyNode { .. })
    }

    ///
    pub fn is_adult(&self) -> bool {
        matches!(self.entity, Entity::AdultNode { .. })
    }

    ///
    pub fn is_elder(&self) -> bool {
        matches!(self.entity, Entity::ElderNode { .. })
    }

    ///
    pub fn is_section(&self) -> bool {
        matches!(self.entity, Entity::Section { .. })
    }
}

impl Entity {
    /// The id of the entity.
    pub fn id(&self) -> EntityId {
        use Entity::*;
        match self {
            Client(key) => EntityId::Client(*key),
            AnyNode(node_id, ..) | AdultNode(node_id, ..) => EntityId::Node(*node_id),
            ElderNode(key, ..) => EntityId::Node(key.node_id),
            Section(key, ..) => EntityId::Section(key.bls_key),
        }
    }

    /// The address of the entity,
    /// used to send messages to it.
    pub fn address(&self) -> Address {
        use Entity::*;
        match self {
            Client(key) => Address::Client((*key).into()),
            AnyNode(key, ..) | AdultNode(key, ..) => Address::Node(PublicKey::Ed25519(*key).into()),
            ElderNode(key, ..) => Address::Node(PublicKey::Ed25519(key.node_id).into()),
            Section(key, ..) => Address::Section(PublicKey::Bls(key.bls_key).into()),
        }
    }

    ///
    pub fn try_verify(&self, sig: Option<EntitySignature>, data: &[u8]) -> bool {
        use Entity::*;
        match self {
            Client(key) => {
                if let Some(EntitySignature::Client(sig)) = sig {
                    key.verify(&sig, data).is_ok()
                } else {
                    false
                }
            }
            AnyNode(key, ..) | AdultNode(key, ..) => {
                if let Some(EntitySignature::Node(sig)) = sig {
                    key.verify(data, &sig).is_ok()
                } else {
                    false
                }
            }
            ElderNode(key, ..) => {
                if let Some(EntitySignature::Elder(sig)) = sig {
                    key.bls_key.verify(&sig, data)
                } else if let Some(EntitySignature::Node(sig)) = sig {
                    key.node_id.verify(data, &sig).is_ok()
                } else {
                    false
                }
            }
            Section(..) => true,
        }
    }
}
