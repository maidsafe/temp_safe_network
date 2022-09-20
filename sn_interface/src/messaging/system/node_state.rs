// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use bls::PublicKey as BlsPublicKey;
use ed25519_dalek::{Signature, Verifier};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use xor_name::{XorName, XOR_NAME_LEN};

use crate::{network_knowledge::NetworkKnowledge, types::Peer};

/// Information about a member of our section.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct NodeState {
    /// Peer's name.
    pub name: XorName,
    /// Peer's address.
    pub addr: SocketAddr,
    /// Current state of the peer
    pub state: MembershipState,
    /// To avoid sybil attack via relocation, a relocated node's original name will be recorded.
    pub previous_name: Option<XorName>,
}

impl NodeState {
    /// Build a `NodeState` in the Joined state.
    pub fn joined(name: XorName, addr: SocketAddr, previous_name: Option<XorName>) -> Self {
        Self {
            name,
            addr,
            state: MembershipState::Joined,
            previous_name,
        }
    }

    /// Returns the peer struct for this node.
    pub fn peer(&self) -> Peer {
        Peer::new(self.name, self.addr)
    }

    /// Returns the age.
    pub fn age(&self) -> u8 {
        self.name[XOR_NAME_LEN - 1]
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
/// Node's current section membership state
pub enum MembershipState {
    /// Node is active member of the section.
    Joined,
    /// Node went offline.
    Left,
    /// Node was relocated to a different section.
    Relocated(Box<RelocateDetails>),
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
