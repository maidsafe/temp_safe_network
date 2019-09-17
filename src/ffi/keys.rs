use crate::api::{ResultReturn};
use crate::api::{Safe};
use super::ffi_structs::{BlsKeyPair};
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use super::helpers::{from_c_str_to_string_option, to_c_str};
use std::os::raw::{c_char, c_void};

const PRELOAD_TESTCOINS_DEFAULT_AMOUNT: &str = "1000.111";

#[no_mangle]
pub unsafe extern "C" fn generate_new_safe_key_pair(
    app: *mut Safe,
    test_coins: bool,
    pay_with: *const c_char,
    preload: *const c_char,
    pk: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        xor_url: *const c_char,
        safe_key: *const BlsKeyPair,
        pre_load: *const c_char)
) {
    catch_unwind_cb(user_data, o_cb,  || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let preload_str = from_c_str_to_string_option(preload);
        let pay_with_str = from_c_str_to_string_option(pay_with);
        let pk_with_str = from_c_str_to_string_option(pk);
        let (xorurl, key_pair, _amount) = if test_coins {
            let (xorurl, key_pair) = (*app).keys_create_preload_test_coins(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT)?;
            (xorurl, key_pair, Some(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT))
        } else {
            let (xorurl, key_pair) = (*app).keys_create(pay_with_str, preload_str, pk_with_str)?;
            (xorurl, key_pair, Some(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT)) // Todo: return amount not the default value
        };
        let key_xor_url = to_c_str(xorurl)?;
        let amount_result = to_c_str(PRELOAD_TESTCOINS_DEFAULT_AMOUNT.to_string())?;
        let ffi_bls_key_pair = key_pair.unwrap().into_repr_c()?;
        o_cb(user_data.0, FFI_RESULT_OK, key_xor_url.as_ptr(), &ffi_bls_key_pair, amount_result.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn query_key_balance(
    app: *mut Safe,
    key: *const c_char,
    secret: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        balance: *const c_char)
) {
    catch_unwind_cb(user_data, o_cb,  || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let key_url = from_c_str(key)?;
        let secret_key = from_c_str_to_string_option(secret);
        let sk = secret_key.unwrap_or_else(|| String::from(""));
        let current_balance = if key_url.is_empty() {
            (*app).keys_balance_from_sk(&sk)? 
        } else {
            (*app).keys_balance_from_url(&key_url, &sk)?
        };
        let amount_result = to_c_str(current_balance)?;
        o_cb(user_data.0, FFI_RESULT_OK, amount_result.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn transfer_key_balance(
    app: *mut Safe,
    from: *const c_char,
    to: *const c_char,
    amount: *const c_char,
    id: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        tx_id: u64),
) {
    catch_unwind_cb(user_data, o_cb,  || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let from_key = from_c_str_to_string_option(from);
        let to_key = from_c_str(to)?;
        let amount_tranfer = from_c_str(amount)?;
        let tx_id = (*app).keys_transfer(&amount_tranfer, from_key, &to_key, Some(id))?;
        o_cb(user_data.0, FFI_RESULT_OK, tx_id);
        Ok(())
    })
}
