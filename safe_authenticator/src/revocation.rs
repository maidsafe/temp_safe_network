// Copyright 2017 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use super::{AccessContainerEntry, AuthError, AuthFuture};
use access_container::{access_container, access_container_entry, access_container_key,
                       delete_access_container_entry};
use config;
use config::AppInfo;
use futures::Future;
use futures::future::{self, Either, Loop};
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::{EntryActions, User};
use rust_sodium::crypto::sign;
use safe_core::{Client, FutureExt, MDataInfo};
use safe_core::ipc::IpcError;
use safe_core::recovery;
use safe_core::utils::{symmetric_decrypt, symmetric_encrypt};
use std::collections::HashMap;

/// Revokes app access using a revocation queue
pub fn revoke_app(client: &Client<()>, app_id: &str) -> Box<AuthFuture<String>> {
    let app_id = app_id.to_string();
    let client = client.clone();

    // 1. Get the topmost queue item which contains an app ID
    //    (or use the provided app_id if it's empty)
    // 2. Revoke a single app from the queue
    // 3. Remove the app_id from the queue, start over with step 1 if the queue is not empty
    future::loop_fn(app_id, move |app_id| {
        let c2 = client.clone();
        let c3 = client.clone();
        let c4 = client.clone();
        let c5 = client.clone();
        let app_id2 = app_id.clone();

        config::get_revocation_queue(&c2)
            .and_then(move |res| {
                let (_version, queue) = res.unwrap_or_else(|| (0, Default::default()));
                let current_item = queue.front().cloned().unwrap_or_else(|| app_id.clone());
                let fut = if !queue.contains(&app_id) {
                    config::push_to_revocation_queue(&c3, app_id)
                } else {
                    // The queue already contains this app, do nothing
                    ok!(())
                };
                fut.map(move |_| current_item)
            })
            .and_then(move |app_id| revoke_single_app(&c4, &app_id))
            .and_then(move |_| config::pop_from_revocation_queue(&c5))
            .and_then(move |opt_queue| {
                let (app_id, queue) = opt_queue.ok_or_else(|| {
                    AuthError::from("No revocation queue found in the config file")
                })?;

                if let Some(next_app_id) = queue.front().cloned() {
                    Ok(Loop::Continue(next_app_id))
                } else {
                    Ok(Loop::Break(app_id.unwrap_or(app_id2)))
                }
            })
    }).into_box()

}

/// Revoke all apps currently in the revocation queue.
pub fn flush_app_revocation_queue(client: &Client<()>) -> Box<AuthFuture<()>> {
    let client = client.clone();

    future::loop_fn((), move |_| {
        let c2 = client.clone();
        let c3 = client.clone();

        config::get_revocation_queue(&client)
            .map(|queue| {
                queue.map(|(_, queue)| queue).unwrap_or_else(
                    Default::default,
                )
            })
            .and_then(move |queue| if let Some(app_id) = queue.front().cloned() {
                let f = revoke_single_app(&c2, &app_id)
                    .and_then(move |_| config::pop_from_revocation_queue(&c3))
                    .and_then(move |_| Ok(Loop::Continue(())));
                Either::A(f)
            } else {
                Either::B(future::ok(Loop::Break(())))
            })
    }).into_box()
}

/// Revoke access for a single app
fn revoke_single_app(client: &Client<()>, app_id: &str) -> Box<AuthFuture<()>> {
    let c2 = client.clone();
    let c3 = client.clone();
    let c4 = client.clone();
    let c5 = client.clone();
    let c6 = client.clone();
    let c7 = client.clone();
    let c8 = client.clone();

    // 1. Put the provided app_id into the revocation queue
    // 2. Delete the app key from MaidManagers
    // 3. Remove the app's key from containers permissions
    // 4. Refresh the containers info from the user's root dir (as the access
    //    container entry is not updated with the new keys info - so we have to
    //    make sure that we use correct encryption keys if the previous revoke\
    //    attempt has failed)
    // 4. Re-encrypt private containers that the app had access to
    // 5. Remove the revoked app from the access container
    config::get_app(client, app_id)
        .and_then(move |app| {
            delete_app_auth_key(&c2, app.keys.sign_pk).map(move |_| app)
        })
        .and_then(move |app| {
            access_container(&c3).map(move |access_container| (app, access_container))
        })
        .and_then(move |(app, access_container)| {
            access_container_entry(&c4, &access_container, &app.info.id, app.keys.clone())
                .and_then(move |(version, permissions)| {
                    Ok((
                        app,
                        access_container,
                        version,
                        permissions.ok_or(AuthError::IpcError(IpcError::UnknownApp))?,
                    ))
                })
        })
        .and_then(move |(app,
               access_container,
               ac_entry_version,
               permissions)| {
            revoke_container_perms(&c5, &permissions, app.keys.sign_pk).map(move |_| {
                (app, access_container, ac_entry_version, permissions)
            })
        })
        .and_then(move |(app,
               access_container,
               ac_entry_version,
               permissions)| {
            refresh_from_user_root_dir(&c6, permissions).map(move |refreshed_containers| {
                (
                    app,
                    access_container,
                    ac_entry_version,
                    refreshed_containers,
                )
            })
        })
        .and_then(move |(app,
               access_container,
               ac_entry_version,
               permissions)| {
            reencrypt_private_containers(&c7, access_container.clone(), permissions.clone(), &app)
                .map(move |_| (app, access_container, ac_entry_version))
        })
        .and_then(move |(app, access_container, version)| {
            delete_access_container_entry(
                &c8,
                &access_container,
                &app.info.id,
                &app.keys,
                version + 1,
            )
        })
        .into_box()
}

/// Delete the app's auth key from the Maid Manager - this prevents the app from
/// performing any more mutations.
fn delete_app_auth_key(client: &Client<()>, key: sign::PublicKey) -> Box<AuthFuture<()>> {
    let client = client.clone();

    client
        .list_auth_keys_and_version()
        .and_then(move |(listed_keys, version)| if listed_keys.contains(
            &key,
        )
        {
            client.del_auth_key(key, version + 1)
        } else {
            // The key has been removed already
            ok!(())
        })
        .map_err(From::from)
        .into_box()
}

// Revokes containers permissions
fn revoke_container_perms(
    client: &Client<()>,
    permissions: &AccessContainerEntry,
    sign_pk: sign::PublicKey,
) -> Box<AuthFuture<()>> {
    let reqs: Vec<_> = permissions
        .values()
        .map(|&(ref mdata_info, _)| {
            let mdata_info = mdata_info.clone();
            let c2 = client.clone();

            client
                .clone()
                .get_mdata_version(mdata_info.name, mdata_info.type_tag)
                .and_then(move |version| {
                    recovery::del_mdata_user_permissions(
                        &c2,
                        mdata_info.name,
                        mdata_info.type_tag,
                        User::Key(sign_pk),
                        version + 1,
                    )
                })
                .map_err(From::from)
        })
        .collect();

    future::join_all(reqs).map(move |_| ()).into_box()
}

// Re-encrypts private containers for a revoked app
fn reencrypt_private_containers(
    client: &Client<()>,
    access_container: MDataInfo,
    containers: AccessContainerEntry,
    revoked_app: &AppInfo,
) -> Box<AuthFuture<()>> {
    // 1. Make sure to get the latest containers info from the root dir (as it
    //    could have been updated on the previous failed revocation)
    // 2. Generate new encryption keys for all the containers to be reencrypted.
    // 3. Update the user root dir and the access container to use the new keys.
    // 4. Perform the actual reencryption of the containers.
    // 5. Update the user root dir and access container again, commiting or aborting
    //    the encryption keys change, depending on whether the re-encryption of the
    //    corresponding container succeeded or failed.
    let c2 = client.clone();
    let c3 = client.clone();

    let containers = start_new_containers_enc_info(containers);
    let app_key = fry!(access_container_key(
        &access_container,
        &revoked_app.info.id,
        &revoked_app.keys,
    ));

    let f0 = update_user_root_dir(client, containers.clone());
    let f1 = update_access_container(
        client,
        access_container.clone(),
        containers.clone(),
        app_key.clone(),
    );

    f0.join(f1)
        .and_then(move |_| reencrypt_containers(&c2, containers))
        .and_then(move |containers| {
            let f0 = update_user_root_dir(&c3, containers.clone());
            let f1 = update_access_container(&c3, access_container, containers, app_key);

            f0.join(f1).map(|_| ())
        })
        .into_box()
}

fn start_new_containers_enc_info(containers: AccessContainerEntry) -> Vec<(String, MDataInfo)> {
    containers
        .into_iter()
        .map(|(container, (mut mdata_info, _))| {
            if mdata_info.new_enc_info.is_none() {
                mdata_info.start_new_enc_info();
            }
            (container, mdata_info)
        })
        .collect()
}

/// Fetches containers info from the user's root dir
fn refresh_from_user_root_dir(
    client: &Client<()>,
    containers: AccessContainerEntry,
) -> Box<AuthFuture<AccessContainerEntry>> {
    let user_root = fry!(client.user_root_dir());

    client
        .list_mdata_entries(user_root.name, user_root.type_tag)
        .and_then(move |entries| {
            let mut refreshed = HashMap::new();
            for (container, (mdata_info, perms)) in containers {
                let key = user_root.enc_entry_key(container.as_bytes())?;

                let _ = refreshed.insert(
                    container,
                    if let Some(new_value) = entries.get(&key) {
                        let decoded = user_root.decrypt(&new_value.content)?;
                        let root_mdata_info = deserialise::<MDataInfo>(&decoded)?;
                        (root_mdata_info, perms)
                    } else {
                        (mdata_info, perms)
                    },
                );
            }
            Ok(refreshed)
        })
        .map_err(AuthError::from)
        .into_box()
}

fn update_user_root_dir(
    client: &Client<()>,
    containers: Vec<(String, MDataInfo)>,
) -> Box<AuthFuture<()>> {
    let c2 = client.clone();
    let user_root = fry!(client.user_root_dir());

    client
        .list_mdata_entries(user_root.name, user_root.type_tag)
        .and_then(move |entries| {
            let mut actions = EntryActions::new();

            for (container, mdata_info) in containers {
                let key = user_root.enc_entry_key(container.as_bytes())?;
                let encoded = serialise(&mdata_info)?;
                let encoded = user_root.enc_entry_value(&encoded)?;

                if let Some(old_value) = entries.get(&key) {
                    actions = actions.update(key, encoded, old_value.entry_version + 1);
                } else {
                    actions = actions.ins(key, encoded, 0);
                }
            }

            Ok((user_root, actions))
        })
        .and_then(move |(user_root, actions)| {
            recovery::mutate_mdata_entries(&c2, user_root.name, user_root.type_tag, actions.into())
        })
        .map_err(From::from)
        .into_box()
}

fn update_access_container(
    client: &Client<()>,
    access_container: MDataInfo,
    mut containers: Vec<(String, MDataInfo)>,
    revoked_app_key: Vec<u8>,
) -> Box<AuthFuture<()>> {
    let c2 = client.clone();

    let f_config = config::list_apps(client).map(|(_, apps)| apps);
    let f_entries = client
        .list_mdata_entries(access_container.name, access_container.type_tag)
        .map_err(From::from)
        .map(move |mut entries| {
            // Remove the revoked app entry from the access container
            // because we don't need it to be reencrypted.
            let _ = entries.remove(&revoked_app_key);
            entries
        });

    f_config
        .join(f_entries)
        .and_then(move |(apps, entries)| {
            let mut actions = EntryActions::new();

            for app in apps.values() {
                let key = access_container_key(&access_container, &app.info.id, &app.keys)?;

                if let Some(raw) = entries.get(&key) {
                    // Skip deleted entries.
                    if raw.content.is_empty() {
                        continue;
                    }

                    let decoded = symmetric_decrypt(&raw.content, &app.keys.enc_key)?;
                    let mut decoded: AccessContainerEntry = deserialise(&decoded)?;

                    for &mut (ref container, ref mdata_info) in &mut containers {
                        if let Some(entry) = decoded.get_mut(container) {
                            entry.0 = mdata_info.clone();
                        }
                    }

                    let encoded = serialise(&decoded)?;
                    let encoded = symmetric_encrypt(&encoded, &app.keys.enc_key, None)?;

                    actions = actions.update(key, encoded, raw.entry_version + 1);
                }
            }

            Ok((access_container, actions))
        })
        .and_then(move |(access_container, actions)| {
            recovery::mutate_mdata_entries(
                &c2,
                access_container.name,
                access_container.type_tag,
                actions.into(),
            ).map_err(From::from)
        })
        .into_box()
}

// Re-encrypt the given `containers` using the `new_enc_info` in the corresponding
// `MDataInfo`. Returns modified `containers` where the enc info regeneration is either
// commited or aborted, depending on if the re-encryption succeeded or failed.
fn reencrypt_containers(
    client: &Client<()>,
    containers: Vec<(String, MDataInfo)>,
) -> Box<AuthFuture<Vec<(String, MDataInfo)>>> {
    let c2 = client.clone();
    let fs = containers.into_iter().map(move |(container, mdata_info)| {
        let mut mdata_info2 = mdata_info.clone();
        let c3 = c2.clone();

        c2.list_mdata_entries(mdata_info.name, mdata_info.type_tag)
            .and_then(move |entries| {
                let mut actions = EntryActions::new();

                for (old_key, value) in entries {
                    // Skip deleted entries.
                    if value.content.is_empty() {
                        continue;
                    }

                    let plain_key = mdata_info.decrypt(&old_key)?;
                    let new_key = mdata_info.enc_entry_key(&plain_key)?;

                    let plain_content = mdata_info.decrypt(&value.content)?;
                    let new_content = mdata_info.enc_entry_value(&plain_content)?;

                    // Delete the old entry with the old key and
                    // insert the re-encrypted entry with a new key
                    actions = actions.del(old_key, value.entry_version + 1).ins(
                        new_key,
                        new_content,
                        0,
                    );
                }

                Ok((mdata_info, actions))
            })
            .and_then(move |(mdata_info, actions)| {
                recovery::mutate_mdata_entries(
                    &c3,
                    mdata_info.name,
                    mdata_info.type_tag,
                    actions.into(),
                ).map_err(From::from)
            })
            .then(move |res| {
                // If the mutation succeeded, commit the enc_info regeneration,
                // otherwise abort it.

                if res.is_ok() {
                    mdata_info2.commit_new_enc_info();
                } else {
                    // TODO: consider logging the error.
                    mdata_info2.abort_new_enc_info();
                }

                Ok((container, mdata_info2))
            })
    });

    future::join_all(fs).into_box()
}
