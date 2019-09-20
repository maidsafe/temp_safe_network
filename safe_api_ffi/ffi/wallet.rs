use super::helpers::{from_c_str_to_string_option, to_c_str};
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use safe_api::{ResultReturn, Safe};
use std::os::raw::{c_char, c_void};

const PRELOAD_TESTCOINS_DEFAULT_AMOUNT: &str = "1000.111";

#[no_mangle]
pub unsafe extern "C" fn wallet_create(
    app: *mut Safe,
    pay_with: *const c_char,
    no_balance: bool,
    name: *const c_char,
    key_url: *const c_char,
    secret_key: *const c_char,
    test_coins: bool,
    preload: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, xorurl: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let wallet_xorurl = (*app).wallet_create()?;
        let key_url_str = from_c_str_to_string_option(key_url);
        let secret_key_str = from_c_str_to_string_option(secret_key);
        let name_str = from_c_str_to_string_option(name);
        let preload_str = from_c_str_to_string_option(preload);
        let pay_with_str = from_c_str_to_string_option(pay_with);
        if !no_balance {
            // get or create keypair
            let sk = match key_url_str {
                Some(linked_key) => {
                    let sk = secret_key_str.unwrap_or_else(|| String::from("")); //Todo: needs to be implementated properly
                    let _pk = (*app).validate_sk_for_url(&sk, &linked_key)?;
                    sk
                }
                None => match secret_key_str {
                    Some(sk) => sk,
                    None => {
                        let key_generated_output = if test_coins {
                            let (xorurl, key_pair) = (*app).keys_create_preload_test_coins(
                                &PRELOAD_TESTCOINS_DEFAULT_AMOUNT,
                            )?;
                            (
                                xorurl,
                                key_pair,
                                Some(PRELOAD_TESTCOINS_DEFAULT_AMOUNT.to_string()),
                            )
                        } else {
                            let (xorurl, key_pair) =
                                (*app).keys_create(pay_with_str, preload_str, None)?;
                            (
                                xorurl,
                                key_pair,
                                Some(PRELOAD_TESTCOINS_DEFAULT_AMOUNT.to_string()),
                            ) // Todo: return amount not the default value
                        };
                        let unwrapped_key_pair = key_generated_output
                            .1
                            .clone()
                            .ok_or("Failed to read the generated key pair")?;
                        unwrapped_key_pair.sk
                    }
                },
            };

            // insert and set as default
            (*app).wallet_insert(&wallet_xorurl, name_str, true, &sk)?;
        }

        let wallet_xorurl = to_c_str(wallet_xorurl)?;
        o_cb(user_data.0, FFI_RESULT_OK, wallet_xorurl.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn wallet_insert(
    app: *mut Safe,
    target: *const c_char,
    _pay_with: *const c_char,
    secret_key: *const c_char,
    name: *const c_char,
    key_url: *const c_char,
    set_default: bool,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, name: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let target_str = from_c_str(target)?;
        let secret_key_str = from_c_str_to_string_option(secret_key);
        let name_str = from_c_str_to_string_option(name);
        let key_url_str = from_c_str_to_string_option(key_url);

        let sk = match key_url_str {
            Some(linked_key) => {
                let sk = secret_key_str.unwrap_or_else(|| String::from("")); // todo:  needs to be updated to use a helper function to get the secret key
                let _pk = (*app).validate_sk_for_url(&sk, &linked_key)?;
                sk
            }
            None => secret_key_str.unwrap_or_else(|| String::from("")), // todo:  needs to be updated to use a helper function to get the secret key
        };
        let the_name = (*app).wallet_insert(&target_str, name_str, set_default, &sk)?;
        let result_name = to_c_str(the_name)?;
        o_cb(user_data.0, FFI_RESULT_OK, result_name.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn wallet_balance(
    app: *mut Safe,
    target: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, balance: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let target_str = from_c_str(target)?;
        let balance = (*app).wallet_balance(&target_str)?;
        let amount_result = to_c_str(balance)?;
        o_cb(user_data.0, FFI_RESULT_OK, amount_result.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn wallet_transfer(
    app: *mut Safe,
    from: *const c_char,
    to: *const c_char,
    amount: *const c_char,
    id: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, tx_id: u64),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let from_key = from_c_str_to_string_option(from);
        let to_key = from_c_str(to)?;
        let amount_tranfer = from_c_str(amount)?;
        let tx_id = (*app).wallet_transfer(&amount_tranfer, from_key, &to_key, Some(id))?;
        o_cb(user_data.0, FFI_RESULT_OK, tx_id);
        Ok(())
    })
}
