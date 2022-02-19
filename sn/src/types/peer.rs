// Copyright 2022 MaidSafe.net limited.
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
/// A Peer not yet with a name
#[derive(Clone, Debug)]
pub struct NamelessPeer {
    addr: SocketAddr,
}

impl NamelessPeer {
    /// Creates a new `NamelessPeer` with given `SocketAddr`.
    pub(crate) fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    /// Returns the `SocketAddr`.
    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns a `NamedPeer`.
    pub(crate) fn with_name(self, name: XorName) -> NamedPeer {
        NamedPeer::new(name, self.addr)
    }
}

/// A Peer with name, derived from its PublicKey, and an address.
#[derive(Clone, Debug)]
pub struct NamedPeer {
    name: XorName,
    addr: SocketAddr,
}

impl NamedPeer {
    pub(crate) fn new(name: XorName, addr: SocketAddr) -> Self {
        Self { name, addr }
    }

    pub(crate) fn name(&self) -> XorName {
        self.name
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the age.
    pub(crate) fn age(&self) -> u8 {
        self.name[XOR_NAME_LEN - 1]
    }

    pub(crate) fn id(&self) -> (XorName, SocketAddr) {
        (self.name, self.addr)
    }
}

impl Display for NamedPeer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.name(), self.addr(),)
    }
}

impl Eq for NamedPeer {}

impl Hash for NamedPeer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.addr().hash(state);
    }
}

impl Ord for NamedPeer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name()
            .cmp(&other.name())
            .then_with(|| self.addr().cmp(&other.addr()))
    }
}

impl PartialEq for NamedPeer {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.addr() == other.addr()
    }
}

impl PartialEq<&Self> for NamedPeer {
    fn eq(&self, other: &&Self) -> bool {
        self.name() == other.name() && self.addr() == other.addr()
    }
}

impl PartialOrd for NamedPeer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
