// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeId;

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    net::SocketAddr,
};
use xor_name::XorName;

/// A NodeId with name, derived from its `PublicKey`,
/// an address, and a reward key to which rewards can be paid.
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RewardPeer {
    node_id: NodeId,
    reward_key: bls::PublicKey,
}

impl RewardPeer {
    pub fn new(node_id: NodeId, reward_key: bls::PublicKey) -> Self {
        Self {
            node_id,
            reward_key,
        }
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(self.node_id.name(), self.node_id.addr())
    }

    pub fn name(&self) -> XorName {
        self.node_id.name()
    }

    pub fn addr(&self) -> SocketAddr {
        self.node_id.addr()
    }

    /// Returns the age of the node_id.
    pub fn age(&self) -> u8 {
        self.node_id.age()
    }

    /// Returns the public key to which rewards to the node_id are paid.
    pub fn reward_key(&self) -> bls::PublicKey {
        self.reward_key
    }

    #[cfg(any(test, feature = "test-utils"))]
    pub fn random() -> Self {
        Self {
            node_id: NodeId::random(),
            reward_key: bls::SecretKey::random().public_key(),
        }
    }

    #[cfg(any(test, feature = "test-utils"))]
    pub fn random_w_name(name: XorName) -> Self {
        Self {
            node_id: NodeId::random_w_name(name),
            reward_key: bls::SecretKey::random().public_key(),
        }
    }

    #[cfg(any(test, feature = "test-utils"))]
    pub fn random_w_key(public_key: ed25519_dalek::PublicKey) -> Self {
        Self {
            node_id: NodeId::random_w_key(public_key),
            reward_key: bls::SecretKey::random().public_key(),
        }
    }

    pub fn random_w_addr(addr: SocketAddr) -> Self {
        Self {
            node_id: NodeId::random_w_addr(addr),
            reward_key: bls::SecretKey::random().public_key(),
        }
    }

    pub fn random_w_node_id(node_id: NodeId) -> Self {
        Self {
            node_id,
            reward_key: bls::SecretKey::random().public_key(),
        }
    }
}

impl Display for RewardPeer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}, w reward key {:?}", self.node_id(), self.reward_key)
    }
}

impl Eq for RewardPeer {}

impl Hash for RewardPeer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
        self.reward_key.hash(state);
    }
}

impl Ord for RewardPeer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.node_id
            .cmp(&other.node_id)
            .then_with(|| self.reward_key.cmp(&other.reward_key))
    }
}

impl PartialEq for RewardPeer {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id && self.reward_key == other.reward_key
    }
}

impl PartialEq<&Self> for RewardPeer {
    fn eq(&self, other: &&Self) -> bool {
        self.node_id == other.node_id && self.reward_key == other.reward_key
    }
}

impl PartialOrd for RewardPeer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
