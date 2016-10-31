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
use ffi::{OpaqueCtx, Session, helper};
use ffi::object_cache::DataIdHandle;
use libc::{c_void, int32_t};
use routing::{DataIdentifier, XOR_NAME_LEN, XorName};

/// Construct DataIdentifier for StructuredData.
#[no_mangle]
pub unsafe extern "C" fn data_id_new_struct_data(session: *const Session,
                                                 type_tag: u64,
                                                 id: *const [u8; XOR_NAME_LEN],
                                                 user_data: *mut c_void,
                                                 o_cb: unsafe extern "C" fn(*mut c_void,
                                                                            int32_t,
                                                                            DataIdHandle))
                                                 -> i32 {
    helper::catch_unwind_i32(|| {
        let xor_id = XorName(*id);
        let data_id = DataIdentifier::Structured(xor_id, type_tag);

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let handle = unwrap!(obj_cache.lock()).insert_data_id(data_id);
            o_cb(user_data.0, 0, handle);
            None
        })));
        0
    })
}

/// Construct DataIdentifier for ImmutableData.
#[no_mangle]
pub unsafe extern "C" fn data_id_new_immut_data(session: *const Session,
                                                id: *const [u8; XOR_NAME_LEN],
                                                user_data: *mut c_void,
                                                o_cb: unsafe extern "C" fn(*mut c_void,
                                                                           int32_t,
                                                                           DataIdHandle))
                                                -> i32 {
    helper::catch_unwind_i32(|| {
        let xor_id = XorName(*id);
        let data_id = DataIdentifier::Immutable(xor_id);

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let handle = unwrap!(obj_cache.lock()).insert_data_id(data_id);
            o_cb(user_data.0, 0, handle);
            None
        })));
        0
    })
}

/// Construct DataIdentifier for AppendableData.
#[no_mangle]
pub unsafe extern "C" fn data_id_new_appendable_data(session: *const Session,
                                                     id: *const [u8; XOR_NAME_LEN],
                                                     is_private: bool,
                                                     user_data: *mut c_void,
                                                     o_cb: unsafe extern "C" fn(*mut c_void,
                                                                                int32_t,
                                                                                DataIdHandle))
                                                     -> i32 {
    helper::catch_unwind_i32(|| {
        let xor_id = XorName(*id);
        let data_id = if is_private {
            DataIdentifier::PrivAppendable(xor_id)
        } else {
            DataIdentifier::PubAppendable(xor_id)
        };

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let handle = unwrap!(obj_cache.lock()).insert_data_id(data_id);
            o_cb(user_data.0, 0, handle);
            None
        })));
        0
    })
}

/// Free DataIdentifier handle
#[no_mangle]
pub unsafe extern "C" fn data_id_free(session: *const Session,
                                      handle: DataIdHandle,
                                      user_data: *mut c_void,
                                      o_cb: unsafe extern "C" fn(*mut c_void, int32_t)) {
    let user_data = OpaqueCtx(user_data);

    helper::catch_unwind_cb(|| {
        let obj_cache = (*session).object_cache();

        (*session).send(CoreMsg::new(move |_| {
            let res = unwrap!(obj_cache.lock()).remove_data_id(handle);
            o_cb(user_data.0, ffi_result_code!(res));
            None
        }))
    },
                            move |err| o_cb(user_data.0, ffi_error_code!(err)));
}

#[cfg(test)]
mod tests {
    use ffi::{FfiError, Session, test_utils};
    use ffi::object_cache::DataIdHandle;
    use libc::c_void;
    use rand;
    use routing::XOR_NAME_LEN;
    use std::sync::mpsc;
    use super::*;

    #[test]
    fn create_and_free() {
        let sess = test_utils::create_session();
        let obj_cache = sess.object_cache();
        let sess_ptr: *const _ = &sess;

        let type_tag = rand::random();
        let struct_id_arr: [u8; XOR_NAME_LEN] = rand::random();

        let immut_id_arr: [u8; XOR_NAME_LEN] = rand::random();

        let priv_app_id_arr: [u8; XOR_NAME_LEN] = rand::random();
        let pub_app_id_arr: [u8; XOR_NAME_LEN] = rand::random();

        let data_id_handle_immut;
        let data_id_handle_struct;
        let data_id_handle_priv_appendable;
        let data_id_handle_pub_appendable;

        unsafe {
            let (mut tx, rx) = mpsc::channel::<DataIdHandle>();
            let tx: *mut _ = &mut tx;
            let tx = tx as *mut c_void;

            assert_eq!(data_id_new_struct_data(sess_ptr, type_tag, &struct_id_arr, tx, data_id_cb),
                       0);
            data_id_handle_struct = unwrap!(rx.recv());

            assert_eq!(data_id_new_immut_data(sess_ptr, &immut_id_arr, tx, data_id_cb),
                       0);
            data_id_handle_immut = unwrap!(rx.recv());

            assert_eq!(data_id_new_appendable_data(sess_ptr,
                                                   &priv_app_id_arr,
                                                   true,
                                                   tx,
                                                   data_id_cb),
                       0);
            data_id_handle_priv_appendable = unwrap!(rx.recv());

            assert_eq!(data_id_new_appendable_data(sess_ptr,
                                                   &pub_app_id_arr,
                                                   false,
                                                   tx,
                                                   data_id_cb),
                       0);
            data_id_handle_pub_appendable = unwrap!(rx.recv());
        }

        {
            let mut obj_cache = unwrap!(obj_cache.lock());
            let _ = unwrap!(obj_cache.get_data_id(data_id_handle_struct));
            let _ = unwrap!(obj_cache.get_data_id(data_id_handle_immut));
            let _ = unwrap!(obj_cache.get_data_id(data_id_handle_priv_appendable));
            let _ = unwrap!(obj_cache.get_data_id(data_id_handle_pub_appendable));
        }

        unsafe {
            assert_free(sess_ptr, data_id_handle_struct, 0);
            assert_free(sess_ptr, data_id_handle_immut, 0);
            assert_free(sess_ptr, data_id_handle_priv_appendable, 0);
            assert_free(sess_ptr, data_id_handle_pub_appendable, 0);
        }

        let err_code = FfiError::InvalidDataIdHandle.into();
        unsafe {
            assert_free(sess_ptr, data_id_handle_struct, err_code);
            assert_free(sess_ptr, data_id_handle_immut, err_code);
            assert_free(sess_ptr, data_id_handle_priv_appendable, err_code);
            assert_free(sess_ptr, data_id_handle_pub_appendable, err_code);
        }

        {
            let mut obj_cache = unwrap!(obj_cache.lock());
            assert!(obj_cache.get_data_id(data_id_handle_struct).is_err());
            assert!(obj_cache.get_data_id(data_id_handle_immut).is_err());
            assert!(obj_cache.get_data_id(data_id_handle_priv_appendable).is_err());
            assert!(obj_cache.get_data_id(data_id_handle_pub_appendable).is_err());
        }

        unsafe extern "C" fn data_id_cb(tx: *mut c_void, errcode: i32, handle: DataIdHandle) {
            assert_eq!(errcode, 0);

            let tx = tx as *mut mpsc::Sender<DataIdHandle>;
            unwrap!((*tx).send(handle));
        }

        unsafe fn assert_free(sess: *const Session, handle: DataIdHandle, expected: i32) {
            let res = test_utils::call_0(|user_data, cb| data_id_free(sess, handle, user_data, cb));
            match res {
                Ok(()) => assert_eq!(expected, 0),
                Err(code) => assert_eq!(expected, code),
            }
        }
    }
}
