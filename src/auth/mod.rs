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

use routing::XorName;
use rust_sodium::crypto::{box_, secretbox, sign};

/// TODO: doc
pub mod ffi;

use self::ffi::PermissionAccess;

// TODO: replace with `crust::Config`
/// empty doc
pub struct Config;

/// TODO: doc
pub struct ContainerPermission {
    /// TODO: doc
    pub container_key: String,
    /// TODO: doc
    pub access: Vec<PermissionAccess>,
}

/// TODO: doc
pub struct AppExchangeInfo {
    /// TODO: doc
    pub id: String,
    /// TODO: doc
    pub scope: Option<String>,
    /// TODO: doc
    pub name: String,
    /// TODO: doc
    pub vendor: String,
}

/// TODO: doc
pub struct AuthRequest {
    /// TODO: doc
    pub app: AppExchangeInfo,
    /// TODO: doc
    pub app_container: bool,
    /// TODO: doc
    pub containers: Vec<ContainerPermission>,
}

/// TODO: doc
pub struct AppAccessToken {
    /// TODO: doc
    pub enc_key: secretbox::Key,
    /// TODO: doc
    pub sign_pk: sign::PublicKey,
    /// TODO: doc
    pub sign_sk: sign::SecretKey,
    /// TODO: doc
    pub enc_pk: box_::PublicKey,
    /// TODO: doc
    pub enc_sk: box_::SecretKey,
}

/// TODO: doc
pub enum AuthResponse {
    /// TODO: doc
    Granted {
        /// TODO: doc
        access_token: AppAccessToken,
        /// TODO: doc
        bootstrap_config: Config,
        /// TODO: doc
        access_container: Option<(XorName, u64)>,
    },
    /// TODO: doc
    Denied,
}
