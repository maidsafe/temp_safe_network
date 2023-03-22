// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Participant;

use crate::types::{utils::calc_age, PublicKey};

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    net::SocketAddr,
};
use xor_name::XorName;

// Above this age, we trust the node to store data
const PRIMARY_NODE_AGE: u8 = 4;

/// The id of a node is the name, derived from its `PublicKey`, and its address.
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeId {
    name: XorName,
    addr: SocketAddr,
}

impl NodeId {
    pub fn new(name: XorName, addr: SocketAddr) -> Self {
        Self { name, addr }
    }

    pub fn name(&self) -> XorName {
        self.name
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the age.
    pub fn age(&self) -> u8 {
        calc_age(&self.name)
    }

    /// Returns if this node can be considered primary storage
    /// (pruimary storage being anything beyond the low age nodes which are much more
    /// likely to be churning, and so only perform backup operations)
    pub fn is_primary_node(&self) -> bool {
        self.age() >= PRIMARY_NODE_AGE
    }

    pub fn from(sender: Participant) -> Self {
        Self::new(sender.name, sender.addr)
    }

    pub fn from_key(addr: SocketAddr, public_key: ed25519_dalek::PublicKey) -> Self {
        Self {
            addr,
            name: XorName::from(PublicKey::from(public_key)),
        }
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.name(), self.addr(),)
    }
}

impl Eq for NodeId {}

impl Hash for NodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.addr().hash(state);
    }
}

impl Ord for NodeId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name()
            .cmp(&other.name())
            .then_with(|| self.addr().cmp(&other.addr()))
    }
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.addr() == other.addr()
    }
}

impl PartialEq<&Self> for NodeId {
    fn eq(&self, other: &&Self) -> bool {
        self.name() == other.name() && self.addr() == other.addr()
    }
}

impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
