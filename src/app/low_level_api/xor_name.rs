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

use app::App;
use app::object_cache::XorNameHandle;
use routing::{XOR_NAME_LEN, XorName};
use std::os::raw::c_void;
use util::ffi::{self, OpaqueCtx};

/// Construct new `XorName`.
#[no_mangle]
pub unsafe extern "C" fn xor_name_new(app: *const App,
                                      id: *const [u8; XOR_NAME_LEN],
                                      user_data: *mut c_void,
                                      o_cb: unsafe extern "C" fn(*mut c_void, i32, XorNameHandle)) {
    ffi::catch_unwind_cb(user_data, o_cb, || {
        let xor_name = XorName(*id);
        let user_data = OpaqueCtx(user_data);

        (*app).send(move |_, context| {
            let handle = context.object_cache().insert_xor_name(xor_name);
            o_cb(user_data.0, 0, handle);
            None
        })
    })
}

/// Free `XorName` handle
#[no_mangle]
pub unsafe extern "C" fn xor_name_free(app: *const App,
                                       handle: XorNameHandle,
                                       user_data: *mut c_void,
                                       o_cb: unsafe extern "C" fn(*mut c_void, i32)) {
    let user_data = OpaqueCtx(user_data);

    ffi::catch_unwind_cb(user_data, o_cb, || {
        (*app).send(move |_, context| {
            let res = context.object_cache().remove_xor_name(handle);
            o_cb(user_data.0, ffi_result_code!(res));
            None
        })
    });
}


#[cfg(test)]
mod tests {
    use app::test_util::{create_app, run_now};
    use rand;
    use routing::{XOR_NAME_LEN, XorName};
    use super::*;
    use util::ffi::test_util::{call_0, call_1};

    #[test]
    fn create_and_free() {
        let session = create_app();
        let array: [u8; XOR_NAME_LEN] = rand::random();

        let handle = unsafe { unwrap!(call_1(|ud, cb| xor_name_new(&session, &array, ud, cb))) };

        run_now(&session, move |_, context| {
            assert_eq!(*unwrap!(context.object_cache().get_xor_name(handle)),
                       XorName(array));
        });

        unsafe { unwrap!(call_0(|ud, cb| xor_name_free(&session, handle, ud, cb))) }

        run_now(&session, move |_, context| {
            assert!(context.object_cache().get_xor_name(handle).is_err());
        });
    }
}
