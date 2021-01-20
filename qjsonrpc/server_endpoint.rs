// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    jsonrpc::parse_jsonrpc_request, Error, JsonRpcRequest, JsonRpcResponse, Result, ALPN_QUIC_HTTP,
};
use crate::utils;
use futures::StreamExt;
use log::{debug, info, warn};
use std::{
    fs, io,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

// JSON-RPC over QUIC server endpoint
pub struct Endpoint {
    config: quinn::ServerConfig,
}

impl Endpoint {
    // cert_base_path: Base path where to locate custom certificate authority to trust, in DER format
    // idle_timeout: Optional number of millis before timing out an idle connection
    pub fn new<P: AsRef<Path>>(cert_base_path: P, idle_timeout: Option<u64>) -> Result<Self> {
        let mut server_config = quinn::ServerConfig::default();
        if let Some(timeout) = idle_timeout {
            server_config.transport = Arc::new(utils::new_transport_cfg(timeout)?)
        };

        let mut server_config = quinn::ServerConfigBuilder::new(server_config);
        server_config.protocols(ALPN_QUIC_HTTP);

        let mut cert_path = PathBuf::new();
        cert_path.push(&cert_base_path);
        cert_path.push("cert.der");

        let mut key_path = PathBuf::new();
        key_path.push(&cert_base_path);
        key_path.push("key.der");

        let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            Ok(x) => x,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("Generating self-signed certificate...");
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).map_err(
                    |err| {
                        Error::GeneralError(format!(
                            "Failed to generate self-signed certificate: {}",
                            err
                        ))
                    },
                )?;
                let key = cert.serialize_private_key_der();
                let cert = cert.serialize_der().map_err(|err| {
                    Error::GeneralError(format!("Failed to serialise certificate: {}", err))
                })?;
                fs::create_dir_all(cert_base_path).map_err(|err| {
                    Error::GeneralError(format!("Failed to create certificate directory: {}", err))
                })?;
                fs::write(&cert_path, &cert).map_err(|err| {
                    Error::GeneralError(format!("Failed to write certificate: {}", err))
                })?;
                fs::write(&key_path, &key).map_err(|err| {
                    Error::GeneralError(format!("Failed to write private key: {}", err))
                })?;
                (cert, key)
            }
            Err(e) => {
                return Err(Error::GeneralError(format!(
                    "Failed to read certificate: {}",
                    e
                )));
            }
        };
        let key = quinn::PrivateKey::from_der(&key).map_err(|err| {
            Error::GeneralError(format!("Failed parse private key from file: {}", err))
        })?;
        let cert = quinn::Certificate::from_der(&cert).map_err(|err| {
            Error::GeneralError(format!("Failed to parse certificate from file: {}", err))
        })?;
        server_config
            .certificate(quinn::CertificateChain::from_certs(vec![cert]), key)
            .map_err(|err| {
                Error::GeneralError(format!(
                    "Failed to set certificate for communication: {}",
                    err
                ))
            })?;

        let config = server_config.build();
        Ok(Self { config })
    }

    // Bind server endpoint to a socket address to start listening for connections on it
    pub fn bind(&self, listen_socket_addr: &SocketAddr) -> Result<IncomingConn> {
        let mut quinn_endpoint_builder = quinn::Endpoint::builder();
        quinn_endpoint_builder.listen(self.config.clone());

        let (_endpoint, incoming) = quinn_endpoint_builder
            .bind(&listen_socket_addr)
            .map_err(|err| Error::GeneralError(format!("Failed to bind QUIC endpoint: {}", err)))?;

        Ok(IncomingConn::new(incoming))
    }
}

// Stream of incoming QUIC connections
pub struct IncomingConn {
    quinn_incoming: quinn::Incoming,
}

impl IncomingConn {
    pub(crate) fn new(quinn_incoming: quinn::Incoming) -> Self {
        Self { quinn_incoming }
    }

    // Returns next QUIC connection established by a peer
    pub async fn get_next(&mut self) -> Option<IncomingJsonRpcRequest> {
        match self.quinn_incoming.next().await {
            Some(quinn_conn) => match quinn_conn.await {
                Ok(quinn::NewConnection { bi_streams, .. }) => {
                    Some(IncomingJsonRpcRequest::new(bi_streams))
                }
                Err(_err) => None,
            },
            None => None,
        }
    }
}

// Stream of incoming JSON-RPC request messages
pub struct IncomingJsonRpcRequest {
    bi_streams: quinn::IncomingBiStreams,
}

impl IncomingJsonRpcRequest {
    pub(crate) fn new(bi_streams: quinn::IncomingBiStreams) -> Self {
        Self { bi_streams }
    }

    // Returns next JSON-RPC request sent by the peer on current QUIC connection
    pub async fn get_next(&mut self) -> Option<(JsonRpcRequest, JsonRpcResponseStream)> {
        // Each stream initiated by the client constitutes a new request
        match self.bi_streams.next().await {
            None => None,
            Some(stream) => {
                let (send, recv): (quinn::SendStream, quinn::RecvStream) = match stream {
                    Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                        debug!("Connection terminated");
                        return None;
                    }
                    Err(err) => {
                        warn!("Failed to read incoming request: {}", err);
                        return None;
                    }
                    Ok(bi_stream) => bi_stream,
                };

                match recv
                    .read_to_end(64 * 1024) // Read the request's bytes, which must be at most 64KiB
                    .await
                {
                    Ok(req_bytes) => {
                        debug!("Got new request's bytes");
                        match parse_jsonrpc_request(req_bytes) {
                            Ok(jsonrpc_req) => {
                                debug!("Request parsed successfully");
                                Some((jsonrpc_req, JsonRpcResponseStream::new(send)))
                            }
                            Err(err) => {
                                warn!("Failed to parse request as JSON-RPC: {}", err);
                                None
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Failed reading request's bytes: {}", err);
                        None
                    }
                }
            }
        }
    }
}

// Stream of outgoing JSON-RPC responses
pub struct JsonRpcResponseStream {
    quinn_send_stream: quinn::SendStream,
}

impl JsonRpcResponseStream {
    pub(crate) fn new(quinn_send_stream: quinn::SendStream) -> Self {
        Self { quinn_send_stream }
    }

    // Write a JsonRpcResponse into the current connection's sending stream
    pub async fn respond(&mut self, response: &JsonRpcResponse) -> Result<()> {
        let serialised_res = serde_json::to_string(response).map_err(|err| {
            Error::GeneralError(format!("Failed to serialise response: {:?}", err))
        })?;

        self.quinn_send_stream
            .write_all(&serialised_res.into_bytes())
            .await
            .map_err(|err| {
                Error::GeneralError(format!(
                    "Failed to write entire buffer to response stream: {}",
                    err
                ))
            })
    }

    // Gracefully finish current connection's stream
    pub async fn finish(&mut self) -> Result<()> {
        self.quinn_send_stream.finish().await.map_err(|err| {
            Error::GeneralError(format!(
                "Failed to shutdown the response stream gracefully: {}",
                err
            ))
        })
    }
}
