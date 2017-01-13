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

use App;
use ffi::helper::send_sync;
use ffi_utils::catch_unwind_cb;
use object_cache::{EncryptKeyHandle, SignKeyHandle};
use rust_sodium::crypto::{box_, sign};
use std::os::raw::c_void;

/// Get the public signing key of the app.
#[no_mangle]
pub unsafe extern "C" fn app_pub_sign_key(app: *const App,
                                          user_data: *mut c_void,
                                          o_cb: extern "C" fn(*mut c_void, i32, SignKeyHandle)) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |client, context| {
            let key = client.public_signing_key()?;
            Ok(context.object_cache().insert_sign_key(key))
        })
    })
}

/// Create new public signing key from raw array.
#[no_mangle]
pub unsafe extern "C" fn sign_key_new(app: *const App,
                                      data: *const [u8; sign::PUBLICKEYBYTES],
                                      user_data: *mut c_void,
                                      o_cb: extern "C" fn(*mut c_void, i32, SignKeyHandle)) {
    catch_unwind_cb(user_data, o_cb, || {
        let key = sign::PublicKey(*data);
        send_sync(app,
                  user_data,
                  o_cb,
                  move |_, context| Ok(context.object_cache().insert_sign_key(key)))
    })
}

/// Retrieve the public signing key as raw array.
#[no_mangle]
pub unsafe extern "C" fn sign_key_get(app: *const App,
                                      handle: SignKeyHandle,
                                      user_data: *mut c_void,
                                      o_cb: extern "C" fn(*mut c_void,
                                                          i32,
                                                          *const [u8; sign::PUBLICKEYBYTES])) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |_, context| {
            let key = context.object_cache().get_sign_key(handle)?;
            Ok(&key.0)
        })
    })
}

/// Free signing key from memory
#[no_mangle]
pub unsafe extern "C" fn sign_key_free(app: *const App,
                                       handle: SignKeyHandle,
                                       user_data: *mut c_void,
                                       o_cb: extern "C" fn(*mut c_void, i32)) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |_, context| {
            let _ = context.object_cache().remove_sign_key(handle)?;
            Ok(())
        })
    })
}

/// Get the public encryption key of the app.
#[no_mangle]
pub unsafe extern "C" fn app_pub_enc_key(app: *const App,
                                         user_data: *mut c_void,
                                         o_cb: extern "C" fn(*mut c_void, i32, EncryptKeyHandle)) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |client, context| {
            let key = client.public_encryption_key()?;
            Ok(context.object_cache().insert_encrypt_key(key))
        })
    })
}

/// Create new public encryption key from raw array.
#[no_mangle]
pub unsafe extern "C" fn enc_key_new(app: *const App,
                                     data: *const [u8; box_::PUBLICKEYBYTES],
                                     user_data: *mut c_void,
                                     o_cb: extern "C" fn(*mut c_void, i32, EncryptKeyHandle)) {
    catch_unwind_cb(user_data, o_cb, || {
        let key = box_::PublicKey(*data);
        send_sync(app,
                  user_data,
                  o_cb,
                  move |_, context| Ok(context.object_cache().insert_encrypt_key(key)))
    })
}

/// Retrieve the public encryption key as raw array.
#[no_mangle]
pub unsafe extern "C" fn enc_key_get(app: *const App,
                                     handle: EncryptKeyHandle,
                                     user_data: *mut c_void,
                                     o_cb: extern "C" fn(*mut c_void,
                                                         i32,
                                                         *const [u8; box_::PUBLICKEYBYTES])) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |_, context| {
            let key = context.object_cache().get_encrypt_key(handle)?;
            Ok(&key.0)
        })
    })
}

/// Free encryption key from memory
#[no_mangle]
pub unsafe extern "C" fn enc_key_free(app: *const App,
                                      handle: EncryptKeyHandle,
                                      user_data: *mut c_void,
                                      o_cb: extern "C" fn(*mut c_void, i32)) {
    catch_unwind_cb(user_data, o_cb, || {
        send_sync(app, user_data, o_cb, move |_, context| {
            let _ = context.object_cache().remove_encrypt_key(handle)?;
            Ok(())
        })
    })
}

/*
/// Reset the object cache (drop all objects stored in it). This will invalidate
/// all currently held object handles.
#[no_mangle]
pub unsafe extern "C" fn misc_object_cache_reset(session: *const Session,
                                                 user_data: *mut c_void,
                                                 o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    let user_data = OpaqueCtx(user_data);

    helper::catch_unwind_cb(user_data, o_cb, || {
        (*session).send(move |_, obj_cache| {
            obj_cache.reset();
            o_cb(user_data.0, 0);
            None
        })
    })
}
*/

#[cfg(test)]
mod tests {
    use ffi_utils::test_utils::call_1;
    use routing::XOR_NAME_LEN;
    use super::*;
    use test_utils::{create_app, run_now};

    #[test]
    fn sign_key_basics() {
        let app = create_app();
        let app_sign_key1_h = unsafe { unwrap!(call_1(|ud, cb| app_pub_sign_key(&app, ud, cb))) };

        let app_sign_key1 = run_now(&app, move |client, context| {
            let app_sign_key1 = unwrap!(client.public_signing_key());
            let app_sign_key2 = unwrap!(context.object_cache().get_sign_key(app_sign_key1_h));
            assert_eq!(app_sign_key1, *app_sign_key2);

            app_sign_key1
        });

        let app_sign_key1_raw: [u8; XOR_NAME_LEN] =
            unsafe { unwrap!(call_1(|ud, cb| sign_key_get(&app, app_sign_key1_h, ud, cb))) };

        let app_sign_key2_h =
            unsafe { unwrap!(call_1(|ud, cb| sign_key_new(&app, &app_sign_key1_raw, ud, cb))) };

        let app_sign_key2 = run_now(&app, move |_, context| {
            *unwrap!(context.object_cache().get_sign_key(app_sign_key2_h))
        });

        assert_eq!(app_sign_key1, app_sign_key2);
    }

    #[test]
    fn enc_key_basics() {
        let app = create_app();
        let app_enc_key1_h = unsafe { unwrap!(call_1(|ud, cb| app_pub_enc_key(&app, ud, cb))) };

        let app_enc_key1 = run_now(&app, move |client, context| {
            let app_enc_key1 = unwrap!(client.public_encryption_key());
            let app_enc_key2 = unwrap!(context.object_cache().get_encrypt_key(app_enc_key1_h));
            assert_eq!(app_enc_key1, *app_enc_key2);

            app_enc_key1
        });

        let app_enc_key1_raw: [u8; XOR_NAME_LEN] =
            unsafe { unwrap!(call_1(|ud, cb| enc_key_get(&app, app_enc_key1_h, ud, cb))) };

        let app_enc_key2_h =
            unsafe { unwrap!(call_1(|ud, cb| enc_key_new(&app, &app_enc_key1_raw, ud, cb))) };

        let app_enc_key2 = run_now(&app, move |_, context| {
            *unwrap!(context.object_cache().get_encrypt_key(app_enc_key2_h))
        });

        assert_eq!(app_enc_key1, app_enc_key2);
    }
}
