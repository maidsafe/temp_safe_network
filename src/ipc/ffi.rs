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

use rust_sodium::crypto::{box_, secretbox, sign};

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq, RustcEncodable, RustcDecodable)]
/// The permission type
pub enum PermissionAccess {
    /// Read
    Read,
    /// Insert
    Insert,
    /// Update
    Update,
    /// Delete
    Delete,
    /// Modify permissions
    ManagePermissions,
}

#[repr(C)]
/// Represents the set of permissions for a given container
pub struct ContainerPermission {
    /// The UTF-8 encoded id
    pub container_key: *const u8,
    /// `container_key`'s length
    pub container_key_len: usize,
    /// Used by the Rust memory allocator
    pub container_key_cap: usize,

    /// The `PermissionAccess` array
    pub access: *mut PermissionAccess,
    /// `access`'s length.
    pub access_len: usize,
    /// Used by the Rust memory allocator
    pub access_cap: usize,
}

/// Free memory
#[no_mangle]
#[allow(unsafe_code)]
pub unsafe extern "C" fn container_permission_free(cp: *mut ContainerPermission) {
    let _ = super::ContainerPermission::from_raw(cp);
}

#[repr(C)]
/// Represents an application ID in the process of asking permissions
pub struct AppExchangeInfo {
    /// UTF-8 encoded id
    pub id: *const u8,
    /// `id`'s length
    pub id_len: usize,
    /// Used by the Rust memory allocator
    pub id_cap: usize,

    /// Reserved by the frontend
    ///
    /// null if not present
    pub scope: *const u8,
    /// `scope`'s length.
    ///
    /// 0 if `scope` is null
    pub scope_len: usize,
    /// Used by the Rust memory allocator.
    ///
    /// 0 if `scope` is null
    pub scope_cap: usize,

    /// UTF-8 encoded application friendly-name.
    pub name: *const u8,
    /// `name`'s length
    pub name_len: usize,
    /// Used by the Rust memory allocator
    pub name_cap: usize,

    /// UTF-8 encoded application provider/vendor (e.g. MaidSafe)
    pub vendor: *const u8,
    /// `vendor`'s length
    pub vendor_len: usize,
    /// Reserved by the Rust allocator
    pub vendor_cap: usize,
}

/// Free memory
#[no_mangle]
#[allow(unsafe_code)]
pub unsafe extern "C" fn app_exchange_info_free(a: *mut AppExchangeInfo) {
    let _ = super::AppExchangeInfo::from_raw(a);
}

#[repr(C)]
/// Represents an authorization request
pub struct AuthReq {
    /// The application identifier for this request
    pub app: *mut AppExchangeInfo,
    /// `true` if the app wants dedicated container for itself. `false`
    /// otherwise.
    pub app_container: bool,

    /// Array of `*mut ContainerPermission`
    pub containers: *mut *mut ContainerPermission,
    /// `containers`'s length
    pub containers_len: usize,
    /// Reserved by the Rust allocator
    pub containers_cap: usize,
}

/// Free memory from the subobjects
#[no_mangle]
#[allow(unsafe_code)]
pub unsafe extern "C" fn auth_request_drop(a: AuthReq) {
    let _ = super::AuthReq::from_ffi(a);
}

#[repr(C)]
/// Represents the needed keys to work with the data
pub struct AppAccessToken {
    /// Data symmetric encryption key
    pub enc_key: [u8; secretbox::KEYBYTES],
    /// Asymmetric sign public key.
    ///
    /// This is the identity of the App in the Network.
    pub sign_pk: [u8; sign::PUBLICKEYBYTES],
    /// Asymmetric sign private key.
    pub sign_sk: [u8; sign::SECRETKEYBYTES],
    /// Asymmetric enc public key.
    pub enc_pk: [u8; box_::PUBLICKEYBYTES],
    /// Asymmetric enc private key.
    pub enc_sk: [u8; box_::SECRETKEYBYTES],
}

/// Free memory
#[no_mangle]
#[allow(unsafe_code)]
pub unsafe extern "C" fn app_access_token_free(a: *mut AppAccessToken) {
    let _ = super::AppAccessToken::from_raw(a);
}

/// TODO: doc
pub struct ContainersReq;

/// TODO: doc
pub struct ContainersGranted;
