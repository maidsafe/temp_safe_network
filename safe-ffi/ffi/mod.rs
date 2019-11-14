mod errors;
/// Fetch API
pub mod fetch;
pub mod ffi_structs;
pub mod files;
pub mod helpers;
pub mod keys;
pub mod nrs;
pub mod wallet;
pub mod xorurl;

use errors::Result;
use ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK};
use helpers::from_c_str_to_str_option;
use safe_api::Safe;
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub unsafe extern "C" fn connect(
    app_id: *const c_char,
    auth_credentials: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, app: *mut Safe),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let app_id = String::clone_from_repr_c(app_id)?;
        let auth_cred = from_c_str_to_str_option(auth_credentials);
        let mut safe = Safe::new(None);
        safe.connect(&app_id, auth_cred)?;
        o_cb(user_data.0, FFI_RESULT_OK, Box::into_raw(Box::new(safe)));
        Ok(())
    })
}
