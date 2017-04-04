// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

//! # Safe Client Libraries
//! [Project GitHub page](https://github.com/maidsafe/safe_client_libs)

#![doc(html_logo_url =
           "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
       html_favicon_url = "https://maidsafe.net/img/favicon.ico",
       html_root_url = "https://docs.rs/safe_client_libs")]

// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(exceeding_bitshifts, mutable_transmutes, no_mangle_const_items,
          unknown_crate_types, warnings)]
#![deny(bad_style, deprecated, improper_ctypes, missing_docs,
        non_shorthand_field_patterns, overflowing_literals, plugin_as_library,
        private_no_mangle_fns, private_no_mangle_statics, stable_features, unconditional_recursion,
        unknown_lints, unsafe_code, unused, unused_allocation, unused_attributes,
        unused_comparisons, unused_features, unused_parens, while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]
#![allow(box_pointers, fat_ptr_transmutes, missing_copy_implementations,
         missing_debug_implementations, variant_size_differences)]

// FIXME(nbaksalyar): temporarily disable new lints when updating clippy on master,
// this should be a separate task
#![cfg_attr(feature = "cargo-clippy", allow(large_enum_variant, cyclomatic_complexity,
                                            match_wild_err_arm, needless_pass_by_value,
                                            should_assert_eq))]

extern crate config_file_handler;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate lru_cache;
extern crate maidsafe_utilities;
extern crate rand;
extern crate routing;
#[cfg(feature = "use-mock-routing")]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate self_encryption;
extern crate rust_sodium;
extern crate chrono;
#[macro_use]
extern crate unwrap;

/// Core module
pub mod core;
/// Nfs module;
pub mod nfs;
/// Dns module;
pub mod dns;
/// Ffi module;
pub mod ffi;

/// Unversioned `StructuredData`
pub const UNVERSIONED_STRUCT_DATA_TYPE_TAG: u64 = 500;
/// Versioned `StructuredData`
pub const VERSIONED_STRUCT_DATA_TYPE_TAG: u64 = 501;
