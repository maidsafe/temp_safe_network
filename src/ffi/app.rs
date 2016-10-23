// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

//! Structure representing application registered with the launcher + set of
//! FFI operations on it.

// TODO(Spandan) - Run through this and make interfaces efficient (return references instead of
// copies etc.) and uniform (i.e. not use get_ prefix for mem functions.

use core::{Client, CoreMsg};
use core::futures::FutureExt;
use ffi::{FfiFuture, OpaqueCtx};
use futures::{self, Future};
use libc::int32_t;
use nfs::DirId;
use rust_sodium::crypto::{box_, secretbox};
use super::Session;
use super::errors::FfiError;
use super::helper;
use super::launcher_config;
use super::object_cache::AppHandle;

/// Represents an application connected to the launcher.
#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub enum App {
    /// Unautorised applicationa
    Unauthorised,
    /// Authorised application
    Registered {
        /// Application directory
        app_dir_id: DirId,
        /// Defines whether the application has access to SAFE Drive
        safe_drive_access: bool,
        /// Asymmetric encryption keys of the app
        asym_enc_keys: (box_::PublicKey, box_::SecretKey),
        /// Symmetric encryption key of the app
        sym_key: secretbox::Key,
    },
}

impl App {
    /// Get app root directory key
    pub fn sym_key(&self) -> Result<secretbox::Key, FfiError> {
        if let App::Registered { ref sym_key, .. } = *self {
            Ok(sym_key.clone())
        } else {
            Err(FfiError::OperationForbiddenForApp)
        }
    }

    /// Get asymmetric encryption key for the app
    pub fn asym_enc_keys(&self) -> Result<(box_::PublicKey, box_::SecretKey), FfiError> {
        if let App::Registered { ref asym_enc_keys, .. } = *self {
            Ok(asym_enc_keys.clone())
        } else {
            Err(FfiError::OperationForbiddenForApp)
        }
    }

    /// Get app root directory ID
    pub fn app_dir(&self) -> Result<DirId, FfiError> {
        if let App::Registered { ref app_dir_id, .. } = *self {
            Ok(app_dir_id.clone())
        } else {
            Err(FfiError::OperationForbiddenForApp)
        }
    }

    /// Get root directory: for shared paths, this is the SAFEdrive directory,
    /// otherwise it's the app directory.
    pub fn root_dir(&self, client: Client, is_shared: bool) -> Box<FfiFuture<DirId>> {
        if is_shared {
            if let App::Registered { ref safe_drive_access, .. } = *self {
                if !safe_drive_access {
                    return err!(FfiError::PermissionDenied);
                }
                helper::safe_drive_metadata(client.clone())
                    .map(move |dir_meta| dir_meta.id())
                    .into_box()
            } else {
                err!(FfiError::from("Safe Drive directory key is not present"))
            }
        } else {
            futures::done(self.app_dir()
                    .map_err(move |_| FfiError::from("Application directory is not present")))
                .into_box()
        }
    }
}

/// Register an app with the launcher. The returned app handle must be disposed
/// of by calling `drop_app` once no longer needed.
#[no_mangle]
pub unsafe extern "C" fn register_app(session: *mut Session,
                                      app_name: *const u8,
                                      app_name_len: usize,
                                      unique_token: *const u8,
                                      token_len: usize,
                                      vendor: *const u8,
                                      vendor_len: usize,
                                      safe_drive_access: bool,
                                      user_data: OpaqueCtx,
                                      o_cb: extern "C" fn(int32_t, OpaqueCtx, AppHandle))
                                      -> int32_t {
    helper::catch_unwind_i32(|| {
        let app_name = ffi_try!(helper::c_utf8_to_string(app_name, app_name_len));
        let unique_token = ffi_try!(helper::c_utf8_to_string(unique_token, token_len));
        let vendor = ffi_try!(helper::c_utf8_to_string(vendor, vendor_len));

        let s2 = (*session).clone();

        ffi_try!((*session)
            .send(CoreMsg::new(move |client| {
                let fut =
                    launcher_config::app(client, app_name, unique_token, vendor, safe_drive_access)
                        .map_err(move |e| o_cb(ffi_error_code!(e), user_data, 0))
                        .map(move |app| {
                            let app_handle = unwrap!(s2.object_cache()).insert_app(app);
                            o_cb(0, user_data, app_handle);
                        })
                        .into_box();
                Some(fut)
            }))
            .map_err(FfiError::from));

        0
    })
}

/// Register an annonymous app with the launcher. Can access only public data
#[no_mangle]
pub unsafe extern "C" fn create_unauthorised_app(session: *mut Session,
                                                 o_app_handle: *mut AppHandle)
                                                 -> int32_t {
    helper::catch_unwind_i32(|| {
        *o_app_handle = unwrap!((*session).object_cache()).insert_app(App::Unauthorised);
        0
    })
}

/// Discard and clean up the previously allocated app.
#[no_mangle]
pub unsafe extern "C" fn drop_app(app_handle: *mut App) {
    let _ = Box::from_raw(app_handle);
}
