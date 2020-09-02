// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#![allow(clippy::missing_safety_doc)]

#[macro_use]
extern crate sn_ffi_utils;

pub mod ffi;

pub use ffi::app::fetch::*;
pub use ffi::app::ffi_structs::*;
pub use ffi::app::files::*;
pub use ffi::app::ipc::*;
pub use ffi::app::keys::*;
pub use ffi::app::nrs::*;
pub use ffi::app::wallet::*;
pub use ffi::app::xorurl::*;
pub use ffi::app::*;

pub use ffi::authenticator::ffi_types::*;
pub use ffi::authenticator::*;
