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

/// This maps to the SocketAddr at the Elders where the EndUser is proxied through.
pub type SocketId = XorName;

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
    /// A single Node with the given name.
    Node {
        /// Name of the Node.
        name: XorName,
        /// Node's section public key
        section_pk: BlsPublicKey,
    },
    /// A Section close to a name.
    Section {
        /// Name of the Section.
        name: XorName,
        /// Section's public key
        section_pk: BlsPublicKey,
    },
}

impl SrcLocation {
    /// Returns the name of this location.
    pub fn name(&self) -> XorName {
        match self {
            Self::EndUser(user) => user.xorname,
            Self::Node { name, .. } => *name,
            Self::Section { name, .. } => *name,
        }
    }

    /// Returns this location as `DstLocation`
    pub fn to_dst(self) -> DstLocation {
        match self {
            Self::EndUser(user) => DstLocation::EndUser(user),
            Self::Node { name, section_pk } => DstLocation::Node { name, section_pk },
            Self::Section { name, section_pk } => DstLocation::Section { name, section_pk },
        }
    }
}

/// Message destination location.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum DstLocation {
    /// An EndUser.
    EndUser(EndUser),
    /// Destination is a single node with the given name.
    Node {
        /// Name of the Node.
        name: XorName,
        /// Node's section public key
        section_pk: BlsPublicKey,
    },
    /// Destination are the nodes of the section whose prefix matches the given name.
    Section {
        /// Name of the Section.
        name: XorName,
        /// Section's public key
        section_pk: BlsPublicKey,
    },
    /// Destination is a specific node to be directly connected to,
    /// and so the message is unrouted. The destination's known section key is provided.
    DirectAndUnrouted(BlsPublicKey),
}

impl DstLocation {
    /// Returns the section pk if it's not EndUser.
    pub fn section_pk(&self) -> Option<BlsPublicKey> {
        match self {
            Self::EndUser(_) => None,
            Self::Node { section_pk, .. } => Some(*section_pk),
            Self::Section { section_pk, .. } => Some(*section_pk),
            Self::DirectAndUnrouted(section_pk) => Some(*section_pk),
        }
    }

    /// Updates the section pk if it's not EndUser.
    pub fn set_section_pk(&mut self, pk: BlsPublicKey) {
        match self {
            Self::EndUser(_) => {}
            Self::Node { section_pk, .. } => *section_pk = pk,
            Self::Section { section_pk, .. } => *section_pk = pk,
            Self::DirectAndUnrouted(section_pk) => *section_pk = pk,
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

    /// Updates the name of this location if it's not `DirectAndUnrouted`.
    pub fn set_name(&mut self, new_name: XorName) {
        match self {
            Self::EndUser(EndUser { xorname, .. }) => *xorname = new_name,
            Self::Node { name, .. } => *name = new_name,
            Self::Section { name, .. } => *name = new_name,
            Self::DirectAndUnrouted(_) => {}
        }
    }
}
