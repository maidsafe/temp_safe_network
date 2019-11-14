// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::errors::Result;
use ffi_utils::ReprC;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;

// NOTE: The returned &str is only valid as long as the data in `ptr` is valid.
/// # Safety
/// Note this is an unsafe function
#[inline]
pub unsafe fn from_c_str_to_str_option(ptr: *const c_char) -> Option<&'static str> {
    if ptr.is_null() {
        None
    } else {
        CStr::from_ptr(ptr).to_str().ok()
    }
}

/// # Safety
/// Note this is an unsafe function
#[inline]
pub fn string_vec_to_c_str_str(argv: Vec<String>) -> Result<*const *const c_char> {
    let cstr_argv = argv
        .iter()
        .map(|arg| CString::new(arg.as_str()))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let p_argv: Vec<_> = cstr_argv.iter().map(|arg| arg.as_ptr()).collect();

    Ok(p_argv.as_ptr())
}

/// # Safety
/// Note this is an unsafe function
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
