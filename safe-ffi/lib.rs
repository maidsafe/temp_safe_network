// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[macro_use]
extern crate ffi_utils;

#[cfg(feature = "mock-network")]
pub use safe_authenticator_ffi::*;

pub mod ffi;

pub use ffi::fetch::*;
pub use ffi::ffi_structs::*;
pub use ffi::files::*;
pub use ffi::helpers::*;
pub use ffi::ipc::*;
pub use ffi::keys::*;
pub use ffi::logging::*;
pub use ffi::nrs::*;
pub use ffi::wallet::*;
pub use ffi::xorurl::*;
pub use ffi::*;
