// Copyright 2017 MaidSafe.net limited.
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

#![allow(unsafe_code)]

use App;
use errors::AppError;
use ffi_utils::{FFI_RESULT_OK, FfiResult, ReprC, catch_unwind_cb, from_c_str};
use safe_core::ffi::ipc::req::AuthReq;
use safe_core::ipc::req::AuthReq as NativeAuthReq;
use std::os::raw::{c_char, c_void};
use test_utils::{create_app_by_req, create_auth_req};

/// Creates a random app instance for testing.
#[no_mangle]
#[allow(unsafe_code)]
pub unsafe extern "C" fn test_create_app(
    app_id: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void,
                        result: *const FfiResult,
                        app: *mut App),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<(), AppError> {
        let app_id = from_c_str(app_id)?;
        let auth_req = create_auth_req(Some(app_id), None);
        let app = create_app_by_req(&auth_req);
        o_cb(user_data, FFI_RESULT_OK, Box::into_raw(Box::new(app)));
        Ok(())
    })
}

/// Create a random app instance for testing, with access to containers.
#[no_mangle]
#[allow(unsafe_code)]
pub unsafe extern "C" fn test_create_app_with_access(
    auth_req: *const AuthReq,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void,
                        result: *const FfiResult,
                        o_app: *mut App),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<(), AppError> {
        let auth_req = NativeAuthReq::clone_from_repr_c(auth_req)?;
        let app = create_app_by_req(&auth_req);
        o_cb(user_data, FFI_RESULT_OK, Box::into_raw(Box::new(app)));
        Ok(())
    })
}
