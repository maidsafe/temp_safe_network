use super::errors::Result;
use ffi_utils::from_c_str;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;

// NOTE: The returned &str is only valid as long as the data in `ptr` is valid.
#[inline]
pub unsafe fn from_c_str_to_str_option(ptr: *const c_char) -> Option<&'static str> {
    if ptr.is_null() {
        None
    } else {
        CStr::from_ptr(ptr).to_str().ok()
    }
}

#[inline]
pub fn string_vec_to_c_str_str(argv: Vec<String>) -> Result<*const *const c_char> {
    let cstr_argv = argv
        .iter()
        .map(|arg| CString::new(arg.as_str()))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let p_argv: Vec<_> = cstr_argv.iter().map(|arg| arg.as_ptr()).collect();

    Ok(p_argv.as_ptr())
}

#[inline]
pub unsafe fn c_str_str_to_string_vec(
    argv: *const *const c_char,
    len: usize,
) -> Result<Vec<String>> {
    let data_vec = slice::from_raw_parts(argv, len).to_vec();
    let string_vec = data_vec
        .iter()
        .map(|s| from_c_str(*s))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(string_vec)
}
