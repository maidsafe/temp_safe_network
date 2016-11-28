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

/// FFI for mutable data entry actions.
pub mod entry_actions;
/// FFI for mutable data entries, keys and values.
pub mod entries;

use core::{Client, CoreError, FutureExt};
use ffi::{MDataEntriesHandle, MDataEntryActionsHandle, MDataKeysHandle, MDataValuesHandle,
          OpaqueCtx, Session, helper};
use ffi::callback::{Callback, CallbackArgs};
use ffi::errors::FfiError;
use ffi::object_cache::ObjectCache;
use futures::Future;
use routing::{MutableData, XOR_NAME_LEN, XorName};
use std::fmt::Debug;
use std::os::raw::c_void;

/// Create new mutable data and put it on the network.
#[no_mangle]
pub unsafe extern "C" fn mdata_put(session: *const Session,
                                   name: *const [u8; XOR_NAME_LEN],
                                   type_tag: u64,
                                   entries_h: MDataEntriesHandle,
                                   user_data: *mut c_void,
                                   o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    helper::catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);
        let name = XorName(*name);

        (*session).send(move |client, object_cache| {
            let sign_pk = try_cb!(client.public_signing_key(), user_data, o_cb);
            let entries = if entries_h != 0 {
                try_cb!(object_cache.get_mdata_entries(entries_h), user_data, o_cb).clone()
            } else {
                Default::default()
            };

            let data = try_cb!(MutableData::new(name,
                                                type_tag,
                                                Default::default(),
                                                entries,
                                                btree_set![sign_pk])
                                   .map_err(CoreError::from),
                               user_data,
                               o_cb);

            client.put_mdata(data, None)
                .then(move |result| {
                    o_cb(user_data.0, ffi_result_code!(result));
                    Ok(())
                })
                .into_box()
                .into()
        })
    })
}

/// Get version of a mutable data.
#[no_mangle]
pub unsafe extern "C" fn mdata_get_version(session: *const Session,
                                           name: *const [u8; XOR_NAME_LEN],
                                           type_tag: u64,
                                           user_data: *mut c_void,
                                           o_cb: unsafe extern "C" fn(*mut c_void, i32, u64)) {
    helper::catch_unwind_cb(user_data, o_cb, || {
        let name = XorName(*name);

        send(session,
             user_data,
             o_cb,
             move |client, _| client.get_mdata_version(name, type_tag, None))
    })
}

/// Get value at the given key from a mutable data.
/// The arguments to the callback are:
///     1. user data
///     2. error code
///     3. pointer to content
///     4. content length
///     5. content capacity
///     6. entry version
#[no_mangle]
pub unsafe extern "C" fn mdata_get_value(session: *const Session,
                                         name: *const [u8; XOR_NAME_LEN],
                                         type_tag: u64,
                                         key_ptr: *const u8,
                                         key_len: usize,
                                         user_data: *mut c_void,
                                         o_cb: unsafe extern "C" fn(*mut c_void,
                                                                    i32,
                                                                    *mut u8,
                                                                    usize,
                                                                    usize,
                                                                    u64)) {
    helper::catch_unwind_cb(user_data, o_cb, || {
        let name = XorName(*name);
        let key = helper::u8_ptr_to_vec(key_ptr, key_len);

        send(session, user_data, o_cb, move |client, _| {
            client.get_mdata_value(name, type_tag, key, None)
                .map(move |value| {
                    let content = helper::u8_vec_to_ptr(value.content);
                    (content.0, content.1, content.2, value.entry_version)
                })
        })
    })
}

/// Get complete list of entries in a mutable data.
#[no_mangle]
pub unsafe extern "C" fn mdata_list_entries(session: *const Session,
                                            name: *const [u8; XOR_NAME_LEN],
                                            type_tag: u64,
                                            user_data: *mut c_void,
                                            o_cb: unsafe extern "C" fn(*mut c_void,
                                                                       i32,
                                                                       MDataEntriesHandle)) {
    helper::catch_unwind_cb(user_data, o_cb, || {
        let name = XorName(*name);

        send(session, user_data, o_cb, move |client, object_cache| {
            let object_cache = object_cache.clone();
            client.list_mdata_entries(name, type_tag, None)
                .map(move |entries| object_cache.insert_mdata_entries(entries))
        })
    })
}

/// Get list of keys in a mutable data.
#[no_mangle]
pub unsafe extern "C" fn mdata_list_keys(session: *const Session,
                                         name: *const [u8; XOR_NAME_LEN],
                                         type_tag: u64,
                                         user_data: *mut c_void,
                                         o_cb: unsafe extern "C" fn(*mut c_void,
                                                                    i32,
                                                                    MDataKeysHandle)) {
    helper::catch_unwind_cb(user_data, o_cb, || {
        let name = XorName(*name);

        send(session, user_data, o_cb, move |client, object_cache| {
            let object_cache = object_cache.clone();
            client.list_mdata_keys(name, type_tag, None)
                .map(move |keys| object_cache.insert_mdata_keys(keys))
        })
    })
}

/// Get list of values in a mutable data.
#[no_mangle]
pub unsafe extern "C" fn mdata_list_values(session: *const Session,
                                           name: *const [u8; XOR_NAME_LEN],
                                           type_tag: u64,
                                           user_data: *mut c_void,
                                           o_cb: unsafe extern "C" fn(*mut c_void,
                                                                      i32,
                                                                      MDataValuesHandle)) {
    helper::catch_unwind_cb(user_data, o_cb, || {
        let name = XorName(*name);

        send(session, user_data, o_cb, move |client, object_cache| {
            let object_cache = object_cache.clone();
            client.list_mdata_values(name, type_tag, None)
                .map(move |values| object_cache.insert_mdata_values(values))
        })
    })
}

/// Mutate entries of a mutable data.
#[no_mangle]
pub unsafe fn mdata_mutate_entries(session: *const Session,
                                   name: *const [u8; XOR_NAME_LEN],
                                   type_tag: u64,
                                   actions_h: MDataEntryActionsHandle,
                                   user_data: *mut c_void,
                                   o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    helper::catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);
        let name = XorName(*name);

        (*session).send(move |client, object_cache| {
            let actions = try_cb!(object_cache.get_mdata_entry_actions(actions_h),
                                  user_data,
                                  o_cb)
                .clone();

            client.mutate_mdata_entries(name, type_tag, actions, None)
                .then(move |result| {
                    o_cb(user_data.0, ffi_result_code!(result));
                    Ok(())
                })
                .into_box()
                .into()
        })
    })
}

// FFI call boilerplate
// TODO: consider moving this over to `Session`, or `helper`, ...
unsafe fn send<C, F, U, E>(session: *const Session,
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
