use crate::api::{ResultReturn, Error};
use crate::api::keys::{BlsKeyPair};
use crate::api::{Safe};
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use std::os::raw::{c_char, c_void};
use std::ffi::{CString};

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
        let preload_option = from_c_str(preload)?;
        let pay_with_option = from_c_str(pay_with)?;
        let pk_with_option = from_c_str(pk)?;
        let (xorurl, key_pair, _amount) = if test_coins {
            let (xorurl, key_pair) = (*app).keys_create_preload_test_coins(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT)?;
            (xorurl, key_pair, Some(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT))
        } else {
            let (xorurl, key_pair) = (*app).keys_create(Some(pay_with_option), Some(preload_option), Some(pk_with_option))?;
            (xorurl, key_pair, Some(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT)) // Todo: return amount not the default value
        };
        let key_xor_url = CString::new(xorurl).map_err(|_| Error::Unexpected("Couldn't convert to string".to_string()))?;
        let amount_result = CString::new(PRELOAD_TESTCOINS_DEFAULT_AMOUNT).map_err(|_| Error::Unexpected("Couldn't convert to string".to_string()))?;
        o_cb(user_data.0, FFI_RESULT_OK, key_xor_url.as_ptr(), &key_pair.unwrap(), amount_result.as_ptr());
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
        let secret_key = from_c_str(secret)?;
        let sk = Some(secret_key).unwrap_or_else(|| String::from(""));
        let current_balance = if key_url.is_empty() {
            (*app).keys_balance_from_sk(&sk)?
        } else {
            (*app).keys_balance_from_url(&key_url, &sk)?
        };
        let amount_result = CString::new(current_balance).map_err(|_| Error::Unexpected("Couldn't convert to string".to_string()))?;
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
        let from_key = from_c_str(from)?;
        let to_key = from_c_str(to)?;
        let amount_tranfer = from_c_str(amount)?;
        let tx_id = (*app).keys_transfer(&amount_tranfer, Some(from_key), &to_key, Some(id))?;
        o_cb(user_data.0, FFI_RESULT_OK, tx_id);
        Ok(())
    })
}
