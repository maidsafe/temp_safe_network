// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::common;

mod constants;
/// Fetch API
pub mod fetch;
pub mod ffi_structs;
pub mod files;
pub mod ipc;
pub mod keys;
pub mod nrs;
pub mod sequence;
pub mod wallet;
pub mod xorurl;

use super::common::{errors::Result, helpers::from_c_str_to_str_option};
use ffi_structs::{bls_key_pair_into_repr_c, BlsKeyPair};
use sn_ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK};
use safe_api::Safe;
use safe_api::{
    fetch::{XorUrlBase, XorUrlEncoder as NativeXorUrlEncoder},
    parse_coins_amount, xorname_from_pk, BlsKeyPair as NativeBlsKeyPair, KeyPair as NativeKeyPair,
};
use safe_core::test_create_balance;
use safe_nd::ClientFullId;
use std::{
    ffi::CString,
    os::raw::{c_char, c_void},
    time::Duration,
};
use tokio::runtime::Runtime;

#[no_mangle]
pub unsafe extern "C" fn allocate_test_coins(
    preload: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        xorurl: *const c_char,
        safe_key: *const BlsKeyPair,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let preload = String::clone_from_repr_c(preload)?;
        let amount = parse_coins_amount(&preload)?;
        let keypair = NativeKeyPair::random();
        let (pk, sk) = keypair.to_hex_key_pair()?;
        let key_pair = Some(NativeBlsKeyPair { pk, sk });
        let xorname = xorname_from_pk(keypair.pk);
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        runtime.block_on(test_create_balance(&ClientFullId::from(keypair.sk), amount))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        let xorurl = NativeXorUrlEncoder::encode_safekey(xorname, XorUrlBase::Base32z)?;
        let xorurl_c_str = CString::new(xorurl)?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            xorurl_c_str.as_ptr(),
            &bls_key_pair_into_repr_c(&key_pair.as_ref().unwrap())?,
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn auth_app(
    app_id: *const c_char,
    app_name: *const c_char,
    app_vendor: *const c_char,
    endpoint: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        auth_response: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let app_id = String::clone_from_repr_c(app_id)?;
        let app_name = String::clone_from_repr_c(app_name)?;
        let app_vendor = String::clone_from_repr_c(app_vendor)?;
        let endpoint = from_c_str_to_str_option(endpoint);
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let auth_response =
            runtime.block_on(Safe::auth_app(&app_id, &app_name, &app_vendor, endpoint))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        let auth_response = CString::new(auth_response)?;
        o_cb(user_data.0, FFI_RESULT_OK, auth_response.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn connect_app(
    app_id: *const c_char,
    auth_credentials: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, app: *mut Safe),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let app_id = String::clone_from_repr_c(app_id)?;
        let auth_cred = from_c_str_to_str_option(auth_credentials);
        let mut safe = Safe::default();
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        runtime.block_on(safe.connect(&app_id, auth_cred))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        o_cb(user_data.0, FFI_RESULT_OK, Box::into_raw(Box::new(safe)));
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn app_is_mock() -> bool {
    cfg!(feature = "scl-mock")
}
