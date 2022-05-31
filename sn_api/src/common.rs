// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{constants::SN_AUTHD_CONNECTION_IDLE_TIMEOUT, Error, Result};
use log::info;
use qjsonrpc::ClientEndpoint;
use serde::de::DeserializeOwned;
use std::path::Path;
use tokio::runtime;

pub mod auth_types {
    use crate::ipc::req::IpcReq;
    use serde::{Deserialize, Serialize};

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
    }

    // Type of the list of authorised applications in a Safe
    pub type AuthedAppsList = Vec<AuthedApp>;
}

/// Send a request to authd using JSON-RPC over QUIC.
pub async fn send_authd_request<T>(
    cert_path: &Path,
    dst_endpoint: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<T>
where
    T: DeserializeOwned,
{
    info!(
        "Sending '{}' request to SAFE Authenticator on {} ...",
        method, dst_endpoint
    );

    let qjsonrpc_client =
        ClientEndpoint::new(&cert_path, Some(SN_AUTHD_CONNECTION_IDLE_TIMEOUT), false).map_err(
            |err| Error::AuthdClientError(format!("Failed to create client endpoint: {}", err)),
        )?;

    let runtime = match runtime::Handle::try_current() {
        Ok(r) => r,
        Err(_) => runtime::Runtime::new()
            .map_err(|err| Error::AuthdClientError(format!("Failed to create runtime: {}", err)))?
            .handle()
            .clone(),
    };
    let mut outgoing_conn = {
        let _ = runtime.enter();
        qjsonrpc_client
            .bind()
            .map_err(|err| Error::AuthdClientError(format!("Failed to bind endpoint: {}", err)))?
    };

    outgoing_conn
        .connect(dst_endpoint, None)
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
