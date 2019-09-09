use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use std::os::raw::{c_char, c_void};
use super::{Safe, ResultReturn};
use crate::fetch::SafeData;

#[no_mangle]
pub unsafe extern "C" fn safe_fetch(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        im_data: *const u8,
        im_data_len: usize),
) {
    catch_unwind_cb(user_data, o_cb,  || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let url = from_c_str(url)?;
        let content = (*app).fetch(&url)?;
        match &content {
        SafeData::PublishedImmutableData {
            data,
            xorname: _,
            resolved_from: _,
        } => {
            o_cb(
                user_data.0,
                FFI_RESULT_OK,
                data.as_ptr(),
                data.len());  
            },
        _ => {}
        }
        Ok(())
    })
}
