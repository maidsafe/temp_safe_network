// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod client_id;
mod node_id;

pub use client_id::ClientId;
pub use node_id::NodeId;

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    net::SocketAddr,
};
use xor_name::XorName;

/// The id is the name, derived from its `PublicKey`, and the address of a client.
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Participant {
    name: XorName,
    addr: SocketAddr,
}

impl Participant {
    pub fn new(name: XorName, addr: SocketAddr) -> Self {
        Self { name, addr }
    }

    pub fn name(&self) -> XorName {
        self.name
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn from_node(id: NodeId) -> Participant {
        Self::new(id.name(), id.addr())
    }

    pub fn from_client(id: ClientId) -> Participant {
        Self::new(id.name(), id.addr())
    }
}

impl Display for Participant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.name(), self.addr(),)
    }
}

impl Eq for Participant {}

impl Hash for Participant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.addr().hash(state);
    }
}

impl Ord for Participant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name()
            .cmp(&other.name())
            .then_with(|| self.addr().cmp(&other.addr()))
    }
}

impl PartialEq for Participant {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.addr() == other.addr()
    }
}

impl PartialEq<&Self> for Participant {
    fn eq(&self, other: &&Self) -> bool {
        self.name() == other.name() && self.addr() == other.addr()
    }
}

impl PartialOrd for Participant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
