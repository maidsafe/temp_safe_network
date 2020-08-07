// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;
use crate::client::AuthActions;
use crate::errors::CoreError;
use safe_nd::{
    AppPermissions, EntryError, Error as SndError, MapAction, MapAddress, MapPermissionSet,
    MapSeqEntries, MapSeqEntryAction, MapSeqEntryActions, MapSeqValue, PublicKey, SeqMap,
};

use std::collections::BTreeMap;

const MAX_ATTEMPTS: usize = 10;

///! Wrapped APIs to provide auto recovery and resiliance to some network errors.

/// Puts mutable data on the network and tries to recover from errors.
///
/// If the data already exists, it tries to mutate it so its entries and permissions
/// are the same as those of the data being put, except it wont delete existing
/// entries or remove existing permissions.
pub async fn put_map(client: Client, data: SeqMap) -> Result<(), CoreError> {
    let client = client.clone();

    match client.put_seq_mutable_data(data.clone()).await {
        Ok(_response) => Ok(()),
        Err(e) => match e {
            CoreError::DataError(SndError::DataExists) => update_map(client, data).await,
            error => Err(error),
        },
    }
}

/// Mutates mutable data entries and tries to recover from errors.
pub async fn mutate_map_entries(
    client: Client,
    address: MapAddress,
    actions: MapSeqEntryActions,
) -> Result<(), CoreError> {
    let mut actions_to_try = actions;
    let mut attempts = 0;
    let mut done_trying = false;
    let mut response: Result<(), CoreError> = Err(CoreError::RequestTimeout);

    while !done_trying {
        response = match client
            .mutate_seq_map_entries(*address.name(), address.tag(), actions_to_try.clone())
            .await
        {
            Ok(()) => {
                done_trying = true;
                Ok(())
            }
            Err(CoreError::DataError(SndError::InvalidEntryActions(errors)))
                if attempts < MAX_ATTEMPTS =>
            {
                actions_to_try = fix_entry_actions(actions_to_try.clone(), &errors).into();
                attempts += 1;
                Ok(())
            }
            Err(CoreError::RequestTimeout) if attempts < MAX_ATTEMPTS => {
                attempts += 1;
                Ok(())
            }
            other => {
                done_trying = true;
                other
            }
        };
    }
    response
}

/// Sets user permission on the mutable data and tries to recover from errors.
pub async fn set_map_user_permissions(
    client: Client,
    address: MapAddress,
    user: PublicKey,
    permissions: MapPermissionSet,
    version: u64,
) -> Result<(), CoreError> {
    let mut version_to_try = version;
    let mut attempts = 0;
    let mut done_trying = false;
    let mut response: Result<(), CoreError> = Err(CoreError::RequestTimeout);

    while !done_trying {
        response = match client
            .set_map_user_permissions(address, user, permissions.clone(), version_to_try)
            .await
        {
            Ok(()) => {
                done_trying = true;
                Ok(())
            }
            Err(CoreError::DataError(SndError::InvalidSuccessor(current_version)))
                if attempts < MAX_ATTEMPTS =>
            {
                version_to_try = current_version + 1;
                attempts += 1;
                Ok(())
            }
            Err(CoreError::RequestTimeout) if attempts < MAX_ATTEMPTS => {
                version_to_try += version;
                attempts += 1;
                Ok(())
            }
            other => {
                done_trying = true;
                other
            }
        }
    }

    response
}

/// Deletes user permission on the mutable data and tries to recover from errors.
pub async fn del_map_user_permissions(
    client: Client,
    address: MapAddress,
    user: PublicKey,
    version: u64,
) -> Result<(), CoreError> {
    let mut version_to_try = version;
    let mut attempts = 0;
    let mut done_trying = false;
    let mut response: Result<(), CoreError> = Err(CoreError::RequestTimeout);

    while !done_trying {
        response = match client
            .del_map_user_permissions(address, user, version_to_try)
            .await
        {
            Ok(_) | Err(CoreError::DataError(SndError::NoSuchKey)) => {
                done_trying = true;
                Ok(())
            }
            Err(CoreError::DataError(SndError::InvalidSuccessor(current_version)))
                if attempts < MAX_ATTEMPTS =>
            {
                attempts += 1;
                version_to_try = current_version + 1;
                Ok(())
            }
            Err(CoreError::RequestTimeout) if attempts < MAX_ATTEMPTS => {
                attempts += 1;
                version_to_try = version;
                Ok(())
            }
            other => {
                done_trying = true;
                other
            }
        }
    }

    response
}

async fn update_map(client: Client, data: SeqMap) -> Result<(), CoreError> {
    let client = client.clone();

    let address = *data.address();
    let entries = client
        .list_seq_map_entries(*data.name(), data.tag())
        .await?;
    let permissions = client.list_map_permissions(address).await?;
    let version = client.get_map_version(address).await?;

    let next_version = version + 1;

    update_map_permissions(
        client.clone(),
        address,
        &permissions,
        data.permissions(),
        next_version,
    )
    .await?;

    update_map_entries(client, address, &entries, data.entries().clone()).await
}

// Update the mutable data on the network so it has all the `desired_entries`.
async fn update_map_entries(
    client: Client,
    address: MapAddress,
    current_entries: &MapSeqEntries,
    desired_entries: MapSeqEntries,
) -> Result<(), CoreError> {
    let actions = desired_entries
        .into_iter()
        .filter_map(|(key, value)| {
            if let Some(current_value) = current_entries.get(&key) {
                if current_value.version <= value.version {
                    Some((key, MapSeqEntryAction::Update(value)))
                } else {
                    None
                }
            } else {
                Some((key, MapSeqEntryAction::Ins(value)))
            }
        })
        .collect::<BTreeMap<_, _>>();

    mutate_map_entries(client, address, actions.into()).await
}

async fn update_map_permissions(
    client: Client,
    address: MapAddress,
    current_permissions: &BTreeMap<PublicKey, MapPermissionSet>,
    desired_permissions: BTreeMap<PublicKey, MapPermissionSet>,
    version: u64,
) -> Result<(), CoreError> {
    let mut permissions: Vec<_> = desired_permissions
        .into_iter()
        .map(|(user, desired_set)| {
            if let Some(current_set) = current_permissions.get(&user) {
                (
                    user,
                    union_permission_sets(current_set.clone(), desired_set),
                )
            } else {
                (user, desired_set)
            }
        })
        .collect();

    let _state = (client.clone(), permissions.clone(), version);

    let mut success = false;
    let mut version_to_try = version;

    while !success {
        if let Some((user, set)) = permissions.pop() {
            match set_map_user_permissions(client.clone(), address, user, set, version_to_try).await
            {
                Ok(()) => {
                    success = true;
                }
                Err(_error) => {
                    version_to_try += 1;
                }
            }
        }
    }

    Ok(())
}

// Modify the given entry actions to fix the entry errors.
fn fix_entry_actions(
    actions: MapSeqEntryActions,
    errors: &BTreeMap<Vec<u8>, EntryError>,
) -> BTreeMap<Vec<u8>, MapSeqEntryAction> {
    actions
        .into_actions()
        .into_iter()
        .fold(BTreeMap::new(), |mut fixed_action, (key, action)| {
            if let Some(error) = errors.get(&key) {
                if let Some(action) = fix_entry_action(&action, error) {
                    let _ = fixed_action.insert(key, action);
                }
            } else {
                let _ = fixed_action.insert(key, action);
            }
            fixed_action
        })
}

fn fix_entry_action(action: &MapSeqEntryAction, error: &EntryError) -> Option<MapSeqEntryAction> {
    match (action, error) {
        (MapSeqEntryAction::Ins(value), EntryError::EntryExists(current_version))
        | (MapSeqEntryAction::Update(value), EntryError::InvalidSuccessor(current_version)) => {
            Some(MapSeqEntryAction::Update(MapSeqValue {
                data: value.data.clone(),
                version: (current_version + 1).into(),
            }))
        }
        (MapSeqEntryAction::Update(value), EntryError::NoSuchEntry) => {
            Some(MapSeqEntryAction::Ins(value.clone()))
        }
        (MapSeqEntryAction::Del(_), EntryError::NoSuchEntry) => None,
        (MapSeqEntryAction::Del(_), EntryError::InvalidSuccessor(current_version)) => {
            Some(MapSeqEntryAction::Del((current_version + 1).into()))
        }
        (action, _) => Some(action.clone()),
    }
}

// Create union of the two permission sets, preferring allows to deny's.
fn union_permission_sets(a: MapPermissionSet, b: MapPermissionSet) -> MapPermissionSet {
    let actions = [
        MapAction::Insert,
        MapAction::Update,
        MapAction::Delete,
        MapAction::ManagePermissions,
    ];
    actions
        .iter()
        .fold(MapPermissionSet::new(), |set, &action| {
            if a.is_allowed(action) | b.is_allowed(action) {
                set.allow(action)
            } else if !a.is_allowed(action) | !b.is_allowed(action) {
                set.deny(action)
            } else {
                set
            }
        })
}


#[cfg(test)]
mod tests {
    use super::*;
    use safe_nd::MapSeqValue;
    use unwrap::unwrap;

    // Test modifying given entry actions to fix entry errors
    #[test]
    fn test_fix_entry_actions() -> Result<(), CoreError> {
        let actions = MapSeqEntryActions::new()
            .ins(vec![0], vec![0], 0)
            .ins(vec![1], vec![1], 0)
            .update(vec![2], vec![2], 1)
            .update(vec![3], vec![3], 1)
            .update(vec![4], vec![4], 1)
            .del(vec![5], 1)
            .del(vec![6], 1)
            .del(vec![7], 1);

        let mut errors = BTreeMap::new();
        let _ = errors.insert(vec![1], EntryError::EntryExists(2));
        let _ = errors.insert(vec![3], EntryError::NoSuchEntry);
        let _ = errors.insert(vec![4], EntryError::InvalidSuccessor(2));
        let _ = errors.insert(vec![6], EntryError::NoSuchEntry);
        let _ = errors.insert(vec![7], EntryError::InvalidSuccessor(2));

        let actions = fix_entry_actions(actions, &errors);

        // 0: insert is OK.
        assert_eq!(
            *unwrap!(actions.get([0].as_ref())),
            MapSeqEntryAction::Ins(MapSeqValue {
                data: vec![0],
                version: 0,
            })
        );

        // 1: insert is transformed to update
        assert_eq!(
            *unwrap!(actions.get([1].as_ref())),
            MapSeqEntryAction::Update(MapSeqValue {
                data: vec![1],
                version: 3,
            })
        );

        // 2: update is OK.
        assert_eq!(
            *unwrap!(actions.get([2].as_ref())),
            MapSeqEntryAction::Update(MapSeqValue {
                data: vec![2],
                version: 1,
            })
        );

        // 3: update is transformed to insert.
        assert_eq!(
            *unwrap!(actions.get([3].as_ref())),
            MapSeqEntryAction::Ins(MapSeqValue {
                data: vec![3],
                version: 1,
            })
        );

        // 4: update version is fixed.
        assert_eq!(
            *unwrap!(actions.get([4].as_ref())),
            MapSeqEntryAction::Update(MapSeqValue {
                data: vec![4],
                version: 3,
            })
        );

        // 5: delete is OK.
        assert_eq!(
            *unwrap!(actions.get([5].as_ref())),
            MapSeqEntryAction::Del(1)
        );

        // 6: delete action is removed, as there is nothing to delete.
        assert!(actions.get([6].as_ref()).is_none());

        // 7: delete version is fixed.
        assert_eq!(
            *unwrap!(actions.get([7].as_ref())),
            MapSeqEntryAction::Del(3)
        );

        Ok(())
    }

    // Test creating a union of two permission sets
    #[test]
    fn test_union_permission_sets() -> Result<(), CoreError> {
        let a = MapPermissionSet::new()
            .allow(MapAction::Insert)
            .deny(MapAction::Update)
            .deny(MapAction::ManagePermissions);
        let b = MapPermissionSet::new()
            .allow(MapAction::Update)
            .allow(MapAction::Delete);

        let c = union_permission_sets(a, b);
        assert_eq!(c.is_allowed(MapAction::Insert), true);
        assert_eq!(c.is_allowed(MapAction::Update), true);
        assert_eq!(c.is_allowed(MapAction::Delete), true);
        assert_eq!(c.is_allowed(MapAction::ManagePermissions), false);

        Ok(())
    }
}

#[cfg(all(test, feature = "mock-network"))]
mod tests_with_mock_routing {
    use super::*;
    use crate::btree_map;
    use crate::utils::test_utils::random_client;
    use safe_nd::{MapSeqValue, XorName};
    use unwrap::unwrap;

    // Test putting map and recovering from errors
    #[tokio::test]
    async fn put_map_with_recovery() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = rand::random();
        let tag = 10_000;
        let owners = client.public_key().await;

        let entries = btree_map![
             vec![0] => MapSeqValue {
                data: vec![0, 0],
                version: 0,
            },
             vec![1] => MapSeqValue {
                data: vec![1, 0],
                version: 1,
            },
             vec![2] => MapSeqValue {
                data: vec![2, 0],
                version: 0,
            }
        ];

        let bls_sk = threshold_crypto::SecretKey::random();
        let user = PublicKey::from(bls_sk.public_key());

        let permissions = btree_map![
            user => MapPermissionSet::new().allow(MapAction::Insert)
        ];
        let data0 = SeqMap::new_with_data(name, tag, entries, permissions, owners);

        let entries1 = btree_map![
            vec![0] => MapSeqValue {
                data: vec![0, 1],
                version: 1,
            },
            vec![1] => MapSeqValue {
                data: vec![1, 1],
                version: 0,
            },
            vec![3] => MapSeqValue {
                data: vec![3, 1],
                version: 0,
            }
        ];

        let bls_sk = threshold_crypto::SecretKey::random();
        let user = PublicKey::from(bls_sk.public_key());

        let permissions = btree_map![
           user => MapPermissionSet::new().allow(MapAction::Delete)
        ];

        let data1 = SeqMap::new_with_data(name, tag, entries1, permissions, owners);

        client.put_seq_mutable_data(data0).await?;
        put_map(client.clone(), data1).await?;
        let entries = client.list_seq_map_entries(name, tag).await?;
        assert_eq!(entries.len(), 4);
        assert_eq!(
            *unwrap!(entries.get([0].as_ref())),
            MapSeqValue {
                data: vec![0, 1],
                version: 1,
            }
        );
        assert_eq!(
            *unwrap!(entries.get([1].as_ref())),
            MapSeqValue {
                data: vec![1, 0],
                version: 1,
            }
        );

        let permissions = client
            .list_map_permissions(MapAddress::Seq { name, tag })
            .await?;
        assert_eq!(permissions.len(), 2);
        assert_eq!(
            *unwrap!(permissions.get(&user)),
            MapPermissionSet::new().allow(MapAction::Delete)
        );

        Ok(())
    }

    // Test mutating map entries and recovering from errors
    #[tokio::test]
    async fn mutate_map_entries_with_recovery() -> Result<(), CoreError> {
        let client = random_client()?;

        let name: XorName = rand::random();
        let tag = 10_000;
        let entries = btree_map![
            vec![1] => MapSeqValue {
                data: vec![1],
                version: 0,
            },
            vec![2] => MapSeqValue {
                data: vec![2],
                version: 0,
            },
            vec![4] => MapSeqValue {
                data: vec![4],
                version: 0,
            },
            vec![5] => MapSeqValue {
                data: vec![5],
                version: 0,
            },
            vec![7] => MapSeqValue {
                data: vec![7],
                version: 0,
            }
        ];
        let owners = client.public_key().await;
        let data = SeqMap::new_with_data(name, tag, entries, Default::default(), owners);

        client.put_seq_mutable_data(data).await?;

        let actions = MapSeqEntryActions::new()
            .ins(vec![0], vec![0], 0) // normal insert
            .ins(vec![1], vec![1, 0], 0) // insert to existing entry
            .update(vec![2], vec![2, 0], 1) // normal update
            .update(vec![3], vec![3], 1) // update of non-existing entry
            .update(vec![4], vec![4, 0], 0) // update with invalid version
            .del(vec![5], 1) // normal delete
            .del(vec![6], 1) // delete of non-existing entry
            .del(vec![7], 0); // delete with invalid version

        mutate_map_entries(client.clone(), MapAddress::Seq { name, tag }, actions).await?;
        let entries = client.list_seq_map_entries(name, tag).await?;
        assert_eq!(entries.len(), 5);

        assert_eq!(
            *unwrap!(entries.get([0].as_ref())),
            MapSeqValue {
                data: vec![0],
                version: 0,
            }
        );
        assert_eq!(
            *unwrap!(entries.get([1].as_ref())),
            MapSeqValue {
                data: vec![1, 0],
                version: 1,
            }
        );
        assert_eq!(
            *unwrap!(entries.get([2].as_ref())),
            MapSeqValue {
                data: vec![2, 0],
                version: 1,
            }
        );
        assert_eq!(
            *unwrap!(entries.get([3].as_ref())),
            MapSeqValue {
                data: vec![3],
                version: 1,
            }
        );
        assert_eq!(
            *unwrap!(entries.get([4].as_ref())),
            MapSeqValue {
                data: vec![4, 0],
                version: 1,
            }
        );
        assert!(entries.get([5].as_ref()).is_none());
        assert!(entries.get([6].as_ref()).is_none());
        assert!(entries.get([7].as_ref()).is_none());

        Ok(())
    }

    // Test setting and deleting user permissions and recovering from errors
    #[tokio::test]
    async fn set_and_del_map_user_permissions_with_recovery() -> Result<(), CoreError> {
        let client = random_client()?;

        let name: XorName = rand::random();
        let tag = 10_000;
        let owners = client.public_key().await;
        let data = SeqMap::new_with_data(name, tag, Default::default(), Default::default(), owners);
        let address = *data.address();
        let bls_sk1 = threshold_crypto::SecretKey::random();
        let bls_sk2 = threshold_crypto::SecretKey::random();

        let user0 = PublicKey::from(bls_sk1.public_key());
        let user1 = PublicKey::from(bls_sk2.public_key());

        client.put_seq_mutable_data(data).await?;
        // set with invalid version
        set_map_user_permissions(
            client.clone(),
            address,
            user0,
            MapPermissionSet::new().allow(MapAction::Insert),
            0,
        )
        .await?;
        let retrieved_permissions = client.list_map_user_permissions(address, user0).await?;
        assert_eq!(
            retrieved_permissions,
            MapPermissionSet::new().allow(MapAction::Insert)
        );

        // delete with invalid version
        del_map_user_permissions(client.clone(), address, user0, 0).await?;
        let res = client.list_map_user_permissions(address, user0).await;
        match res {
            Err(CoreError::DataError(SndError::NoSuchKey)) => (),
            x => panic!("Unexpected {:?}", x),
        }

        // delete of non-existing user
        del_map_user_permissions(client, address, user1, 3).await?;

        Ok(())
    }
}
