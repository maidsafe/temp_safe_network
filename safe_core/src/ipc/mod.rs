// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Request module
pub mod req;
/// Response module
pub mod resp;

mod errors;

pub use self::errors::IpcError;
pub use self::req::{AppExchangeInfo, AuthReq, ContainersReq, IpcReq, Permission, ShareMData,
                    ShareMDataReq};
pub use self::resp::{AccessContInfo, AccessContainerEntry, AppKeys, AuthGranted, IpcResp,
                     access_container_enc_key};

use ffi_utils::{base64_decode, base64_encode};
use maidsafe_utilities::serialisation::{deserialise, serialise};
use rand::{self, Rng};
pub use routing::BootstrapConfig;
use std::u32;

/// IPC message
#[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum IpcMsg {
    /// Request
    Req {
        /// Request ID
        req_id: u32,
        /// Request
        req: IpcReq,
    },
    /// Response
    Resp {
        /// Request ID
        req_id: u32,
        /// Response
        resp: IpcResp,
    },
    /// Revoked
    Revoked {
        /// Application ID
        app_id: String,
    },
    /// Generic error like couldn't parse IpcMsg etc.
    Err(IpcError),
}

/// Encode `IpcMsg` into string, using base64 encoding.
pub fn encode_msg(msg: &IpcMsg) -> Result<String, IpcError> {
    Ok(base64_encode(&serialise(msg)?))
}

/// Decode `IpcMsg` encoded with base64 encoding.
pub fn decode_msg(encoded: &str) -> Result<IpcMsg, IpcError> {
    Ok(deserialise(&base64_decode(encoded)?)?)
}

/// Generate unique request ID.
pub fn gen_req_id() -> u32 {
    // Generate the number in range 1..MAX inclusive.
    rand::thread_rng().gen_range(0, u32::MAX) + 1
}
