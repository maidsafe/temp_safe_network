// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

#![allow(unsafe_code)]

pub mod callback;
#[macro_use]
mod macros;
pub mod string;
#[cfg(test)]
pub mod test_util;

use self::callback::{Callback, CallbackArgs};
pub use self::string::FfiString;
use std::{mem, slice, str};
use std::fmt::Debug;
use std::os::raw::c_void;
use std::panic::{self, AssertUnwindSafe};

/// Type that holds opaque user data handed into FFI functions
#[derive(Clone, Copy)]
pub struct OpaqueCtx(pub *mut c_void);
unsafe impl Send for OpaqueCtx {}

impl Into<*mut c_void> for OpaqueCtx {
    fn into(self) -> *mut c_void {
        self.0
    }
}

fn catch_unwind_result<'a, F, T, E>(f: F) -> Result<T, E>
    where F: FnOnce() -> Result<T, E>,
          E: Debug + From<&'a str>
{
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Err(_) => Err(E::from("panic")),
        Ok(result) => result,
    }
}

/// Catch panics. On error return the error code.
pub fn catch_unwind_error_code<'a, F, E>(f: F) -> i32
    where F: FnOnce() -> Result<(), E>,
          E: Debug + Into<i32> + From<&'a str>
{
    ffi_result_code!(catch_unwind_result(f))
}

/// Catch panics. On error call the callback.
pub fn catch_unwind_cb<'a, U, C, F, E>(user_data: U, cb: C, f: F)
    where U: Into<*mut c_void>,
          C: Callback,
          F: FnOnce() -> Result<(), E>,
          E: Debug + Into<i32> + From<&'a str>
{
    if let Err(err) = catch_unwind_result(f) {
        cb.call(user_data.into(),
                ffi_error_code!(err),
                CallbackArgs::default());
    }
}

/// Converts a byte pointer to Vec<u8>
pub unsafe fn u8_ptr_to_vec(ptr: *const u8, len: usize) -> Vec<u8> {
    slice::from_raw_parts(ptr, len).to_owned()
}

/// Converts null pointer to None and a valid pointer to Some(Vec<u8>)
pub unsafe fn u8_ptr_to_opt_vec(ptr: *const u8, len: usize) -> Option<Vec<u8>> {
    if ptr.is_null() {
        None
    } else {
        Some(u8_ptr_to_vec(ptr, len))
    }
}

/// Converts Vec<u8> to (byte pointer, size, capacity)
pub fn u8_vec_to_ptr(mut v: Vec<u8>) -> (*mut u8, usize, usize) {
    v.shrink_to_fit();
    let ptr = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    mem::forget(v);
    (ptr, len, cap)
}

#[cfg(test)]
mod tests {}
