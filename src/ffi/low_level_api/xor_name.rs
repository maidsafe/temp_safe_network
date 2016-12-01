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

use ffi::{OpaqueCtx, Session, XorNameHandle, helper};
use routing::{XOR_NAME_LEN, XorName};
use std::os::raw::c_void;

/// Construct new `XorName`.
#[no_mangle]
pub unsafe extern "C" fn xor_name_new(session: *const Session,
                                      id: *const [u8; XOR_NAME_LEN],
                                      user_data: *mut c_void,
                                      o_cb: unsafe extern "C" fn(*mut c_void, i32, XorNameHandle)) {
    helper::catch_unwind_cb(user_data, o_cb, || {
        let xor_name = XorName(*id);
        let user_data = OpaqueCtx(user_data);

        (*session).send(move |_, obj_cache| {
            let handle = obj_cache.insert_xor_name(xor_name);
            o_cb(user_data.0, 0, handle);
            None
        })
    })
}

/// Free `XorName` handle
#[no_mangle]
pub unsafe extern "C" fn xor_name_free(session: *const Session,
                                       handle: XorNameHandle,
                                       user_data: *mut c_void,
                                       o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    let user_data = OpaqueCtx(user_data);

    helper::catch_unwind_cb(user_data, o_cb, || {
        (*session).send(move |_, obj_cache| {
            let res = obj_cache.remove_xor_name(handle);
            o_cb(user_data.0, ffi_result_code!(res));
            None
        })
    });
}


#[cfg(test)]
mod tests {
    use ffi::test_utils;
    use rand;
    use routing::{XOR_NAME_LEN, XorName};
    use super::*;

    #[test]
    fn create_and_free() {
        let session = test_utils::create_session();
        let array: [u8; XOR_NAME_LEN] = rand::random();

        let handle =
            unsafe { unwrap!(test_utils::call_1(|ud, cb| xor_name_new(&session, &array, ud, cb))) };

        test_utils::run_now(&session, move |_, obj_cache| {
            assert_eq!(*unwrap!(obj_cache.get_xor_name(handle)), XorName(array));
        });

        unsafe { unwrap!(test_utils::call_0(|ud, cb| xor_name_free(&session, handle, ud, cb))) }

        test_utils::run_now(&session, move |_, obj_cache| {
            assert!(obj_cache.get_xor_name(handle).is_err());
        });
    }
}
