// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    notifs::monitor_pending_auth_reqs, requests::process_jsonrpc_request, shared::*, Error, Result,
};
use futures::{lock::Mutex, StreamExt, TryFutureExt};
use jsonrpc_quic::parse_jsonrpc_request;
use safe_api::SafeAuthenticator;
use std::{collections::BTreeMap, fs, io, net::SocketAddr, str, sync::Arc};
use tokio::runtime::Builder;
use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;
use url::Url;

const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-24"];

// Number of milliseconds to allow an idle connection before closing it
const CONNECTION_IDLE_TIMEOUT: u64 = 60_000;

pub fn run(
    listen: &str,
    cert_base_path: Option<&str>,
    config_dir_path: Option<&str>,
) -> Result<()> {
    let safe_auth_handle: SharedSafeAuthenticatorHandle =
        Arc::new(Mutex::new(SafeAuthenticator::new(config_dir_path)));

    // We keep a queue for all the authorisation requests
    let auth_reqs_handle = Arc::new(Mutex::new(AuthReqsList::new()));

    // We keep a list of the notifications subscribers' endpoints
    let notif_endpoints_handle = Arc::new(Mutex::new(BTreeMap::new()));

    // Let's spawn a task which will monitor pending auth reqs
    // and get them allowed/denied by the user using any of the subcribed endpoints
    let auth_reqs_handle2 = auth_reqs_handle.clone();
    let notif_endpoints_handle2 = notif_endpoints_handle.clone();
    let runtime = Builder::new().threaded_scheduler().enable_all().build()?;
    runtime.spawn(async move {
        monitor_pending_auth_reqs(auth_reqs_handle2, notif_endpoints_handle2).await;
    });

    let base_path = cert_base_path.map_or_else(
        || match directories::ProjectDirs::from("net", "maidsafe", "safe-authd") {
            Some(dirs) => Ok(dirs.config_dir().display().to_string()),
            None => Err(Error::GeneralError(
                "Failed to obtain local project directory where to write certificate from"
                    .to_string(),
            )),
        },
        |path| Ok(path.to_string()),
    )?;

    jsonrpc_listen(
        listen,
        &base_path,
        Some(CONNECTION_IDLE_TIMEOUT),
        safe_auth_handle,
        auth_reqs_handle,
        notif_endpoints_handle,
    )
    .map_err(|err| Error::GeneralError(err.to_string()))
}

// TODO: move to jsonrpc-quic crate
fn jsonrpc_listen(
    listen: &str,
    cert_base_path: &str,
    idle_timeout: Option<u64>,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<()> {
    let listen2 = listen.to_string();
    let cert_base_path2 = cert_base_path.to_string();
    let endpoint_socket_addr = Url::parse(&listen2)
        .map_err(|_| Error::GeneralError("Invalid endpoint address".to_string()))?
        .socket_addrs(|| None)
        .map_err(|_| Error::GeneralError("Invalid endpoint address".to_string()))?[0];
    start_quic_endpoint(
        endpoint_socket_addr,
        &cert_base_path2,
        idle_timeout,
        safe_auth_handle,
        auth_reqs_handle,
        notif_endpoints_handle,
    )
}

fn start_quic_endpoint(
    listen_socket_addr: SocketAddr,
    cert_base_path: &str,
    idle_timeout: Option<u64>,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<()> {
    let server_config = quinn::ServerConfig {
        transport: Arc::new(quinn::TransportConfig {
            idle_timeout: idle_timeout.unwrap_or_else(|| 0),
            stream_window_uni: 0,
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut server_config = quinn::ServerConfigBuilder::new(server_config);
    server_config.protocols(ALPN_QUIC_HTTP);

    let cert_path = std::path::Path::new(&cert_base_path).join("cert.der");
    let key_path = std::path::Path::new(&cert_base_path).join("key.der");
    let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
        Ok(x) => x,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            info!("Generating self-signed certificate...");
            let cert =
                rcgen::generate_simple_self_signed(vec!["localhost".into()]).map_err(|err| {
                    Error::GeneralError(format!(
                        "Failed to generate self-signed certificate: {}",
                        err
                    ))
                })?;
            let key = cert.serialize_private_key_der();
            let cert = cert.serialize_der().map_err(|err| {
                Error::GeneralError(format!("Failed to serialise certificate: {}", err))
            })?;
            fs::create_dir_all(&std::path::Path::new(&cert_base_path)).map_err(|err| {
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

    let mut endpoint = quinn::Endpoint::builder();
    endpoint.listen(server_config.build());

    let mut runtime = Builder::new().threaded_scheduler().enable_all().build()?;
    let (endpoint_driver, mut incoming) = {
        let (driver, endpoint, incoming) = runtime
            .enter(|| endpoint.bind(&listen_socket_addr))
            .map_err(|err| Error::GeneralError(format!("Failed to bind endpoint: {}", err)))?;
        info!("Listening on {}", endpoint.local_addr()?);
        (driver, incoming)
    };

    runtime.spawn(async move {
        while let Some(conn) = incoming.next().await {
            tokio::spawn({
                handle_connection(
                    conn,
                    safe_auth_handle.clone(),
                    auth_reqs_handle.clone(),
                    notif_endpoints_handle.clone(),
                )
                .unwrap_or_else(move |e| error!("{reason}", reason = e.to_string()))
            });
        }
    });
    runtime.block_on(endpoint_driver)?;

    Ok(())
}

// Private helpers

async fn handle_connection(
    conn: quinn::Connecting,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<()> {
    let quinn::NewConnection {
        driver,
        connection,
        mut bi_streams,
        ..
    } = conn
        .await
        .map_err(|e| Error::GeneralError(format!("Failed to stablish connection: {}", e)))?;

    let span = info_span!("Connection stablished",
          remote_id = %connection.remote_id(),
          address = %connection.remote_address(),
          protocol = %connection.protocol().map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned()));

    tokio::spawn(driver.unwrap_or_else(|_| ()).instrument(span.clone()));

    async {
        // Each stream initiated by the client constitutes a new request.
        while let Some(stream) = bi_streams.next().await {
            let stream = match stream {
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("Connection terminated");
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(s) => s,
            };

            tokio::spawn(
                handle_request(
                    stream,
                    safe_auth_handle.clone(),
                    auth_reqs_handle.clone(),
                    notif_endpoints_handle.clone(),
                )
                .unwrap_or_else(move |e| error!("{reason}", reason = e.to_string())),
            );
        }
        Ok(())
    }
    .instrument(span)
    .await
    .map_err(|e| Error::GeneralError(format!("Failed to stablish connection: {}", e)))?;

    Ok(())
}

async fn handle_request(
    (mut send, recv): (quinn::SendStream, quinn::RecvStream),
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<()> {
    let req = recv
        .read_to_end(64 * 1024) // Read the request, which must be at most 64KiB
        .await
        .map_err(|e| Error::GeneralError(format!("Failed reading request: {}", e)))?;

    info!("Got new request");

    // Execute the request
    let resp = process_request(
        req,
        safe_auth_handle,
        auth_reqs_handle,
        notif_endpoints_handle,
    )
    .await?;

    // Write the response
    send.write_all(&resp)
        .await
        .map_err(|e| Error::GeneralError(format!("Failed to send response: {}", e)))?;

    // Gracefully terminate the stream
    send.finish()
        .await
        .map_err(|e| Error::GeneralError(format!("Failed to shutdown stream: {}", e)))?;

    info!("Request complete");
    Ok(())
}

async fn process_request(
    req: Vec<u8>,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<Box<[u8]>> {
    let jsonrpc_req = parse_jsonrpc_request(req)
        .map_err(|e| Error::GeneralError(format!("Failed to parse request as JSON-RPC: {}", e)))?;

    process_jsonrpc_request(
        jsonrpc_req,
        safe_auth_handle,
        auth_reqs_handle,
        notif_endpoints_handle,
    )
    .await
}
