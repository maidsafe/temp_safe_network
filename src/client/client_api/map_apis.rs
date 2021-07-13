// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;
use crate::client::Error;
use tracing::trace;

use crate::types::{
    Map, MapAddress, MapEntries, MapEntryActions, MapKind, MapPermissionSet, MapValue, PublicKey,
};

use crate::messaging::client::{DataCmd, DataQuery, MapRead, MapWrite, QueryResponse};

use xor_name::XorName;

use std::collections::{BTreeMap, BTreeSet};

impl Client {
    //-------------------
    // Store
    // ------------------

    /// Store a new Map
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn store_map(
        &self,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        kind: MapKind,
        entries: Option<MapEntries>,
        permissions: Option<BTreeMap<PublicKey, MapPermissionSet>>,
    ) -> Result<MapAddress, Error> {
        let data = Map::new_with_data(
            name,
            tag,
            entries.unwrap_or_else(MapEntries::default),
            permissions.unwrap_or_else(BTreeMap::default),
            owner,
            kind,
        );
        let address = *data.address();
        let cmd = DataCmd::Map(MapWrite::New(data));

        self.pay_and_send_data_command(cmd).await?;

        Ok(address)
    }

    /// Delete Map
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn delete_map(&self, address: MapAddress) -> Result<(), Error> {
        let cmd = DataCmd::Map(MapWrite::Delete(address));

        self.pay_and_send_data_command(cmd).await
    }

    /// Delete map user permission
    pub async fn delete_map_user_perms(
        &self,
        address: MapAddress,
        user: PublicKey,
        version: u64,
    ) -> Result<(), Error> {
        let cmd = DataCmd::Map(MapWrite::DelUserPermissions {
            address,
            user,
            version,
        });

        self.pay_and_send_data_command(cmd).await
    }

    /// Set map user permissions
    pub async fn set_map_user_perms(
        &self,
        address: MapAddress,
        user: PublicKey,
        permissions: MapPermissionSet,
        version: u64,
    ) -> Result<(), Error> {
        let cmd = DataCmd::Map(MapWrite::SetUserPermissions {
            address,
            user,
            permissions,
            version,
        });

        self.pay_and_send_data_command(cmd).await
    }

    /// Mutate map user entries
    pub async fn edit_map_entries(
        &self,
        address: MapAddress,
        changes: MapEntryActions,
    ) -> Result<(), Error> {
        let cmd = DataCmd::Map(MapWrite::Edit { address, changes });

        self.pay_and_send_data_command(cmd).await
    }

    //-------------------
    // Gets
    // ------------------

    /// Fetch map data from the network
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_map(&self, address: MapAddress) -> Result<Map, Error>
    where
        Self: Sized,
    {
        trace!("Fetch Sequenced Mutable Data");

        let query_result = self
            .send_query(DataQuery::Map(MapRead::Get(address)))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::GetMap(res) => res.map_err(|err| Error::from((err, msg_id))),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Fetch the value for a given key in a map
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_map_value(&self, address: MapAddress, key: Vec<u8>) -> Result<MapValue, Error>
    where
        Self: Sized,
    {
        trace!("Fetch MapValue for {:?}", address);

        let query_result = self
            .send_query(DataQuery::Map(MapRead::GetValue { address, key }))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::GetMapValue(res) => res.map_err(|err| Error::from((err, msg_id))),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Get a shell (bare bones) version of `Map` from the network.
    pub async fn get_map_shell(&self, address: MapAddress) -> Result<Map, Error>
    where
        Self: Sized,
    {
        trace!("GetMapShell for {:?}", address);

        let query_result = self
            .send_query(DataQuery::Map(MapRead::GetShell(address)))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::GetMapShell(res) => res.map_err(|err| Error::from((err, msg_id))),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Get a current version of `Map` from the network.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_map_version(&self, address: MapAddress) -> Result<u64, Error>
    where
        Self: Sized,
    {
        trace!("GetMapVersion for {:?}", address);

        let query_result = self
            .send_query(DataQuery::Map(MapRead::GetVersion(address)))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::GetMapVersion(res) => res.map_err(|err| Error::from((err, msg_id))),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    //----------
    // Entries
    //----------

    /// Mutates public `Map` entries in bulk
    pub async fn mutate_map_entries(
        &self,
        address: MapAddress,
        actions: MapEntryActions,
    ) -> Result<(), Error>
    where
        Self: Sized,
    {
        trace!("Mutate Map for {:?}", address.name());
        self.edit_map_entries(address, actions).await
    }

    /// Return a complete list of entries in `Map`.
    pub async fn list_map_entries(&self, address: MapAddress) -> Result<MapEntries, Error>
    where
        Self: Sized,
    {
        trace!("ListMapEntries for {:?}", address.name());

        let query_result = self
            .send_query(DataQuery::Map(MapRead::ListEntries(address)))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::ListMapEntries(res) => {
                Ok(res.map_err(|err| Error::from((err, msg_id)))?)
            }
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Return a list of keys in `Map` stored on the network.
    pub async fn list_map_keys(&self, address: MapAddress) -> Result<BTreeSet<Vec<u8>>, Error>
    where
        Self: Sized,
    {
        trace!("ListMapKeys for {:?}", address);

        let query_result = self
            .send_query(DataQuery::Map(MapRead::ListKeys(address)))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::ListMapKeys(res) => res.map_err(|err| Error::from((err, msg_id))),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Return a list of values in a Map
    pub async fn list_map_values(&self, address: MapAddress) -> Result<Vec<MapValue>, Error>
    where
        Self: Sized,
    {
        trace!("List MapValues for {:?}", address.name());

        let query_result = self
            .send_query(DataQuery::Map(MapRead::ListValues(address)))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::ListMapValues(res) => Ok(res.map_err(|err| Error::from((err, msg_id)))?),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    //-----------------
    // Permissions
    //-----------------

    /// Return the permissions set for a particular user
    pub async fn list_map_user_permissions(
        &self,
        address: MapAddress,
        user: PublicKey,
    ) -> Result<MapPermissionSet, Error>
    where
        Self: Sized,
    {
        trace!("GetMapUserPermissions for {:?}", address);

        let query_result = self
            .send_query(DataQuery::Map(MapRead::ListUserPermissions {
                address,
                user,
            }))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::ListMapUserPermissions(res) => {
                res.map_err(|err| Error::from((err, msg_id)))
            }
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Return a list of permissions in `Map` stored on the network.
    pub async fn list_map_permissions(
        &self,
        address: MapAddress,
    ) -> Result<BTreeMap<PublicKey, MapPermissionSet>, Error>
    where
        Self: Sized,
    {
        trace!("List MapPermissions for {:?}", address);

        let query_result = self
            .send_query(DataQuery::Map(MapRead::ListPermissions(address)))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::ListMapPermissions(res) => res.map_err(|err| Error::from((err, msg_id))),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Updates or inserts a permissions set for a user
    pub async fn set_map_user_permissions(
        &self,
        address: MapAddress,
        user: PublicKey,
        permissions: MapPermissionSet,
        version: u64,
    ) -> Result<(), Error>
    where
        Self: Sized,
    {
        trace!("SetMapUserPermissions for {:?}", address);

        self.set_map_user_perms(address, user, permissions, version)
            .await
    }

    /// Updates or inserts a permissions set for a user
    pub async fn del_map_user_permissions(
        &self,
        address: MapAddress,
        user: PublicKey,
        version: u64,
    ) -> Result<(), Error>
    where
        Self: Sized,
    {
        trace!("DelMapUserPermissions for {:?}", address);

        self.delete_map_user_perms(address, user, version).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::utils::test_utils::{create_test_client, gen_ed_keypair, run_w_backoff};
    use crate::messaging::client::{CmdError, Error as ErrorMessage};
    use crate::types::{MapAction, MapKind, MapValues};
    use anyhow::{anyhow, bail, Result};
    use std::time::Duration;
    use xor_name::XorName;

    // 1. Create unseq. map with some entries and perms and put it on the network
    // 2. Fetch the shell version, entries, keys, values anv verify them
    // 3. Fetch the entire. data object and verify
    #[tokio::test]
    async fn public_map_test() -> Result<()> {
        let client = create_test_client(None).await?;

        let name = XorName::random();
        let tag = 15001;
        let mut entries: MapEntries = Default::default();
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new().allow(MapAction::Read);
        let _ = permissions.insert(client.public_key(), permission_set);
        let _ = entries.insert(
            b"key".to_vec(),
            MapValue {
                pointer: name,
                version: 0,
            },
        );
        let entries_keys = entries.keys().cloned().collect();
        let entries_values: MapValues = entries.values().cloned().collect();
        let owner = client.public_key();
        let address = client
            .store_map(
                name,
                tag,
                owner,
                MapKind::Public,
                Some(entries.clone()),
                Some(permissions),
            )
            .await?;

        let mut res: Result<u64> = Err(anyhow!("Timeout!".to_string()));
        while res.is_err() {
            tokio::time::sleep(Duration::from_millis(200)).await;
            res = match client.get_map_version(address).await {
                Ok(res) => Ok(res),
                Err(error) => Err(error.into()), // into anyhow error
            };
        }

        let version = res?;

        assert_eq!(version, 0);
        let fetched_entries = client.list_map_entries(address).await?;
        assert_eq!(fetched_entries, entries);
        let keys = client.list_map_keys(address).await?;
        assert_eq!(keys, entries_keys);
        let values = client.list_map_values(address).await?;
        assert_eq!(values, entries_values);
        let fetched_data = client.get_map(address).await?;
        assert_eq!(*fetched_data.name(), name);
        assert_eq!(fetched_data.tag(), tag);
        Ok(())
    }

    // 1. Store a private map on the network with some entries and permissions.
    // 2. Fetch the shell version, entries, keys, values anv verify them
    // 3. Fetch the entire. data object and verify
    #[tokio::test]
    async fn private_map_test() -> Result<()> {
        let client = create_test_client(None).await?;

        let name = XorName::random();
        let tag = 15001;
        let mut entries: MapEntries = Default::default();
        let _ = entries.insert(
            b"key".to_vec(),
            MapValue {
                pointer: name,
                version: 0,
            },
        );
        let entries_keys = entries.keys().cloned().collect();
        let entries_values: Vec<MapValue> = entries.values().cloned().collect();
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new().allow(MapAction::Read);
        let _ = permissions.insert(client.public_key(), permission_set);
        let owner = client.public_key();

        let address = client
            .store_map(
                name,
                tag,
                owner,
                MapKind::Private,
                Some(entries.clone()),
                Some(permissions),
            )
            .await?;

        let mut res: Result<MapEntries> = Err(anyhow!("Timeout!".to_string()));
        while res.is_err() {
            tokio::time::sleep(Duration::from_millis(200)).await;
            res = match client.list_map_entries(address).await {
                Ok(res) => Ok(res),
                Err(error) => Err(error.into()), // into anyhow error
            };
        }
        let fetched_entries = res?;

        assert_eq!(fetched_entries, entries);
        let map_shell = client.get_map_shell(address).await?;
        assert_eq!(*map_shell.name(), name);
        assert_eq!(map_shell.tag(), tag);
        assert_eq!(map_shell.entries().len(), 0);
        let keys = client.list_map_keys(address).await?;
        assert_eq!(keys, entries_keys);
        let values = client.list_map_values(address).await?;
        assert_eq!(values, entries_values);
        let fetched_data = client.get_map(address).await?;
        assert_eq!(*fetched_data.name(), name);
        assert_eq!(fetched_data.tag(), tag);
        assert_eq!(fetched_data.entries().len(), 1);
        Ok(())
    }

    // 1. Put seq. map on the network and then delete it
    // 2. Try getting the data object. It should bail
    #[tokio::test]
    // TODO: reenable all this when reworked and CRDT
    #[ignore = "flaky test, can hang"]
    async fn del_private_map_test() -> Result<()> {
        let mut client = create_test_client(None).await?;
        let name = XorName(rand::random());
        let tag = 15001;
        let mapref = MapAddress::Private { name, tag };
        let owner = client.public_key();

        let address = client
            .store_map(name, tag, owner, MapKind::Private, None, None)
            .await?;

        client.delete_map(mapref).await?;

        client.query_timeout = Duration::from_secs(5); // override with a short timeout
        let mut res = client.get_map(address).await;
        while res.is_ok() {
            tokio::time::sleep(Duration::from_millis(200)).await;
            // Keep trying until it fails
            res = client.get_map(address).await;
        }

        match res {
            Err(Error::NoResponse) => (),
            _ => bail!("Unexpected success"),
        }
        Ok(())
    }

    // 1. Put unseq. map on the network and then delete it
    // 2. Try getting the data object. It should bail
    #[tokio::test]
    // TODO: reenable all this when reworked and CRDT
    #[ignore = "flaky test, can hang"]
    async fn del_public_map_test() -> Result<()> {
        let mut client = create_test_client(None).await?;
        let name = XorName(rand::random());
        let tag = 15001;
        let mapref = MapAddress::Public { name, tag };
        let owner = client.public_key();

        let address = client
            .store_map(name, tag, owner, MapKind::Public, None, None)
            .await?;

        client.delete_map(mapref).await?;

        client.query_timeout = Duration::from_secs(5); // override with a short timeout
        let mut res = client.get_map(address).await;
        while res.is_ok() {
            // Keep trying until it fails
            res = client.get_map(address).await;
        }

        match res {
            Err(Error::NoResponse) => (),
            _ => bail!("Unexpected success"),
        }

        Ok(())
    }

    // 1. Create a client that PUTs some map on the network
    // 2. Create a different client that tries to delete the data. It should bail.
    #[tokio::test]
    async fn del_public_map_permission_test() -> Result<()> {
        let name = XorName(rand::random());
        let tag = 15001;
        let mapref = MapAddress::Public { name, tag };

        let some_client = create_test_client(None).await?;
        let owner = some_client.public_key();

        let _ = run_w_backoff(
            || some_client.store_map(name, tag, owner, MapKind::Public, None, None),
            10,
        )
        .await?;

        let mut client = create_test_client(None).await?;

        client.delete_map(mapref).await?;

        match client.expect_cmd_error().await {
            Some(CmdError::Data(ErrorMessage::AccessDenied(_))) => Ok(()),
            _ => bail!("Unexpected: Deletion by non-owners should fail"),
        }
    }

    #[tokio::test]
    #[ignore = "Has been failing for a long time, fix coming up."]
    async fn map_cannot_initially_put_data_with_another_owner_than_current_client() -> Result<()> {
        let client = create_test_client(None).await?;
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::ManagePermissions);
        let user = client.public_key();
        let random_user = gen_ed_keypair().public_key();
        let random_pk = gen_ed_keypair().public_key();

        let _ = permissions.insert(user, permission_set.clone());
        let _ = permissions.insert(random_user, permission_set);

        let test_data_name = XorName(rand::random());
        let address = run_w_backoff(
            || {
                client.store_map(
                    test_data_name,
                    15000u64,
                    random_pk,
                    MapKind::Private,
                    None,
                    Some(permissions.clone()),
                )
            },
            10,
        )
        .await?;

        let res = client.get_map_shell(address).await;
        match res {
            Err(Error::NoResponse) => (),
            Ok(data) => bail!(
                "Unexpected Success: Validating owners should fail.  Data received : {:?}",
                data
            ),
            Err(e) => bail!("Unexpected: {:?}", e),
        };

        // TODO: Refunds not yet in place.... Reenable this check when that's the case

        // Check token was not taken
        // let balance = client.get_balance().await?;
        // let expected_bal = calculate_new_balance(start_bal, Some(2), None);
        // assert_eq!(balance, expected_bal);

        Ok(())
    }

    // 1. Create a map with some permissions and store it on the network.
    // 2. Modify the permissions of a user in the permission set.
    // 3. Fetch the list of permissions and verify the edit.
    // 4. Delete a user's permissions from the permission set and verify the deletion.
    #[tokio::test]
    async fn map_can_modify_permissions_test() -> Result<()> {
        let client = create_test_client(None).await?;
        let name = XorName(rand::random());
        let tag = 15001;
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::ManagePermissions);
        let user = client.public_key();
        let random_user = gen_ed_keypair().public_key();

        let _ = permissions.insert(user, permission_set.clone());
        let _ = permissions.insert(random_user, permission_set);

        let owner = client.public_key();

        // Store the data
        let address = client
            .store_map(name, tag, owner, MapKind::Private, None, Some(permissions))
            .await?;

        // Assert that the data is stored.
        let mut res = client.get_map(address).await;
        while res.is_err() {
            tokio::time::sleep(Duration::from_millis(200)).await;
            res = client.get_map(address).await;
        }

        let new_perm_set = MapPermissionSet::new()
            .allow(MapAction::ManagePermissions)
            .allow(MapAction::Read);

        // Set new perms to the data
        client
            .set_map_user_permissions(MapAddress::Private { name, tag }, user, new_perm_set, 1)
            .await?;

        // Assert that the new perms are set.
        let mut permissions = client
            .list_map_user_permissions(MapAddress::Private { name, tag }, user)
            .await?;
        while permissions.is_allowed(MapAction::Insert) {
            tokio::time::sleep(Duration::from_millis(200)).await;
            permissions = client
                .list_map_user_permissions(MapAddress::Private { name, tag }, user)
                .await?;
        }
        assert!(!permissions.is_allowed(MapAction::Insert));
        assert!(permissions.is_allowed(MapAction::Read));
        assert!(permissions.is_allowed(MapAction::ManagePermissions));

        // Delete user perms
        client
            .del_map_user_permissions(MapAddress::Private { name, tag }, random_user, 2)
            .await?;

        // Assert perms deletion.
        let mut permissions = client
            .list_map_permissions(MapAddress::Private { name, tag })
            .await?;
        while permissions.len() != 1 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            permissions = client
                .list_map_permissions(MapAddress::Private { name, tag })
                .await?;
        }

        Ok(())
    }

    // 1. Create a map and store it on the network
    // 2. Create some entry actions and mutate the data on the network.
    // 3. List the entries and verify that the mutation was applied.
    // 4. Fetch a value for a particular key and verify
    #[tokio::test]
    async fn map_mutations_test() -> Result<()> {
        let client = create_test_client(None).await?;
        let name = XorName::random();
        let val_1 = XorName::random();
        let tag = 15001;
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::Update)
            .allow(MapAction::Delete);
        let user = client.public_key();
        let _ = permissions.insert(user, permission_set);
        let mut entries: MapEntries = Default::default();
        let _ = entries.insert(
            b"key1".to_vec(),
            MapValue {
                pointer: val_1,
                version: 0,
            },
        );
        let _ = entries.insert(
            b"key2".to_vec(),
            MapValue {
                pointer: val_1,
                version: 0,
            },
        );
        let owner = client.public_key();

        let address = client
            .store_map(
                name,
                tag,
                owner,
                MapKind::Private,
                Some(entries.clone()),
                Some(permissions),
            )
            .await?;

        // Assert that the data is stored.
        let mut res = client.get_map(address).await;

        while res.is_err() {
            tokio::time::sleep(Duration::from_millis(200)).await;
            res = client.get_map(address).await;
        }
        let fetched_entries = client.list_map_entries(address).await?;

        assert_eq!(fetched_entries, entries);

        let new_value = XorName::random();
        let value = XorName::random();
        let entry_actions: MapEntryActions = MapEntryActions::new()
            .update(b"key1".to_vec(), new_value, 1)
            .delete(b"key2".to_vec(), 1)
            .insert(b"key3".to_vec(), value, 0);

        client.mutate_map_entries(address, entry_actions).await?;

        let mut fetched_entries = client.list_map_entries(address).await?;
        while fetched_entries.contains_key(&b"key2".to_vec()) {
            fetched_entries = client.list_map_entries(address).await?;
        }

        let mut expected_entries: BTreeMap<_, _> = Default::default();
        let _ = expected_entries.insert(
            b"key1".to_vec(),
            MapValue {
                pointer: new_value,
                version: 1,
            },
        );
        let _ = expected_entries.insert(
            b"key3".to_vec(),
            MapValue {
                pointer: value,
                version: 0,
            },
        );

        assert_eq!(fetched_entries, expected_entries);

        let fetched_value = client.get_map_value(address, b"key3".to_vec()).await?;

        assert_eq!(
            fetched_value,
            MapValue {
                pointer: value,
                version: 0
            }
        );

        let res = client.get_map_value(address, b"wrongKey".to_vec()).await;
        match res {
            Ok(_) => bail!("Unexpected: Entry should not exist"),
            Err(Error::ErrorMessage {
                source: ErrorMessage::NoSuchEntry,
                ..
            }) => (),
            Err(err) => bail!("Unexpected error: {:?}", err),
        };

        let client = create_test_client(None).await?;
        let name = XorName(rand::random());
        let tag = 15001;
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::Update)
            .allow(MapAction::Delete);
        let user = client.public_key();
        let _ = permissions.insert(user, permission_set);
        let mut entries: BTreeMap<Vec<u8>, MapValue> = Default::default();
        let _ = entries.insert(
            b"key1".to_vec(),
            MapValue {
                pointer: val_1,
                version: 0,
            },
        );
        let _ = entries.insert(
            b"key2".to_vec(),
            MapValue {
                pointer: val_1,
                version: 0,
            },
        );

        let owner = client.public_key();
        let address = client
            .store_map(
                name,
                tag,
                owner,
                MapKind::Public,
                Some(entries.clone()),
                Some(permissions),
            )
            .await?;

        // Assert that the data is stored.
        let mut res = client.get_map(address).await;
        while res.is_err() {
            tokio::time::sleep(Duration::from_millis(200)).await;
            res = client.get_map(address).await;
        }

        let fetched_entries = client.list_map_entries(address).await?;
        assert_eq!(fetched_entries, entries);
        let entry_actions = MapEntryActions::new()
            .update(b"key1".to_vec(), new_value, 1)
            .delete(b"key2".to_vec(), 1)
            .insert(b"key3".to_vec(), value, 0);

        client.mutate_map_entries(address, entry_actions).await?;

        let mut fetched_entries = client.list_map_entries(address).await?;
        while fetched_entries.contains_key(&b"key2".to_vec()) {
            fetched_entries = client.list_map_entries(address).await?;
        }

        let mut expected_entries: BTreeMap<_, _> = Default::default();
        let _ = expected_entries.insert(
            b"key1".to_vec(),
            MapValue {
                pointer: new_value,
                version: 1,
            },
        );
        let _ = expected_entries.insert(
            b"key3".to_vec(),
            MapValue {
                pointer: value,
                version: 0,
            },
        );
        assert_eq!(fetched_entries, expected_entries);
        let fetched_value = client.get_map_value(address, b"key1".to_vec()).await?;

        assert_eq!(
            fetched_value,
            MapValue {
                pointer: new_value,
                version: 1,
            }
        );
        let res = client.get_map_value(address, b"wrongKey".to_vec()).await;
        match res {
            Ok(_) => bail!("Unexpected: Entry should not exist"),
            Err(Error::ErrorMessage {
                source: ErrorMessage::NoSuchEntry,
                ..
            }) => Ok(()),
            Err(err) => bail!("Unexpected error: {:?}", err),
        }
    }
}
