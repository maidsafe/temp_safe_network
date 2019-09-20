use super::ffi_structs::{bls_key_pair_into_repr_c, BlsKeyPair};
use super::helpers::{from_c_str_to_string_option, to_c_str};
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use safe_api::{ResultReturn, Safe};
use std::os::raw::{c_char, c_void};

const PRELOAD_TESTCOINS_DEFAULT_AMOUNT: &str = "1000.111";

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
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let keypair = bls_key_pair_into_repr_c(&(*app).keypair()?)?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            &keypair,
        );
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
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let preload_option = from_c_str_to_string_option(preload);
        let from_option = from_c_str_to_string_option(from);
        let pk_option = from_c_str_to_string_option(pk);
        let (xorurl, keypair) = &(*app).keys_create(from_option, preload_option, pk_option)?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            to_c_str(xorurl.to_string())?.as_ptr(),
            &bls_key_pair_into_repr_c(&keypair.as_ref().unwrap())?,
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
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let preload_option = from_c_str(preload)?;
        let (xorurl, keypair) = &(*app).keys_create_preload_test_coins(&preload_option)?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            to_c_str(xorurl.to_string())?.as_ptr(),
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
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let secret_key = from_c_str(sk)?;
        let balance = (*app).keys_balance_from_sk(&secret_key)?;
        let amount_result = to_c_str(balance)?;
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
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let key_url = from_c_str(url)?;
        let secret_key = from_c_str(sk)?;
        let balance = (*app).keys_balance_from_url(&key_url, &secret_key)?;
        let amount_result = to_c_str(balance)?;
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
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let key_url = from_c_str(url)?;
        let secret_key = from_c_str(sk)?;
        let balance = (*app).validate_sk_for_url(&secret_key, &key_url)?;
        let amount_result = to_c_str(balance)?;
        o_cb(user_data.0, FFI_RESULT_OK, amount_result.as_ptr());
        Ok(())
    })
}


#[no_mangle]
pub unsafe extern "C" fn generate_safe_key_pair(
    app: *mut Safe,
    test_coins: bool,
    pay_with: *const c_char,
    preload: *const c_char,
    pk: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        xorurl: *const c_char,
        safe_key: *const BlsKeyPair,
        pre_load: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let preload_str = from_c_str_to_string_option(preload);
        let pay_with_str = from_c_str_to_string_option(pay_with);
        let pk_with_str = from_c_str_to_string_option(pk);
        let (xorurl, key_pair, _amount) = if test_coins {
            let (xorurl, key_pair) =
                (*app).keys_create_preload_test_coins(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT)?;
            (xorurl, key_pair, Some(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT))
        } else {
            let (xorurl, key_pair) = (*app).keys_create(pay_with_str, preload_str, pk_with_str)?;
            (xorurl, key_pair, Some(&PRELOAD_TESTCOINS_DEFAULT_AMOUNT)) // Todo: return amount not the default value
        };
        let key_xorurl = to_c_str(xorurl)?;
        let amount_result = to_c_str(PRELOAD_TESTCOINS_DEFAULT_AMOUNT.to_string())?;
        let ffi_bls_key_pair = bls_key_pair_into_repr_c(&key_pair.unwrap())?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            key_xorurl.as_ptr(),
            &ffi_bls_key_pair,
            amount_result.as_ptr(),
        );
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
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let from_key = from_c_str_to_string_option(from);
        let to_key = from_c_str(to)?;
        let amount_tranfer = from_c_str(amount)?;
        let tx_id = (*app).keys_transfer(&amount_tranfer, from_key, &to_key, Some(id))?;
        o_cb(user_data.0, FFI_RESULT_OK, tx_id);
        Ok(())
    })
}
