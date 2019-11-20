// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use futures::Future;
use log::{debug, error, info};
use std::fs;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use tokio::runtime::current_thread::Runtime;
use url::Url;

// QUIC client
// url_str: QUIC destination endpoint URL
// request: Request payload
// keylog: Perform NSS-compatible TLS key logging to the file specified in `SSLKEYLOGFILE`
// cert_host: Override hostname used for certificate verification
// cert_ca: Custom certificate authority to trust, in DER format
// rebind: Simulate NAT rebinding after connecting
// timeout: Optional number of millis before timing out an idle connection
pub fn quic_send(
    url_str: &str,
    request: &str,
    keylog: bool,
    cert_host: Option<&str>,
    cert_ca: Option<&str>,
    rebind: bool,
    timeout: Option<u64>,
) -> Result<Vec<u8>, String> {
    let url = Url::parse(url_str).map_err(|_| "Invalid end point address".to_string())?;
    let remote = url
        .to_socket_addrs()
        .map_err(|_| "Invalid end point address".to_string())?
        .next()
        .ok_or_else(|| "The end point is an invalid address".to_string())?;

    let client_config = if let Some(idle_timeout) = timeout {
        quinn::ClientConfig {
            transport: Arc::new(quinn::TransportConfig {
                idle_timeout,
                ..Default::default()
            }),
            ..Default::default()
        }
    } else {
        quinn::ClientConfig::default()
    };

    let mut client_config = quinn::ClientConfigBuilder::new(client_config);
    client_config.protocols(&[quinn::ALPN_QUIC_HTTP]);

    if keylog {
        client_config.enable_keylog();
    }

    let ca_path = if let Some(ca_path) = cert_ca {
        PathBuf::from(ca_path).join("cert.der")
    } else {
        let dirs = match directories::ProjectDirs::from("net", "maidsafe", "safe-authd-client") {
            Some(dirs) => dirs,
            None => {
                return Err(
                    "Failed to obtain local home directory where to read certificate from"
                        .to_string(),
                )
            }
        };
        dirs.data_local_dir().join("cert.der")
    };

    let ca_certificate = fs::read(&ca_path).map_err(|err| {
        format!(
            "Failed to read certificate from '{}': {}",
            ca_path.display(),
            err
        )
    })?;
    let ca_authority = quinn::Certificate::from_der(&ca_certificate).map_err(|err| {
        format!(
            "Failed to obtain CA authority from certificate found at '{}': {}",
            ca_path.display(),
            err
        )
    })?;
    client_config
        .add_certificate_authority(ca_authority)
        .map_err(|err| {
            format!(
                "Failed to add CA authority to QUIC client configuration: {}",
                err
            )
        })?;

    let mut endpoint = quinn::Endpoint::builder();
    endpoint.default_client_config(client_config.build());

    let (endpoint_driver, endpoint, _) = endpoint
        .bind("[::]:0")
        .map_err(|err| format!("Failed to bind client endpoint: {}", err))?;
    let mut runtime = Runtime::new()
        .map_err(|err| format!("Unexpected error setting up client endpoint: {}", err))?;
    runtime.spawn(endpoint_driver.map_err(|e| error!("IO error: {}", e)));

    let start = Instant::now();
    let host = cert_host
        .as_ref()
        .map_or_else(|| url.host_str(), |x| Some(&x))
        .ok_or_else(|| "No hostname specified".to_string())?;

    let (tx, rx) = mpsc::channel();
    runtime.block_on(
        endpoint
            .connect(&remote, &host)
            .map_err(|err| format!("{}", err))?
            .map_err(|err| format!("Failed to establish connection with authd: {}", err))
            .and_then(move |(conn_driver, conn, _)| {
                debug!("Connected with authd at {:?}", start.elapsed());
                tokio_current_thread::spawn(
                    conn_driver.map_err(|e| error!("Connection lost: {}", e)),
                );
                conn.open_bi()
                    .map_err(|e| format!("Failed to open stream: {}", e))
                    .and_then(move |(send, recv)| {
                        if rebind {
                            let socket = std::net::UdpSocket::bind("[::]:0").unwrap();
                            let addr = socket.local_addr().unwrap();
                            info!("Rebinding to {}", addr);
                            let _ = endpoint
                                .rebind(socket, &tokio_reactor::Handle::default())
                                .map_err(|err| format!("Rebind failed: {}", err));
                        }

                        tokio::io::write_all(send, request.as_bytes().to_owned())
                            .map_err(|e| format!("Failed to send request: {}", e))
                            .and_then(|(send, _)| {
                                tokio::io::shutdown(send)
                                    .map_err(|e| format!("Failed to shutdown stream: {}", e))
                            })
                            .and_then(move |_| {
                                let response_start = Instant::now();
                                debug!("Request sent at {:?}", response_start - start);
                                recv.read_to_end(usize::max_value())
                                    .map_err(|e| format!("Failed to read response: {}", e))
                                    .map(move |x| (x, response_start))
                            })
                    })
                    .map(move |((_, data), response_start)| {
                        let duration = response_start.elapsed();
                        info!(
                            "Response received from authd in {:?} - {} KiB/s",
                            duration,
                            data.len() as f32 / (duration_secs(&duration) * 1024.0)
                        );
                        let _ = tx
                            .send(data)
                            .map_err(|err| error!("Failed to obtain the response data: {}", err));
                        conn.close(0, b"");
                    })
            }),
    )?;

    let received_bytes = rx
        .try_recv()
        .map_err(|err| format!("Failed to obtain the response data: {}", err))?;

    // Let the connection to finish closing gracefully
    runtime
        .run()
        .map_err(|err| format!("Failed to connect with authd: {}", err))?;

    Ok(received_bytes)
}

fn duration_secs(x: &Duration) -> f32 {
    x.as_secs() as f32 + x.subsec_nanos() as f32 * 1e-9
}
