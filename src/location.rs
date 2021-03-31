// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use serde::{Deserialize, Serialize};
use sn_data_types::PublicKey;
use xor_name::{Prefix, XorName};

type SocketId = XorName;

/// The planned route of a message.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Itinerary {
    /// Source
    pub src: SrcLocation,
    /// Destionation
    pub dst: DstLocation,
    /// Wether this will be aggregated, and where.
    pub aggregation: Aggregation,
}

impl Itinerary {
    /// Elders will send their signed message, which
    /// recipients aggregate.
    pub fn aggregate_at_dst(&self) -> bool {
        matches!(self.aggregation, Aggregation::AtDestination)
    }

    /// Name of the source
    pub fn src_name(&self) -> XorName {
        self.src.name()
    }

    /// Name of the destionation
    pub fn dst_name(&self) -> Option<XorName> {
        self.dst.name()
    }
}

/// Aggregation scheme
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum Aggregation {
    /// No aggregation is made, eg. when the payload contains full authority.
    None,
    /// Elders will send their signed message, which
    /// recipients aggregate.
    AtDestination,
}

/// An EndUser is repreented by a PublicKey.
/// It uses 1-n clients to access the network.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum EndUser {
    /// All clients of this end user.
    AllClients(PublicKey),
    /// An EndUser can instantiate multiple Clients.
    /// The Clients use the same PublicKey, but different SocketAddr.
    Client {
        /// The EndUser PublicKey
        public_key: PublicKey,
        /// A hash of the EndUser signature over their socket address.
        /// This maps to the SocketAddr at the Elders where the EndUser registered.
        socket_id: SocketId,
    },
}

impl EndUser {
    /// Returns the name of this location, or `None` if it is `Direct`.
    pub fn id(&self) -> &PublicKey {
        match self {
            Self::Client { public_key, .. } => public_key,
            Self::AllClients(public_key) => public_key,
        }
    }

    /// Returns the name of this location, or `None` if it is `Direct`.
    pub fn name(&self) -> XorName {
        (*self.id()).into()
    }

    /// Returns true if the provided name equals the enduser name.
    pub fn equals(&self, name: &XorName) -> bool {
        match self {
            Self::Client { public_key, .. } => name == &(*public_key).into(),
            Self::AllClients(public_key) => name == &(*public_key).into(),
        }
    }
}

/// Message source location.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum SrcLocation {
    /// An EndUser uses one or more Clients.
    EndUser(EndUser),
    /// A single node with the given name.
    Node(XorName),
    /// A section close to a name.
    Section(XorName),
}

impl SrcLocation {
    /// Returns whether this location is a section.
    pub fn is_section(&self) -> bool {
        matches!(self, Self::Section(_))
    }

    /// Returns whether this location is a section.
    pub fn is_user(&self) -> bool {
        matches!(self, Self::EndUser(_))
    }

    /// Returns whether the given name is part of this location
    pub fn equals(&self, name: &XorName) -> bool {
        match self {
            Self::EndUser(user) => user.equals(name),
            Self::Node(self_name) => name == self_name,
            Self::Section(some_name) => name == some_name,
        }
    }

    /// Returns the name of this location, or `None` if it is `Direct`.
    pub fn name(&self) -> XorName {
        match self {
            Self::EndUser(user) => user.name(),
            Self::Node(name) => *name,
            Self::Section(name) => *name,
        }
    }

    /// Returns this location as `DstLocation`
    pub fn to_dst(&self) -> DstLocation {
        match self {
            Self::EndUser(user) => DstLocation::EndUser(*user),
            Self::Node(name) => DstLocation::Node(*name),
            Self::Section(name) => DstLocation::Section(*name),
        }
    }
}

/// Message destination location.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum DstLocation {
    /// An EndUser uses one or more Clients.
    EndUser(EndUser),
    /// Destination is a single node with the given name.
    Node(XorName),
    /// Destination are the nodes of the section whose prefix matches the given name.
    Section(XorName),
    /// Destination is the node at the `ConnectionInfo` the message is directly sent to.
    Direct,
}

impl DstLocation {
    /// Returns whether this location is a section.
    pub fn is_section(&self) -> bool {
        matches!(self, Self::Section(_))
    }

    /// Returns whether this location is a section.
    pub fn is_user(&self) -> bool {
        matches!(self, Self::EndUser(_))
    }

    /// Returns whether the given name of the given prefix is part of this location.
    ///
    /// Returns None if `prefix` does not match `name`.
    pub fn contains(&self, name: &XorName, prefix: &Prefix) -> bool {
        if !prefix.matches(name) {
            return false;
        }

        match self {
            Self::EndUser(user) => prefix.matches(&user.name()),
            Self::Node(self_name) => name == self_name,
            Self::Section(self_name) => prefix.matches(self_name),
            Self::Direct => true,
        }
    }

    /// Returns the name of this location, or `None` if it is `Direct`.
    pub fn name(&self) -> Option<XorName> {
        match self {
            Self::EndUser(user) => Some(user.name()),
            Self::Node(name) => Some(*name),
            Self::Section(name) => Some(*name),
            Self::Direct => None,
        }
    }
}
