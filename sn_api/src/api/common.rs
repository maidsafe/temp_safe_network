// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{constants::SAFE_AUTHD_CONNECTION_IDLE_TIMEOUT, Error, Result};
use log::info;
use qjsonrpc::ClientEndpoint;
use serde::de::DeserializeOwned;
use threshold_crypto::SecretKey;
use tokio::runtime;

pub mod auth_types {
    use safe_core::ipc::req::{ContainerPermissions, IpcReq};
    use safe_nd::AppPermissions;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    pub type SafeAuthReq = IpcReq;
    pub type SafeAuthReqId = u32;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AuthedApp {
        /// The App ID. It must be unique.
        pub id: String,
        /// The application friendly-name.
        pub name: String,
        /// The application provider/vendor (e.g. MaidSafe)
        pub vendor: String,
        /// Permissions granted, e.g. allowing to work with the user's coin balance.
        pub app_permissions: AppPermissions,
        /// Permissions granted to the app for named containers
        // TODO: ContainerPermissions will/shall be refactored to expose a struct defined in this crate
        pub containers: HashMap<String, ContainerPermissions>,
        /// If the app was given a dedicated named container for itself
        pub own_container: bool,
    }

    // Type of the list of authorised applications in a SAFE account
    pub type AuthedAppsList = Vec<AuthedApp>;
}

pub fn parse_hex(hex_str: &str) -> Vec<u8> {
    let mut hex_bytes = hex_str
        .as_bytes()
        .iter()
        .filter_map(|b| match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        })
        .fuse();

    let mut bytes = Vec::new();
    while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
        bytes.push(h << 4 | l)
    }
    bytes
}

pub fn sk_from_hex(hex_str: &str) -> Result<SecretKey> {
    let sk_bytes = parse_hex(&hex_str);
    bincode::deserialize(&sk_bytes)
        .map_err(|_| Error::InvalidInput("Failed to deserialize provided secret key".to_string()))
}

// Send a request to authd using JSON-RPC over QUIC
pub async fn send_authd_request<T>(
    dest_endpoint: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<T>
where
    T: DeserializeOwned,
{
    info!(
        "Sending '{}' request to SAFE Authenticator on {} ...",
        method, dest_endpoint
    );

    match directories::ProjectDirs::from("net", "maidsafe", "sn_authd") {
        None => Err(Error::AuthdClientError(
            "Failed to obtain local project directory where to read certificate from".to_string(),
        )),
        Some(dirs) => {
            let cert_base_path = dirs.config_dir().display().to_string();

            let qjsonrpc_client = ClientEndpoint::new(
                &cert_base_path,
                Some(SAFE_AUTHD_CONNECTION_IDLE_TIMEOUT),
                false,
            )
            .map_err(|err| {
                Error::AuthdClientError(format!("Failed to create client endpoint: {}", err))
            })?;

            // We try to obtain current runtime or create a new one if there is none
            let runtime = match runtime::Handle::try_current() {
                Ok(r) => r,
                Err(_) => runtime::Runtime::new()
                    .map_err(|err| {
                        Error::AuthdClientError(format!("Failed to create runtime: {}", err))
                    })?
                    .handle()
                    .clone(),
            };

            let mut outgoing_conn = {
                runtime.enter(|| qjsonrpc_client.bind()).map_err(|err| {
                    Error::AuthdClientError(format!("Failed to bind endpoint: {}", err))
                })?
            };

            // Establish a new connection
            outgoing_conn
                .connect(dest_endpoint, None)
                .await
                .map_err(|err| {
                    Error::AuthdClientError(format!(
                        "Failed to establish connection with authd: {}",
                        err
                    ))
                })?
                // Send request and await for response
                .send(method, params)
                .await
                .map_err(|err| match err {
                    qjsonrpc::Error::RemoteEndpointError(msg) => Error::AuthdError(msg),
                    other => Error::AuthdClientError(other.to_string()),
                })
        }
    }
}
