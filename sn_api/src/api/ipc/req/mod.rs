// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#![allow(unsafe_code)]

mod auth;
// mod containers;
// mod share_map;

pub use self::auth::AuthReq;
// pub use self::containers::ContainersReq;
// pub use self::share_map::{ShareMap, ShareMapReq};

// use crate::ffi::ipc::req::{
//     AppExchangeInfo as FfiAppExchangeInfo, ContainerPermissions as FfiContainerPermissions,
//     PermissionSet as FfiPermissionSet,
// };

// use ffi_utils::{ReprC, StringError};
use serde::{Deserialize, Serialize};

// use std::ffi::{CString, NulError};

/// Permission enum - use for internal storage only.
// #[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
// pub enum Permission {
//     /// Read.
//     Read,
//     /// Insert.
//     Insert,
//     /// Update.
//     Update,
//     /// Delete.
//     Delete,
//     /// Modify permissions.
//     ManagePermissions,
// }

/// Permissions stored internally in the access container.
/// In FFI represented as `ffi::PermissionSet`
// pub type ContainerPermissions = BTreeSet<Permission>;

/// IPC request.
// TODO: `TransOwnership` variant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum IpcReq {
    /// Authentication request.
    Auth(AuthReq),
    // /// Containers request.
    // Containers(ContainersReq),
    // /// Unregistered client authenticator request.
    /// Takes arbitrary user data as `Vec<u8>`, returns bootstrap config.
    Unregistered(Vec<u8>),
    // /// Share mutable data.
    // ShareMap(ShareMapReq),
}
