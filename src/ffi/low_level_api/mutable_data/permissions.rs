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

use ffi::{MDataPermissionSetHandle, Session};
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
