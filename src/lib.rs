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

#![crate_name = "maidsafe_client"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://maidsafe.net/img/Resources/branding/maidsafe_logo.fab2.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
              html_root_url = "http://dirvine.github.io/dirvine/maidsafe_client/")]

///////////////////////////////////////////////////
//               LINT
///////////////////////////////////////////////////

#![forbid(bad_style, warnings)] 

#![deny(deprecated, improper_ctypes, missing_docs, non_shorthand_field_patterns,
overflowing_literals, plugin_as_library, private_no_mangle_fns, private_no_mangle_statics,
raw_pointer_derive, stable_features, unconditional_recursion, unknown_lints, unsafe_code,
unsigned_negation, unused, unused_allocation, unused_attributes, unused_comparisons,
unused_features, unused_parens, while_true)] 

#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
unused_qualifications, variant_size_differences)]

///////////////////////////////////////////////////

//! #Maidsafe-Client Library
//! [Project github page](https://github.com/maidsafe/maidsafe_client)

#![allow(variant_size_differences)]

extern crate cbor;
extern crate rand;
extern crate time;
extern crate crypto;
extern crate routing;
extern crate sodiumoxide;
extern crate rustc_serialize;

extern crate maidsafe_types;
extern crate lru_time_cache;
extern crate self_encryption;

/// NFS module
pub mod nfs;
/// Self-Auth and Gateway Module
pub mod client;

/// Representation of input/output error
pub type IoError = std::io::Error;

/// CryptoError - To be removed
pub enum CryptoError {
    /// TODO
    SymmetricCryptoError(crypto::symmetriccipher::SymmetricCipherError),
    /// TODO
    BadBuffer
}

impl From<crypto::symmetriccipher::SymmetricCipherError> for CryptoError {
    fn from(error : crypto::symmetriccipher::SymmetricCipherError) -> CryptoError {
        return CryptoError::SymmetricCryptoError(error);
    }
}

/// Under Construction - Will undergo lot of modification in upcoming sprint
pub enum MaidsafeError {
    /// Under Construction
    CryptoError(CryptoError),
    /// Under Construction
    EncodingError(cbor::CborError),
}

impl From<CryptoError> for MaidsafeError {
    fn from(error : CryptoError) -> MaidsafeError {
        return MaidsafeError::CryptoError(error);
    }
}

impl From<cbor::CborError> for MaidsafeError {
    fn from(error : cbor::CborError) -> MaidsafeError {
        return MaidsafeError::EncodingError(error);
    }
}

impl From<crypto::symmetriccipher::SymmetricCipherError> for MaidsafeError {
    fn from(error : crypto::symmetriccipher::SymmetricCipherError) -> MaidsafeError {
        return MaidsafeError::CryptoError(CryptoError::SymmetricCryptoError(error));
    }
}

mod utility {
    #[allow(dead_code)]
    pub fn generate_random_string(length: usize) -> String {
        (0..length).map(|_| ::rand::random::<char>()).collect()
    }

    #[allow(dead_code)]
    pub fn generate_random_pin() -> u32 {
        ::rand::random::<u32>() % 10000
    }
}
