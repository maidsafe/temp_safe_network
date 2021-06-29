// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use bls::PublicKey as BlsPublicKey;
use serde::{Deserialize, Serialize};
use xor_name::{Prefix, XorName};

type SocketId = XorName;

/// The planned route of a message.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Itinerary {
    /// Source
    pub src: SrcLocation,
    /// Destionation
    pub dst: DstLocation,
    // /// Wether this will be aggregated, and where.
    // pub aggregation: Aggregation,
}

impl Itinerary {
    /*
        /// Elders will send their signed message, where recipients aggregate.
        pub fn aggregate_at_dst(&self) -> bool {
            matches!(self.aggregation, Aggregation::AtDestination)
        }

        /// Elders will aggregate a group sig before they each send one copy of it to dst.
        pub fn aggregate_at_src(&self) -> bool {
            matches!(self.aggregation, Aggregation::AtSource)
        }
    */
    /// Name of the source
    pub fn src_name(&self) -> XorName {
        self.src.name()
    }

    /// Name of the destionation
    pub fn dst_name(&self) -> Option<XorName> {
        self.dst.name()
    }
}

/// An EndUser is represented by the name
/// it's proxied through, and its socket id.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct EndUser {
    /// The name it's proxied through
    pub xorname: XorName,
    /// This maps to the SocketAddr at the Elders where the EndUser is proxied through.
    pub socket_id: SocketId,
}

/// Message source location.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum SrcLocation {
    /// An EndUser.
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

    /// Returns whether this location is an end user.
    pub fn is_user(&self) -> bool {
        matches!(self, Self::EndUser(_))
    }

    /// Returns whether the given name is part of this location
    pub fn equals(&self, name: &XorName) -> bool {
        match self {
            Self::EndUser(user) => &user.xorname == name,
            Self::Node(self_name) => name == self_name,
            Self::Section(some_name) => name == some_name,
        }
    }

    /// Returns the name of this location.
    pub fn name(&self) -> XorName {
        match self {
            Self::EndUser(user) => user.xorname,
            Self::Node(name) => *name,
            Self::Section(name) => *name,
        }
    }

    /// Returns this location as `DstLocation`
    pub fn to_dst(self) -> DstLocation {
        unimplemented!();
        /*match self {
            Self::EndUser(user) => DstLocation::EndUser(user),
            Self::Node(name) => DstLocation::Node(name),
            Self::Section(name) => DstLocation::Section(name),
        }*/
    }
}

/// Message destination location.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum DstLocation {
    /// An EndUser.
    EndUser(EndUser),
    /// Destination is a single node with the given name.
    Node {
        name: XorName,
        section_pk: BlsPublicKey,
    },
    /// Destination are the nodes of the section whose prefix matches the given name.
    Section {
        name: XorName,
        section_pk: BlsPublicKey,
    },
    /// Destination is a specific node to be directly connected to,
    /// and so the message is unrouted. The destination's known section key is provided.
    DirectAndUnrouted(BlsPublicKey),
}

impl DstLocation {
    /// Returns whether this location is a section.
    pub fn is_section(&self) -> bool {
        matches!(self, Self::Section { .. })
    }

    /// Returns whether this location is an end user.
    pub fn is_user(&self) -> bool {
        matches!(self, Self::EndUser(_))
    }

    /// Returns the section pk if it's not EndUser.
    pub fn section_pk(&self) -> Option<BlsPublicKey> {
        match self {
            Self::EndUser(_) => None,
            Self::Node { section_pk, .. } => Some(*section_pk),
            Self::Section { section_pk, .. } => Some(*section_pk),
            Self::DirectAndUnrouted(section_pk) => Some(*section_pk),
        }
    }

    /// Returns whether the given name of the given prefix is part of this location.
    ///
    /// Returns None if `prefix` does not match `name`.
    pub fn contains(&self, name: &XorName, prefix: &Prefix) -> bool {
        if !prefix.matches(name) {
            return false;
        }

        match self {
            Self::EndUser(user) => prefix.matches(&user.xorname),
            Self::Node {
                name: self_name, ..
            } => name == self_name,
            Self::Section {
                name: self_name, ..
            } => prefix.matches(self_name),
            Self::DirectAndUnrouted(_) => true,
        }
    }

    /// Returns the name of this location, or `None` if it is `Direct`.
    pub fn name(&self) -> Option<XorName> {
        match self {
            Self::EndUser(user) => Some(user.xorname),
            Self::Node { name, .. } => Some(*name),
            Self::Section { name, .. } => Some(*name),
            Self::DirectAndUnrouted(_) => None,
        }
    }
}
