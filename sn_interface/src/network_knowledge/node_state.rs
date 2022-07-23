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
use crate::network_knowledge::{section_has_room_for_node, Error, Result};
use crate::types::Peer;

use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use xor_name::{Prefix, XorName};

/// Information about a member of our section.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeState {
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
    pub fn joined(peer: Peer, previous_name: Option<XorName>) -> Self {
        Self {
            peer,
            state: MembershipState::Joined,
            previous_name,
        }
    }

    // Creates a `NodeState` in the `Left` state.
    #[cfg(feature = "test-utils")]
    pub fn left(peer: Peer, previous_name: Option<XorName>) -> Self {
        Self {
            peer,
            state: MembershipState::Left,
            previous_name,
        }
    }

    // Creates a `NodeState` in the `Relocated` state.
    #[cfg(feature = "test-utils")]
    pub fn relocated(
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

    pub fn validate(
        &self,
        prefix: &Prefix,
        members: &BTreeMap<XorName, Self>,
        archived: &BTreeSet<XorName>,
    ) -> Result<()> {
        let name = self.name();
        info!("Validating node state for {name}");

        if !prefix.matches(&name) {
            info!("Membership - rejecting node {name}, name doesn't match our prefix {prefix:?}");
            return Err(Error::WrongSection);
        }

        self.validate_relocation_details(prefix)?;

        match self.state {
            MembershipState::Joined | MembershipState::Relocated(_) => {
                if members.contains_key(&name) {
                    info!("Rejecting join from existing member {name}");
                    Err(Error::ExistingMemberConflict)
                } else if !section_has_room_for_node(name, prefix, members.keys().copied()) {
                    info!("Rejecting join since we are at capacity");
                    Err(Error::TryJoinLater)
                } else if let Some(existing_node) = members
                    .values()
                    .find(|n| n.peer().addr() == self.peer().addr())
                {
                    info!("Rejecting join since we have an existing node with this address: {existing_node:?}");
                    Err(Error::ExistingMemberConflict)
                } else if archived.contains(&name) {
                    Err(Error::ArchivedNodeRejoined)
                } else {
                    Ok(())
                }
            }
            MembershipState::Left => {
                if !members.contains_key(&name) {
                    info!("Rejecting leave from non-existing member");
                    Err(Error::NotAMember)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn validate_relocation_details(&self, prefix: &Prefix) -> Result<()> {
        if let MembershipState::Relocated(details) = &self.state {
            let name = self.name();
            let dest = details.dst;

            if !prefix.matches(&dest) {
                info!("Invalid relocation request from {name} - {dest} doesn't match our prefix {prefix:?}.");
                return Err(Error::WrongSection);
            }

            // We requires the node name matches the relocation details age.
            let age = details.age;
            let state_age = self.age();
            if age != state_age {
                info!(
		    "Invalid relocation request from {name} - relocation age ({age}) doesn't match peer's age ({state_age})."
		);
                return Err(Error::InvalidRelocationDetails);
            }
        }

        Ok(())
    }

    pub fn peer(&self) -> &Peer {
        &self.peer
    }

    pub fn name(&self) -> XorName {
        self.peer.name()
    }

    pub fn addr(&self) -> SocketAddr {
        self.peer.addr()
    }

    pub fn state(&self) -> MembershipState {
        self.state.clone()
    }

    pub fn previous_name(&self) -> Option<XorName> {
        self.previous_name
    }

    pub fn age(&self) -> u8 {
        self.peer.age()
    }

    // Returns true if the state is a Relocated node
    pub fn is_relocated(&self) -> bool {
        matches!(self.state, MembershipState::Relocated(_))
    }

    pub fn leave(self) -> Result<Self, Error> {
        // Do not allow switching to `Left` when already relocated,
        assert_eq!(self.state, MembershipState::Joined);

        Ok(Self {
            state: MembershipState::Left,
            ..self
        })
    }

    // Convert this info into one with the state changed to `Relocated`.
    pub fn relocate(self, relocate_details: RelocateDetails) -> Self {
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
    pub fn to_msg(&self) -> NodeStateMsg {
        NodeStateMsg {
            name: self.name(),
            addr: self.addr(),
            state: self.state.clone(),
            previous_name: self.previous_name,
        }
    }
}

impl SectionAuth<NodeState> {
    pub fn into_authed_msg(self) -> SectionAuth<NodeStateMsg> {
        SectionAuth {
            value: self.value.to_msg(),
            sig: self.sig,
        }
    }
}

impl NodeStateMsg {
    pub fn into_state(self) -> NodeState {
        NodeState {
            peer: Peer::new(self.name, self.addr),
            state: self.state,
            previous_name: self.previous_name,
        }
    }
}

impl SectionAuth<NodeStateMsg> {
    pub fn into_authed_state(self) -> SectionAuth<NodeState> {
        SectionAuth {
            value: self.value.into_state(),
            sig: self.sig,
        }
    }
}
