// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{
    MembershipState, NodeState as NodeStateMsg, RelocateDetails, SectionAuth,
};
use crate::node::error::Error;
use crate::peer::Peer;

use std::net::SocketAddr;
use xor_name::XorName;

/// The minimum age a node becomes an adult node.
pub const MIN_ADULT_AGE: u8 = 5;

/// During the first section, nodes can start at a range of age to avoid too many nodes having the
/// same time get relocated at the same time.
/// Defines the lower bound of this range.
pub const FIRST_SECTION_MIN_AGE: u8 = MIN_ADULT_AGE + 1;
/// Defines the higher bound of this range.
pub const FIRST_SECTION_MAX_AGE: u8 = 100;

/// Information about a member of our section.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct NodeState {
    peer: Peer,
    state: MembershipState,
    previous_name: Option<XorName>,
}

impl serde::Serialize for NodeState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as a `NodeStateMsg`
        self.to_msg().serialize(serializer)
    }
}

impl NodeState {
    // Creates a `NodeState` in the `Joined` state.
    pub(crate) fn joined(peer: Peer, previous_name: Option<XorName>) -> Self {
        Self {
            peer,
            state: MembershipState::Joined,
            previous_name,
        }
    }

    // Creates a `NodeState` in the `Left` state.
    #[cfg(test)]
    pub(crate) fn left(peer: Peer, previous_name: Option<XorName>) -> Self {
        Self {
            peer,
            state: MembershipState::Left,
            previous_name,
        }
    }

    // Creates a `NodeState` in the `Relocated` state.
    #[cfg(test)]
    pub(crate) fn relocated(
        peer: Peer,
        previous_name: Option<XorName>,
        relocate_details: RelocateDetails,
    ) -> Self {
        Self {
            peer,
            state: MembershipState::Relocated(Box::new(relocate_details)),
            previous_name,
        }
    }

    pub(crate) fn peer(&self) -> &Peer {
        &self.peer
    }

    pub(crate) fn name(&self) -> XorName {
        self.peer.name()
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.peer.addr()
    }

    pub(crate) fn state(&self) -> MembershipState {
        self.state.clone()
    }

    pub(crate) fn previous_name(&self) -> Option<XorName> {
        self.previous_name
    }

    pub(crate) fn age(&self) -> u8 {
        self.peer.age()
    }

    // Returns true if the state is a Relocated node
    pub(crate) fn is_relocated(&self) -> bool {
        matches!(self.state, MembershipState::Relocated(_))
    }

    pub(crate) fn leave(self) -> Result<Self, Error> {
        // Do not allow switching to `Left` when already relocated,
        // to avoid rejoining with the same name.
        if let MembershipState::Relocated(_) = self.state {
            return Err(Error::InvalidState);
        }

        Ok(Self {
            state: MembershipState::Left,
            ..self
        })
    }

    // Convert this info into one with the state changed to `Relocated`.
    pub(crate) fn relocate(self, relocate_details: RelocateDetails) -> Self {
        Self {
            state: MembershipState::Relocated(Box::new(relocate_details)),
            ..self
        }
    }
}

// Add conversion methods to/from `messaging::...::NodeState`
// We prefer this over `From<...>` to make it easier to read the conversion.

impl NodeState {
    /// Create a message from the current state.
    pub(crate) fn to_msg(&self) -> NodeStateMsg {
        NodeStateMsg {
            name: self.name(),
            addr: self.addr(),
            state: self.state.clone(),
            previous_name: self.previous_name,
        }
    }
}

impl SectionAuth<NodeState> {
    pub(crate) fn into_authed_msg(self) -> SectionAuth<NodeStateMsg> {
        SectionAuth {
            value: self.value.to_msg(),
            sig: self.sig,
        }
    }
}

impl NodeStateMsg {
    pub(crate) fn into_state(self) -> NodeState {
        NodeState {
            peer: Peer::new(self.name, self.addr),
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
