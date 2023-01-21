// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::SectionSigned;
use crate::network_knowledge::{NetworkKnowledge, section_has_room_for_node, Error, Result};
use crate::types::Peer;

use bls::PublicKey as BlsPublicKey;
use ed25519_dalek::{Signature, Verifier, PublicKey};
use hex_fmt::HexFmt;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fmt::{self, Debug, Formatter}, net::SocketAddr};
use xor_name::{Prefix, XorName};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
/// Node's current section membership state
pub enum MembershipState {
    /// Node is active member of the section.
    Joined,
    /// Node went offline.
    Left,
    /// Node was relocated to a different section.
    Relocated(Box<RelocationDst>),
}

/// Information about a member of our section.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NodeState {
    peer: Peer,
    /// Current state of the peer
    state: MembershipState,
    /// To avoid sybil attack via relocation, a relocated node's original name will be recorded.
    previous_name: Option<XorName>,
}

impl Debug for NodeState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut f = f.debug_tuple("NodeState");
        let f = f
            .field(&self.name())
            .field(&self.addr())
            .field(&self.state());

        let f = if let Some(prev_name) = self.previous_name() {
            f.field(&format!("prev_name: {prev_name:?}"))
        } else {
            f
        };
        f.finish()
    }
}

impl NodeState {
    // Creates a `NodeState` in the `Joined` state.
    pub fn joined(peer: Peer, previous_name: Option<XorName>) -> Self {
        Self {
            peer,
            state: MembershipState::Joined,
            previous_name,
        }
    }

    // Creates a `NodeState` in the `Left` state.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn left(peer: Peer, previous_name: Option<XorName>) -> Self {
        Self {
            peer,
            state: MembershipState::Left,
            previous_name,
        }
    }

    // Creates a `NodeState` in the `Relocated` state.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn relocated(
        peer: Peer,
        previous_name: Option<XorName>,
        relocation_dst: RelocationDst,
    ) -> Self {
        Self {
            peer,
            state: MembershipState::Relocated(Box::new(relocation_dst)),
            previous_name,
        }
    }

    pub fn validate(
        &self,
        prefix: &Prefix,
        members: &BTreeMap<XorName, Self>,
        archived: &BTreeSet<XorName>,
    ) -> Result<()> {
        let name = self.name();
        info!("Validating node state for {name} - {:?}", self.state);

        if !prefix.matches(&name) {
            info!("Membership - rejecting node {name}, name doesn't match our prefix {prefix:?}");
            return Err(Error::WrongSection);
        }

        match self.state {
            MembershipState::Joined => {
                if members.contains_key(&name) {
                    info!("Rejecting join from existing member {name}");
                    Err(Error::ExistingMemberConflict)
                } else if !section_has_room_for_node(name, prefix, members.keys().copied()) {
                    info!("Rejecting join since we are at capacity");
                    Err(Error::TryJoinLater)
                } else if let Some(existing_node) = members
                    .values()
                    .find(|n| n.peer().addr() == self.peer().addr())
                {
                    info!("Rejecting join since we have an existing node with this address: {existing_node:?}");
                    Err(Error::ExistingMemberConflict)
                } else if archived.contains(&name) {
                    Err(Error::ArchivedNodeRejoined)
                } else {
                    Ok(())
                }
            }
            MembershipState::Relocated(_) => {
                // A node relocation is always OK
                Ok(())
            }
            MembershipState::Left => {
                if !members.contains_key(&name) {
                    info!("Rejecting leave from non-existing member");
                    Err(Error::NotAMember)
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn peer(&self) -> &Peer {
        &self.peer
    }

    pub fn name(&self) -> XorName {
        self.peer.name()
    }

    pub fn addr(&self) -> SocketAddr {
        self.peer.addr()
    }

    pub fn state(&self) -> MembershipState {
        self.state.clone()
    }

    pub fn previous_name(&self) -> Option<XorName> {
        self.previous_name
    }

    pub fn age(&self) -> u8 {
        self.peer.age()
    }

    // Returns true if the state is a Relocated node
    pub fn is_relocated(&self) -> bool {
        matches!(self.state, MembershipState::Relocated(_))
    }

    pub fn leave(self) -> Result<Self, Error> {
        // Do not allow switching to `Left` when already relocated,
        assert_eq!(self.state, MembershipState::Joined);

        Ok(Self {
            state: MembershipState::Left,
            ..self
        })
    }

    // Convert this info into one with the state changed to `Relocated`.
    pub fn relocate(self, previous_name: XorName, relocation_dst: RelocationDst) -> Self {
        let previous_name = Some(previous_name);
        Self {
            state: MembershipState::Relocated(Box::new(relocation_dst)),
            previous_name,
            ..self
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct RelocationDst(XorName);

impl RelocationDst {
    pub fn name(&self) -> &XorName {
        &self.0
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct RelocationInfo {
    signed_relocation: SectionSigned<NodeState>,
    new_name: XorName,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, custom_debug::Debug)]
pub struct RelocationProof {
    info: RelocationInfo,
    #[serde(with = "serde_bytes")]
    #[debug(with = "Self::fmt_ed25519")]
    peer_sig: Signature,
    peer_key: PublicKey,
}

impl RelocationInfo {
    pub fn new(signed_relocation: SectionSigned<NodeState>, new_name: XorName) -> Self {
        Self {
            signed_relocation,
            new_name,
        }
    }
}

impl RelocationProof {
    pub fn new(info: RelocationInfo, peer_sig: Signature, peer_key: PublicKey) -> Self {
        Self {
            info,
            peer_sig,
            peer_key,
        }
    }

    pub fn signed_by(&self) -> &bls::PublicKey {
        &self.info.signed_relocation.sig.public_key
    }

    pub fn verify(&self) -> Result<()> {
        let serialized_info = bincode::serialize(&self.info).map_err(|err| Error::InvalidSignature)?;
        self.peer_key.verify(&serialized_info, &self.peer_sig).map_err(|err| Error::InvalidSignature)?;
        let serialized_state = bincode::serialize(&self.info.signed_relocation.value).map_err(|err| Error::InvalidSignature)?;
        if !self.info.signed_relocation.sig.verify(&serialized_state) {
            Err(Error::InvalidSignature)
        } else {
            Ok(())
        }
    }

    pub fn previous_name(&self) -> XorName {
        self.info.signed_relocation.name()
    }

    pub fn previous_age(&self) -> u8 {
        self.info.signed_relocation.age()
    }

    // ed25519_dalek::Signature has overly verbose debug output, so we provide our own
    pub fn fmt_ed25519(sig: &Signature, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({:0.10})", HexFmt(sig))
    }
}

/// Details of a node that has been relocated
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RelocateDetails {
    /// Name of the node to relocate (this is the node's name before relocation).
    pub previous_name: XorName,
    /// Relocation destination, the node will be relocated to
    /// a section whose prefix matches this name.
    pub dst: XorName,
    /// The BLS key of the destination section used by the relocated node to verify messages.
    pub dst_section_key: BlsPublicKey,
    /// The age the node will have post-relocation.
    pub age: u8,
}

impl RelocateDetails {
    /// Constructs RelocateDetails given current network knowledge
    pub fn with_age(
        network_knowledge: &NetworkKnowledge,
        peer: &Peer,
        dst: XorName,
        age: u8,
    ) -> Self {
        let genesis_key = *network_knowledge.genesis_key();

        let dst_section_key = network_knowledge
            .section_auth_by_name(&dst)
            .map_or_else(|_| genesis_key, |section_auth| section_auth.section_key());

        Self {
            previous_name: peer.name(),
            dst,
            dst_section_key,
            age,
        }
    }

    pub fn verify_identity(&self, new_name: &XorName, new_name_sig: &Signature) -> bool {
        let pub_key = if let Ok(pub_key) = crate::types::keys::ed25519::pub_key(&self.previous_name)
        {
            pub_key
        } else {
            return false;
        };

        pub_key.verify(&new_name.0, new_name_sig).is_ok()
    }
}
