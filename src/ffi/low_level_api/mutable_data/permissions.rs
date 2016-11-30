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

//! FFI for mutable data permissions and permission sets.

use ffi::{MDataPermissionSetHandle, MDataPermissionsHandle, OpaqueCtx, Session, SignKeyHandle};
use ffi::errors::FfiError;
use ffi::helper as ffi_helper;
use routing::{Action, PermissionSet};
use std::os::raw::c_void;
use super::helper;

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
            MDataAction::ManagePermissions => Action::ManagePermission,
        }
    }
}

/// Create new permission set.
#[no_mangle]
pub unsafe extern "C"
fn mdata_permission_set_new(session: *const Session,
                            user_data: *mut c_void,
                            o_cb: unsafe extern "C" fn(*mut c_void,
                                                       i32,
                                                       MDataPermissionSetHandle)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, |object_cache| {
            Ok(object_cache.insert_mdata_permission_set(PermissionSet::new()))
        })
    })
}

/// Allow the action in the permission set.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_set_allow(session: *const Session,
                                                     set_h: MDataPermissionSetHandle,
                                                     action: MDataAction,
                                                     user_data: *mut c_void,
                                                     o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, move |object_cache| {
            let mut set = object_cache.get_mdata_permission_set(set_h)?;
            let _ = set.allow(action.into());
            Ok(())
        })
    })
}

/// Deny the action in the permission set.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_set_deny(session: *const Session,
                                                    set_h: MDataPermissionSetHandle,
                                                    action: MDataAction,
                                                    user_data: *mut c_void,
                                                    o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, move |object_cache| {
            let mut set = object_cache.get_mdata_permission_set(set_h)?;
            let _ = set.deny(action.into());
            Ok(())
        })
    })
}

/// Clear the actions in the permission set.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_set_clear(session: *const Session,
                                                     set_h: MDataPermissionSetHandle,
                                                     action: MDataAction,
                                                     user_data: *mut c_void,
                                                     o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, move |object_cache| {
            let mut set = object_cache.get_mdata_permission_set(set_h)?;
            let _ = set.clear(action.into());
            Ok(())
        })
    })
}

/// Free the permission set from memory.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_set_free(session: *const Session,
                                                    set_h: MDataPermissionSetHandle,
                                                    user_data: *mut c_void,
                                                    o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, move |object_cache| {
            let _ = object_cache.remove_mdata_permission_set(set_h)?;
            Ok(())
        })
    })
}

/// Create new permissions.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_new(session: *const Session,
                                               user_data: *mut c_void,
                                               o_cb: unsafe extern "C" fn(*mut c_void,
                                                                          i32,
                                                                          MDataPermissionsHandle)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, |object_cache| {
            Ok(object_cache.insert_mdata_permissions(Default::default()))
        })
    })
}

/// Get the number of entries in the permissions.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_len(session: *const Session,
                                               permissions_h: MDataPermissionsHandle,
                                               user_data: *mut c_void,
                                               o_cb: unsafe extern "C" fn(*mut c_void,
                                                                          i32,
                                                                          usize)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, move |object_cache| {
            let permissions = object_cache.get_mdata_permissions(permissions_h)?;
            Ok(permissions.len())
        })
    })
}

/// Get the permission set corresponding to the given user (0 means anyone).
#[no_mangle]
pub unsafe extern "C"
fn mdata_permissions_get(session: *const Session,
                         permissions_h: MDataPermissionsHandle,
                         user_h: SignKeyHandle,
                         user_data: *mut c_void,
                         o_cb: unsafe extern "C" fn(*mut c_void, i32, MDataPermissionSetHandle)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, move |object_cache| {
            let permissions = object_cache.get_mdata_permissions(permissions_h)?;
            let handle = *permissions.get(&user_h).ok_or(FfiError::InvalidSignKeyHandle)?;

            Ok(handle)
        })
    })
}

/// Iterate over the permissions.
/// The `each_cb` is called for each (user, permission set) pair in the permissions.
/// The `done_cb` is called after the iterations is over, or in case of error.
#[no_mangle]
pub unsafe extern "C"
fn mdata_permissions_for_each(session: *const Session,
                              permissions_h: MDataPermissionsHandle,
                              each_cb: unsafe extern "C" fn(*mut c_void,
                                                            SignKeyHandle,
                                                            MDataPermissionSetHandle),
                              user_data: *mut c_void,
                              done_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    ffi_helper::catch_unwind_cb(user_data, done_cb, || {
        let user_data = OpaqueCtx(user_data);

        helper::send_sync(session, user_data.0, done_cb, move |object_cache| {
            let permissions = object_cache.get_mdata_permissions(permissions_h)?;
            for (user_h, permission_set_h) in &*permissions {
                each_cb(user_data.0, *user_h, *permission_set_h);
            }

            Ok(())
        })
    })
}

/// Insert permission set for the given user to the permissions.
///
/// To insert permissions for "Anyone", pass 0 as the user handle.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_insert(session: *const Session,
                                                  permissions_h: MDataPermissionsHandle,
                                                  user_h: SignKeyHandle,
                                                  permission_set_h: MDataPermissionSetHandle,
                                                  user_data: *mut c_void,
                                                  o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, move |object_cache| {
            let mut permissions = object_cache.get_mdata_permissions(permissions_h)?;
            let _ = permissions.insert(user_h, permission_set_h);

            Ok(())
        })
    })
}

/// Free the permissions from memory.
#[no_mangle]
pub unsafe extern "C" fn mdata_permissions_free(session: *const Session,
                                                permissions_h: MDataPermissionsHandle,
                                                user_data: *mut c_void,
                                                o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    ffi_helper::catch_unwind_cb(user_data, o_cb, || {
        helper::send_sync(session, user_data, o_cb, move |object_cache| {
            let _ = object_cache.remove_mdata_permissions(permissions_h)?;
            Ok(())
        })
    })
}
