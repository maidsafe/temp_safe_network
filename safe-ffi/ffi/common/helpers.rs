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
    let res = p_argv.as_ptr() as *const *const c_char;
    std::mem::forget(cstr_argv);
    std::mem::forget(p_argv);
    Ok(res)
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

#[cfg(test)]
mod tests {
    use super::*;
    use ffi_utils::{vec_from_raw_parts, vec_into_raw_parts};
    use unwrap::unwrap;

    #[test]
    fn test_str_vector_converter() {
        let str_vec: Vec<String> = vec![
            "string1".to_string(),
            "string2".to_string(),
            "string3".to_string(),
        ];
        let str_vec_2 = str_vec.clone();
        let str_vec_len = str_vec.len();
        let c_str_vec = unwrap!(string_vec_to_c_str_str(str_vec));
        let converted_vec: Vec<String> =
            unsafe { unwrap!(c_str_str_to_string_vec(c_str_vec, str_vec_len)) };
        println!("{:?}", converted_vec);
        assert_eq!(str_vec_2, converted_vec);
    }

    #[test]
    fn test_simple_vector_converter() {
        let str_vec: Vec<u8> = vec![1, 2, 3, 4, 5];
        let (c_vec_ptr, c_vec_len) = vec_into_raw_parts(str_vec.clone());
        let converted_vec: Vec<u8> = unsafe { vec_from_raw_parts(c_vec_ptr, c_vec_len) };
        assert_eq!(str_vec, converted_vec);
    }
}
