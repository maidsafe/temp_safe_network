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

use core::{CLIENT_STRUCTURED_DATA_TAG, CoreError};
use core::futures::FutureExt;
use core::structured_data::{self, unversioned, versioned};
use ffi::{FfiError, helper, Session};
use ffi::object_cache::{AppHandle, CipherOptHandle, DataIdHandle, StructDataHandle};
use futures::{self, Future};
use libc::{c_void, int32_t, uint64_t};
use routing::{Data, StructuredData, XorName, XOR_NAME_LEN};
use std::{ptr, slice};
use super::cipher_opt::CipherOpt;


// use core::immut_data_operations;
// use core::client::Client;
// use ffi::low_level_api::object_cache::object_cache;
// use maidsafe_utilities::serialisation::{deserialise, serialise};
// use routing::{DataIdentifier, ImmutableData, NO_OWNER_PUB_KEY};
// use std::mem;
// use std::sync::{Arc, Mutex};

// TOOD: consider moving this macro to ffi::macros as it might be useful elsewhere.
macro_rules! try_cb {
    ($result:expr, $cb:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                $cb(ffi_error_code!(err));
                return None;
            }
        }
    }
}

/// Create new StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_new(session: *const Session,
                                         app_h: AppHandle,
                                         type_tag: u64,
                                         id: *const [u8; XOR_NAME_LEN],
                                         version: u64,
                                         cipher_opt_h: CipherOptHandle,
                                         data: *const u8,
                                         data_len: usize,
                                         user_data: *mut c_void,
                                         o_cb: extern "C" fn(*mut c_void, int32_t, StructDataHandle)) {
    helper::catch_unwind_cb(|| {
        let id = XorName(*id);
        let data = slice::from_raw_parts(data, data_len);

        (*session).send_cb(user_data, move |client, object_cache, user_data| {
            let (sign_pk, sign_sk) = try_cb!(client.signing_keypair(), |code| o_cb(user_data, code, 0));

            let encrypted_data = {
                let mut object_cache = unwrap!(object_cache.lock());
                let (app, cipher_opt) =
                    try_cb!(object_cache.get_app_and_cipher_opt(app_h, cipher_opt_h),
                            |code| o_cb(user_data, code, 0));

                try_cb!(cipher_opt.encrypt(app, data), |code| o_cb(user_data, code, 0))
            };

            let fut = match type_tag {
                ::UNVERSIONED_STRUCT_DATA_TYPE_TAG => {
                    unversioned::create(client,
                                        type_tag,
                                        id,
                                        version,
                                        encrypted_data,
                                        vec![sign_pk],
                                        vec![],
                                        sign_sk,
                                        None)
                }
                ::VERSIONED_STRUCT_DATA_TYPE_TAG => {
                    versioned::create(client,
                                      type_tag,
                                      id,
                                      encrypted_data,
                                      vec![sign_pk],
                                      sign_sk,
                                      None)
                }
                x if x >= CLIENT_STRUCTURED_DATA_TAG => {
                    futures::done(StructuredData::new(type_tag,
                                                      id,
                                                      version,
                                                      encrypted_data,
                                                      vec![sign_pk],
                                                      vec![],
                                                      Some(&sign_sk)))
                        .map_err(CoreError::from)
                        .into_box()
                }
                _ => err!(CoreError::InvalidStructuredDataTypeTag),
            };

            fut.map(move |data| {
                let mut object_cache = unwrap!(object_cache.lock());
                let handle = object_cache.insert_sd(data);
                o_cb(user_data, 0, handle);
            })
            .map_err(move |err| {
                let err = FfiError::from(err);
                o_cb(user_data, ffi_error_code!(err), 0);
            })
            .into()
        })
    }, move |error| o_cb(user_data, error, 0))
}

/// Fetch an existing StructuredData from Network.
#[no_mangle]
pub unsafe extern "C" fn struct_data_fetch(session: *const Session,
                                           data_id_h: DataIdHandle,
                                           user_data: *mut c_void,
                                           o_cb: extern "C" fn(*mut c_void, int32_t, StructDataHandle)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |client, object_cache, user_data| {
            let data_id = {
                let mut object_cache = unwrap!(object_cache.lock());
                *try_cb!(object_cache.get_data_id(data_id_h),
                         |error| o_cb(user_data, error, 0))
            };

            client.get(data_id, None)
                .and_then(|data| match data {
                    Data::Structured(data) => Ok(data),
                    _ => Err(CoreError::ReceivedUnexpectedData),
                })
                .map(move |data| {
                    let mut object_cache = unwrap!(object_cache.lock());
                    let handle = object_cache.insert_sd(data);

                    o_cb(user_data, 0, handle);
                })
                .map_err(move |err| {
                    let err = FfiError::from(err);
                    o_cb(user_data, ffi_error_code!(err), 0);
                })
                .into()
        })
    }, move |error| o_cb(user_data, error, 0))
}

/// Extract DataIdentifier from StructuredData.
#[no_mangle]
pub unsafe extern "C" fn struct_data_extract_data_id(session: *const Session,
                                                     sd_h: StructDataHandle,
                                                     user_data: *mut c_void,
                                                     o_cb: extern "C" fn(*mut c_void, int32_t, DataIdHandle)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |_, object_cache, user_data| -> Option<Result<_, _>> {
            let mut object_cache = unwrap!(object_cache.lock());
            let data_id = {
                let data = try_cb!(object_cache.get_sd(sd_h),
                                   |error| o_cb(user_data, error, 0));
                data.identifier()
            };

            let handle = object_cache.insert_data_id(data_id);
            o_cb(user_data, 0, handle);
            None
        })
    }, move |error| o_cb(user_data, error, 0))
}


/// Put new data into StructuredData.
#[no_mangle]
pub unsafe extern "C" fn struct_data_update(session: *const Session,
                                            app_h: AppHandle,
                                            sd_h: StructDataHandle,
                                            cipher_opt_h: CipherOptHandle,
                                            data: *const u8,
                                            data_len: usize,
                                            user_data: *mut c_void,
                                            o_cb: extern "C" fn(*mut c_void, int32_t)) {
    helper::catch_unwind_cb(|| {
        let data = slice::from_raw_parts(data, data_len);

        (*session).send_cb(user_data, move |client, object_cache, user_data| {
            let sign_sk = try_cb!(client.secret_signing_key(),
                                  |error| o_cb(user_data, error));

            let encrypted_data = {
                let mut object_cache = unwrap!(object_cache.lock());
                let (app, cipher_opt) =
                    try_cb!(object_cache.get_app_and_cipher_opt(app_h, cipher_opt_h),
                            |error| o_cb(user_data, error));
                try_cb!(cipher_opt.encrypt(app, data),
                        |error| o_cb(user_data, error))
            };

            let old_sd = {
                let mut object_cache = unwrap!(object_cache.lock());
                try_cb!(object_cache.remove_sd(sd_h),
                        |error| o_cb(user_data, error))
            };

            let fut = match old_sd.get_type_tag() {
                ::UNVERSIONED_STRUCT_DATA_TYPE_TAG => {
                    unversioned::create(client,
                                        old_sd.get_type_tag(),
                                        *old_sd.name(),
                                        old_sd.get_version() + 1,
                                        encrypted_data,
                                        old_sd.get_owner_keys().clone(),
                                        old_sd.get_previous_owner_keys().clone(),
                                        sign_sk,
                                        None)
                }
                ::VERSIONED_STRUCT_DATA_TYPE_TAG => {
                    let owner_keys = old_sd.get_owner_keys().clone();
                    versioned::update(client,
                                      old_sd,
                                      encrypted_data,
                                      owner_keys,
                                      sign_sk,
                                      None)
                }
                x if x >= CLIENT_STRUCTURED_DATA_TAG => {
                    futures::done(StructuredData::new(old_sd.get_type_tag(),
                                                      *old_sd.name(),
                                                      old_sd.get_version() + 1,
                                                      encrypted_data,
                                                      old_sd.get_owner_keys().clone(),
                                                      old_sd.get_previous_owner_keys().clone(),
                                                      Some(&sign_sk)))
                        .map_err(CoreError::from)
                        .into_box()
                }
                _ => err!(CoreError::InvalidStructuredDataTypeTag),
            };

            fut.map(move |new_sd| {
                // Replace the SD in the object cache with the updated one.
                let mut object_cache = unwrap!(object_cache.lock());
                object_cache.insert_sd_at(sd_h, new_sd);
                o_cb(user_data, 0);
            })
            .map_err(move |err| {
                // TODO: should we put the old SD back to the object cache here?
                // (it would require cloning the SD though)
                o_cb(user_data, ffi_error_code!(err));
            })
            .into()
        })
    }, move |error| o_cb(user_data, error))
}

/// Extract data from StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_extract_data(session: *const Session,
                                                  app_h: AppHandle,
                                                  sd_h: StructDataHandle,
                                                  user_data: *mut c_void,
                                                  o_cb: extern "C" fn(*mut c_void, int32_t, *mut u8, usize, usize)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |client, object_cache, user_data| {
            let fut = {
                let mut object_cache = unwrap!(object_cache.lock());
                let sd = try_cb!(object_cache.get_sd(sd_h),
                                 |error| o_cb(user_data, error, ptr::null_mut(), 0, 0));

                match sd.get_type_tag() {
                    ::UNVERSIONED_STRUCT_DATA_TYPE_TAG => {
                        unversioned::extract_value(client, sd, None)
                    }
                    ::VERSIONED_STRUCT_DATA_TYPE_TAG => {
                        versioned::extract_current_value(client, sd, None)
                    }
                    x if x >= CLIENT_STRUCTURED_DATA_TAG => {
                        ok!(sd.get_data().clone())
                    }
                    _ => err!(CoreError::InvalidStructuredDataTypeTag),
                }
            };

            fut.map_err(FfiError::from)
                .and_then(move |encrypted_data| {
                    let mut object_cache = unwrap!(object_cache.lock());
                    let app = try!(object_cache.get_app(app_h));

                    CipherOpt::decrypt(app, &encrypted_data)
                })
                .map(move |data| {
                    let (ptr, size, capacity) = helper::u8_vec_to_ptr(data);
                    o_cb(user_data, 0, ptr, size, capacity);
                })
                .map_err(move |err| {
                    o_cb(user_data, ffi_error_code!(err), ptr::null_mut(), 0, 0);
                })
                .into()
        })
    }, move |error| o_cb(user_data, error, ptr::null_mut(), 0, 0))
}

/// Get number of versions from a versioned StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_num_of_versions(session: *const Session,
                                                     sd_h: StructDataHandle,
                                                     user_data: *mut c_void,
                                                     o_cb: extern "C" fn(*mut c_void, int32_t, uint64_t)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |_, object_cache, user_data| -> Option<Result<_, _>> {
            let mut object_cache = unwrap!(object_cache.lock());

            let sd = try_cb!(object_cache.get_sd(sd_h),
                            |error| o_cb(user_data, error, 0));
            let num = try_cb!(versioned::version_count(&sd),
                              |error| o_cb(user_data, error, 0));

            o_cb(user_data, 0, num);

            None
        })
    }, move |error| o_cb(user_data, error, 0))
}

/// Get nth (starts from 0) version from a versioned StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_nth_version(session: *const Session,
                                                 app_h: AppHandle,
                                                 sd_h: StructDataHandle,
                                                 n: uint64_t,
                                                 user_data: *mut c_void,
                                                 o_cb: extern "C" fn(*mut c_void, int32_t, *mut u8, usize, usize)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |client, object_cache, user_data| {
            let fut = {
                let mut object_cache = unwrap!(object_cache.lock());
                let sd = try_cb!(object_cache.get_sd(sd_h),
                                 |error| o_cb(user_data, error, ptr::null_mut(), 0, 0));

                if sd.get_type_tag() == ::VERSIONED_STRUCT_DATA_TYPE_TAG {
                    versioned::extract_value(client, sd, n, None)
                } else {
                    err!(CoreError::InvalidStructuredDataTypeTag)
                }
            };

            fut.map_err(FfiError::from)
                .and_then(move |encrypted_data| {
                    let mut object_cache = unwrap!(object_cache.lock());
                    let app = try!(object_cache.get_app(app_h));

                    CipherOpt::decrypt(app, &encrypted_data)
                })
                .map(move |data| {
                    let (ptr, size, capacity) = helper::u8_vec_to_ptr(data);
                    o_cb(user_data, 0, ptr, size, capacity);
                })
                .map_err(move |err| {
                    o_cb(user_data, ffi_error_code!(err), ptr::null_mut(), 0, 0);
                })
                .into()

        })
    }, move |error| o_cb(user_data, error, ptr::null_mut(), 0, 0))
}

/// Get the current version of StructuredData by its handle
#[no_mangle]
pub unsafe extern "C" fn struct_data_version(session: *const Session,
                                             handle: StructDataHandle,
                                             user_data: *mut c_void,
                                             o_cb: extern "C" fn(*mut c_void, int32_t, uint64_t)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |_, object_cache, user_data| -> Option<Result<_, _>> {
            let mut object_cache = unwrap!(object_cache.lock());
            let sd = try_cb!(object_cache.get_sd(handle), |error| o_cb(user_data, error, 0));

            o_cb(user_data, 0, sd.get_version());
            None
        })
    }, |error| o_cb(user_data, error, 0))
}

/// PUT StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_put(session: *const Session,
                                         sd_h: StructDataHandle,
                                         user_data: *mut c_void,
                                         o_cb: extern "C" fn(*mut c_void, int32_t)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |client, object_cache, user_data| {
            let sd = {
                let mut object_cache = unwrap!(object_cache.lock());
                try_cb!(object_cache.get_sd(sd_h),
                        |error| o_cb(user_data, error)).clone()
            };

            client.put(Data::Structured(sd), None)
                .map(move |_| o_cb(user_data, 0))
                .map_err(move |err| {
                    let err = FfiError::from(err);
                    o_cb(user_data, ffi_error_code!(err));
                })
                .into()
        })
    }, |error| o_cb(user_data, error))
}

/// POST StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_post(session: *const Session,
                                          sd_h: StructDataHandle,
                                          user_data: *mut c_void,
                                          o_cb: extern "C" fn(*mut c_void, int32_t)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |client, object_cache, user_data| {
            let sd = {
                let mut object_cache = unwrap!(object_cache.lock());
                try_cb!(object_cache.get_sd(sd_h),
                        |error| o_cb(user_data, error)).clone()
            };

            client.post(Data::Structured(sd), None)
                .map(move |_| o_cb(user_data, 0))
                .map_err(move |err| {
                    let err = FfiError::from(err);
                    o_cb(user_data, ffi_error_code!(err));
                })
                .into()
        })
    }, |error| o_cb(user_data, error))
}

/// DELETE StructuredData. Version will be bumped.
#[no_mangle]
pub unsafe extern "C" fn struct_data_delete(session: *const Session,
                                            sd_h: StructDataHandle,
                                            user_data: *mut c_void,
                                            o_cb: extern "C" fn(*mut c_void, int32_t)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |client, object_cache, user_data| {
            let sd = {
                let mut object_cache = unwrap!(object_cache.lock());
                try_cb!(object_cache.remove_sd(sd_h),
                        |error| o_cb(user_data, error))
            };

            let sign_sk = try_cb!(client.secret_signing_key(),
                                  |error| o_cb(user_data, error));

            structured_data::delete(client, sd, &sign_sk)
                .map(move |_| o_cb(user_data, 0))
                .map_err(move |err| {
                    let err = FfiError::from(err);
                    o_cb(user_data, ffi_error_code!(err));
                })
                .into()
        })
    }, |error| o_cb(user_data, error))
}

/// See if StructuredData size is valid.
#[no_mangle]
pub unsafe extern "C" fn struct_data_validate_size(session: *const Session,
                                                   handle: StructDataHandle,
                                                   user_data: *mut c_void,
                                                   o_cb: extern "C" fn(*mut c_void, int32_t, bool)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |_, object_cache, user_data| -> Option<Result<_, _>> {
            let mut object_cache = unwrap!(object_cache.lock());
            let sd = try_cb!(object_cache.get_sd(handle), |error| o_cb(user_data, error, false));

            o_cb(user_data, 0, sd.validate_size());
            None
        })
    }, |error| o_cb(user_data, error, false))
}

/// Returns true if we are one of the owners of the provided StructuredData.
#[no_mangle]
pub unsafe extern "C" fn struct_data_is_owned(session: *const Session,
                                              sd_h: StructDataHandle,
                                              user_data: *mut c_void,
                                              o_cb: extern "C" fn(*mut c_void, int32_t, bool)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |client, object_cache, user_data| -> Option<Result<_, _>> {
            let mut object_cache = unwrap!(object_cache.lock());
            let sd = try_cb!(object_cache.get_sd(sd_h),
                             |error| o_cb(user_data, error, false));
            let my_key = try_cb!(client.public_signing_key(),
                                 |error| o_cb(user_data, error, false));

            o_cb(user_data, 0, sd.get_owner_keys().contains(&my_key));
            None
        })
    }, |error| o_cb(user_data, error, false))
}

/// Free StructuredData handle
#[no_mangle]
pub unsafe extern "C" fn struct_data_free(session: *const Session,
                                          handle: StructDataHandle,
                                          user_data: *mut c_void,
                                          o_cb: extern "C" fn(*mut c_void, int32_t)) {
    helper::catch_unwind_cb(|| {
        (*session).send_cb(user_data, move |_, object_cache, user_data| -> Option<Result<_, _>> {
            let mut object_cache = unwrap!(object_cache.lock());
            let _ = try_cb!(object_cache.remove_sd(handle), |error| o_cb(user_data, error));
            None
        })
    }, move |error| o_cb(user_data, error))
}

#[cfg(test)]
mod tests {
    /*
    use core::{CLIENT_STRUCTURED_DATA_TAG, utility};
    use ffi::app::App;
    use ffi::errors::FfiError;
    use ffi::low_level_api::{CipherOptHandle, DataIdHandle, StructDataHandle};
    use ffi::low_level_api::cipher_opt::*;
    use ffi::low_level_api::object_cache::object_cache;
    use ffi::test_utils;
    use rand;
    use std::ptr;
    use super::*;

    #[test]
    fn unversioned_struct_data_crud() {
        let app = test_utils::create_app(false);

        let mut cipher_opt_h: CipherOptHandle = 0;
        let mut sd_h: StructDataHandle = 0;
        let mut data_id_h: DataIdHandle = 0;
        let id = rand::random();
        let mut plain_text = unwrap!(utility::generate_random_vector::<u8>(10));
        unsafe {
            assert_eq!(cipher_opt_new_symmetric(&mut cipher_opt_h), 0);

            // Create
            assert_eq!(struct_data_new(&app,
                                       ::UNVERSIONED_STRUCT_DATA_TYPE_TAG,
                                       &id,
                                       0,
                                       cipher_opt_h,
                                       plain_text.as_ptr(),
                                       plain_text.len(),
                                       &mut sd_h),
                       0);
            assert_eq!(struct_data_extract_data_id(sd_h, &mut data_id_h), 0);

            // Put
            assert_eq!(struct_data_put(&app, sd_h), 0);
            let _ = unwrap!(object_cache()).get_sd(sd_h);
            assert_eq!(struct_data_free(sd_h), 0);
            assert!(unwrap!(object_cache()).get_sd(sd_h).is_err());

            // Fetch
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);
            let _ = unwrap!(object_cache()).get_sd(sd_h);

            // Extract Data
            let rx_plain_text_0 = extract_data(&app, sd_h);
            assert_eq!(rx_plain_text_0, plain_text);

            // New Data
            plain_text = unwrap!(utility::generate_random_vector::<u8>(10));
            assert_eq!(struct_data_new_data(&app,
                                            sd_h,
                                            cipher_opt_h,
                                            plain_text.as_ptr(),
                                            plain_text.len()),
                       0);

            // Post
            assert_eq!(struct_data_post(&app, sd_h), 0);
            let _ = unwrap!(object_cache()).get_sd(sd_h);
            assert_eq!(struct_data_free(sd_h), 0);
            assert!(unwrap!(object_cache()).get_sd(sd_h).is_err());

            // Fetch
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);
            let _ = unwrap!(object_cache()).get_sd(sd_h);

            // Extract Data
            let rx_plain_text_1 = extract_data(&app, sd_h);
            assert_eq!(rx_plain_text_1, plain_text);
            assert!(rx_plain_text_1 != rx_plain_text_0);


            // Perform Invalid Operations - should error out
            let mut versions = 0;
            assert_eq!(struct_data_num_of_versions(sd_h, &mut versions),
                       FfiError::InvalidStructuredDataTypeTag.into());
            {
                let mut data_ptr: *mut u8 = ptr::null_mut();
                let mut data_size = 0;
                let mut capacity = 0;
                assert_eq!(struct_data_nth_version(&app,
                                                   sd_h,
                                                   0,
                                                   &mut data_ptr,
                                                   &mut data_size,
                                                   &mut capacity),
                           FfiError::InvalidStructuredDataTypeTag.into());
            }

            // Check StructData owners
            let mut is_owned = false;
            assert_eq!(struct_data_is_owned(&app, sd_h, &mut is_owned), 0);
            assert_eq!(is_owned, true);

            let app_fake = test_utils::create_app(false);
            assert_eq!(struct_data_is_owned(&app_fake, sd_h, &mut is_owned), 0);
            assert_eq!(is_owned, false);

            // Delete
            assert_eq!(struct_data_delete(&app, sd_h), 0);
            let _ = unwrap!(object_cache()).get_sd(sd_h);

            // Re-delete shold fail - MutationError::NoSuchData; Fetch should be successful
            assert_eq!(struct_data_delete(&app, sd_h), -22);
            assert_eq!(struct_data_free(sd_h), 0);
            assert_eq!(struct_data_free(sd_h),
                       FfiError::InvalidStructDataHandle.into());
            assert!(unwrap!(object_cache()).get_sd(sd_h).is_err());

            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);
            assert_eq!(struct_data_free(sd_h), 0);
            assert_eq!(struct_data_free(sd_h),
                       FfiError::InvalidStructDataHandle.into());

            // Re-claim via PUT
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);
            let mut version = 0;
            assert_eq!(struct_data_version(sd_h, &mut version), 0);
            assert_eq!(struct_data_free(sd_h), 0);
            // Create
            assert_eq!(struct_data_new(&app,
                                       ::UNVERSIONED_STRUCT_DATA_TYPE_TAG,
                                       &id,
                                       version + 1,
                                       cipher_opt_h,
                                       plain_text.as_ptr(),
                                       plain_text.len(),
                                       &mut sd_h),
                       0);
            // Put - Reclaim
            assert_eq!(struct_data_put(&app, sd_h), 0);
        }
    }

    #[test]
    fn versioned_struct_data_crud() {
        let app = test_utils::create_app(false);

        let mut cipher_opt_h: CipherOptHandle = 0;
        let mut sd_h: StructDataHandle = 0;
        let mut data_id_h: DataIdHandle = 0;

        let name = rand::random();
        let data0 = unwrap!(utility::generate_random_vector(10));
        let data1 = unwrap!(utility::generate_random_vector(10));

        unsafe {
            assert_eq!(cipher_opt_new_symmetric(&mut cipher_opt_h), 0);

            // Create
            assert_eq!(struct_data_new(&app,
                                       ::VERSIONED_STRUCT_DATA_TYPE_TAG,
                                       &name,
                                       0,
                                       cipher_opt_h,
                                       data0.as_ptr(),
                                       data0.len(),
                                       &mut sd_h),
                       0);
            assert_eq!(struct_data_extract_data_id(sd_h, &mut data_id_h), 0);

            // Put and re-fetch
            assert_eq!(struct_data_put(&app, sd_h), 0);
            assert_eq!(struct_data_free(sd_h), 0);
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);

            // Check content
            let mut num_versions = 0usize;
            assert_eq!(struct_data_num_of_versions(sd_h, &mut num_versions), 0);
            assert_eq!(num_versions, 1);
            assert_eq!(nth_version(&app, sd_h, 0), data0);

            let mut version = 0;
            assert_eq!(struct_data_version(sd_h, &mut version), 0);
            assert_eq!(version, 0);

            assert_eq!(extract_data(&app, sd_h), data0);

            // Update the content
            assert_eq!(struct_data_new_data(&app, sd_h, cipher_opt_h, data1.as_ptr(), data1.len()),
                       0);

            // Post and re-fetch
            assert_eq!(struct_data_post(&app, sd_h), 0);
            assert_eq!(struct_data_free(sd_h), 0);
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);

            // Check content
            assert_eq!(struct_data_num_of_versions(sd_h, &mut num_versions), 0);
            assert_eq!(num_versions, 2);
            assert_eq!(nth_version(&app, sd_h, 0), data0);
            assert_eq!(nth_version(&app, sd_h, 1), data1);

            assert_eq!(extract_data(&app, sd_h), data1);

            let mut version = 0;
            assert_eq!(struct_data_version(sd_h, &mut version), 0);
            assert_eq!(version, 1);

            // Delete
            assert_eq!(struct_data_delete(&app, sd_h), 0);
            // -26 is CoreError::MutationFailure { reason: MutationError::InvalidOperation }
            assert_eq!(struct_data_delete(&app, sd_h), -26);
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);
        }
    }

    #[test]
    fn client_struct_data_crud() {
        let app = test_utils::create_app(false);

        let mut cipher_opt_h: CipherOptHandle = 0;
        let mut sd_h: StructDataHandle = 0;
        let mut data_id_h: DataIdHandle = 0;

        let name = rand::random();
        let data0 = unwrap!(utility::generate_random_vector(10));
        let data1 = unwrap!(utility::generate_random_vector(10));

        unsafe {
            assert_eq!(cipher_opt_new_symmetric(&mut cipher_opt_h), 0);

            // Invalid client tag
            assert_eq!(struct_data_new(&app,
                                       CLIENT_STRUCTURED_DATA_TAG - 1,
                                       &name,
                                       0,
                                       cipher_opt_h,
                                       data0.as_ptr(),
                                       data0.len(),
                                       &mut sd_h),
                       FfiError::InvalidStructuredDataTypeTag.into());

            // Create
            assert_eq!(struct_data_new(&app,
                                       CLIENT_STRUCTURED_DATA_TAG + 1,
                                       &name,
                                       0,
                                       cipher_opt_h,
                                       data0.as_ptr(),
                                       data0.len(),
                                       &mut sd_h),
                       0);
            assert_eq!(struct_data_extract_data_id(sd_h, &mut data_id_h), 0);

            // Put and re-fetch
            assert_eq!(struct_data_put(&app, sd_h), 0);
            assert_eq!(struct_data_free(sd_h), 0);
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);

            // Check content
            assert_eq!(extract_data(&app, sd_h), data0);

            // Update the content
            assert_eq!(struct_data_new_data(&app, sd_h, cipher_opt_h, data1.as_ptr(), data1.len()),
                       0);

            // Post and re-fetch
            assert_eq!(struct_data_post(&app, sd_h), 0);
            assert_eq!(struct_data_free(sd_h), 0);
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);

            // Check content
            assert_eq!(extract_data(&app, sd_h), data1);

            // Invalid operations
            let mut num_versions = 0;
            assert_eq!(struct_data_num_of_versions(sd_h, &mut num_versions),
                       FfiError::InvalidStructuredDataTypeTag.into());

            // Delete
            assert_eq!(struct_data_delete(&app, sd_h), 0);
            // -26 is CoreError::MutationFailure { reason: MutationError::InvalidOperation }
            assert_eq!(struct_data_delete(&app, sd_h), -26);
            assert_eq!(struct_data_fetch(&app, data_id_h, &mut sd_h), 0);
        }
    }

    // Helper function to fetch the current data from the structured data using FFI.
    fn extract_data(app: &App, sd_h: StructDataHandle) -> Vec<u8> {
        let mut data_ptr = ptr::null_mut();
        let mut data_size = 0usize;
        let mut data_cap = 0usize;

        unsafe {
            assert_eq!(struct_data_extract_data(app,
                                                sd_h,
                                                &mut data_ptr,
                                                &mut data_size,
                                                &mut data_cap),
                       0);

            Vec::from_raw_parts(data_ptr, data_size, data_cap)
        }
    }

    // Helper function to fetch the nth version from the structured data using FFI.
    fn nth_version(app: &App, sd_h: StructDataHandle, n: usize) -> Vec<u8> {
        let mut data_ptr = ptr::null_mut();
        let mut data_size = 0usize;
        let mut data_cap = 0usize;

        unsafe {
            assert_eq!(struct_data_nth_version(app,
                                               sd_h,
                                               n,
                                               &mut data_ptr,
                                               &mut data_size,
                                               &mut data_cap),
                       0);

            Vec::from_raw_parts(data_ptr, data_size, data_cap)
        }
    }

    */
}
