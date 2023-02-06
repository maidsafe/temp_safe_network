// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::SectionSigned;
use crate::network_knowledge::{section_has_room_for_node, Error, Result};
use crate::types::Peer;

use ed25519_dalek::{PublicKey, Signature, Verifier};
use hex_fmt::HexFmt;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug, Formatter},
    net::SocketAddr,
};
use xor_name::{Prefix, XorName};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
/// Node's current section membership state
pub enum MembershipState {
    /// Node is active member of the section.
    Joined,
    /// Node went offline.
    Left,
    /// Node was relocated to a different section.
    Relocated(RelocationDst),
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
            state: MembershipState::Relocated(relocation_dst),
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
            warn!("Membership - rejecting node {name}, name doesn't match our prefix {prefix:?}");
            return Err(Error::WrongSection);
        }

        match self.state {
            MembershipState::Joined => {
                if members.contains_key(&name) {
                    warn!("Rejecting join from existing member {name}");
                    Err(Error::ExistingMemberConflict)
                } else if !section_has_room_for_node(name, prefix, members.keys().copied()) {
                    warn!("Rejecting join since we are at capacity");
                    Err(Error::TryJoinLater)
                } else if let Some(existing_node) = members
                    .values()
                    .find(|n| n.peer().addr() == self.peer().addr())
                {
                    warn!("Rejecting join since we have an existing node with this address: {existing_node:?}");
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
                    warn!("Rejecting leave from non-existing member");
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
    pub fn relocate(self, relocation_dst: RelocationDst) -> Self {
        Self {
            state: MembershipState::Relocated(relocation_dst),
            ..self
        }
    }
}

/// We are relocating to the section that matches the contained XorName.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct RelocationDst(XorName);

impl RelocationDst {
    /// We are relocating to the section that matches `dst_name`.
    pub fn new(dst_name: XorName) -> Self {
        Self(dst_name)
    }

    pub fn name(&self) -> &XorName {
        &self.0
    }
}

/// The relocation info contains the dst (in `NodeState`),
/// the old name, the new name and the source section signature
/// over the fact that the section considered the node to be relocated.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct RelocationInfo {
    signed_relocation: SectionSigned<NodeState>,
    new_name: XorName,
}

/// A relocation proof proves that a section started a relocation
/// of one of its nodes, and that the new name provided was created by that node.
///
/// NB: Upper layers will need to verify that said section is also a known section,
/// only then is the relocation fully valid.
#[derive(Clone, PartialEq, Serialize, Deserialize, custom_debug::Debug)]
pub struct RelocationProof {
    info: RelocationInfo,
    // This sig proves that the new name was actually created by the node holding the old keys.
    #[serde(with = "serde_bytes")]
    #[debug(with = "Self::fmt_ed25519")]
    self_sig: Signature,
    /// The old key that identified the node in the source section.
    self_old_key: PublicKey,
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
    pub fn new(info: RelocationInfo, self_sig: Signature, self_old_key: PublicKey) -> Self {
        Self {
            info,
            self_sig,
            self_old_key,
        }
    }

    /// The key of the section that the node is relocating from.
    pub fn signed_by(&self) -> &bls::PublicKey {
        &self.info.signed_relocation.sig.public_key
    }

    /// This verifies that the new name was actually created by the node holding the old name,
    /// and that the section signature is signed by the provided section key.
    /// Calling context will need to verify that said section key is also a known section.
    pub fn verify(&self) -> Result<()> {
        // the key that we use to verify the sig over the new name, must match the name of the relocated node
        if self.old_key_name() != self.info.signed_relocation.name() {
            return Err(Error::InvalidRelocationProof);
        }
        let serialized_info =
            bincode::serialize(&self.info).map_err(|_err| Error::InvalidRelocationProof)?;
        self.self_old_key
            .verify(&serialized_info, &self.self_sig)
            .map_err(|_err| Error::InvalidRelocationProof)?;
        let serialized_state = bincode::serialize(&self.info.signed_relocation.value)
            .map_err(|_err| Error::InvalidRelocationProof)?;
        if !self.info.signed_relocation.sig.verify(&serialized_state) {
            Err(Error::InvalidRelocationProof)
        } else {
            Ok(())
        }
    }

    /// Previous name of the relocating node.
    pub fn previous_name(&self) -> XorName {
        self.info.signed_relocation.name()
    }

    /// Previous age of the relocating node.
    pub fn previous_age(&self) -> u8 {
        self.info.signed_relocation.age()
    }

    // ed25519_dalek::Signature has overly verbose debug output, so we provide our own
    pub fn fmt_ed25519(sig: &Signature, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({:0.10})", HexFmt(sig))
    }

    fn old_key_name(&self) -> XorName {
        use crate::types::PublicKey::Ed25519;
        XorName::from(Ed25519(self.self_old_key))
    }
}
