// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::super::{utils, Result, XorName};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// An action on Register data type.
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Action {
    /// Read from the data.
    Read,
    /// Write to the data.
    Write,
}

/// An entry in a Register.
pub type Entry = Vec<u8>;

/// Address of a Register.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Address {
    /// Public sequence namespace.
    Public {
        /// Name.
        name: XorName,
        /// Tag.
        tag: u64,
    },
    /// Private sequence namespace.
    Private {
        /// Name.
        name: XorName,
        /// Tag.
        tag: u64,
    },
}

impl Address {
    /// Constructs a new `Address` given `kind`, `name`, and `tag`.
    pub fn from_kind(kind: Kind, name: XorName, tag: u64) -> Self {
        match kind {
            Kind::Public => Address::Public { name, tag },
            Kind::Private => Address::Private { name, tag },
        }
    }

    /// Returns the kind.
    pub fn kind(&self) -> Kind {
        match self {
            Address::Public { .. } => Kind::Public,
            Address::Private { .. } => Kind::Private,
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &XorName {
        match self {
            Address::Public { ref name, .. } | Address::Private { ref name, .. } => name,
        }
    }

    /// Returns the tag.
    pub fn tag(&self) -> u64 {
        match self {
            Address::Public { tag, .. } | Address::Private { tag, .. } => *tag,
        }
    }

    /// Returns true if public.
    pub fn is_public(&self) -> bool {
        self.kind().is_public()
    }

    /// Returns true if private.
    pub fn is_private(&self) -> bool {
        self.kind().is_private()
    }

    /// Returns the `Address` serialised and encoded in z-base-32.
    pub fn encode_to_zbase32(&self) -> Result<String> {
        utils::encode(&self)
    }

    /// Creates from z-base-32 encoded string.
    pub fn decode_from_zbase32<I: AsRef<str>>(encoded: I) -> Result<Self> {
        utils::decode(encoded)
    }
}

/// Kind of a Register.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Kind {
    /// Public sequence.
    Public,
    /// Private sequence.
    Private,
}

impl Kind {
    /// Returns true if public.
    pub fn is_public(self) -> bool {
        self == Kind::Public
    }

    /// Returns true if private.
    pub fn is_private(self) -> bool {
        !self.is_public()
    }
}
