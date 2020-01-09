// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::errors::Result;
use ffi_utils::ReprC;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    slice,
};

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
        .map(|s| String::clone_from_repr_c(*s))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(string_vec)
}
