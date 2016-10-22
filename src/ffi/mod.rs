// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

//! This module provides FFI-bindings to the Client Modules (`core`, `nfs`, `dns`)
//! In the current implementation the allocations made by this crate are managed within the crate
//! itself and is guaranteed that management of such allocations will not be pushed beyond the FFI
//! boundary. This has a 2-fold outcome: firstly, the passing of data is done by filling of the
//! allocations passed by the caller and is caller's responsibility to manage those. For this every
//! function that fills an allocated memory also has a companion function to return the size of
//! data which the caller can call to find out how much space needs to be allocated in the first
//! place. Second and consequently, the caller does not have to bother calling functions within
//! this crate which only serve to free resources allocated by the crate itself. This otherwise
//! would be error prone and cumbersome. Instead the caller can use whatever idiom in his language
//! to manage memory much more naturally and conveniently (eg., RAII idioms etc)
//!
//! The only exception to the above rule is the obtainment of the client engine itself. The client
//! engine is allocated and managed by the crate. This is necessary because it serves as a context
//! to all operations provided by the crate. Hence the user will obtain the engine on calling any
//! one of the functions to create it and must preserve it for all subsequent operations. When
//! done, to release the resources, `drop_client` may be called.

#![allow(unsafe_code)]

#[macro_use]
mod macros;

mod app;
pub mod dir_details;
// pub mod dns;
pub mod errors;
// pub mod file_details;
// pub mod logging;
pub mod low_level_api;
// pub mod nfs;
mod session;
// pub mod string_list;

mod config;
mod helper;
mod launcher_config;
mod object_cache;
#[cfg(test)]
mod test_utils;

pub use ffi::app::App;
pub use ffi::errors::FfiError;
pub use ffi::session::Session;

use futures::Future;
/// Helper type to represent the FFI future result
pub type FfiFuture<T> = Future<Item = T, Error = FfiError>;

/// Type that holds opaque user data handed into FFI functions
#[derive(Clone, Copy)]
pub struct OpaqueCtx(*const ::libc::c_void);
unsafe impl Send for OpaqueCtx {}
