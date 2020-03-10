// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    errors::Result,
    ffi_structs::{bls_key_pair_into_repr_c, BlsKeyPair},
    helpers::from_c_str_to_str_option,
};
use ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK};
use safe_api::{BlsKeyPair as NativeBlsKeyPair, Safe};
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub unsafe extern "C" fn generate_keypair(
    app: *mut Safe,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        safe_key: *const BlsKeyPair,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let keypair = bls_key_pair_into_repr_c(&(*app).keypair()?)?;
        o_cb(user_data.0, FFI_RESULT_OK, &keypair);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn keys_create(
    app: *mut Safe,
    from: *const c_char,
    preload: *const c_char,
    pk: *const c_char,
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
        let from_option = from_c_str_to_str_option(from);
        let preload_option = from_c_str_to_str_option(preload);
        let pk_option = from_c_str_to_str_option(pk);
        let (xorurl, keypair) =
            async_std::task::block_on((*app).keys_create(from_option, preload_option, pk_option))?;
        let xorurl_c_str = CString::new(xorurl)?;
        let keypair = match keypair {
            Some(keypair) => keypair,
            None => NativeBlsKeyPair {
                pk: String::clone_from_repr_c(pk)?,
                sk: String::new(),
            },
        };
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            xorurl_c_str.as_ptr(),
            &bls_key_pair_into_repr_c(&keypair)?,
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn keys_create_preload_test_coins(
    app: *mut Safe,
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
        let preload_option = String::clone_from_repr_c(preload)?;
        let (xorurl, keypair) =
            async_std::task::block_on((*app).keys_create_preload_test_coins(&preload_option))?;
        let xorurl_c_str = CString::new(xorurl)?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            xorurl_c_str.as_ptr(),
            &bls_key_pair_into_repr_c(&keypair.as_ref().unwrap())?,
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn keys_balance_from_sk(
    app: *mut Safe,
    sk: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, balance: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let secret_key = String::clone_from_repr_c(sk)?;
        let balance = async_std::task::block_on((*app).keys_balance_from_sk(&secret_key))?;
        let amount_result = CString::new(balance)?;
        o_cb(user_data.0, FFI_RESULT_OK, amount_result.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn keys_balance_from_url(
    app: *mut Safe,
    url: *const c_char,
    sk: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, balance: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let key_url = String::clone_from_repr_c(url)?;
        let secret_key = String::clone_from_repr_c(sk)?;
        let balance =
            async_std::task::block_on((*app).keys_balance_from_url(&key_url, &secret_key))?;
        let amount_result = CString::new(balance)?;
        o_cb(user_data.0, FFI_RESULT_OK, amount_result.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn validate_sk_for_url(
    app: *mut Safe,
    sk: *const c_char,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, balance: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let key_url = String::clone_from_repr_c(url)?;
        let secret_key = String::clone_from_repr_c(sk)?;
        let balance = async_std::task::block_on((*app).validate_sk_for_url(&secret_key, &key_url))?;
        let amount_result = CString::new(balance)?;
        o_cb(user_data.0, FFI_RESULT_OK, amount_result.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn keys_transfer(
    app: *mut Safe,
    amount: *const c_char,
    from: *const c_char,
    to: *const c_char,
    id: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, tx_id: u64),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let from_key = from_c_str_to_str_option(from);
        let to_key = String::clone_from_repr_c(to)?;
        let amount_tranfer = String::clone_from_repr_c(amount)?;
        let tx_id = async_std::task::block_on((*app).keys_transfer(
            &amount_tranfer,
            from_key,
            &to_key,
            Some(id),
        ))?;
        o_cb(user_data.0, FFI_RESULT_OK, tx_id);
        Ok(())
    })
}
