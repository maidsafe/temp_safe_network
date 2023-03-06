// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Participant;

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    net::SocketAddr,
};
use xor_name::XorName;

/// The id of a client is the name, derived from its `PublicKey`, and its address.
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClientId {
    name: XorName,
    addr: SocketAddr,
}

impl ClientId {
    pub fn new(name: XorName, addr: SocketAddr) -> Self {
        Self { name, addr }
    }

    pub fn name(&self) -> XorName {
        self.name
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn from(sender: Participant) -> Self {
        Self::new(sender.name, sender.addr)
    }
}

impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.name(), self.addr(),)
    }
}

impl Eq for ClientId {}

impl Hash for ClientId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.addr().hash(state);
    }
}

impl Ord for ClientId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name()
            .cmp(&other.name())
            .then_with(|| self.addr().cmp(&other.addr()))
    }
}

impl PartialEq for ClientId {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.addr() == other.addr()
    }
}

impl PartialEq<&Self> for ClientId {
    fn eq(&self, other: &&Self) -> bool {
        self.name() == other.name() && self.addr() == other.addr()
    }
}

impl PartialOrd for ClientId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
