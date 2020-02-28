// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{constants::SAFE_AUTHD_CONNECTION_IDLE_TIMEOUT, Error, Result};
use jsonrpc_quic::ClientEndpoint;
use serde::de::DeserializeOwned;
use threshold_crypto::SecretKey;
use tokio::runtime::Builder;

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

pub fn send_authd_request<T>(
    dest_endpoint: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<T>
where
    T: DeserializeOwned,
{
    match directories::ProjectDirs::from("net", "maidsafe", "safe-authd") {
        None => Err(Error::AuthdClientError(
            "Failed to obtain local project directory where to read certificate from".to_string(),
        )),
        Some(dirs) => {
            let cert_base_path = dirs.config_dir().display().to_string();

            let jsonrpc_quic_client = ClientEndpoint::new(
                &cert_base_path,
                Some(SAFE_AUTHD_CONNECTION_IDLE_TIMEOUT),
                false,
            )
            .map_err(|err| {
                Error::AuthdClientError(format!("Failed to create client endpoint: {}", err))
            })?;

            let mut runtime = Builder::new()
                .threaded_scheduler()
                .enable_all()
                .build()
                .map_err(|err| {
                    Error::AuthdClientError(format!("Failed to create runtime: {}", err))
                })?;

            let (endpoint_driver, mut outgoing_conn) = {
                runtime
                    .enter(|| jsonrpc_quic_client.bind())
                    .map_err(|err| {
                        Error::AuthdClientError(format!("Failed to bind endpoint: {}", err))
                    })?
            };

            let _handle = runtime.spawn(endpoint_driver);

            runtime.block_on(async {
                let (driver, mut new_conn) = outgoing_conn
                    .connect(dest_endpoint, None)
                    .await
                    .map_err(|err| {
                        Error::AuthdClientError(format!(
                            "Failed to establish connection with authd: {}",
                            err
                        ))
                    })?;

                tokio::spawn(driver);

                let res = new_conn
                    .send(method, params)
                    .await
                    .map_err(|err| match err {
                        jsonrpc_quic::Error::RemoteEndpointError(msg) => Error::AuthdError(msg),
                        other => Error::AuthdClientError(other.to_string()),
                    });

                // Allow the endpoint driver to automatically shut down
                drop(outgoing_conn);

                res
            })
        }
    }
}
