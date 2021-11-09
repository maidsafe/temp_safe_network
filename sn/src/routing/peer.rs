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

/// Network p2p peer identity.
/// When a node knows another p2p_node as a `Peer` it's implicitly connected to it. This is separate
/// from being connected at the network layer, which currently is handled by quic-p2p.
#[derive(Clone, Debug)]
pub struct Peer {
    name: XorName,
    addr: SocketAddr,

    // The connection through which we learned about the `Peer`. This may be set when connecting to
    // a peer, or when receiving an incoming connection. This provides a means for connection reuse,
    // but there's no guarantee that the connection will still be valid at call time, so
    // reconnection may be necessary.
    pub(crate) connection: Option<qp2p::Connection>,
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

    /// Crates a new `Peer` given a `name`, and an existing connection to the peer.
    ///
    /// Note that the connection doesn't have to be alive (i.e. may be disconnected). This needs to
    /// be considered when reading `self.connection`.
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
}

/// A peer whose name we do not yet know.
///
/// An [`UnknownPeer`] can be [`identify`]'d to become a [`Peer`].
#[derive(Debug)]
pub(crate) struct UnknownPeer {
    addr: SocketAddr,
    connection: Option<qp2p::Connection>,
}

impl UnknownPeer {
    #[cfg(test)]
    pub(crate) fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            connection: None,
        }
    }

    pub(crate) fn connected(connection: qp2p::Connection) -> Self {
        Self {
            addr: connection.remote_address(),
            connection: Some(connection),
        }
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub(crate) fn identify(self, name: XorName) -> Peer {
        Peer {
            name,
            addr: self.addr,
            connection: self.connection,
        }
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
