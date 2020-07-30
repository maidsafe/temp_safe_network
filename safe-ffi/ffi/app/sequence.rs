// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{common::errors::Result, ffi_structs::XorNameArray};
use ffi_utils::{
    catch_unwind_cb, vec_clone_from_raw_parts, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK,
};
use safe_api::Safe;
use std::{
    ffi::CString,
    os::raw::{c_char, c_void},
    time::Duration,
};
use tokio::runtime::Runtime;
use xor_name::XorName;

#[no_mangle]
pub unsafe extern "C" fn create_sequence(
    app: *mut Safe,
    data: *const u8,
    data_len: usize,
    name: *const XorNameArray,
    type_tag: u64,
    is_private: bool,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, xorurl: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let name = if name.is_null() {
            None
        } else {
            Some(XorName(*name))
        };
        let data_vec = vec_clone_from_raw_parts(data, data_len);
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let xorurl =
            runtime.block_on((*app).sequence_create(&data_vec, name, type_tag, is_private))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        let xorurl_string = CString::new(xorurl)?;
        o_cb(user_data.0, FFI_RESULT_OK, xorurl_string.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn get_sequence(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        version: u64,
        data: *const u8,
        data_len: usize,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let url = String::clone_from_repr_c(url)?;
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let (version, data) = runtime.block_on((*app).sequence_get(&url))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            version,
            data.as_ptr(),
            data.len(),
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn append_sequence(
    app: *mut Safe,
    url: *const c_char,
    data: *const u8,
    data_len: usize,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let url = String::clone_from_repr_c(url)?;
        let data_vec = vec_clone_from_raw_parts(data, data_len);
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        runtime.block_on((*app).sequence_append(&url, &data_vec))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        o_cb(user_data.0, FFI_RESULT_OK);
        Ok(())
    })
}
