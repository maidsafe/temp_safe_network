use crate::api::Error;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[inline]
pub unsafe fn from_c_str_to_string_option(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        CStr::from_ptr(ptr).to_owned().into_string().ok()
    }
}

#[inline]
pub unsafe fn from_c_str_to_str_option(ptr: *const c_char) -> Option<&'static str> {
    if ptr.is_null() {
        None
    } else {
        CStr::from_ptr(ptr).to_str().ok()
    }
}

#[inline]
pub unsafe fn to_c_str(native_string: String) -> Result<CString, Error> {
    CString::new(native_string).map_err(|_| Error::StringError("Couldn't convert to string".to_string()))
}
