// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::notifs::monitor_pending_auth_reqs;
use super::requests::process_request;
use super::shared::*;
use failure::{Error, Fail, ResultExt};
use futures::{Future, Stream};
use safe_api::SafeAuthenticator;
use slog::{Drain, Logger};
use std::collections::BTreeSet;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::{ascii, fmt, fs, str};
use tokio::runtime::current_thread::Runtime;

// Number of milliseconds to allow an idle connection before closing it
const CONNECTION_IDLE_TIMEOUT: u64 = 60_000;

pub struct PrettyErr<'a>(&'a dyn Fail);
impl<'a> fmt::Display for PrettyErr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?;
        let mut x: &dyn Fail = self.0;
        while let Some(cause) = x.cause() {
            f.write_str(": ")?;
            fmt::Display::fmt(&cause, f)?;
            x = cause;
        }
        Ok(())
    }
}

pub trait ErrorExt {
    fn pretty(&self) -> PrettyErr<'_>;
}

impl ErrorExt for Error {
    fn pretty(&self) -> PrettyErr<'_> {
        PrettyErr(self.as_fail())
    }
}

pub fn run(listen: SocketAddr) -> Result<(), Error> {
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

    /*if options.keylog {
        server_config.enable_keylog();
    }

    if options.stateless_retry {
        server_config.use_stateless_retry(true);
    }*/

    /*if let (Some(ref key_path), Some(ref cert_path)) = (options.key, options.cert) {
        let key = fs::read(key_path).context("Failed to read private key")?;
        let key = if key_path.extension().map_or(false, |x| x == "der") {
            quinn::PrivateKey::from_der(&key)?
        } else {
            quinn::PrivateKey::from_pem(&key)?
        };
        let cert_chain = fs::read(cert_path).context("Failed to read certificate chain")?;
        let cert_chain = if cert_path.extension().map_or(false, |x| x == "der") {
            quinn::CertificateChain::from_certs(quinn::Certificate::from_der(&cert_chain))
        } else {
            quinn::CertificateChain::from_pem(&cert_chain)?
        };
        server_config.certificate(cert_chain, key)?;
    } else {*/
    let dirs = match directories::ProjectDirs::from("net", "maidsafe", "authd") {
        Some(dirs) => dirs,
        None => bail!("Failed to obtain local home directory where to read certificate from"),
    };
    let path = dirs.data_local_dir();
    let cert_path = path.join("cert.der");
    let key_path = path.join("key.der");
    let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
        Ok(x) => x,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            info!(log, "Generating self-signed certificate...");
            let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]);
            let key = cert.serialize_private_key_der();
            let cert = cert.serialize_der();
            fs::create_dir_all(&path).context("Failed to create certificate directory")?;
            fs::write(&cert_path, &cert).context("Failed to write certificate")?;
            fs::write(&key_path, &key).context("Failed to write private key")?;
            (cert, key)
        }
        Err(e) => {
            bail!("Failed to read certificate: {}", e);
        }
    };
    let key = quinn::PrivateKey::from_der(&key)?;
    let cert = quinn::Certificate::from_der(&cert)?;
    server_config.certificate(quinn::CertificateChain::from_certs(vec![cert]), key)?;
    //}

    let mut endpoint = quinn::Endpoint::builder();
    endpoint.logger(log.clone());

    endpoint.listen(server_config.build());

    let (endpoint_driver, incoming) = {
        let (driver, endpoint, incoming) = endpoint.bind(listen)?;
        info!(log, "Listening on {}", endpoint.local_addr()?);
        (driver, incoming)
    };

    let safe_auth_handle: SharedSafeAuthenticatorHandle =
        Arc::new(Mutex::new(SafeAuthenticator::new()));

    // We keep a queue for all the authorisation requests
    let auth_reqs_handle = Arc::new(Mutex::new(AuthReqsList::new()));

    // We keep a list of the notifications subscriptors' endpoints
    let notif_endpoints_handle = Arc::new(Mutex::new(BTreeSet::new()));

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
    let safe_auth_handle = safe_auth_handle.clone();
    let auth_reqs_handle = auth_reqs_handle.clone();
    let notif_endpoints_handle = notif_endpoints_handle.clone();
    let log = log.clone();
    let log2 = log.clone();
    let log3 = log.clone();

    tokio_current_thread::spawn(
        recv.read_to_end(64 * 1024) // Read the request, which must be at most 64KiB
            .map_err(|e| format_err!("Failed reading request: {}", e))
            .and_then(move |(_, req)| {
                let mut escaped = String::new();
                for &x in &req[..] {
                    let part = ascii::escape_default(x).collect::<Vec<_>>();
                    escaped.push_str(str::from_utf8(&part).unwrap());
                }
                info!(log, "Got request");
                // Execute the request
                process_request(
                    safe_auth_handle,
                    auth_reqs_handle,
                    notif_endpoints_handle,
                    &req,
                )
                .and_then(|resp| {
                    // Write the response
                    tokio::io::write_all(send, resp)
                        .map_err(|e| format_err!("Failed to send response: {}", e))
                })
            })
            // Gracefully terminate the stream
            .and_then(|(send, _)| {
                tokio::io::shutdown(send)
                    .map_err(|e| format_err!("Failed to shutdown stream: {}", e))
            })
            .map(move |_| info!(log3, "Request complete"))
            .map_err(move |e| error!(log2, "Request Failed"; "reason" => %e.pretty())),
    )
}
