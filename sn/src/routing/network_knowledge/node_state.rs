// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{MembershipState, NodeState as NodeStateMsg, SectionAuth};
use crate::routing::{error::Error, Peer};
use std::net::SocketAddr;
use xor_name::{XorName, XOR_NAME_LEN};

/// The minimum age a node can have. The Infants will start at age 4. This is to prevent frequent
/// relocations during the beginning of a node's lifetime.
pub const MIN_AGE: u8 = 4;

/// The minimum age a node becomes an adult node.
pub const MIN_ADULT_AGE: u8 = MIN_AGE + 1;

/// During the first section, nodes can start at a range of age to avoid too many nodes having the
/// same time get relocated at the same time.
/// Defines the lower bound of this range.
pub const FIRST_SECTION_MIN_AGE: u8 = MIN_ADULT_AGE + 1;
/// Defines the higher bound of this range.
pub const FIRST_SECTION_MAX_AGE: u8 = 100;

/// Information about a member of our section.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize)]
pub(crate) struct NodeState {
    name: XorName,
    addr: SocketAddr,
    state: MembershipState,
    previous_name: Option<XorName>,
}

impl NodeState {
    // Creates a `NodeState` in the `Joined` state.
    pub(crate) fn joined(peer: &Peer, previous_name: Option<XorName>) -> Self {
        Self {
            name: peer.name(),
            addr: peer.addr(),
            state: MembershipState::Joined,
            previous_name,
        }
    }

    // Creates a `NodeState` in the `Left` state.
    #[cfg(test)]
    pub(crate) fn left(peer: &Peer, previous_name: Option<XorName>) -> Self {
        Self {
            name: peer.name(),
            addr: peer.addr(),
            state: MembershipState::Left,
            previous_name,
        }
    }

    pub(crate) fn name(&self) -> XorName {
        self.name
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub(crate) fn state(&self) -> MembershipState {
        self.state
    }

    pub(crate) fn previous_name(&self) -> Option<XorName> {
        self.previous_name
    }

    pub(crate) fn age(&self) -> u8 {
        self.name[XOR_NAME_LEN - 1]
    }

    // Is the age > `MIN_AGE`?
    pub(crate) fn is_mature(&self) -> bool {
        self.age() > MIN_AGE
    }

    pub(crate) fn leave(self) -> Result<Self, Error> {
        // Do not allow switching to `Left` when already relocated, to avoid rejoining with the
        // same name.
        if let MembershipState::Relocated(_) = self.state {
            return Err(Error::InvalidState);
        }
        Ok(Self {
            state: MembershipState::Left,
            ..self
        })
    }

    // Convert this info into one with the state changed to `Relocated`.
    pub(crate) fn relocate(self, dst: XorName) -> Self {
        Self {
            state: MembershipState::Relocated(dst),
            ..self
        }
    }

    pub(crate) fn to_peer(self) -> Peer {
        Peer::new(self.name, self.addr)
    }
}

// Add conversion methods to/from `messaging::...::NodeState`
// We prefer this over `From<...>` to make it easier to read the conversion.

impl NodeState {
    pub(crate) fn into_msg(self) -> NodeStateMsg {
        NodeStateMsg {
            name: self.name,
            addr: self.addr,
            state: self.state,
            previous_name: self.previous_name,
        }
    }
}

impl SectionAuth<NodeState> {
    pub(crate) fn into_authed_msg(self) -> SectionAuth<NodeStateMsg> {
        SectionAuth {
            value: self.value.into_msg(),
            sig: self.sig,
        }
    }
}

impl NodeStateMsg {
    pub(crate) fn into_state(self) -> NodeState {
        NodeState {
            name: self.name,
            addr: self.addr,
            state: self.state,
            previous_name: self.previous_name,
        }
    }
}

impl SectionAuth<NodeStateMsg> {
    pub(crate) fn into_authed_state(self) -> SectionAuth<NodeState> {
        SectionAuth {
            value: self.value.into_state(),
            sig: self.sig,
        }
    }
}
