// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::ALPN_QUIC_HTTP;
use futures::TryFutureExt;
use log::{debug, error, info};
use std::{fs, path::PathBuf, sync::Arc, time::Instant};
use tokio::runtime::Builder;
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
        .socket_addrs(|| None)
        .map_err(|_| "Invalid end point address".to_string())?[0];

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
    client_config.protocols(ALPN_QUIC_HTTP);

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

    let mut runtime = Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .map_err(|err| format!("Unexpected error setting up client endpoint: {}", err))?;
    let sock_addr = "[::]:0"
        .parse()
        .map_err(|_| "Invalid endpoint address".to_string())?;
    let (endpoint_driver, endpoint, _) = runtime
        .enter(|| endpoint.bind(&sock_addr))
        .map_err(|err| format!("Failed to bind client endpoint: {}", err))?;

    let handle = runtime.spawn(endpoint_driver.unwrap_or_else(|e| error!("IO error: {}", e)));

    let start = Instant::now();
    let host = cert_host
        .as_ref()
        .map_or_else(|| url.host_str(), |x| Some(&x))
        .ok_or_else(|| "No hostname specified".to_string())?;

    let response: Result<Vec<u8>, String> = runtime.block_on(async {
        let new_conn = endpoint
            .connect(&remote, &host)
            .map_err(|err| format!("Failed to establish connection with authd: {}", err))?
            .await
            .map_err(|err| format!("Failed to establish connection with authd: {}", err))?;

        debug!("Connected with authd at {:?}", start.elapsed());
        let quinn::NewConnection {
            driver,
            connection: conn,
            ..
        } = { new_conn };

        tokio::spawn(driver.unwrap_or_else(|e| eprintln!("Connection lost: {}", e)));
        let (mut send, recv) = conn
            .open_bi()
            .await
            .map_err(|e| format!("Failed to open stream: {}", e))?;
        if rebind {
            let socket = std::net::UdpSocket::bind("[::]:0").unwrap();
            let addr = socket.local_addr().unwrap();
            info!("Rebinding to {}", addr);
            endpoint
                .rebind(socket)
                .map_err(|err| format!("Rebind failed: {}", err))?;
        }

        send.write_all(request.as_bytes())
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;
        send.finish()
            .await
            .map_err(|e| format!("Failed to shutdown stream: {}", e))?;

        let response_start = Instant::now();
        debug!("Request sent at {:?}", response_start - start);
        let data = recv
            .read_to_end(usize::max_value())
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let duration = response_start.elapsed();
        let duration_secs = duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9;
        info!(
            "Response received from authd in {:?} - {} KiB/s",
            duration,
            data.len() as f32 / (duration_secs * 1024.0)
        );
        conn.close(0u32.into(), b"");
        Ok(data)
    });

    let received_bytes =
        response.map_err(|err| format!("Failed to obtain the response data: {}", err))?;

    // Allow the endpoint driver to automatically shut down
    drop(endpoint);

    // Let the connection finish closing gracefully
    runtime.block_on(handle).unwrap();

    Ok(received_bytes)
}
