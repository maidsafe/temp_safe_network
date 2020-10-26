// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#![allow(unsafe_code)]

use super::{BootstrapConfig, IpcError};

use serde::{Deserialize, Serialize};
use sn_data_types::Keypair;
use std::sync::Arc;

/// IPC response.
// TODO: `TransOwnership` variant
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IpcResp {
    /// Authentication.
    Auth(Result<AuthGranted, IpcError>),
    /// Unregistered client.
    Unregistered(Result<BootstrapConfig, IpcError>),
}

/// It represents the authentication response.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AuthGranted {
    /// The access keys.
    pub app_keypair: Arc<Keypair>,

    /// The crust config.
    /// Useful to reuse bootstrap nodes and speed up access.
    pub bootstrap_config: BootstrapConfig,
}
