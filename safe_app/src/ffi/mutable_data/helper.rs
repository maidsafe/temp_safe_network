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

use errors::AppError;
use object_cache::{MDataPermissionsHandle, ObjectCache, SignKeyHandle};
use routing::{PermissionSet, User};
use std::collections::BTreeMap;

// Retrieve the sign key corresponding to the handle from the object cache and wrap it
// in `User`. If the handle is 0, return `User::Anyone`.
pub fn get_user(object_cache: &ObjectCache, handle: SignKeyHandle) -> Result<User, AppError> {
    let user = if handle != 0 {
        let sign_key = object_cache.get_sign_key(handle)?;
        User::Key(*sign_key)
    } else {
        User::Anyone
    };

    Ok(user)
}

// Insert the permissions into the object cache.
pub fn insert_permissions(object_cache: &ObjectCache,
                          permissions: BTreeMap<User, PermissionSet>)
                          -> MDataPermissionsHandle {
    let permissions = permissions.into_iter()
        .map(|(user, permission_set)| {
                 let permission_set_h = object_cache.insert_mdata_permission_set(permission_set);
                 (user, permission_set_h)
             })
        .collect();

    object_cache.insert_mdata_permissions(permissions)
}

// Retrieve permissions from the object cache.
pub fn get_permissions(object_cache: &ObjectCache,
                       handle: MDataPermissionsHandle)
                       -> Result<BTreeMap<User, PermissionSet>, AppError> {
    let input = object_cache.get_mdata_permissions(handle)?.clone();
    let mut output = BTreeMap::new();

    for (user, permission_set_h) in input {
        let permission_set = *object_cache.get_mdata_permission_set(permission_set_h)?;
        let _ = output.insert(user, permission_set);
    }

    Ok(output)
}
