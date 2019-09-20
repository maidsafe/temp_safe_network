/// Fetch API
pub mod fetch;
pub mod xorurl;
pub mod keys;
pub mod nrs;
pub mod wallet;
pub mod helpers;
pub mod ffi_structs;

use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use std::os::raw::{c_char, c_void};
use crate::api::{Safe, ResultReturn};
use helpers::{from_c_str_to_str_option};

#[no_mangle]
pub unsafe extern "C" fn connect(
    app_id: *const c_char,
    auth_credentials: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, app: *mut Safe),
) {
    catch_unwind_cb(user_data, o_cb,  || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let app_id = from_c_str(app_id)?;
        let auth_cred = from_c_str_to_str_option(auth_credentials);
        let mut safe = Safe::new("base32z");
        safe.connect(&app_id, auth_cred)?;
        o_cb(user_data.0, FFI_RESULT_OK, Box::into_raw(Box::new(safe)));
        Ok(())
    })
}
