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
/// TODO: doc
pub enum PermissionAccess {
    /// TODO: doc
    Read,
    /// TODO: doc
    Insert,
    /// TODO: doc
    Update,
    /// TODO: doc
    Delete,
    /// TODO: doc
    ManagePermissions,
}

#[repr(C)]
/// TODO: doc
pub struct ContainerPermission {
    /// TODO: doc
    pub container_key: *mut u8,
    /// TODO: doc
    pub container_key_len: usize,

    /// TODO: doc
    pub access: *mut PermissionAccess,
    /// TODO: doc
    pub access_len: usize,
}

#[repr(C)]
/// TODO: doc
pub struct AppExchangeInfo {
    /// TODO: doc
    pub id: *mut u8,
    /// TODO: doc
    pub id_len: usize,
    /// TODO: doc
    pub id_cap: usize,

    /// TODO: doc
    ///
    /// null if not present
    pub scope: *mut u8,
    /// TODO: doc
    ///
    /// 0 if above is null
    pub scope_len: usize,
    /// TODO: doc
    ///
    /// 0 if above is null
    pub scope_cap: usize,

    /// TODO: doc
    pub name: *mut u8,
    /// TODO: doc
    pub name_len: usize,
    /// TODO: doc
    pub name_cap: usize,

    /// TODO: doc
    pub vendor: *mut u8,
    /// TODO: doc
    pub vendor_len: usize,
    /// TODO: doc
    pub vendor_cap: usize,
}

#[repr(C)]
/// TODO: doc
pub struct AuthRequest {
    /// TODO: doc
    pub app: AppExchangeInfo,
    /// TODO: doc
    pub app_container: bool,

    /// TODO: doc
    pub containers: *mut ContainerPermission,
    /// TODO: doc
    pub containers_len: usize,
    /// TODO: doc
    pub containers_cap: usize,
}

#[repr(C)]
/// TODO: doc
pub struct AppAccessToken {
    /// TODO: doc
    pub enc_key: [u8; secretbox::KEYBYTES],
    /// TODO: doc
    pub sign_pk: [u8; sign::PUBLICKEYBYTES],
    /// TODO: doc
    pub sign_sk: [u8; sign::SECRETKEYBYTES],
    /// TODO: doc
    pub enc_pk: [u8; box_::PUBLICKEYBYTES],
    /// TODO: doc
    pub enc_sk: [u8; box_::SECRETKEYBYTES],
}
