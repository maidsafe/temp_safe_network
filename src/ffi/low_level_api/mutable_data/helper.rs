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

use core::{Client, FutureExt};
use ffi::{OpaqueCtx, Session};
use ffi::callback::{Callback, CallbackArgs};
use ffi::errors::FfiError;
use ffi::object_cache::ObjectCache;
use futures::Future;
use std::fmt::Debug;
use std::os::raw::c_void;

// TODO: consider moving these functions to ffi::helper, or even make them methods of
// Session.

// Convenience wrapper around `Session::send` which automatically handles the callback
// boilerplate.
// Use this if the lambda never returns future.
pub unsafe fn send_sync<C, F>(session: *const Session,
                              user_data: *mut c_void,
                              o_cb: C,
                              f: F)
                              -> Result<(), FfiError>
    where C: Callback + Copy + Send + 'static,
          F: FnOnce(&ObjectCache) -> Result<C::Args, FfiError> + Send + 'static
{
    let user_data = OpaqueCtx(user_data);

    (*session).send(move |_, object_cache| {
        match f(object_cache) {
            Ok(args) => o_cb.call(user_data.0, 0, args),
            Err(err) => o_cb.call(user_data.0, ffi_error_code!(err), C::Args::default()),
        }

        None
    })
}

// Convenience wrapper around `Session::send` which automatically handles the callback
// boilerplate.
// Use this if the lambda always returns future.
pub unsafe fn send_async<C, F, U, E>(session: *const Session,
                                     user_data: *mut c_void,
                                     cb: C,
                                     f: F)
                                     -> Result<(), FfiError>
    where C: Callback + Copy + Send + 'static,
          F: FnOnce(&Client, &ObjectCache) -> U + Send + 'static,
          U: Future<Item = C::Args, Error = E> + 'static,
          E: Debug,
          FfiError: From<E>
{
    let user_data = OpaqueCtx(user_data);

    (*session).send(move |client, object_cache| {
        f(client, object_cache)
            .map(move |args| cb.call(user_data.0, 0, args))
            .map_err(move |err| cb.call(user_data.0, ffi_error_code!(err), C::Args::default()))
            .into_box()
            .into()
    })
}
