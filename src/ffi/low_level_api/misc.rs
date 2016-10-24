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

use core::CoreMsg;
use ffi::{FfiError, OpaqueCtx, Session, helper};
use ffi::object_cache::{DataIdHandle, EncryptKeyHandle, SignKeyHandle};
use libc::{c_void, int32_t, size_t, uint8_t};
use maidsafe_utilities::serialisation::{deserialise, serialise};
use std::{mem, ptr, slice};

/// Free Encrypt Key handle
#[no_mangle]
pub unsafe extern "C" fn misc_encrypt_key_free(session: *const Session,
                                               user_data: *mut c_void,
                                               handle: EncryptKeyHandle,
                                               o_cb: unsafe extern "C" fn(*mut c_void, int32_t))
                                               -> i32 {
    helper::catch_unwind_i32(|| {
        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            let res = obj_cache.remove_encrypt_key(handle);
            o_cb(user_data.0, ffi_result_code!(res));
            None
        })));
        0
    })
}

/// Free Sign Key handle
#[no_mangle]
pub unsafe extern "C" fn misc_sign_key_free(session: *const Session,
                                            handle: SignKeyHandle,
                                            user_data: *mut c_void,
                                            o_cb: unsafe extern "C" fn(*mut c_void, int32_t))
                                            -> i32 {
    helper::catch_unwind_i32(|| {
        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            let res = obj_cache.remove_sign_key(handle);
            o_cb(user_data.0, ffi_result_code!(res));
            None
        })));
        0
    })
}

/// Serialise DataIdentifier
/// Callback arguments are (error_code, user_data, data, size, capacity)
#[no_mangle]
pub unsafe extern "C" fn misc_serialise_data_id(session: *const Session,
                                                data_id_h: DataIdHandle,
                                                user_data: *mut c_void,
                                                o_cb: unsafe extern "C" fn(*mut c_void,
                                                                           int32_t,
                                                                           *mut uint8_t,
                                                                           size_t,
                                                                           size_t))
                                                -> i32 {
    helper::catch_unwind_i32(|| {
        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            let data_id = match obj_cache.get_data_id(data_id_h) {
                Ok(data_id) => data_id,
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e), ptr::null_mut(), 0, 0);
                    return None;
                }
            };

            let mut ser_data_id = match serialise(data_id).map_err(FfiError::from) {
                Ok(ser_data_id) => ser_data_id,
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e), ptr::null_mut(), 0, 0);
                    return None;
                }
            };

            let data = ser_data_id.as_mut_ptr();
            let size = ser_data_id.len();
            let capacity = ser_data_id.capacity();
            o_cb(user_data.0, 0, data, size, capacity);

            mem::forget(ser_data_id);
            None
        })));

        0
    })
}

/// Deserialise DataIdentifier
#[no_mangle]
pub unsafe extern "C" fn misc_deserialise_data_id(session: *const Session,
                                                  data: *const u8,
                                                  size: usize,
                                                  user_data: *mut c_void,
                                                  o_cb: unsafe extern "C" fn(*mut c_void,
                                                                             int32_t,
                                                                             DataIdHandle))
                                                  -> i32 {
    helper::catch_unwind_i32(|| {
        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);
        let data = OpaqueCtx(data as *mut _);

        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let mut obj_cache = unwrap!(obj_cache.lock());

            let data: *const u8 = data.0 as *const _;
            let ser_data_id = slice::from_raw_parts(data, size);
            let data_id = match deserialise(ser_data_id).map_err(FfiError::from) {
                Ok(data_id) => data_id,
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e), 0);
                    return None;
                }
            };

            let handle = obj_cache.insert_data_id(data_id);
            o_cb(user_data.0, 0, handle);
            None
        })));

        0
    })
}

/// Deallocate pointer obtained via FFI and allocated by safe_core
#[no_mangle]
pub unsafe extern "C" fn misc_u8_ptr_free(ptr: *mut u8, size: usize, capacity: usize) {
    // TODO: refactor implementation to remove the need for `cap`. Related issue:
    // <https://github.com/rust-lang/rust/issues/36284>.
    let _ = Vec::from_raw_parts(ptr, size, capacity);
}

/// Reset the object cache (drop all objects stored in it). This will invalidate
/// all currently held object handles.
#[no_mangle]
pub unsafe extern "C" fn misc_object_cache_reset(session: *const Session,
                                                 user_data: *mut c_void,
                                                 o_cb: unsafe extern "C" fn(*mut c_void, int32_t))
                                                 -> i32 {
    let obj_cache = (*session).object_cache();
    let user_data = OpaqueCtx(user_data);

    ffi_try!((*session).send(CoreMsg::new(move |_| {
        let mut object_cache = unwrap!(obj_cache.lock());
        object_cache.reset();
        o_cb(user_data.0, 0);
        None
    })));

    0
}

#[cfg(test)]
mod tests {
    use ffi::low_level_api::data_id::*;
    use ffi::object_cache::DataIdHandle;
    use ffi::test_utils;
    use libc::c_void;
    use rand;
    use routing::DataIdentifier;
    use std::hash::{Hash, Hasher, SipHasher};
    use std::sync::mpsc;
    use std::time::Duration;
    use super::*;

    #[test]
    fn data_id_serialisation() {
        let sess = test_utils::create_session();
        let obj_cache = sess.object_cache();
        let sess_ptr = Box::into_raw(Box::new(sess));

        let data_id_sd = DataIdentifier::Structured(rand::random(), rand::random());
        let data_id_id = DataIdentifier::Immutable(rand::random());
        let data_id_ad = DataIdentifier::PrivAppendable(rand::random());
        assert!(data_id_sd != data_id_id);
        assert!(data_id_sd != data_id_ad);
        assert!(data_id_ad != data_id_id);

        let (sd_data_id_h, id_data_id_h, ad_data_id_h) = {
            let mut object_cache = unwrap!(obj_cache.lock());

            (object_cache.insert_data_id(data_id_sd),
             object_cache.insert_data_id(data_id_id),
             object_cache.insert_data_id(data_id_ad))
        };

        unsafe {
            let (tx, rx) = mpsc::channel::<(*mut u8, usize, usize)>();
            assert_eq!(misc_serialise_data_id(sess_ptr,
                                              sd_data_id_h,
                                              Box::into_raw(Box::new(tx.clone())) as *mut _,
                                              serialise_cb),
                       0);
            let (data_ptr, data_size, capacity) = unwrap!(rx.recv_timeout(Duration::from_secs(15)));

            let (tx, rx) = mpsc::channel::<DataIdHandle>();
            assert_eq!(misc_deserialise_data_id(sess_ptr,
                                                data_ptr,
                                                data_size,
                                                Box::into_raw(Box::new(tx.clone())) as *mut _,
                                                deserialise_cb),
                       0);
            let data_id_h = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
            assert!(data_id_h != sd_data_id_h);

            {
                let mut object_cache = unwrap!(obj_cache.lock());
                let before_id = *unwrap!(object_cache.get_data_id(sd_data_id_h));
                let after_id = unwrap!(object_cache.get_data_id(data_id_h));

                assert_eq!(before_id, *after_id);
                assert_eq!(data_id_sd, *after_id);
            }

            let (tx, rx) = mpsc::channel::<()>();
            assert_eq!(data_id_free(sess_ptr,
                                    data_id_h,
                                    Box::into_raw(Box::new(tx.clone())) as *mut _,
                                    free_cb),
                       0);
            let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));

            misc_u8_ptr_free(data_ptr, data_size, capacity);
        }

        unsafe {
            let (tx, rx) = mpsc::channel::<(*mut u8, usize, usize)>();
            assert_eq!(misc_serialise_data_id(sess_ptr,
                                              id_data_id_h,
                                              Box::into_raw(Box::new(tx.clone())) as *mut _,
                                              serialise_cb),
                       0);
            let (data_ptr, data_size, capacity) = unwrap!(rx.recv_timeout(Duration::from_secs(15)));

            let (tx, rx) = mpsc::channel::<DataIdHandle>();
            assert_eq!(misc_deserialise_data_id(sess_ptr,
                                                data_ptr,
                                                data_size,
                                                Box::into_raw(Box::new(tx.clone())) as *mut _,
                                                deserialise_cb),
                       0);
            let data_id_h = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
            assert!(data_id_h != id_data_id_h);

            {
                let mut object_cache = unwrap!(obj_cache.lock());
                let before_id = *unwrap!(object_cache.get_data_id(id_data_id_h));
                let after_id = unwrap!(object_cache.get_data_id(data_id_h));

                assert_eq!(before_id, *after_id);
                assert_eq!(data_id_id, *after_id);
            }

            let (tx, rx) = mpsc::channel::<()>();
            assert_eq!(data_id_free(sess_ptr,
                                    data_id_h,
                                    Box::into_raw(Box::new(tx.clone())) as *mut _,
                                    free_cb),
                       0);
            let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));

            misc_u8_ptr_free(data_ptr, data_size, capacity);
        }

        unsafe {
            let (tx, rx) = mpsc::channel::<(*mut u8, usize, usize)>();
            assert_eq!(misc_serialise_data_id(sess_ptr,
                                              ad_data_id_h,
                                              Box::into_raw(Box::new(tx.clone())) as *mut _,
                                              serialise_cb),
                       0);
            let (data_ptr, data_size, capacity) = unwrap!(rx.recv_timeout(Duration::from_secs(15)));

            let (tx, rx) = mpsc::channel::<DataIdHandle>();
            assert_eq!(misc_deserialise_data_id(sess_ptr,
                                                data_ptr,
                                                data_size,
                                                Box::into_raw(Box::new(tx.clone())) as *mut _,
                                                deserialise_cb),
                       0);
            let data_id_h = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
            assert!(data_id_h != ad_data_id_h);

            {
                let mut object_cache = unwrap!(obj_cache.lock());
                let before_id = *unwrap!(object_cache.get_data_id(ad_data_id_h));
                let after_id = unwrap!(object_cache.get_data_id(data_id_h));

                assert_eq!(before_id, *after_id);
                assert_eq!(data_id_ad, *after_id);
            }

            let (tx, rx) = mpsc::channel::<()>();
            assert_eq!(data_id_free(sess_ptr,
                                    data_id_h,
                                    Box::into_raw(Box::new(tx.clone())) as *mut _,
                                    free_cb),
                       0);
            let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));

            misc_u8_ptr_free(data_ptr, data_size, capacity);
        }

        unsafe {
            let (tx, rx) = mpsc::channel::<()>();
            let tx_ptr = Box::into_raw(Box::new(tx.clone())) as *mut _;

            assert_eq!(data_id_free(sess_ptr, sd_data_id_h, tx_ptr, free_cb), 0);
            let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));

            assert_eq!(data_id_free(sess_ptr, id_data_id_h, tx_ptr, free_cb), 0);
            let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));

            assert_eq!(data_id_free(sess_ptr, ad_data_id_h, tx_ptr, free_cb), 0);
            let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
        }

        unsafe extern "C" fn serialise_cb(tx: *mut c_void,
                                          err_code: i32,
                                          data: *mut u8,
                                          size: usize,
                                          cap: usize) {
            assert_eq!(err_code, 0);

            let tx: *mut mpsc::Sender<(*mut u8, usize, usize)> = tx as *mut _;
            unwrap!((*tx).send((data, size, cap)));
        }

        unsafe extern "C" fn deserialise_cb(tx: *mut c_void, err_code: i32, handle: DataIdHandle) {
            assert_eq!(err_code, 0);

            let tx: *mut mpsc::Sender<DataIdHandle> = tx as *mut _;
            unwrap!((*tx).send(handle));
        }

        unsafe extern "C" fn free_cb(tx: *mut c_void, err_code: i32) {
            assert_eq!(err_code, 0);

            let tx: *mut mpsc::Sender<()> = tx as *mut _;
            unwrap!((*tx).send(()));
        }
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = SipHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}
