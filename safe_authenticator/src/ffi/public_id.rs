// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use AuthError;
use Authenticator;
use ffi_utils::{FFI_RESULT_OK, FfiResult, OpaqueCtx, catch_unwind_cb, from_c_str};
use futures::Future;
use public_id;
use safe_core::FutureExt;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;

/// Create Public ID.
#[no_mangle]
pub unsafe extern "C" fn auth_public_id_create(auth: *const Authenticator,
                                               public_id: *const c_char,
                                               user_data: *mut c_void,
                                               o_cb: extern "C" fn(*mut c_void, FfiResult)) {
    catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);
        let public_id = from_c_str(public_id)?;

        (*auth).send(move |client| {
            public_id::create(client, public_id)
                .then(move |res| {
                    let (error_code, description) = ffi_result!(res);
                    o_cb(user_data.0,
                         FfiResult {
                             error_code,
                             description: description.as_ptr(),
                         });
                    Ok(())
                })
                .into_box()
                .into()
        })
    })
}

/// Retrieve the Public ID.
#[no_mangle]
pub unsafe extern "C" fn auth_public_id(auth: *const Authenticator,
                                        user_data: *mut c_void,
                                        o_cb: extern "C" fn(*mut c_void,
                                                            FfiResult,
                                                            *const c_char)) {
    catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);

        (*auth).send(move |client| {
            public_id::get(client)
                .map(move |public_id| {
                    let c_str = match CString::new(public_id) {
                        Ok(c_str) => c_str,
                        Err(e) => {
                            let e = AuthError::from(e);
                            let (error_code, description) = ffi_error!(e);
                            return o_cb(user_data.0,
                                        FfiResult {
                                            error_code,
                                            description: description.as_ptr(),
                                        },
                                        ptr::null());
                        }
                    };
                    o_cb(user_data.0, FFI_RESULT_OK, c_str.as_ptr());
                })
                .map_err(move |err| {
                    let (error_code, description) = ffi_error!(err);
                    o_cb(user_data.0,
                         FfiResult {
                             error_code,
                             description: description.as_ptr(),
                         },
                         ptr::null())
                })
                .into_box()
                .into()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use errors::{ERR_NO_SUCH_PUBLIC_ID, ERR_PUBLIC_ID_EXISTS};

    use ffi_utils::test_utils::{call_0, call_1};
    use safe_core::utils;
    use std::ffi::CString;
    use test_utils::create_authenticator;

    #[test]
    fn create() {
        let authenticator = create_authenticator();
        let public_id = unwrap!(utils::generate_random_string(10));
        let ffi_public_id = unwrap!(CString::new(public_id));

        // Create public id first time succeeds.
        unsafe {
            unwrap!(call_0(|ud, cb| {
                               auth_public_id_create(&authenticator, ffi_public_id.as_ptr(), ud, cb)
                           }))
        }

        // Attempt to create already existing public id fails.
        let res = unsafe {
            call_0(|ud, cb| auth_public_id_create(&authenticator, ffi_public_id.as_ptr(), ud, cb))
        };

        match res {
            Err(code) if code == ERR_PUBLIC_ID_EXISTS => (),
            Err(err) => panic!("Unexpected {:?}", err),
            Ok(_) => panic!("Unexpected success"),
        }
    }

    #[test]
    fn get() {
        let authenticator = create_authenticator();

        // There is no Public ID yet, so attempt to retrieve it fails.
        let res: Result<String, _> =
            unsafe { call_1(|ud, cb| auth_public_id(&authenticator, ud, cb)) };

        match res {
            Err(code) if code == ERR_NO_SUCH_PUBLIC_ID => (),
            Err(err) => panic!("Unexpected {:?}", err),
            Ok(_) => panic!("Unexpected success"),
        }

        // Create public ID.
        let public_id = unwrap!(utils::generate_random_string(10));
        let ffi_public_id = unwrap!(CString::new(public_id.clone()));

        unsafe {
            unwrap!(call_0(|ud, cb| {
                               auth_public_id_create(&authenticator, ffi_public_id.as_ptr(), ud, cb)
                           }))
        }

        // Now retrieving it succeeds.
        let retrieved_public_id: String =
            unsafe { unwrap!(call_1(|ud, cb| auth_public_id(&authenticator, ud, cb))) };

        assert_eq!(retrieved_public_id, public_id);
    }
}
