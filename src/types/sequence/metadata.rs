// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::super::{utils, Error, PublicKey, Result, XorName};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

/// An action on Sequence data type.
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Action {
    /// Read from the data.
    Read,
    /// Append to the data.
    Append,
}

/// List of entries.
pub type Entries = Vec<Entry>;

/// An entry in a Sequence.
pub type Entry = Vec<u8>;

/// Address of a Sequence.
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

/// Kind of a Sequence.
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

/// Index of some data.
#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Index {
    /// Absolute index.
    FromStart(u64),
    /// Relative index - start counting from the end.
    FromEnd(u64),
}

impl From<u64> for Index {
    fn from(index: u64) -> Self {
        Index::FromStart(index)
    }
}

/// Set of public permissions for a user.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct PublicPermissions {
    /// `Some(true)` if the user can append.
    /// `Some(false)` explicitly denies this permission (even if `Anyone` has permissions).
    /// Use permissions for `Anyone` if `None`.
    append: Option<bool>,
}

impl PublicPermissions {
    /// Constructs a new public permission set.
    pub fn new(append: impl Into<Option<bool>>) -> Self {
        Self {
            append: append.into(),
        }
    }

    /// Sets permissions.
    pub fn set_perms(&mut self, append: impl Into<Option<bool>>) {
        self.append = append.into();
    }

    /// Returns `Some(true)` if `action` is allowed and `Some(false)` if it's not permitted.
    /// `None` means that default permissions should be applied.
    pub fn is_allowed(self, action: Action) -> Option<bool> {
        match action {
            Action::Read => Some(true), // It's public data, so it's always allowed to read it.
            Action::Append => self.append,
        }
    }
}

/// Set of private permissions for a user.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct PrivatePermissions {
    /// `true` if the user can read.
    read: bool,
    /// `true` if the user can append.
    append: bool,
}

impl PrivatePermissions {
    /// Constructs a new private permission set.
    pub fn new(read: bool, append: bool) -> Self {
        Self { read, append }
    }

    /// Sets permissions.
    pub fn set_perms(&mut self, read: bool, append: bool) {
        self.read = read;
        self.append = append;
    }

    /// Returns `true` if `action` is allowed.
    pub fn is_allowed(self, action: Action) -> bool {
        match action {
            Action::Read => self.read,
            Action::Append => self.append,
        }
    }
}

/// User that can access Sequence.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug)]
pub enum User {
    /// Any user.
    Anyone,
    /// User identified by its public key.
    Key(PublicKey),
}

/// Public permissions.
#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct PublicPolicy {
    /// An owner could represent an individual user, or a group of users,
    /// depending on the `public_key` type.
    pub owner: PublicKey,
    /// Map of users to their public permission set.
    pub permissions: BTreeMap<User, PublicPermissions>,
}

impl PublicPolicy {
    /// Returns `Some(true)` if `action` is allowed for the provided user and `Some(false)` if it's
    /// not permitted. `None` means that default permissions should be applied.
    fn is_action_allowed_by_user(&self, user: &User, action: Action) -> Option<bool> {
        self.permissions
            .get(user)
            .and_then(|perms| perms.is_allowed(action))
    }
}

/// Private permissions.
#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct PrivatePolicy {
    /// An owner could represent an individual user, or a group of users,
    /// depending on the `public_key` type.
    pub owner: PublicKey,
    /// Map of users to their private permission set.
    pub permissions: BTreeMap<PublicKey, PrivatePermissions>,
}

pub trait Perm {
    /// Returns true if `action` is allowed for the provided user.
    fn is_action_allowed(&self, requester: PublicKey, action: Action) -> Result<()>;
    /// Gets the permissions for a user if applicable.
    fn permissions(&self, user: User) -> Option<Permissions>;
    /// Returns the owner.
    fn owner(&self) -> &PublicKey;
}

impl Perm for PublicPolicy {
    /// Returns `Ok(())` if `action` is allowed for the provided user and `Err(AccessDenied)` if
    /// this action is not permitted.
    fn is_action_allowed(&self, requester: PublicKey, action: Action) -> Result<()> {
        // First checks if the requester is the owner.
        if action == Action::Read || requester == self.owner {
            Ok(())
        } else {
            match self
                .is_action_allowed_by_user(&User::Key(requester), action)
                .or_else(|| self.is_action_allowed_by_user(&User::Anyone, action))
            {
                Some(true) => Ok(()),
                Some(false) => Err(Error::AccessDenied(requester)),
                None => Err(Error::AccessDenied(requester)),
            }
        }
    }

    /// Gets the permissions for a user if applicable.
    fn permissions(&self, user: User) -> Option<Permissions> {
        self.permissions.get(&user).map(|p| Permissions::Public(*p))
    }

    /// Returns the owner.
    fn owner(&self) -> &PublicKey {
        &self.owner
    }
}

impl Perm for PrivatePolicy {
    /// Returns `Ok(())` if `action` is allowed for the provided user and `Err(AccessDenied)` if
    /// this action is not permitted.
    fn is_action_allowed(&self, requester: PublicKey, action: Action) -> Result<()> {
        // First checks if the requester is the owner.
        if requester == self.owner {
            Ok(())
        } else {
            match self.permissions.get(&requester) {
                Some(perms) => {
                    if perms.is_allowed(action) {
                        Ok(())
                    } else {
                        Err(Error::AccessDenied(requester))
                    }
                }
                None => Err(Error::AccessDenied(requester)),
            }
        }
    }

    /// Gets the permissions for a user if applicable.
    fn permissions(&self, user: User) -> Option<Permissions> {
        match user {
            User::Anyone => None,
            User::Key(key) => self.permissions.get(&key).map(|p| Permissions::Private(*p)),
        }
    }

    /// Returns the owner.
    fn owner(&self) -> &PublicKey {
        &self.owner
    }
}

/// Wrapper type for permissions, which can be public or private.
#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Policy {
    /// Public permissions.
    Public(PublicPolicy),
    /// Private permissions.
    Private(PrivatePolicy),
}

impl From<PrivatePolicy> for Policy {
    fn from(policy: PrivatePolicy) -> Self {
        Policy::Private(policy)
    }
}

impl From<PublicPolicy> for Policy {
    fn from(policy: PublicPolicy) -> Self {
        Policy::Public(policy)
    }
}

/// Wrapper type for permissions set, which can be public or private.
#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Permissions {
    /// Public permissions set.
    Public(PublicPermissions),
    /// Private permissions set.
    Private(PrivatePermissions),
}

impl From<PrivatePermissions> for Permissions {
    fn from(permission_set: PrivatePermissions) -> Self {
        Permissions::Private(permission_set)
    }
}

impl From<PublicPermissions> for Permissions {
    fn from(permission_set: PublicPermissions) -> Self {
        Permissions::Public(permission_set)
    }
}
