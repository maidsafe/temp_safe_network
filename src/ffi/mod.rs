// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#![allow(unsafe_code)]

/// Type definitions for arrays that are FFI input params.
pub mod arrays;
/// FFI Error Codes
pub mod error_codes;
/// IPC utilities.
pub mod ipc;
/// NFS API.
pub mod nfs;
use self::arrays::*;
use safe_nd::MapKind as NativeMapKind;

/// FFI wrapper for `MapInfo`.
#[repr(C)]
#[derive(Clone)]
pub struct MapInfo {
    // NOTE: `repr[C]` enums don't seem to be supported by JNI right now, so we use a bool.
    /// The kind of the mutable data.
    pub seq: bool,
    /// Name of the mutable data.
    pub name: XorNameArray,
    /// Type tag of the mutable data.
    pub type_tag: u64,

    /// Flag indicating whether the encryption info (`enc_key` and `enc_nonce`) is set.
    pub has_enc_info: bool,
    /// Encryption key. Meaningful only if `has_enc_info` is `true`.
    pub enc_key: SymSecretKey,
    /// Encryption nonce. Meaningful only if `has_enc_info` is `true`.
    pub enc_nonce: SymNonce,

    /// Flag indicating whether the new encryption info is set.
    pub has_new_enc_info: bool,
    /// New encryption key (used for two-phase reencryption). Meaningful only if
    /// `has_new_enc_info` is `true`.
    pub new_enc_key: SymSecretKey,
    /// New encryption nonce (used for two-phase reencryption). Meaningful only if
    /// `has_new_enc_info` is `true`.
    pub new_enc_nonce: SymNonce,
}

// TODO: Implement `into_repr_c` for MapKind once we move FfiMapKind to safe-nd.
/// Convert from native to FFI representation for `MapKind`.
pub fn md_kind_into_repr_c(kind: NativeMapKind) -> bool {
    match kind {
        NativeMapKind::Seq => true,
        NativeMapKind::Unseq => false,
    }
}

// TODO: Implement `clone_from_repr_c` for `MapKind` once we move
// `FfiMapKind` to safe-nd.
/// Convert from FFI to native representation for `MapKind`.
pub fn md_kind_clone_from_repr_c(kind: bool) -> NativeMapKind {
    if kind {
        NativeMapKind::Seq
    } else {
        NativeMapKind::Unseq
    }
}
