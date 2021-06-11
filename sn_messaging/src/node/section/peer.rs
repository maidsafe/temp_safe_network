// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    hash::Hash,
    net::SocketAddr,
};
use xor_name::XorName;

/// Network p2p peer identity.
/// When a node knows another p2p_node as a `Peer` it's implicitly connected to it. This is separate
/// from being connected at the network layer, which currently is handled by quic-p2p.
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct Peer {
    pub name: XorName,
    pub addr: SocketAddr,
    // A node's connectivity will be checked when first time join the network or got relocated.
    // An adult can only got promoted to elder when this flag is set to `true`.
    // Taking a default of `false` to enforce the connectivity check.
    pub reachable: bool,
}

impl Display for Peer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.name, self.addr)
    }
}
