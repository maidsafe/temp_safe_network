// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    net::SocketAddr,
};
use xor_name::{XorName, XOR_NAME_LEN};

/// Network peer identity.
///
/// When a node knows another node as a `Peer` it's logically connected to it. This is separate from
/// being physically connected at the network layer, which is indicated by the optional `connection`
/// field.
#[derive(Clone, Debug)]
pub struct Peer {
    name: XorName,
    addr: SocketAddr,

    // An existing connection to the peer. There are no guarantees about the state of the connection
    // except that it once connected to this peer's `addr` (e.g. it may already be closed or
    // otherwise unusable).
    connection: Option<qp2p::Connection>,
}

impl Display for Peer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.name, self.addr)
    }
}

impl Eq for Peer {}

impl Hash for Peer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.addr.hash(state);
    }
}

impl Ord for Peer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .cmp(&other.name)
            .then_with(|| self.addr.cmp(&other.addr))
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.addr == other.addr
    }
}

impl PartialEq<&Self> for Peer {
    fn eq(&self, other: &&Self) -> bool {
        self.name == other.name && self.addr == other.addr
    }
}

impl PartialOrd for Peer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Peer {
    /// Creates a new `Peer` given `Name`, `SocketAddr`.
    pub fn new(name: XorName, addr: SocketAddr) -> Self {
        Self {
            name,
            addr,
            connection: None,
        }
    }

    /// Creates a new `Peer` given a `name` and a `connection` to the peer.
    pub(crate) fn connected(name: XorName, connection: qp2p::Connection) -> Self {
        Self {
            name,
            addr: connection.remote_address(),
            connection: Some(connection),
        }
    }

    /// Returns the `XorName` of the peer.
    pub fn name(&self) -> XorName {
        self.name
    }

    /// Returns the `SocketAddr`.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the age.
    pub fn age(&self) -> u8 {
        self.name[XOR_NAME_LEN - 1]
    }

    /// Get an existing connection to the peer, if any.
    pub(crate) fn connection(&self) -> Option<&qp2p::Connection> {
        self.connection.as_ref()
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::*;
    use proptest::{collection::SizeRange, prelude::*};
    use xor_name::XOR_NAME_LEN;

    pub(crate) fn arbitrary_bytes() -> impl Strategy<Value = [u8; XOR_NAME_LEN]> {
        any::<[u8; XOR_NAME_LEN]>()
    }

    // Generate Vec<Peer> where no two peers have the same name.
    pub(crate) fn arbitrary_unique_peers(
        count: impl Into<SizeRange>,
        age: impl Strategy<Value = u8>,
    ) -> impl Strategy<Value = Vec<Peer>> {
        proptest::collection::btree_map(arbitrary_bytes(), (any::<SocketAddr>(), age), count)
            .prop_map(|peers| {
                peers
                    .into_iter()
                    .map(|(mut bytes, (addr, age))| {
                        bytes[XOR_NAME_LEN - 1] = age;
                        let name = XorName(bytes);
                        Peer::new(name, addr)
                    })
                    .collect()
            })
    }
}
