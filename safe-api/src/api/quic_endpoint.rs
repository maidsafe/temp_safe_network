// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{AuthReq, SafeAuthReqId};
use failure::{Error, Fail, ResultExt};
use futures::{Future, Stream};
use log::debug;
use safe_nd::AppPermissions;
use slog::{Drain, Logger};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::{ascii, fmt, fs, str};
use tokio::runtime::current_thread::Runtime;
use url::Url;

struct PrettyErr<'a>(&'a dyn Fail);
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

trait ErrorExt {
    fn pretty(&self) -> PrettyErr<'_>;
}

impl ErrorExt for Error {
    fn pretty(&self) -> PrettyErr<'_> {
        PrettyErr(self.as_fail())
    }
}

pub fn quic_listen(
    url_str: &str,
    notif_channel: mpsc::Sender<AuthReq>,
    cert_base_path: PathBuf,
) -> Result<(), String> {
    debug!("Launching new QUIC endpoint on '{}'", url_str);

    let decorator = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let drain = slog_term::FullFormat::new(decorator)
        .use_original_order()
        .build()
        .fuse();

    let url = Url::parse(url_str).map_err(|err| format!("Invalid end point address: {}", err))?;
    let endpoint = url
        .to_socket_addrs()
        .map_err(|err| format!("Invalid end point address: {}", err))?
        .next()
        .ok_or_else(|| "The end point is an invalid address".to_string())?;

    if let Err(e) = start_quic_endpoint(
        Logger::root(drain, o!()),
        endpoint,
        notif_channel,
        cert_base_path,
    ) {
        Err(format!("{}", e.pretty()))
    } else {
        Ok(())
    }
}

fn start_quic_endpoint(
    log: Logger,
    listen: SocketAddr,
    notif_channel: mpsc::Sender<AuthReq>,
    cert_base_path: PathBuf,
) -> Result<(), Error> {
    let server_config = quinn::ServerConfig {
        transport: Arc::new(quinn::TransportConfig {
            stream_window_uni: 0,
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut server_config = quinn::ServerConfigBuilder::new(server_config);
    server_config.protocols(&[quinn::ALPN_QUIC_HTTP]);

    let cert_path = cert_base_path.join("cert.der");
    let key_path = cert_base_path.join("key.der");
    let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
        Ok(x) => x,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            // info!(log, "Generating self-signed certificate...");
            let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]);
            let key = cert.serialize_private_key_der();
            let cert = cert.serialize_der();
            fs::create_dir_all(&cert_base_path)
                .context("Failed to create certificate directory")?;
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

    let mut endpoint = quinn::Endpoint::builder();
    endpoint.logger(log.clone());
    endpoint.listen(server_config.build());

    let (endpoint_driver, incoming) = {
        let (driver, _endpoint, incoming) = endpoint.bind(listen)?;
        // info!(log, "Listening on {}", endpoint.local_addr()?);
        (driver, incoming)
    };

    let mut runtime = Runtime::new()?;
    runtime.spawn(incoming.for_each(move |conn| {
        handle_connection(&log, conn, notif_channel.clone());
        Ok(())
    }));
    runtime.block_on(endpoint_driver)?;

    Ok(())
}

fn handle_connection(
    log: &Logger,
    conn: (
        quinn::ConnectionDriver,
        quinn::Connection,
        quinn::IncomingStreams,
    ),
    notif_channel: mpsc::Sender<AuthReq>,
) {
    let (conn_driver, _conn, incoming_streams) = conn;
    let log = log.clone();
    // info!(log, "got connection";
    //      "remote_id" => %conn.remote_id(),
    //      "address" => %conn.remote_address(),
    //      "protocol" => conn.protocol().map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned()));
    //let log2 = log.clone();

    // We ignore errors from the driver because they'll be reported by the `incoming` handler anyway.
    tokio_current_thread::spawn(conn_driver.map_err(|_| ()));

    // Each stream initiated by the client constitutes a new request.
    tokio_current_thread::spawn(
        incoming_streams
            .map_err(move |_e| ()) // info!(log2, "Connection terminated"; "reason" => %e))
            .for_each(move |stream| {
                handle_request(&log, stream, notif_channel.clone());
                Ok(())
            }),
    );
}

fn handle_request(_log: &Logger, stream: quinn::NewStream, notif_channel: mpsc::Sender<AuthReq>) {
    let (send, recv) = match stream {
        quinn::NewStream::Bi(send, recv) => (send, recv),
        quinn::NewStream::Uni(_) => unreachable!("Disabled by endpoint configuration"),
    };
    //let log = log.clone();
    //let log2 = log.clone();
    //let log3 = log.clone();

    tokio_current_thread::spawn(
        recv.read_to_end(64 * 1024) // Read the request, which must be at most 64KiB
            .map_err(|e| format_err!("Failed reading request: {}", e))
            .and_then(move |(_, req)| {
                let mut escaped = String::new();
                for &x in &req[..] {
                    let part = ascii::escape_default(x).collect::<Vec<_>>();
                    escaped.push_str(str::from_utf8(&part).unwrap());
                }
                // info!(log, "Got request");
                // Execute the request
                let resp = process_get(&req, notif_channel).unwrap_or_else(move |e| {
                    // error!(log, "Failed to process request"; "reason" => %e.pretty());
                    // TODO: implement JSON-RPC rather.
                    // Temporarily prefix message with "[AUTHD_ERROR]" to signal error to the caller,
                    // once we have JSON-RPC we can adhere to its format for errors.
                    format!("[ERROR]: {}", e.pretty()).into_bytes().into()
                });

                // Write the response
                tokio::io::write_all(send, resp)
                    .map_err(|e| format_err!("Failed to send response: {}", e))
            })
            // Gracefully terminate the stream
            .and_then(|(send, _)| {
                tokio::io::shutdown(send)
                    .map_err(|e| format_err!("Failed to shutdown stream: {}", e))
            })
            .map(move |_| ()) // info!(log3, "Request complete"))
            .map_err(move |_e| ()), // error!(log2, "Request Failed"; "reason" => %e.pretty())),
    )
}

fn process_get(x: &[u8], notif_channel: mpsc::Sender<AuthReq>) -> Result<Box<[u8]>, Error> {
    if x.len() < 4 || &x[0..4] != b"GET " {
        bail!("Missing GET");
    }
    if x[4..].len() < 2 || &x[x.len() - 2..] != b"\r\n" {
        bail!("Missing \\r\\n");
    }
    let x = &x[4..x.len() - 2];
    let end = x.iter().position(|&c| c == b' ').unwrap_or_else(|| x.len());
    let path = str::from_utf8(&x[..end]).context("Path is malformed UTF-8")?;
    let req_args: Vec<&str> = path.split('/').collect();

    if req_args.len() != 3 {
        bail!(
            "Incorrect number of arguments for authorisation request notification, missing app ID"
        )
    } else {
        let app_id = req_args[1];
        let req_id = req_args[2].parse::<SafeAuthReqId>()?;

        // TODO: get the rest of auth req info from the request
        let auth_req = AuthReq {
            req_id,
            app_id: app_id.to_string(),
            app_name: String::from("Unknown"),
            app_vendor: String::from("Unknown"),
            app_permissions: AppPermissions {
                get_balance: true,
                perform_mutations: true,
                transfer_coins: true,
            },
            own_container: false,
            containers: HashMap::default(),
        };

        // New notification for auth req to be sent to user
        let msg = match notif_channel.send(auth_req) {
            Ok(_) => format!(
                "Ok - auth req from app ID: {} ready to be notified to user",
                app_id
            ),
            Err(err) => format!("Auth req notification couldn't be sent to user: {}", err),
        };

        Ok(msg.as_bytes().into())
    }
}
