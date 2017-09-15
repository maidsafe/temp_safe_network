// Copyright 2016 MaidSafe.net limited.
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

//! FFI for mutable data permissions and permission sets.

use App;
use errors::AppError;
use ffi::helper::send_sync;
use ffi::mutable_data::helper;
use ffi_utils::{FfiResult, OpaqueCtx, catch_unwind_cb};
use object_cache::{MDataPermissionsHandle, SignKeyHandle};
use routing::{Action, User};
use safe_core::ffi::ipc::req::PermissionSet as FfiPermissionSet;
use safe_core::ipc::req::{permission_set_clone_from_repr_c, permission_set_into_repr_c};
use std::os::raw::c_void;

/// Special value that represents `User::Anyone` in permission sets.
#[no_mangle]
pub static USER_ANYONE: u64 = 0;

/// Permission actions.
#[repr(C)]
pub enum MDataAction {
    /// Permission to insert new entries.
    Insert,
    /// Permission to update existing entries.
    Update,
    /// Permission to delete existing entries.
    Delete,
    /// Permission to manage permissions.
    ManagePermissions,
}

impl Into<Action> for MDataAction {
    fn into(self) -> Action {
        match self {
            MDataAction::Insert => Action::Insert,
            MDataAction::Update => Action::Update,
            MDataAction::Delete => Action::Delete,
            MDataAction::ManagePermissions => Action::ManagePermissions,
        }
    }
}

/// Create new permissions.
///
/// Callback parameters: user data, error code, permissions handle
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_new(
    app: *const App,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void,
                        result: FfiResult,
                        perm_h: MDataPermissionsHandle),
) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, |_, context| {
            Ok(context.object_cache().insert_mdata_permissions(
                Default::default(),
            ))
        })
    })
}

/// Get the number of entries in the permissions.
///
/// Callback parameters: user data, error code, size
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_len(
    app: *const App,
    permissions_h: MDataPermissionsHandle,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: FfiResult, size: usize),
) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |_, context| {
            let permissions = context.object_cache().get_mdata_permissions(permissions_h)?;
            Ok(permissions.len())
        })
    })
}

/// Get the permission set corresponding to the given user.
/// Use a constant `USER_ANYONE` for anyone.
///
/// Callback parameters: user data, error code, permission set handle
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_get(
    app: *const App,
    permissions_h: MDataPermissionsHandle,
    user_h: SignKeyHandle,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void,
                        result: FfiResult,
                        perm_set: FfiPermissionSet),
) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |_, context| {
            let permissions = context.object_cache().get_mdata_permissions(permissions_h)?;
            let permission_set = *permissions
                .get(&helper::get_user(context.object_cache(), user_h)?)
                .ok_or(AppError::InvalidSignKeyHandle)?;

            Ok(permission_set_into_repr_c(permission_set))
        })
    })
}

/// Iterate over the permissions.
/// The `o_each_cb` is called for each (user, permission set) pair in the permissions.
/// The `o_done_cb` is called after the iterations is over, or in case of error.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_for_each(
    app: *const App,
    permissions_h: MDataPermissionsHandle,
    user_data: *mut c_void,
    o_each_cb: extern "C" fn(user_data: *mut c_void,
                             sign_key_h: SignKeyHandle,
                             perm_set: FfiPermissionSet),
    o_done_cb: extern "C" fn(user_data: *mut c_void, result: FfiResult),
) {
    catch_unwind_cb(user_data, o_done_cb, || {
        let user_data = OpaqueCtx(user_data);

        send_sync(app, user_data.0, o_done_cb, move |_, context| {
            let permissions = context.object_cache().get_mdata_permissions(permissions_h)?;
            for (user_key, permission_set) in &*permissions {
                let user_h = match *user_key {
                    User::Key(key) => context.object_cache().insert_sign_key(key),
                    User::Anyone => USER_ANYONE,
                };
                o_each_cb(
                    user_data.0,
                    user_h,
                    permission_set_into_repr_c(*permission_set),
                );
            }

            Ok(())
        })
    })
}

/// Insert permission set for the given user to the permissions.
///
/// To insert permissions for "Anyone", pass `USER_ANYONE` as the user handle.
///
/// Note: the permission sets are stored by reference, which means they must
/// remain alive (not be disposed of with `mdata_permission_set_free`) until
/// the whole permissions collection is no longer needed. The users, on the
/// other hand, are stored by value (copied).
///
/// Callback parameters: user data, error code
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_insert(
    app: *const App,
    permissions_h: MDataPermissionsHandle,
    user_h: SignKeyHandle,
    permission_set: FfiPermissionSet,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |_, context| {
            let mut permissions = context.object_cache().get_mdata_permissions(permissions_h)?;
            let _ = permissions.insert(
                helper::get_user(context.object_cache(), user_h)?,
                unwrap!(permission_set_clone_from_repr_c(&permission_set)),
            );

            Ok(())
        })
    })
}

/// Free the permissions from memory.
///
/// Note: this doesn't free the individual permission sets. Those have to be
/// disposed of manually by calling `mdata_permission_set_free`.
///
/// Callback parameters: user data, error code
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_free(
    app: *const App,
    permissions_h: MDataPermissionsHandle,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |_, context| {
            let _ = context.object_cache().remove_mdata_permissions(
                permissions_h,
            )?;
            Ok(())
        })
    })
}
