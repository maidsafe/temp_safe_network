// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::api::common::auth_types::SafeAuthReqId;
use serde::{Deserialize, Serialize};

/// IPC request.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum IpcReq {
    /// Application authorisation request, used to obtain keypair/s the application
    /// shall sign the network requests with.
    Auth(AuthReq),
    /// Unregistered application request, used to obtain network bootstrapping information.
    /// Takes arbitrary user data as `Vec<u8>`, returns bootstrap information.
    Unregistered(Vec<u8>),
}

/// Represents an authorisation request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AuthReq {
    /// The authorisation request ID
    pub req_id: SafeAuthReqId,
    /// The application ID. It must be unique.
    pub app_id: String,
    /// The application friendly-name.
    pub app_name: String,
    /// The application provider/vendor (e.g. MaidSafe)
    pub app_vendor: String,
}
