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

use super::{AuthError, AuthFuture};
use access_container::{self, AUTHENTICATOR_ENTRY};
use config::{self, AppInfo, RevocationQueue};
use futures::Future;
use futures::future::{self, Either, Loop};
use routing::{ClientError, EntryActions, User};
use rust_sodium::crypto::sign;
use safe_core::{Client, CoreError, FutureExt, MDataInfo};
use safe_core::ipc::IpcError;
use safe_core::recovery;
use std::collections::HashMap;

/// Revokes app access using a revocation queue
pub fn revoke_app(client: &Client<()>, app_id: &str) -> Box<AuthFuture<()>> {
    let app_id = app_id.to_string();
    let client = client.clone();
    let c2 = client.clone();

    config::get_app_revocation_queue(&client)
        .and_then(move |(version, queue)| {
            config::push_to_app_revocation_queue(
                &client,
                queue,
                config::next_version(version),
                app_id,
            )
        })
        .and_then(move |(version, queue)| {
            flush_app_revocation_queue_impl(&c2, queue, version + 1)
        })
        .into_box()
}

/// Revoke all apps currently in the revocation queue.
pub fn flush_app_revocation_queue(client: &Client<()>) -> Box<AuthFuture<()>> {
    let client = client.clone();

    config::get_app_revocation_queue(&client)
        .and_then(move |(version, queue)| if let Some(version) = version {
            flush_app_revocation_queue_impl(&client, queue, version + 1)
        } else {
            future::ok(()).into_box()
        })
        .into_box()
}

fn flush_app_revocation_queue_impl(
    client: &Client<()>,
    queue: RevocationQueue,
    version: u64,
) -> Box<AuthFuture<()>> {
    let client = client.clone();

    future::loop_fn((queue, version), move |(queue, version)| {
        let c2 = client.clone();
        let c3 = client.clone();

        if let Some(app_id) = queue.front().cloned() {
            let f = revoke_single_app(&c2, &app_id)
                .and_then(move |_| {
                    config::remove_from_app_revocation_queue(&c3, queue, version, app_id)
                })
                .and_then(move |(version, queue)| {
                    Ok(Loop::Continue((queue, version + 1)))
                });
            Either::A(f)
        } else {
            Either::B(future::ok(Loop::Break(())))
        }
    }).into_box()
}

/// Revoke access for a single app
fn revoke_single_app(client: &Client<()>, app_id: &str) -> Box<AuthFuture<()>> {
    let c2 = client.clone();
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
            access_container::fetch_entry(&c4, &app.info.id, app.keys.clone())
                .and_then(move |(version, ac_entry)| {
                    let containers: HashMap<_, _> = ac_entry
                        .ok_or(AuthError::IpcError(IpcError::UnknownApp))?
                        .into_iter()
                        .map(|(name, (mdata_info, _))| (name, mdata_info))
                        .collect();

                    Ok((app, version, containers))
                })
        })
        .and_then(move |(app, ac_entry_version, containers)| {
            revoke_container_perms(&c5, &containers, app.keys.sign_pk)
                .map(move |_| (app, ac_entry_version, containers))
        })
        .and_then(move |(app, ac_entry_version, containers)| {
            refresh_from_access_container_root(&c6, containers).map(move |refreshed_containers| {
                (app, ac_entry_version, refreshed_containers)
            })
        })
        .and_then(move |(app, ac_entry_version, containers)| {
            reencrypt_containers_and_update_access_container(&c7, containers, &app)
                .map(move |_| (app, ac_entry_version))
        })
        .and_then(move |(app, version)| {
            access_container::delete_entry(&c8, &app.info.id, &app.keys, version + 1)
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
        .or_else(|error| match error {
            CoreError::RoutingClientError(ClientError::NoSuchKey) => Ok(()),
            error => Err(AuthError::from(error)),
        })
        .into_box()
}

// Revokes containers permissions
fn revoke_container_perms(
    client: &Client<()>,
    containers: &HashMap<String, MDataInfo>,
    sign_pk: sign::PublicKey,
) -> Box<AuthFuture<()>> {
    let reqs: Vec<_> = containers
        .values()
        .map(|mdata_info| {
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

/// Fetches containers info from the user's root dir
fn refresh_from_access_container_root(
    client: &Client<()>,
    containers: HashMap<String, MDataInfo>,
) -> Box<AuthFuture<HashMap<String, MDataInfo>>> {
    access_container::fetch_authenticator_entry(client)
        .and_then(move |(_, mut entries)| {
            Ok(
                containers
                    .into_iter()
                    .map(|(name, mdata_info)| if let Some(root_mdata_info) =
                        entries.remove(&name)
                    {
                        (name, root_mdata_info)
                    } else {
                        (name, mdata_info)
                    })
                    .collect(),
            )
        })
        .map_err(AuthError::from)
        .into_box()
}

// Re-encrypts private containers for a revoked app
fn reencrypt_containers_and_update_access_container(
    client: &Client<()>,
    containers: HashMap<String, MDataInfo>,
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

    let ac_info = fry!(client.access_container().map_err(AuthError::from));
    let app_key = fry!(access_container::enc_key(
        &ac_info,
        &revoked_app.info.id,
        &revoked_app.keys.enc_key,
    ));

    update_access_container(
        client,
        ac_info.clone(),
        containers,
        app_key.clone(),
        MDataInfoAction::Start,
    ).and_then(move |containers| {
        reencrypt_containers(&c2, containers.clone()).map(move |_| containers)
    })
        .and_then(move |containers| {
            update_access_container(&c3, ac_info, containers, app_key, MDataInfoAction::Commit)
        })
        .map(|_| ())
        .into_box()
}

// Update `MDataInfo`s of the given containers in all the entries of the access
// container.
fn update_access_container(
    client: &Client<()>,
    ac_info: MDataInfo,
    mut containers: HashMap<String, MDataInfo>,
    revoked_app_key: Vec<u8>,
    mdata_info_action: MDataInfoAction,
) -> Box<AuthFuture<HashMap<String, MDataInfo>>> {
    let c2 = client.clone();
    let c3 = client.clone();

    let apps = config::list_apps(client).map(|(_, apps)| apps);
    let entries = client
        .list_mdata_entries(ac_info.name, ac_info.type_tag)
        .map_err(From::from)
        .map(move |mut entries| {
            // Remove the revoked app entry from the access container
            // because we don't need it to be reencrypted.
            let _ = entries.remove(&revoked_app_key);
            entries
        });

    let auth_key = {
        let sk = fry!(client.secret_symmetric_key());
        fry!(access_container::enc_key(
            &ac_info,
            AUTHENTICATOR_ENTRY,
            &sk,
        ))
    };

    apps.join(entries)
        .and_then(move |(apps, entries)| {
            let mut actions = EntryActions::new();

            // Update the authenticator entry
            if let Some(raw) = entries.get(&auth_key) {
                let sk = c2.secret_symmetric_key()?;
                let mut decoded = access_container::decode_authenticator_entry(&raw.content, &sk)?;

                for (container, mdata_info) in &mut containers {
                    if let Some(entry) = decoded.get_mut(container) {
                        mdata_info_action.apply(entry, mdata_info);
                    }
                }

                let encoded = access_container::encode_authenticator_entry(&decoded, &sk)?;
                actions = actions.update(auth_key, encoded, raw.entry_version + 1);
            }

            // Update apps' entries
            for app in apps.values() {
                let key = access_container::enc_key(&ac_info, &app.info.id, &app.keys.enc_key)?;

                if let Some(raw) = entries.get(&key) {
                    // Skip deleted entries.
                    if raw.content.is_empty() {
                        continue;
                    }

                    let mut decoded =
                        access_container::decode_app_entry(&raw.content, &app.keys.enc_key)?;

                    for (container, mdata_info) in &mut containers {
                        if let Some(entry) = decoded.get_mut(container) {
                            mdata_info_action.apply(&mut entry.0, mdata_info);
                        }
                    }

                    let encoded = access_container::encode_app_entry(&decoded, &app.keys.enc_key)?;
                    actions = actions.update(key, encoded, raw.entry_version + 1);
                }
            }

            Ok((ac_info, actions, containers))
        })
        .and_then(move |(ac_info, actions, containers)| {
            c3.mutate_mdata_entries(ac_info.name, ac_info.type_tag, actions.into())
                .map(move |_| containers)
                .map_err(From::from)
        })
        .into_box()
}

// Action to be performed on `MDataInfo` during access container update.
enum MDataInfoAction {
    // Start new enc info.
    Start,
    // Commit new enc info.
    Commit,
}

impl MDataInfoAction {
    fn apply(&self, stored: &mut MDataInfo, cached: &mut MDataInfo) {
        match *self {
            MDataInfoAction::Start => {
                if stored.new_enc_info.is_none() {
                    cached.start_new_enc_info();
                    *stored = cached.clone();
                } else {
                    *cached = stored.clone();
                }
            }
            MDataInfoAction::Commit => {
                if stored.new_enc_info.is_some() {
                    cached.commit_new_enc_info();
                    *stored = cached.clone();
                } else {
                    *cached = stored.clone();
                }
            }
        }
    }
}

// Re-encrypt the given `containers` using the `new_enc_info` in the corresponding
// `MDataInfo`. Returns modified `containers` where the enc info regeneration is either
// commited or aborted, depending on if the re-encryption succeeded or failed.
fn reencrypt_containers(
    client: &Client<()>,
    containers: HashMap<String, MDataInfo>,
) -> Box<AuthFuture<()>> {
    let c2 = client.clone();
    let fs = containers.into_iter().map(move |(_, mdata_info)| {
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

                    if old_key == new_key {
                        // The entry is already re-encrypted.
                        if value.content != new_content {
                            // It's very unlikely that the key would be already re-encrypted,
                            // but the value not. But let's handle this case anyway, just to
                            // be sure.
                            actions = actions.update(new_key, new_content, value.entry_version + 1);
                        }
                    } else {
                        // Delete the old entry with the old key and
                        // insert the re-encrypted entry with a new key
                        actions = actions.del(old_key, value.entry_version + 1).ins(
                            new_key,
                            new_content,
                            0,
                        );
                    }
                }

                Ok((mdata_info, actions))
            })
            .and_then(move |(mdata_info, actions)| {
                c3.mutate_mdata_entries(mdata_info.name, mdata_info.type_tag, actions.into())
            })
            .map_err(From::from)
    });

    future::join_all(fs).map(|_| ()).into_box()
}
