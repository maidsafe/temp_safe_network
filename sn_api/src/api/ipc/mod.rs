// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Request module.
pub mod req;
/// Response module.
pub mod resp;

mod errors;

pub use self::errors::IpcError;
pub use self::req::{AuthReq, IpcReq};
pub use self::resp::{AuthGranted, IpcResp};

use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::SocketAddr, u32};

/// `QuicP2P` bootstrap info, shared from Authenticator to apps.
pub type BootstrapConfig = HashSet<SocketAddr>;

/// IPC message.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum IpcMsg {
    /// Request.
    Req(IpcReq),
    /// Response.
    Resp(IpcResp),
    /// Generic error like couldn't parse IpcMsg etc.
    Err(IpcError),
}

impl IpcMsg {
    pub fn new_auth_req(app_id: &str, app_name: &str, app_vendor: &str) -> Self {
        let req_id: u32 = gen_req_id();
        Self::Req(IpcReq::Auth(AuthReq {
            req_id,
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
            app_vendor: app_vendor.to_string(),
        }))
    }

    pub fn new_unreg_req(user_data: &[u8]) -> Self {
        Self::Req(IpcReq::Unregistered(user_data.to_vec()))
    }

    pub fn to_string(&self) -> Result<String, IpcError> {
        serde_json::to_string(self).map_err(|_| IpcError::EncodeDecodeError)
    }

    pub fn from_string(msg_str: &str) -> Result<IpcMsg, IpcError> {
        serde_json::from_str(msg_str).map_err(|_| IpcError::EncodeDecodeError)
    }
}

// Generate a unique request ID.
fn gen_req_id() -> u32 {
    use rand::Rng;
    // Generate the number in range 1..MAX inclusive.
    rand::thread_rng().gen_range(0, u32::max_value()) + 1
}
