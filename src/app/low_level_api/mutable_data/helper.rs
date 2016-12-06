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

use app::{App, AppContext};
use app::errors::AppError;
use app::object_cache::{MDataPermissionsHandle, ObjectCache, SignKeyHandle};
use core::{Client, FutureExt};
use futures::Future;
use routing::{PermissionSet, User};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::os::raw::c_void;
use util::ffi::OpaqueCtx;
use util::ffi::callback::{Callback, CallbackArgs};

// TODO: consider moving the send_* functions to ffi::helper, or even make them methods of
// Session.

// Convenience wrapper around `App::send` which automatically handles the callback
// boilerplate.
// Use this if the lambda never returns future.
pub unsafe fn send_sync<C, F>(app: *const App,
                              user_data: *mut c_void,
                              o_cb: C,
                              f: F)
                              -> Result<(), AppError>
    where C: Callback + Copy + Send + 'static,
          F: FnOnce(&AppContext) -> Result<C::Args, AppError> + Send + 'static
{
    let user_data = OpaqueCtx(user_data);

    (*app).send(move |_, context| {
        match f(context) {
            Ok(args) => o_cb.call(user_data.0, 0, args),
            Err(err) => o_cb.call(user_data.0, ffi_error_code!(err), C::Args::default()),
        }

        None
    })
}

// Convenience wrapper around `App::send` which automatically handles the callback
// boilerplate.
// Use this if the lambda always returns future.
pub unsafe fn send_async<C, F, U, E>(app: *const App,
                                     user_data: *mut c_void,
                                     cb: C,
                                     f: F)
                                     -> Result<(), AppError>
    where C: Callback + Copy + Send + 'static,
          F: FnOnce(&Client, &AppContext) -> U + Send + 'static,
          U: Future<Item = C::Args, Error = E> + 'static,
          E: Debug + 'static,
          AppError: From<E>
{
    let user_data = OpaqueCtx(user_data);

    (*app).send(move |client, context| {
        f(client, context)
            .map(move |args| cb.call(user_data.0, 0, args))
            .map_err(AppError::from)
            .map_err(move |err| cb.call(user_data.0, ffi_error_code!(err), C::Args::default()))
            .into_box()
            .into()
    })
}

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
            let user_h = match user {
                User::Anyone => 0,
                User::Key(key) => object_cache.insert_sign_key(key),
            };
            let permission_set_h = object_cache.insert_mdata_permission_set(permission_set);

            (user_h, permission_set_h)
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

    for (user_h, permission_set_h) in input {
        let user = get_user(object_cache, user_h)?;
        let permission_set = *object_cache.get_mdata_permission_set(permission_set_h)?;

        let _ = output.insert(user, permission_set);
    }

    Ok(output)
}
