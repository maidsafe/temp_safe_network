// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

use AccessContainerEntry;
use AuthError;
use Authenticator;
use access_container::{access_container, access_container_key, access_container_nonce};
use ffi_utils::{FfiString, OpaqueCtx, catch_unwind_cb, vec_into_raw_parts};
use ffi_utils::callback::CallbackArgs;
use futures::Future;
use ipc::{AppState, app_state, get_config, remove_app_container, update_config};
use maidsafe_utilities::serialisation::deserialise;
use rust_sodium::crypto::hash::sha256;
use safe_core::FutureExt;
use safe_core::ipc::IpcError;
use safe_core::ipc::req::ffi::{AppExchangeInfo, AppExchangeInfoArray, ContainerPermissions,
                               ContainerPermissionsArray, PermissionArray, app_exchange_info_drop,
                               container_permissions_array_free};
use safe_core::utils::symmetric_decrypt;
use std::os::raw::c_void;
use std::ptr;

/// Application registered in the authenticator
#[repr(C)]
pub struct RegisteredApp {
    /// Unique application identifier
    pub app_info: AppExchangeInfo,
    /// List of containers that this application has access to
    pub containers: ContainerPermissionsArray,
}

impl Drop for RegisteredApp {
    fn drop(&mut self) {
        unsafe {
            app_exchange_info_drop(self.app_info);
            container_permissions_array_free(self.containers);
        }
    }
}

/// Removes a revoked app from the authenticator config
pub unsafe extern "C" fn authenticator_rm_revoked_app(auth: *const Authenticator,
                                                      app_id: FfiString,
                                                      user_data: *mut c_void,
                                                      o_cb: extern "C" fn(*mut c_void, i32)) {

    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data.0, o_cb, || -> Result<_, AuthError> {
        let app_id = app_id.to_string()?;
        let app_id2 = app_id.clone();
        let app_id_hash = sha256::hash(app_id.clone().as_bytes());

        (*auth).send(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();

            get_config(client)
                .and_then(move |(cfg_version, auth_cfg)| {
                    app_state(c2, &auth_cfg, app_id)
                        .map(move |app_state| (app_state, auth_cfg, cfg_version))
                })
                .and_then(move |(app_state, auth_cfg, cfg_version)| {
                    match app_state {
                        AppState::Revoked => ok!((auth_cfg, cfg_version)),
                        AppState::Authenticated => {
                            err!(AuthError::Unexpected("App is not revoked".to_owned()))
                        }
                        AppState::NotAuthenticated => {
                            err!(AuthError::IpcError(IpcError::UnknownApp))
                        }
                    }
                })
                .and_then(move |(mut auth_cfg, cfg_version)| {
                    let _app = fry!(auth_cfg.remove(&app_id_hash)
                        .ok_or(AuthError::Unexpected("Logical error: app isn't found in \
                                                      authenticator config"
                            .to_owned())));

                    update_config(c3, Some(cfg_version + 1), auth_cfg)
                })
                .and_then(move |_| remove_app_container(c4, app_id2))
                .map(move |_| o_cb(user_data.0, 0))
                .map_err(move |e| o_cb(user_data.0, ffi_error_code!(e)))
                .into_box()
                .into()
        })
    });
}

/// Get a list of apps revoked from authenticator
pub unsafe extern "C" fn authenticator_revoked_apps(auth: *const Authenticator,
                                                    user_data: *mut c_void,
                                                    o_cb: extern "C" fn(*mut c_void,
                                                                        i32,
                                                                        AppExchangeInfoArray))
                                                    -> i32 {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data.0, o_cb, || -> Result<_, AuthError> {
        (*auth).send(move |client| {
                let c2 = client.clone();
                let c3 = client.clone();

                get_config(client)
                    .and_then(move |(_, auth_cfg)| {
                        access_container(c2)
                            .map(move |access_container| (access_container, auth_cfg))
                    })
                    .and_then(move |(access_container, auth_cfg)| {
                        c3.list_mdata_entries(access_container.name, access_container.type_tag)
                            .map_err(From::from)
                            .map(move |entries| (access_container, entries, auth_cfg))
                    })
                    .and_then(move |(access_container, entries, auth_cfg)| {
                        let mut apps = Vec::new();

                        for app in auth_cfg.values() {
                            let nonce = access_container_nonce(&access_container)?;
                            let key = access_container_key(&app.info.id, &app.keys, nonce);

                            if !entries.contains_key(&key) {
                                // If the app is not in access container, then it's revoked
                                apps.push(app.info.clone().into_repr_c());
                            }
                        }

                        o_cb(user_data.0, 0, AppExchangeInfoArray::from_vec(apps));

                        Ok(())
                    })
                    .map_err(move |e| {
                        o_cb(user_data.0,
                             ffi_error_code!(e),
                             AppExchangeInfoArray::default())
                    })
                    .into_box()
                    .into()
            })?;

        Ok(())
    });

    0
}

/// Get a list of apps registered in authenticator
#[no_mangle]
pub unsafe extern "C" fn authenticator_registered_apps(auth: *const Authenticator,
                                                       user_data: *mut c_void,
                                                       o_cb: extern "C" fn(*mut c_void,
                                                                           i32,
                                                                           *mut RegisteredApp,
                                                                           usize,
                                                                           usize))
                                                       -> i32 {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data.0, o_cb, || -> Result<_, AuthError> {
        (*auth).send(move |client| {
                let c2 = client.clone();
                let c3 = client.clone();

                get_config(client)
                    .and_then(move |(_, auth_cfg)| {
                        access_container(c2)
                            .map(move |access_container| (access_container, auth_cfg))
                    })
                    .and_then(move |(access_container, auth_cfg)| {
                        c3.list_mdata_entries(access_container.name, access_container.type_tag)
                            .map_err(From::from)
                            .map(move |entries| (access_container, entries, auth_cfg))
                    })
                    .and_then(move |(access_container, entries, auth_cfg)| {
                        let mut apps = Vec::new();

                        for app in auth_cfg.values() {
                            let nonce = access_container_nonce(&access_container)?;
                            let key = access_container_key(&app.info.id, &app.keys, nonce);

                            if let Some(entry) = entries.get(&key) {
                                let plaintext = symmetric_decrypt(&entry.content,
                                                                  &app.keys.enc_key)?;
                                let app_access = deserialise::<AccessContainerEntry>(&plaintext)?;

                                let mut containers = Vec::new();

                                for (key, (_, perms)) in app_access {
                                    let perms = perms.iter().cloned().collect::<Vec<_>>();

                                    containers.push(ContainerPermissions {
                                        cont_name: FfiString::from_string(key),
                                        access: PermissionArray::from_vec(perms),
                                    });
                                }

                                let reg_app = RegisteredApp {
                                    app_info: app.info.clone().into_repr_c(),
                                    containers: ContainerPermissionsArray::from_vec(containers),
                                };

                                apps.push(reg_app);
                            }
                        }

                        let (ptr, len, cap) = vec_into_raw_parts(apps);
                        o_cb(user_data.0, 0, ptr, len, cap);
                        Ok(())
                    })
                    .map_err(move |e| {
                        o_cb(user_data.0, ffi_error_code!(e), ptr::null_mut(), 0, 0)
                    })
                    .into_box()
                    .into()
            })?;

        Ok(())
    });

    0
}

/// Free memory allocated for a `RegisteredApp` structure
#[no_mangle]
pub unsafe extern "C" fn authenticator_registered_app_free(app: *mut RegisteredApp) {
    let _ = Box::from_raw(app);
}

/// Free memory allocated to a vector of registered applications
#[no_mangle]
pub unsafe extern "C" fn authenticator_registered_apps_free(apps: *mut RegisteredApp,
                                                            len: usize,
                                                            cap: usize) {
    let _ = Vec::from_raw_parts(apps, len, cap);
}
