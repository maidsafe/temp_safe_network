// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Map
//!
//! Map can be either private or public.
//!
//! The client must specify the next version number of a value while
//! modifying/deleting keys. Similarly, while modifying the Map shell (permissions,
//! ownership, etc.), the next version number must be passed.

use super::{utils, Error, PublicKey, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    fmt::{self, Debug, Formatter},
    mem,
};
use xor_name::XorName;

/// Map that is unpublished on the network. This data can only be fetched by the owner or
/// those in the permissions fields with `Permission::Read` access.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct Map {
    /// Network address.
    address: Address,
    /// Key-Value semantics.
    data: Entries,
    /// Maps an application key to a list of allowed or forbidden actions.
    permissions: BTreeMap<PublicKey, PermissionSet>,
    /// Version should be increased for any changes to Map fields except for data.
    version: u64,
    /// Contains the public key of an owner or owners of this data.
    ///
    /// Data Handlers in nodes enforce that a mutation request has a valid signature of the owner.
    owner: PublicKey,
}

impl Debug for Map {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Map {:?}", self.name())
    }
}

/// A value in a Map.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct Value {
    /// Pointer.
    pub pointer: XorName,
    /// Version, incremented sequentially for any change to `data`.
    pub version: u64,
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:<8} :: {}", hex::encode(&self.pointer), self.version)
    }
}

/// Wrapper type for lists of values.
pub type Values = Vec<Value>;

/// Entries (key-value pairs, with versioned values).
pub type Entries = BTreeMap<Vec<u8>, Value>;

/// Set of user permissions.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct PermissionSet {
    permissions: BTreeSet<Action>,
}

impl PermissionSet {
    /// Constructs new permission set.
    pub fn new() -> PermissionSet {
        PermissionSet {
            permissions: Default::default(),
        }
    }

    /// Allows the given action.
    pub fn allow(mut self, action: Action) -> Self {
        let _ = self.permissions.insert(action);
        self
    }

    /// Denies the given action.
    pub fn deny(mut self, action: Action) -> Self {
        let _ = self.permissions.remove(&action);
        self
    }

    /// Is the given action allowed according to this permission set?
    pub fn is_allowed(&self, action: Action) -> bool {
        self.permissions.contains(&action)
    }
}

/// Set of Actions that can be performed on the Map.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Action {
    /// Permission to read entries.
    Read,
    /// Permission to insert new entries.
    Insert,
    /// Permission to update existing entries.
    Update,
    /// Permission to delete existing entries.
    Delete,
    /// Permission to modify permissions for other users.
    ManagePermissions,
}

macro_rules! impl_map {
    ($flavour:ident) => {
        impl $flavour {
            /// Returns the address.
            pub fn address(&self) -> &Address {
                &self.address
            }

            /// Returns the name.
            pub fn name(&self) -> &XorName {
                self.address.name()
            }

            /// Returns the tag type.
            pub fn tag(&self) -> u64 {
                self.address.tag()
            }

            /// Returns the kind.
            pub fn kind(&self) -> Kind {
                self.address.kind()
            }

            /// Returns the version of the Map fields (not the data version).
            pub fn version(&self) -> u64 {
                self.version
            }

            /// Returns the owner key.
            pub fn owner(&self) -> &PublicKey {
                &self.owner
            }

            /// Returns all the keys in the data.
            pub fn keys(&self) -> BTreeSet<Vec<u8>> {
                self.data.keys().cloned().collect()
            }

            /// Returns the shell of this Map (the fields without the data).
            pub fn shell(&self) -> Self {
                Self {
                    address: self.address.clone(),
                    data: BTreeMap::new(),
                    permissions: self.permissions.clone(),
                    version: self.version,
                    owner: self.owner,
                }
            }

            /// Gets a complete list of permissions.
            pub fn permissions(&self) -> BTreeMap<PublicKey, PermissionSet> {
                self.permissions.clone()
            }

            /// Gets the permissions for the provided user.
            pub fn user_permissions(&self, user: &PublicKey) -> Result<&PermissionSet> {
                self.permissions.get(user).ok_or(Error::NoSuchKey)
            }

            /// Checks if the provided user is an owner.
            ///
            /// Returns `Ok(())` on success and `Err(Error::AccessDenied)` if the user is not an
            /// owner.
            pub fn check_is_owner(&self, requester: &PublicKey) -> Result<()> {
                if &self.owner == requester {
                    Ok(())
                } else {
                    Err(Error::AccessDenied(*requester))
                }
            }

            /// Checks permissions for given `action` for the provided user.
            ///
            /// Returns `Err(Error::AccessDenied)` if the permission check has failed.
            pub fn check_permissions(&self, action: Action, requester: &PublicKey) -> Result<()> {
                if &self.owner == requester {
                    Ok(())
                } else {
                    let permissions = self
                        .user_permissions(requester)
                        .map_err(|_| Error::AccessDenied(*requester))?;
                    if permissions.is_allowed(action) {
                        Ok(())
                    } else {
                        Err(Error::AccessDenied(*requester))
                    }
                }
            }

            /// Inserts or updates permissions for the provided user.
            ///
            /// Requires the new `version` of the Map fields. If it does not match the
            /// current version + 1, an error will be returned.
            pub fn set_user_permissions(
                &mut self,
                user: PublicKey,
                permissions: PermissionSet,
                version: u64,
            ) -> Result<()> {
                if version != self.version + 1 {
                    return Err(Error::InvalidSuccessor(self.version));
                }

                let _prev = self.permissions.insert(user, permissions);
                self.version = version;

                Ok(())
            }

            /// Deletes permissions for the provided user.
            ///
            /// Requires the new `version` of the Map fields. If it does not match the
            /// current version + 1, an error will be returned.
            pub fn del_user_permissions(&mut self, user: PublicKey, version: u64) -> Result<()> {
                if version != self.version + 1 {
                    return Err(Error::InvalidSuccessor(self.version));
                }
                if !self.permissions.contains_key(&user) {
                    return Err(Error::NoSuchKey);
                }

                let _ = self.permissions.remove(&user);
                self.version = version;

                Ok(())
            }

            /// Deletes user permissions without performing any validation.
            ///
            /// Requires the new `version` of the Map fields. If it does not match the
            /// current version + 1, an error will be returned.
            pub fn del_user_permissions_without_validation(
                &mut self,
                user: PublicKey,
                version: u64,
            ) -> bool {
                if version <= self.version {
                    return false;
                }

                let _ = self.permissions.remove(&user);
                self.version = version;

                true
            }

            /// Changes the owner.
            ///
            /// Requires the new `version` of the Map fields. If it does not match the
            /// current version + 1, an error will be returned.
            pub fn change_owner(&mut self, new_owner: PublicKey, version: u64) -> Result<()> {
                if version != self.version + 1 {
                    return Err(Error::InvalidSuccessor(self.version));
                }

                self.owner = new_owner;
                self.version = version;

                Ok(())
            }

            /// Changes the owner without performing any validation.
            ///
            /// Requires the new `version` of the Map fields. If it does not match the
            /// current version + 1, an error will be returned.
            pub fn change_owner_without_validation(
                &mut self,
                new_owner: PublicKey,
                version: u64,
            ) -> bool {
                if version <= self.version {
                    return false;
                }

                self.owner = new_owner;
                self.version = version;

                true
            }

            /// Returns true if `action` is allowed for the provided user.
            pub fn is_action_allowed(&self, requester: &PublicKey, action: Action) -> bool {
                match self.permissions.get(requester) {
                    Some(perms) => perms.is_allowed(action),
                    None => false,
                }
            }
        }
    };
}

impl_map!(Map);

/// Implements functions for Map.
impl Map {
    /// Creates a new Map.
    pub fn new(name: XorName, tag: u64, owner: PublicKey, kind: Kind) -> Self {
        Self {
            address: Address::from_kind(kind, name, tag),
            data: Default::default(),
            permissions: Default::default(),
            version: 0,
            owner,
        }
    }

    /// Creates a new Map with entries and permissions.
    pub fn new_with_data(
        name: XorName,
        tag: u64,
        data: Entries,
        permissions: BTreeMap<PublicKey, PermissionSet>,
        owner: PublicKey,
        kind: Kind,
    ) -> Self {
        Self {
            address: Address::from_kind(kind, name, tag),
            data,
            permissions,
            version: 0,
            owner,
        }
    }

    /// Returns a value by the given key
    pub fn get(&self, key: &[u8]) -> Option<&Value> {
        self.data.get(key)
    }

    /// Returns values of all entries
    pub fn values(&self) -> Vec<Value> {
        self.data.values().cloned().collect()
    }

    /// Returns all entries
    pub fn entries(&self) -> &Entries {
        &self.data
    }

    /// Removes and returns all entries
    pub fn take_entries(&mut self) -> Entries {
        mem::take(&mut self.data)
    }

    /// Mutates entries (key + value pairs) in bulk.
    ///
    /// Returns `Err(InvalidEntryActions)` if the mutation parameters are invalid.
    pub fn mutate_entries(&mut self, actions: EntryActions, requester: &PublicKey) -> Result<()> {
        // Deconstruct actions into inserts, updates, and deletes
        let (insert, update, delete) = actions.actions.into_iter().fold(
            (BTreeMap::new(), BTreeMap::new(), BTreeMap::new()),
            |(mut insert, mut update, mut delete), (key, item)| {
                match item {
                    EntryAction::Insert(value) => {
                        let _ = insert.insert(key, value);
                    }
                    EntryAction::Update(value) => {
                        let _ = update.insert(key, value);
                    }
                    EntryAction::Delete(version) => {
                        let _ = delete.insert(key, version);
                    }
                };
                (insert, update, delete)
            },
        );

        if self.owner() != requester
            && ((!insert.is_empty() && !self.is_action_allowed(requester, Action::Insert))
                || (!update.is_empty() && !self.is_action_allowed(requester, Action::Update))
                || (!delete.is_empty() && !self.is_action_allowed(requester, Action::Delete)))
        {
            return Err(Error::AccessDenied(*requester));
        }

        let mut new_data = self.data.clone();
        let mut errors = BTreeMap::new();

        for (key, val) in insert {
            match new_data.entry(key) {
                Entry::Occupied(entry) => {
                    let _ = errors.insert(
                        entry.key().clone(),
                        Error::EntryExists(entry.get().version as u8),
                    );
                }
                Entry::Vacant(entry) => {
                    let _ = entry.insert(val);
                }
            }
        }

        for (key, val) in update {
            match new_data.entry(key) {
                Entry::Occupied(mut entry) => {
                    let current_version = entry.get().version;
                    if val.version == current_version + 1 {
                        let _ = entry.insert(val);
                    } else {
                        let _ = errors.insert(
                            entry.key().clone(),
                            Error::InvalidSuccessor(current_version),
                        );
                    }
                }
                Entry::Vacant(entry) => {
                    let _ = errors.insert(entry.key().clone(), Error::NoSuchEntry);
                }
            }
        }

        for (key, version) in delete {
            match new_data.entry(key.clone()) {
                Entry::Occupied(entry) => {
                    let current_version = entry.get().version;
                    if version == current_version + 1 {
                        let _ = new_data.remove(&key);
                    } else {
                        let _ = errors.insert(
                            entry.key().clone(),
                            Error::InvalidSuccessor(current_version),
                        );
                    }
                }
                Entry::Vacant(entry) => {
                    let _ = errors.insert(entry.key().clone(), Error::NoSuchEntry);
                }
            }
        }

        if !errors.is_empty() {
            return Err(Error::InvalidEntryActions(errors));
        }

        let _old_data = mem::replace(&mut self.data, new_data);

        Ok(())
    }
}
/// Kind of a Map.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Kind {
    /// Public map.
    Public,
    /// Private map.
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

/// Address of an Map.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Address {
    ///
    Public {
        /// Name.
        name: XorName,
        /// Tag.
        tag: u64,
    },
    ///
    Private {
        /// Name.
        name: XorName,
        /// Tag.
        tag: u64,
    },
}

impl Address {
    /// Constructs an `Address` given `kind`, `name`, and `tag`.
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
            Address::Private { ref name, .. } | Address::Public { ref name, .. } => name,
        }
    }

    /// Returns the tag.
    pub fn tag(&self) -> u64 {
        match self {
            Address::Private { tag, .. } | Address::Public { tag, .. } => *tag,
        }
    }

    /// Return `true` if public.
    pub fn is_public(&self) -> bool {
        self.kind().is_public()
    }

    /// Return `true` if private.
    pub fn is_private(&self) -> bool {
        self.kind().is_private()
    }

    /// Returns the Address serialised and encoded in z-base-32.
    pub fn encode_to_zbase32(&self) -> Result<String> {
        utils::encode(&self)
    }

    /// Creates from z-base-32 encoded string.
    pub fn decode_from_zbase32<T: AsRef<str>>(encoded: T) -> Result<Self> {
        utils::decode(encoded)
    }
}

/// Action for a Entry.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub enum EntryAction {
    /// Inserts a new entry.
    Insert(Value),
    /// Updates an entry with a new value and version.
    Update(Value),
    /// Deletes an entry.
    Delete(u64),
}

impl EntryAction {
    /// Returns the version for this action.
    pub fn version(&self) -> u64 {
        match *self {
            Self::Insert(ref value) => value.version,
            Self::Update(ref value) => value.version,
            Self::Delete(v) => v,
        }
    }

    /// Sets the version for this action.
    pub fn set_version(&mut self, version: u64) {
        match *self {
            Self::Insert(ref mut value) => value.version = version,
            Self::Update(ref mut value) => value.version = version,
            Self::Delete(ref mut v) => *v = version,
        }
    }
}

/// Entry Actions for given entry keys.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntryActions {
    // A map containing keys and corresponding entry actions to perform.
    actions: BTreeMap<Vec<u8>, EntryAction>,
}

impl EntryActions {
    /// Creates a new Entry Actions list.
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets the actions.
    pub fn actions(&self) -> &BTreeMap<Vec<u8>, EntryAction> {
        &self.actions
    }

    /// Converts `self` to a map of the keys with their corresponding action.
    pub fn into_actions(self) -> BTreeMap<Vec<u8>, EntryAction> {
        self.actions
    }

    /// Inserts a new key-value pair.
    ///
    /// Requires the new `version` of the entry content. If it does not match the current
    /// version + 1, an error will be returned.
    pub fn insert(mut self, key: Vec<u8>, pointer: XorName, version: u64) -> Self {
        let _ = self
            .actions
            .insert(key, EntryAction::Insert(Value { pointer, version }));
        self
    }

    /// Updates an existing key-value pair.
    ///
    /// Requires the new `version` of the entry content. If it does not match the current
    /// version + 1, an error will be returned.
    pub fn update(mut self, key: Vec<u8>, pointer: XorName, version: u64) -> Self {
        let _ = self
            .actions
            .insert(key, EntryAction::Update(Value { pointer, version }));
        self
    }

    /// Deletes an entry.
    ///
    /// Requires the new `version` of the entry content. If it does not match the current
    /// version + 1, an error will be returned.
    pub fn delete(mut self, key: Vec<u8>, version: u64) -> Self {
        let _ = self.actions.insert(key, EntryAction::Delete(version));
        self
    }

    /// Adds an action to the list of actions, replacing it if it is already present.
    pub fn add_action(&mut self, key: Vec<u8>, action: EntryAction) {
        let _ = self.actions.insert(key, action);
    }
}

impl From<EntryActions> for BTreeMap<Vec<u8>, EntryAction> {
    fn from(actions: EntryActions) -> Self {
        actions.actions
    }
}

impl From<BTreeMap<Vec<u8>, EntryAction>> for EntryActions {
    fn from(actions: BTreeMap<Vec<u8>, EntryAction>) -> Self {
        EntryActions { actions }
    }
}

#[cfg(test)]
mod tests {
    use super::Result;
    use super::{Address, XorName};

    #[test]
    fn zbase32_encode_decode_map_address() -> Result<()> {
        let name = XorName(rand::random());
        let address = Address::Public { name, tag: 15000 };
        let encoded = address.encode_to_zbase32()?;
        let decoded = self::Address::decode_from_zbase32(&encoded)?;
        assert_eq!(address, decoded);
        Ok(())
    }
}
