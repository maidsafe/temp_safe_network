// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{jsonrpc::parse_jsonrpc_response, Error, JsonRpcRequest, Result, ALPN_QUIC_HTTP};
use crate::utils;
use log::debug;
use serde::de::DeserializeOwned;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use url::Url;

// JSON-RPC over QUIC client endpoint
pub struct ClientEndpoint {
    config: quinn::ClientConfig,
}

impl ClientEndpoint {
    // cert_base_path: Base path where to locate custom certificate authority to trust, in DER format
    // idle_timeout: Optional number of millis before timing out an idle connection
    // keylog: Perform NSS-compatible TLS key logging to the file specified in `SSLKEYLOGFILE`
    pub fn new<P: AsRef<Path>>(
        cert_base_path: P,
        idle_timeout: Option<u64>,
        keylog: bool,
    ) -> Result<Self> {
        let mut client_config = quinn::ClientConfigBuilder::default();
        client_config.protocols(ALPN_QUIC_HTTP);

        if keylog {
            client_config.enable_keylog();
        }

        let mut ca_path = PathBuf::new();
        ca_path.push(cert_base_path);
        ca_path.push("cert.der");

        let ca_certificate = fs::read(&ca_path).map_err(|err| {
            Error::ClientError(format!(
                "Failed to read certificate from '{}': {}",
                ca_path.display(),
                err
            ))
        })?;
        let ca_authority = quinn::Certificate::from_der(&ca_certificate).map_err(|err| {
            Error::ClientError(format!(
                "Failed to obtain CA authority from certificate found at '{}': {}",
                ca_path.display(),
                err
            ))
        })?;

        client_config
            .add_certificate_authority(ca_authority)
            .map_err(|err| {
                Error::ClientError(format!(
                    "Failed to add CA authority to QUIC client configuration: {}",
                    err
                ))
            })?;

        let mut config = client_config.build();
        if let Some(timeout) = idle_timeout {
            config.transport = Arc::new(utils::new_transport_cfg(timeout)?)
        };
        Ok(Self { config })
    }

    pub fn bind(&self) -> Result<OutgoingConn> {
        let mut quinn_endpoint_builder = quinn::Endpoint::builder();
        quinn_endpoint_builder.default_client_config(self.config.clone());

        let socket_addr = "[::]:0".parse().map_err(|err| {
            Error::ClientError(format!("Failed to parse client endpoint address: {}", err))
        })?;

        let (endpoint, _) = quinn_endpoint_builder.bind(&socket_addr).map_err(|err| {
            Error::ClientError(format!("Failed to bind client endpoint: {}", err))
        })?;

        Ok(OutgoingConn::new(endpoint))
    }
}

// Outgoing QUIC connections
pub struct OutgoingConn {
    pub quinn_endpoint: quinn::Endpoint,
}

impl OutgoingConn {
    pub(crate) fn new(quinn_endpoint: quinn::Endpoint) -> Self {
        Self { quinn_endpoint }
    }

    // Connect to a remote peer to send JSON-RPC requests
    // dest_endpoint: QUIC destination endpoint URL
    // cert_host: Override hostname used for certificate verification
    pub async fn connect(
        &mut self,
        dest_endpoint: &str,
        cert_host: Option<&str>,
    ) -> Result<OutgoingJsonRpcRequest> {
        let start = Instant::now();
        let url = Url::parse(dest_endpoint).map_err(|_| {
            Error::ClientError("Failed to parse remote end point address".to_string())
        })?;
        let remote = url
            .socket_addrs(|| None)
            .map_err(|_| Error::ClientError("Invalid remote end point address".to_string()))?[0];
        let host = cert_host
            .as_ref()
            .map_or_else(|| url.host_str(), |x| Some(&x))
            .ok_or_else(|| Error::ClientError("No certificate hostname specified".to_string()))?;

        let new_conn = self
            .quinn_endpoint
            .connect(&remote, &host)
            .map_err(|err| {
                Error::ClientError(format!(
                    "Failed when attempting to create a connection with remote QUIC endpoint: {}",
                    err
                ))
            })?
            .await
            .map_err(|err| {
                Error::ClientError(format!(
                    "Failed to establish connection with remote QUIC endpoint: {}",
                    err
                ))
            })?;

        debug!(
            "Connected with remote QUIC endpoint at {:?}",
            start.elapsed()
        );
        let quinn::NewConnection {
            connection: conn, ..
        } = { new_conn };

        Ok(OutgoingJsonRpcRequest::new(conn))
    }
}

// Stream of outgoing JSON-RPC request messages
pub struct OutgoingJsonRpcRequest {
    quinn_connection: quinn::Connection,
}

impl OutgoingJsonRpcRequest {
    pub(crate) fn new(quinn_connection: quinn::Connection) -> Self {
        Self { quinn_connection }
    }

    // Send a JSON_RPC request to the remote peer on current QUIC connection,
    // awaiting for a JSON-RPC response which result is of type T
    // method: JSON-RPC request method
    // params: JSON-RPC request params
    pub async fn send<T>(&mut self, method: &str, params: serde_json::Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let (mut send, recv) = self.quinn_connection.open_bi().await.map_err(|err| {
            Error::ClientError(format!("Failed to open communication stream: {}", err))
        })?;

        let jsonrpc_req = JsonRpcRequest::new(method, params);

        let serialised_req = serde_json::to_string(&jsonrpc_req).map_err(|err| {
            Error::ClientError(format!("Failed to serialise request to be sent: {}", err))
        })?;

        // Send request over QUIC, and await for JSON-RPC response
        send.write_all(serialised_req.as_bytes())
            .await
            .map_err(|err| Error::ClientError(format!("Failed to send request: {}", err)))?;

        send.finish().await.map_err(|err| {
            Error::ClientError(format!(
                "Failed to gracefully shutdown communication stream: {}",
                err
            ))
        })?;

        debug!("Request sent to remote endpoint");
        let received_bytes = recv
            .read_to_end(usize::max_value())
            .await
            .map_err(|err| Error::ClientError(format!("Response not received: {}", err)))?;

        self.quinn_connection.close(0u32.into(), b"");

        parse_jsonrpc_response(received_bytes.as_slice())
    }
}
