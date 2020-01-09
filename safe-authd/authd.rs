// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    notifs::monitor_pending_auth_reqs, requests::process_request, shared::*, Error, Result,
};
use futures::{Future, Stream};
use safe_api::SafeAuthenticator;
use slog::{Drain, Logger};
use std::{
    collections::BTreeMap,
    fs, io,
    net::ToSocketAddrs,
    str,
    sync::{Arc, Mutex},
};
use tokio::runtime::current_thread::Runtime;
use url::Url;

// Number of milliseconds to allow an idle connection before closing it
const CONNECTION_IDLE_TIMEOUT: u64 = 60_000;

pub fn run(
    listen: &str,
    cert_base_path: Option<&str>,
    config_dir_path: Option<&str>,
) -> Result<()> {
    let url = Url::parse(&listen)
        .map_err(|_| Error::GeneralError("Invalid endpoint address".to_string()))?;
    let endpoint_socket_addr = url
        .to_socket_addrs()
        .map_err(|_| Error::GeneralError("Invalid endpoint address".to_string()))?
        .next()
        .ok_or_else(|| Error::GeneralError("The endpoint is an invalid address".to_string()))?;

    let decorator = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let drain = slog_term::FullFormat::new(decorator)
        .use_original_order()
        .build()
        .fuse();
    let log = Logger::root(drain, o!());

    let server_config = quinn::ServerConfig {
        transport: Arc::new(quinn::TransportConfig {
            idle_timeout: CONNECTION_IDLE_TIMEOUT,
            stream_window_uni: 0,
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut server_config = quinn::ServerConfigBuilder::new(server_config);
    server_config.protocols(&[quinn::ALPN_QUIC_HTTP]);

    let base_path = match cert_base_path {
        Some(path) => path.to_string(),
        None => get_certificate_base_path().map_err(|err| Error::GeneralError(err.to_string()))?,
    };

    let cert_path = std::path::Path::new(&base_path).join("cert.der");
    let key_path = std::path::Path::new(&base_path).join("key.der");

    let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
        Ok(x) => x,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            info!(log, "Generating self-signed certificate...");
            let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]);
            let key = cert.serialize_private_key_der();
            let cert = cert.serialize_der();
            fs::create_dir_all(&std::path::Path::new(&base_path)).map_err(|err| {
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
    endpoint.logger(log.clone());

    endpoint.listen(server_config.build());

    let (endpoint_driver, incoming) = {
        let (driver, endpoint, incoming) = endpoint
            .bind(endpoint_socket_addr)
            .map_err(|err| Error::GeneralError(format!("Failed to bind endpoint: {}", err)))?;
        let local_addr = endpoint.local_addr()?;
        info!(log, "Listening on {}", local_addr);
        println!("Listening on {}", local_addr);
        (driver, incoming)
    };

    let safe_auth_handle: SharedSafeAuthenticatorHandle =
        Arc::new(Mutex::new(SafeAuthenticator::new(config_dir_path)));

    // We keep a queue for all the authorisation requests
    let auth_reqs_handle = Arc::new(Mutex::new(AuthReqsList::new()));

    // We keep a list of the notifications subscribers' endpoints
    let notif_endpoints_handle = Arc::new(Mutex::new(BTreeMap::new()));

    // Let's spawn a thread which will monitor pending auth reqs
    // and get them allowed/denied by the user using any of the subcribed endpoints
    // TODO: this can also be a Future with a Stream and schudule it as a task rather than having a thread
    monitor_pending_auth_reqs(auth_reqs_handle.clone(), notif_endpoints_handle.clone());

    // Finally let's spawn the task to handle the incoming connections
    let mut runtime = Runtime::new()?;
    runtime.spawn(incoming.for_each(move |conn| {
        handle_connection(
            safe_auth_handle.clone(),
            auth_reqs_handle.clone(),
            notif_endpoints_handle.clone(),
            &log,
            conn,
        );
        Ok(())
    }));
    println!("SAFE Authenticator services initialised sucessfully");
    runtime.block_on(endpoint_driver)?;

    Ok(())
}

// Private helpers

pub fn get_certificate_base_path() -> Result<String> {
    match directories::ProjectDirs::from("net", "maidsafe", "safe-authd") {
        Some(dirs) => Ok(dirs.config_dir().display().to_string()),
        None => Err(Error::GeneralError(
            "Failed to obtain local project directory where to write certificate from".to_string(),
        )),
    }
}

fn handle_connection(
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
    log: &Logger,
    conn: (
        quinn::ConnectionDriver,
        quinn::Connection,
        quinn::IncomingStreams,
    ),
) {
    let (conn_driver, conn, incoming_streams) = conn;
    let log = log.clone();
    info!(log, "Got connection";
          "remote_id" => %conn.remote_id(),
          "address" => %conn.remote_address(),
          "protocol" => conn.protocol().map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned()));
    let log2 = log.clone();

    // We ignore errors from the driver because they'll be reported by the `incoming` handler anyway.
    tokio_current_thread::spawn(conn_driver.map_err(|_| ()));

    // Each stream initiated by the client constitutes a new request.
    tokio_current_thread::spawn(
        incoming_streams
            .map_err(move |e| info!(log2, "Connection terminated"; "reason" => %e))
            .for_each(move |stream| {
                handle_request(
                    safe_auth_handle.clone(),
                    auth_reqs_handle.clone(),
                    notif_endpoints_handle.clone(),
                    &log,
                    stream,
                );
                Ok(())
            }),
    );
}

fn handle_request(
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
    log: &Logger,
    stream: quinn::NewStream,
) {
    let (send, recv) = match stream {
        quinn::NewStream::Bi(send, recv) => (send, recv),
        quinn::NewStream::Uni(_) => unreachable!("Disabled by endpoint configuration"),
    };
    let log = log.clone();
    let log2 = log.clone();
    let log3 = log.clone();

    tokio_current_thread::spawn(
        recv.read_to_end(64 * 1024) // Read the request, which must be at most 64KiB
            .map_err(|e| Error::GeneralError(format!("Failed reading request: {}", e)))
            .and_then(move |(_, req)| {
                info!(log, "Got request");
                // Execute the request
                process_request(
                    safe_auth_handle,
                    auth_reqs_handle,
                    notif_endpoints_handle,
                    req,
                )
                .and_then(|resp| {
                    // Write the response
                    tokio::io::write_all(send, resp)
                        .map_err(|e| Error::GeneralError(format!("Failed to send response: {}", e)))
                })
            })
            // Gracefully terminate the stream
            .and_then(|(send, _)| {
                tokio::io::shutdown(send)
                    .map_err(|e| Error::GeneralError(format!("Failed to shutdown stream: {}", e)))
            })
            .map(move |_| info!(log3, "Request complete"))
            .map_err(move |e| error!(log2, "Request Failed"; "reason" => %e)),
    )
}
