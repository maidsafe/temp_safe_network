use super::errors::Result;
use super::ffi_structs::{
    wallet_spendable_balance_into_repr_c, wallet_spendable_balances_into_repr_c,
    WalletSpendableBalance, WalletSpendableBalances,
};
use super::helpers::from_c_str_to_str_option;
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use safe_api::Safe;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub unsafe extern "C" fn wallet_create(
    app: *mut Safe,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, xorurl: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let wallet_xorurl = (*app).wallet_create()?;
        let wallet_xorurl_c_str = CString::new(wallet_xorurl)?;
        o_cb(user_data.0, FFI_RESULT_OK, wallet_xorurl_c_str.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn wallet_insert(
    app: *mut Safe,
    key_url: *const c_char,
    name: *const c_char,
    set_default: bool,
    secret_key: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, name: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let key_url_str = from_c_str(key_url)?;
        let secret_key_str = from_c_str(secret_key)?;
        let name_str = from_c_str_to_str_option(name);
        let wallet_name =
            (*app).wallet_insert(&key_url_str, name_str, set_default, &secret_key_str)?;
        let wallet_name_c_str = CString::new(wallet_name)?;
        o_cb(user_data.0, FFI_RESULT_OK, wallet_name_c_str.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn wallet_balance(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, balance: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let wallet_url = from_c_str(url)?;
        let balance = (*app).wallet_balance(&wallet_url)?;
        let amount_result = CString::new(balance)?;
        o_cb(user_data.0, FFI_RESULT_OK, amount_result.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn wallet_get_default_balance(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        spendable_wallet_balance: *const WalletSpendableBalance,
        version: u64,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let wallet_url = from_c_str(url)?;
        let (spendable, version) = (*app).wallet_get_default_balance(&wallet_url)?;
        let wallet_spendable = wallet_spendable_balance_into_repr_c(&spendable)?;
        o_cb(user_data.0, FFI_RESULT_OK, &wallet_spendable, version);
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
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let from_key = from_c_str_to_str_option(from);
        let to_key = from_c_str(to)?;
        let amount_tranfer = from_c_str(amount)?;
        let tx_id = (*app).wallet_transfer(&amount_tranfer, from_key, &to_key, Some(id))?;
        o_cb(user_data.0, FFI_RESULT_OK, tx_id);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn wallet_get(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        spendable_wallet_balance: *const WalletSpendableBalances,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let wallet_url = from_c_str(url)?;
        let spendables = (*app).wallet_get(&wallet_url)?;
        let wallet_spendable = wallet_spendable_balances_into_repr_c(&spendables)?;
        o_cb(user_data.0, FFI_RESULT_OK, &wallet_spendable);
        Ok(())
    })
}
