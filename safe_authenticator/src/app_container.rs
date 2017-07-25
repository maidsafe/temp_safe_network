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

//! Routines to handle an apps dedicated containers

use AuthFuture;
use futures::Future;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::{Action, ClientError, EntryActions, PermissionSet, User};
use rust_sodium::crypto::sign;
use safe_core::{Client, CoreError, FutureExt, MDataInfo, nfs};

/// Checks if an app's dedicated container is available and stored in the user's root dir.
/// If no previously created container has been found, then it will be created.
pub fn fetch(
    client: Client<()>,
    app_id: String,
    app_sign_pk: sign::PublicKey,
) -> Box<AuthFuture<MDataInfo>> {
    let root = fry!(client.user_root_dir());
    let app_cont_name = format!("apps/{}", app_id);
    let key = fry!(root.enc_entry_key(app_cont_name.as_bytes()));

    let c2 = client.clone();

    client
        .get_mdata_value(root.name, root.type_tag, key)
        .then(move |res| {
            match res {
                // If the container is not found, create it
                Ok(ref v) if v.content.is_empty() => create(client, &app_id, app_sign_pk),
                Err(CoreError::RoutingClientError(ClientError::NoSuchEntry)) => {
                    create(client, &app_id, app_sign_pk)
                }
                // Reuse the already existing app container
                Ok(val) => {
                    let plaintext = fry!(root.decrypt(&val.content));
                    let mdata_info = fry!(deserialise::<MDataInfo>(&plaintext));

                    // Update permissions for the app container
                    let ps = PermissionSet::new()
                        .allow(Action::Insert)
                        .allow(Action::Update)
                        .allow(Action::Delete)
                        .allow(Action::ManagePermissions);

                    c2.get_mdata_version(mdata_info.name, mdata_info.type_tag)
                        .and_then(move |version| {
                            c2.set_mdata_user_permissions(
                                mdata_info.name,
                                mdata_info.type_tag,
                                User::Key(app_sign_pk),
                                ps,
                                version + 1,
                            ).map(move |_| mdata_info)
                        })
                        .map_err(From::from)
                        .into_box()
                }
                Err(e) => err!(e),
            }
        })
        .into_box()
}

/// Creates a new app dedicated container
pub fn create(
    client: Client<()>,
    app_id: &str,
    app_sign_pk: sign::PublicKey,
) -> Box<AuthFuture<MDataInfo>> {
    let root = fry!(client.user_root_dir());
    let app_cont_name = format!("apps/{}", app_id);

    let c2 = client.clone();

    nfs::create_dir(&client, false)
        .map_err(From::from)
        .and_then(move |dir| {
            let serialised = fry!(serialise(&dir));
            let key = fry!(root.enc_entry_key(app_cont_name.as_bytes()));
            let ciphertext = fry!(root.enc_entry_value(&serialised));

            let actions = EntryActions::new().ins(key, ciphertext, 0);
            client
                .mutate_mdata_entries(root.name, root.type_tag, actions.into())
                .map_err(From::from)
                .map(move |_| dir)
                .into_box()
        })
        .and_then(move |dir| {
            let ps = PermissionSet::new()
                .allow(Action::Insert)
                .allow(Action::Update)
                .allow(Action::Delete)
                .allow(Action::ManagePermissions);

            c2.set_mdata_user_permissions(dir.name, dir.type_tag, User::Key(app_sign_pk), ps, 1)
                .map_err(From::from)
                .map(move |_| dir)
        })
        .into_box()
}

/// Removes an app's dedicated container if it's available and stored in the user's root dir.
/// Returns `true` if it was removed successfully and `false` if it wasn't found in the parent dir.
pub fn remove(client: Client<()>, app_id: &str) -> Box<AuthFuture<bool>> {
    let root = fry!(client.user_root_dir());
    let app_cont_name = format!("apps/{}", app_id);
    let key = fry!(root.enc_entry_key(app_cont_name.as_bytes()));

    let c2 = client.clone();

    client
        .get_mdata_value(root.name, root.type_tag, key.clone())
        .then(move |res| {
            match res {
                Err(CoreError::RoutingClientError(ClientError::NoSuchEntry)) => {
                    // App container doesn't exist
                    ok!(false)
                }
                Err(e) => err!(e),
                Ok(val) => {
                    let decrypted = fry!(root.decrypt(&val.content));
                    let mdata_info = fry!(deserialise::<MDataInfo>(&decrypted));

                    let c3 = c2.clone();

                    c2.list_mdata_entries(mdata_info.name, mdata_info.type_tag)
                        .and_then(move |entries| {
                            // Remove all entries in MData
                            let actions = entries.iter().fold(EntryActions::new(), |actions,
                             (entry_name, val)| {
                                actions.del(entry_name.clone(), val.entry_version + 1)
                            });
                            c3.mutate_mdata_entries(
                                mdata_info.name,
                                mdata_info.type_tag,
                                actions.into(),
                            )
                        })
                        .and_then(move |_| {
                            // Remove MData itself
                            let actions = EntryActions::new().del(key, val.entry_version + 1);
                            client.mutate_mdata_entries(root.name, root.type_tag, actions.into())

                            // TODO(nbaksalyar): when MData deletion is implemented properly,
                            // also delete the actual MutableData related to app
                        })
                        .map_err(From::from)
                        .map(move |_| true)
                        .into_box()
                }
            }
        })
        .into_box()
}
