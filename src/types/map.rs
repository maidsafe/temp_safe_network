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
//! All Map is unpublished. Map can be either sequenced or unsequenced.
//!
//! ## Private data
//!
//! Please see `append_only_data.rs` for more about unpublished versus published data.
//!
//! ## Sequenced and unsequenced data.
//!
//! Explicitly sequencing all mutations is an option provided for clients to allow them to avoid
//! dealing with conflicting mutations. However, we don't need the version for preventing replay
//! attacks.
//!
//! For sequenced Map the client must specify the next version number of a value while
//! modifying/deleting keys. Similarly, while modifying the Map shell (permissions,
//! ownership, etc.), the next version number must be passed. For unsequenced Map the client
//! does not have to pass version numbers for keys, but it still must pass the next version number
//! while modifying the Map shell.

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
pub struct SeqData {
    /// Network address.
    address: Address,
    /// Key-Value semantics.
    data: SeqEntries,
    /// Maps an application key to a list of allowed or forbidden actions.
    permissions: BTreeMap<PublicKey, PermissionSet>,
    /// Version should be increased for any changes to Map fields except for data.
    version: u64,
    /// Contains the public key of an owner or owners of this data.
    ///
    /// Data Handlers in nodes enforce that a mutation request has a valid signature of the owner.
    owner: PublicKey,
}

impl Debug for SeqData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "SeqMap {:?}", self.name())
    }
}

/// Map that is unpublished on the network. This data can only be fetched by the owner or
/// those in the permissions fields with `Permission::Read` access.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct UnseqData {
    /// Network address.
    address: Address,
    /// Key-Value semantics.
    data: UnseqEntries,
    /// Maps an application key to a list of allowed or forbidden actions.
    permissions: BTreeMap<PublicKey, PermissionSet>,
    /// Version should be increased for any changes to Map fields except for data.
    version: u64,
    /// Contains the public key of an owner or owners of this data.
    ///
    /// Data Handlers in nodes enforce that a mutation request has a valid signature of the owner.
    owner: PublicKey,
}

impl Debug for UnseqData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "UnseqMap {:?}", self.name())
    }
}

/// A value in sequenced Map.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct SeqValue {
    /// Actual data.
    pub data: Vec<u8>,
    /// Version, incremented sequentially for any change to `data`.
    pub version: u64,
}

impl Debug for SeqValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:<8} :: {}", hex::encode(&self.data), self.version)
    }
}

/// Wrapper type for values, which can be sequenced or unsequenced.
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum Value {
    /// Sequenced value.
    Seq(SeqValue),
    /// Unsequenced value.
    Unseq(Vec<u8>),
}

impl From<SeqValue> for Value {
    fn from(value: SeqValue) -> Self {
        Value::Seq(value)
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Unseq(value)
    }
}

/// Wrapper type for lists of sequenced or unsequenced values.
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum Values {
    /// List of sequenced values.
    Seq(Vec<SeqValue>),
    /// List of unsequenced values.
    Unseq(Vec<Vec<u8>>),
}

impl From<Vec<SeqValue>> for Values {
    fn from(values: Vec<SeqValue>) -> Self {
        Values::Seq(values)
    }
}

impl From<Vec<Vec<u8>>> for Values {
    fn from(values: Vec<Vec<u8>>) -> Self {
        Values::Unseq(values)
    }
}

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

impl_map!(SeqData);
impl_map!(UnseqData);

impl UnseqData {
    /// Creates a new unsequenced Map.
    pub fn new(name: XorName, tag: u64, owner: PublicKey) -> Self {
        Self {
            address: Address::Unseq { name, tag },
            data: Default::default(),
            permissions: Default::default(),
            version: 0,
            owner,
        }
    }

    /// Creates a new unsequenced Map with entries and permissions.
    pub fn new_with_data(
        name: XorName,
        tag: u64,
        data: UnseqEntries,
        permissions: BTreeMap<PublicKey, PermissionSet>,
        owner: PublicKey,
    ) -> Self {
        Self {
            address: Address::Unseq { name, tag },
            data,
            permissions,
            version: 0,
            owner,
        }
    }

    /// Returns a value for the given key.
    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.data.get(key)
    }

    /// Returns values of all entries.
    pub fn values(&self) -> Vec<Vec<u8>> {
        self.data.values().cloned().collect()
    }

    /// Returns all entries.
    pub fn entries(&self) -> &UnseqEntries {
        &self.data
    }

    /// Removes and returns all entries.
    pub fn take_entries(&mut self) -> UnseqEntries {
        mem::replace(&mut self.data, BTreeMap::new())
    }

    /// Mutates entries based on `actions` for the provided user.
    ///
    /// Returns `Err(InvalidEntryActions)` if the mutation parameters are invalid.
    pub fn mutate_entries(
        &mut self,
        actions: UnseqEntryActions,
        requester: &PublicKey,
    ) -> Result<()> {
        let (insert, update, delete) = actions.actions.into_iter().fold(
            (
                BTreeMap::<Vec<u8>, Vec<u8>>::new(),
                BTreeMap::<Vec<u8>, Vec<u8>>::new(),
                BTreeSet::<Vec<u8>>::new(),
            ),
            |(mut insert, mut update, mut delete), (key, item)| {
                match item {
                    UnseqEntryAction::Ins(value) => {
                        let _ = insert.insert(key, value);
                    }
                    UnseqEntryAction::Update(value) => {
                        let _ = update.insert(key, value);
                    }
                    UnseqEntryAction::Del => {
                        let _ = delete.insert(key);
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
                    let _ = errors.insert(entry.key().clone(), Error::EntryExists(0));
                }
                Entry::Vacant(entry) => {
                    let _ = entry.insert(val);
                }
            }
        }

        for (key, val) in update {
            match new_data.entry(key) {
                Entry::Occupied(mut entry) => {
                    let _ = entry.insert(val);
                }
                Entry::Vacant(entry) => {
                    let _ = errors.insert(entry.key().clone(), Error::NoSuchEntry);
                }
            }
        }

        for key in delete {
            match new_data.entry(key.clone()) {
                Entry::Occupied(_) => {
                    let _ = new_data.remove(&key);
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

/// Implements functions for sequenced Map.
impl SeqData {
    /// Creates a new sequenced Map.
    pub fn new(name: XorName, tag: u64, owner: PublicKey) -> Self {
        Self {
            address: Address::Seq { name, tag },
            data: Default::default(),
            permissions: Default::default(),
            version: 0,
            owner,
        }
    }

    /// Creates a new sequenced Map with entries and permissions.
    pub fn new_with_data(
        name: XorName,
        tag: u64,
        data: SeqEntries,
        permissions: BTreeMap<PublicKey, PermissionSet>,
        owner: PublicKey,
    ) -> Self {
        Self {
            address: Address::Seq { name, tag },
            data,
            permissions,
            version: 0,
            owner,
        }
    }

    /// Returns a value by the given key
    pub fn get(&self, key: &[u8]) -> Option<&SeqValue> {
        self.data.get(key)
    }

    /// Returns values of all entries
    pub fn values(&self) -> Vec<SeqValue> {
        self.data.values().cloned().collect()
    }

    /// Returns all entries
    pub fn entries(&self) -> &SeqEntries {
        &self.data
    }

    /// Removes and returns all entries
    pub fn take_entries(&mut self) -> SeqEntries {
        mem::replace(&mut self.data, BTreeMap::new())
    }

    /// Mutates entries (key + value pairs) in bulk.
    ///
    /// Returns `Err(InvalidEntryActions)` if the mutation parameters are invalid.
    pub fn mutate_entries(
        &mut self,
        actions: SeqEntryActions,
        requester: &PublicKey,
    ) -> Result<()> {
        // Deconstruct actions into inserts, updates, and deletes
        let (insert, update, delete) = actions.actions.into_iter().fold(
            (BTreeMap::new(), BTreeMap::new(), BTreeMap::new()),
            |(mut insert, mut update, mut delete), (key, item)| {
                match item {
                    SeqEntryAction::Ins(value) => {
                        let _ = insert.insert(key, value);
                    }
                    SeqEntryAction::Update(value) => {
                        let _ = update.insert(key, value);
                    }
                    SeqEntryAction::Del(version) => {
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
    /// Unsequenced.
    Unseq,
    /// Sequenced.
    Seq,
}

impl Kind {
    /// Creates `Kind` from a `sequenced` flag.
    pub fn from_flag(sequenced: bool) -> Self {
        if sequenced {
            Kind::Seq
        } else {
            Kind::Unseq
        }
    }

    /// Returns `true` if sequenced.
    pub fn is_seq(self) -> bool {
        self == Kind::Seq
    }

    /// Returns `true` if unsequenced.
    pub fn is_unseq(self) -> bool {
        !self.is_seq()
    }
}

/// Address of an Map.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Address {
    /// Unsequenced namespace.
    Unseq {
        /// Name.
        name: XorName,
        /// Tag.
        tag: u64,
    },
    /// Sequenced namespace.
    Seq {
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
            Kind::Seq => Address::Seq { name, tag },
            Kind::Unseq => Address::Unseq { name, tag },
        }
    }

    /// Returns the kind.
    pub fn kind(&self) -> Kind {
        match self {
            Address::Seq { .. } => Kind::Seq,
            Address::Unseq { .. } => Kind::Unseq,
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &XorName {
        match self {
            Address::Unseq { ref name, .. } | Address::Seq { ref name, .. } => name,
        }
    }

    /// Returns the tag.
    pub fn tag(&self) -> u64 {
        match self {
            Address::Unseq { tag, .. } | Address::Seq { tag, .. } => *tag,
        }
    }

    /// Returns `true` if sequenced.
    pub fn is_seq(&self) -> bool {
        self.kind().is_seq()
    }

    /// Returns `true` if unsequenced.
    pub fn is_unseq(&self) -> bool {
        self.kind().is_unseq()
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

/// Object storing a Map variant.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Data {
    /// Sequenced Map.
    Seq(SeqData),
    /// Unsequenced Map.
    Unseq(UnseqData),
}

impl Data {
    /// Returns the address of the data.
    pub fn address(&self) -> &Address {
        match self {
            Data::Seq(data) => data.address(),
            Data::Unseq(data) => data.address(),
        }
    }

    /// Returns the kind of the data.
    pub fn kind(&self) -> Kind {
        self.address().kind()
    }

    /// Returns the name of the data.
    pub fn name(&self) -> &XorName {
        self.address().name()
    }

    /// Returns the tag of the data.
    pub fn tag(&self) -> u64 {
        self.address().tag()
    }

    /// Returns true if the data is sequenced.
    pub fn is_seq(&self) -> bool {
        self.kind().is_seq()
    }

    /// Returns true if the data is unsequenced.
    pub fn is_unseq(&self) -> bool {
        self.kind().is_unseq()
    }

    /// Returns the version of this data.
    pub fn version(&self) -> u64 {
        match self {
            Data::Seq(data) => data.version(),
            Data::Unseq(data) => data.version(),
        }
    }

    /// Returns all the keys in the data.
    pub fn keys(&self) -> BTreeSet<Vec<u8>> {
        match self {
            Data::Seq(data) => data.keys(),
            Data::Unseq(data) => data.keys(),
        }
    }

    /// Returns the shell of the data.
    pub fn shell(&self) -> Self {
        match self {
            Data::Seq(data) => Data::Seq(data.shell()),
            Data::Unseq(data) => Data::Unseq(data.shell()),
        }
    }

    /// Gets a complete list of permissions.
    pub fn permissions(&self) -> BTreeMap<PublicKey, PermissionSet> {
        match self {
            Data::Seq(data) => data.permissions(),
            Data::Unseq(data) => data.permissions(),
        }
    }

    /// Gets the permissions for the provided user.
    pub fn user_permissions(&self, user: &PublicKey) -> Result<&PermissionSet> {
        match self {
            Data::Seq(data) => data.user_permissions(user),
            Data::Unseq(data) => data.user_permissions(user),
        }
    }

    /// Inserts or update permissions for the provided user.
    pub fn set_user_permissions(
        &mut self,
        user: PublicKey,
        permissions: PermissionSet,
        version: u64,
    ) -> Result<()> {
        match self {
            Data::Seq(data) => data.set_user_permissions(user, permissions, version),
            Data::Unseq(data) => data.set_user_permissions(user, permissions, version),
        }
    }

    /// Deletes permissions for the provided user.
    pub fn del_user_permissions(&mut self, user: PublicKey, version: u64) -> Result<()> {
        match self {
            Data::Seq(data) => data.del_user_permissions(user, version),
            Data::Unseq(data) => data.del_user_permissions(user, version),
        }
    }

    /// Checks permissions for given `action` for the provided user.
    pub fn check_permissions(&self, action: Action, requester: &PublicKey) -> Result<()> {
        match self {
            Data::Seq(data) => data.check_permissions(action, requester),
            Data::Unseq(data) => data.check_permissions(action, requester),
        }
    }

    /// Checks if the provided user is an owner.
    pub fn check_is_owner(&self, requester: &PublicKey) -> Result<()> {
        match self {
            Data::Seq(data) => data.check_is_owner(requester),
            Data::Unseq(data) => data.check_is_owner(requester),
        }
    }

    /// Returns the owner key.
    pub fn owner(&self) -> PublicKey {
        match self {
            Data::Seq(data) => data.owner,
            Data::Unseq(data) => data.owner,
        }
    }

    /// Mutates entries (key + value pairs) in bulk.
    pub fn mutate_entries(&mut self, actions: EntryActions, requester: &PublicKey) -> Result<()> {
        match self {
            Data::Seq(data) => {
                if let EntryActions::Seq(actions) = actions {
                    return data.mutate_entries(actions, requester);
                }
            }
            Data::Unseq(data) => {
                if let EntryActions::Unseq(actions) = actions {
                    return data.mutate_entries(actions, requester);
                }
            }
        }

        Err(Error::InvalidOperation)
    }
}

impl From<SeqData> for Data {
    fn from(data: SeqData) -> Self {
        Data::Seq(data)
    }
}

impl From<UnseqData> for Data {
    fn from(data: UnseqData) -> Self {
        Data::Unseq(data)
    }
}

/// Action for a sequenced Entry.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub enum SeqEntryAction {
    /// Inserts a new sequenced entry.
    Ins(SeqValue),
    /// Updates an entry with a new value and version.
    Update(SeqValue),
    /// Deletes an entry.
    Del(u64),
}

impl SeqEntryAction {
    /// Returns the version for this action.
    pub fn version(&self) -> u64 {
        match *self {
            Self::Ins(ref value) => value.version,
            Self::Update(ref value) => value.version,
            Self::Del(v) => v,
        }
    }

    /// Sets the version for this action.
    pub fn set_version(&mut self, version: u64) {
        match *self {
            Self::Ins(ref mut value) => value.version = version,
            Self::Update(ref mut value) => value.version = version,
            Self::Del(ref mut v) => *v = version,
        }
    }
}

/// Action for an unsequenced Entry.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub enum UnseqEntryAction {
    /// Inserts a new unsequenced entry.
    Ins(Vec<u8>),
    /// Updates an entry with a new value.
    Update(Vec<u8>),
    /// Deletes an entry.
    Del,
}

/// Sequenced Entry Actions for given entry keys.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug, Default)]
pub struct SeqEntryActions {
    // A map containing keys and corresponding sequenced entry actions to perform.
    actions: BTreeMap<Vec<u8>, SeqEntryAction>,
}

impl SeqEntryActions {
    /// Creates a new sequenced Entry Actions list.
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets the actions.
    pub fn actions(&self) -> &BTreeMap<Vec<u8>, SeqEntryAction> {
        &self.actions
    }

    /// Converts `self` to a map of the keys with their corresponding action.
    pub fn into_actions(self) -> BTreeMap<Vec<u8>, SeqEntryAction> {
        self.actions
    }

    /// Inserts a new key-value pair.
    ///
    /// Requires the new `version` of the sequenced entry content. If it does not match the current
    /// version + 1, an error will be returned.
    pub fn ins(mut self, key: Vec<u8>, content: Vec<u8>, version: u64) -> Self {
        let _ = self.actions.insert(
            key,
            SeqEntryAction::Ins(SeqValue {
                data: content,
                version,
            }),
        );
        self
    }

    /// Updates an existing key-value pair.
    ///
    /// Requires the new `version` of the sequenced entry content. If it does not match the current
    /// version + 1, an error will be returned.
    pub fn update(mut self, key: Vec<u8>, content: Vec<u8>, version: u64) -> Self {
        let _ = self.actions.insert(
            key,
            SeqEntryAction::Update(SeqValue {
                data: content,
                version,
            }),
        );
        self
    }

    /// Deletes an entry.
    ///
    /// Requires the new `version` of the sequenced entry content. If it does not match the current
    /// version + 1, an error will be returned.
    pub fn del(mut self, key: Vec<u8>, version: u64) -> Self {
        let _ = self.actions.insert(key, SeqEntryAction::Del(version));
        self
    }

    /// Adds an action to the list of actions, replacing it if it is already present.
    pub fn add_action(&mut self, key: Vec<u8>, action: SeqEntryAction) {
        let _ = self.actions.insert(key, action);
    }
}

impl From<SeqEntryActions> for BTreeMap<Vec<u8>, SeqEntryAction> {
    fn from(actions: SeqEntryActions) -> Self {
        actions.actions
    }
}

impl From<BTreeMap<Vec<u8>, SeqEntryAction>> for SeqEntryActions {
    fn from(actions: BTreeMap<Vec<u8>, SeqEntryAction>) -> Self {
        SeqEntryActions { actions }
    }
}

/// Unsequenced Entry Actions for given entry keys.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug, Default)]
pub struct UnseqEntryActions {
    // A BTreeMap containing keys to which the corresponding unsequenced entry action is to be
    // performed.
    actions: BTreeMap<Vec<u8>, UnseqEntryAction>,
}

impl UnseqEntryActions {
    /// Creates a new unsequenced Entry Actions list.
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets the actions.
    pub fn actions(&self) -> &BTreeMap<Vec<u8>, UnseqEntryAction> {
        &self.actions
    }

    /// Converts UnseqEntryActions struct to a BTreeMap of the keys with their corresponding action.
    pub fn into_actions(self) -> BTreeMap<Vec<u8>, UnseqEntryAction> {
        self.actions
    }

    /// Insert a new key-value pair
    pub fn ins(mut self, key: Vec<u8>, content: Vec<u8>) -> Self {
        let _ = self.actions.insert(key, UnseqEntryAction::Ins(content));
        self
    }

    /// Update existing key-value pair
    pub fn update(mut self, key: Vec<u8>, content: Vec<u8>) -> Self {
        let _ = self.actions.insert(key, UnseqEntryAction::Update(content));
        self
    }

    /// Delete existing key
    pub fn del(mut self, key: Vec<u8>) -> Self {
        let _ = self.actions.insert(key, UnseqEntryAction::Del);
        self
    }

    /// Adds a UnseqEntryAction to the list of actions, replacing it if it is already present
    pub fn add_action(&mut self, key: Vec<u8>, action: UnseqEntryAction) {
        let _ = self.actions.insert(key, action);
    }
}

impl From<UnseqEntryActions> for BTreeMap<Vec<u8>, UnseqEntryAction> {
    fn from(actions: UnseqEntryActions) -> Self {
        actions.actions
    }
}

impl From<BTreeMap<Vec<u8>, UnseqEntryAction>> for UnseqEntryActions {
    fn from(actions: BTreeMap<Vec<u8>, UnseqEntryAction>) -> Self {
        UnseqEntryActions { actions }
    }
}

/// Wrapper type for entry actions, which can be sequenced or unsequenced.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub enum EntryActions {
    /// Sequenced entry actions.
    Seq(SeqEntryActions),
    /// Unsequenced entry actions.
    Unseq(UnseqEntryActions),
}

impl EntryActions {
    /// Gets the kind.
    pub fn kind(&self) -> Kind {
        match self {
            EntryActions::Seq(_) => Kind::Seq,
            EntryActions::Unseq(_) => Kind::Unseq,
        }
    }
}

impl From<SeqEntryActions> for EntryActions {
    fn from(entry_actions: SeqEntryActions) -> Self {
        EntryActions::Seq(entry_actions)
    }
}

impl From<UnseqEntryActions> for EntryActions {
    fn from(entry_actions: UnseqEntryActions) -> Self {
        EntryActions::Unseq(entry_actions)
    }
}

/// Sequenced entries (key-value pairs, with versioned values).
pub type SeqEntries = BTreeMap<Vec<u8>, SeqValue>;
/// Unsequenced entries (key-value pairs, without versioned values).
pub type UnseqEntries = BTreeMap<Vec<u8>, Vec<u8>>;

/// Wrapper type for entries, which can be sequenced or unsequenced.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub enum Entries {
    /// Sequenced entries.
    Seq(SeqEntries),
    /// Unsequenced entries.
    Unseq(UnseqEntries),
}

impl From<SeqEntries> for Entries {
    fn from(entries: SeqEntries) -> Self {
        Entries::Seq(entries)
    }
}

impl From<UnseqEntries> for Entries {
    fn from(entries: UnseqEntries) -> Self {
        Entries::Unseq(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::Result;
    use super::{Address, XorName};

    #[test]
    fn zbase32_encode_decode_map_address() -> Result<()> {
        let name = XorName(rand::random());
        let address = Address::Seq { name, tag: 15000 };
        let encoded = address.encode_to_zbase32()?;
        let decoded = self::Address::decode_from_zbase32(&encoded)?;
        assert_eq!(address, decoded);
        Ok(())
    }
}
