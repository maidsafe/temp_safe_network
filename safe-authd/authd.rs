// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::debug;
use structopt::{self, StructOpt};

use super::quic_client::quic_send;
use super::update::update_commander;
use daemonize::Daemonize;
use failure::{Error, Fail, ResultExt};
use futures::{Async, Future, Poll, Stream};
use safe_api::{SafeAuthReq, SafeAuthenticator};
use slog::{Drain, Logger};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{ascii, fmt, fs, str};
use tokio::runtime::current_thread::Runtime;
use tokio::sync::mpsc;
use url::Url;

// Frequency for checking pensing auth requests
const AUTH_REQS_CHECK_FREQ: u64 = 2000;

// Maximum number of allowed auth reqs notifs subscriptors
const MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS: usize = 3;

#[derive(Debug, Clone)]
struct AuthReq {
    pub app_id: String,
    pub tx: mpsc::Sender<bool>,
}

// List of authorisation requests indexed by their request id
type AuthReqsList = BTreeMap<u32, AuthReq>;

// A thread-safe queue to keep the list of authorisation requests
type SharedAuthReqsHandle = Arc<Mutex<AuthReqsList>>;

// A thread-safe handle to keep the SafeAuthenticator instance
type SharedSafeAuthenticatorHandle = Arc<Mutex<SafeAuthenticator>>;

// A thread-safe handle to keep the list of notifications subscriptors' endpoints
type SharedNotifEndpointsHandle = Arc<Mutex<BTreeSet<String>>>;

const SAFE_AUTHD_PID_FILE: &str = "/tmp/safe-authd.pid";
const SAFE_AUTHD_STDOUT_FILE: &str = "/tmp/safe-authd.out";
const SAFE_AUTHD_STDERR_FILE: &str = "/tmp/safe-authd.err";

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

#[derive(StructOpt, Debug)]
/// SAFE Authenticator daemon
#[structopt(raw(global_settings = "&[structopt::clap::AppSettings::ColoredHelp]"))]
enum CmdArgs {
    /// Start the safe-authd daemon
    #[structopt(name = "start")]
    Start {
        /// File to log TLS keys to for debugging
        #[structopt(long = "keylog")]
        keylog: bool,
        /// TLS private key in PEM format
        #[structopt(parse(from_os_str), short = "k", long = "key", requires = "cert")]
        key: Option<PathBuf>,
        /// TLS certificate in PEM format
        #[structopt(parse(from_os_str), short = "c", long = "cert", requires = "key")]
        cert: Option<PathBuf>,
        /// Enable stateless retries
        #[structopt(long = "stateless-retry")]
        stateless_retry: bool,
        /// Address to listen on
        #[structopt(long = "listen", default_value = "https://localhost:33000")]
        listen: String,
    },
    /// Stop a running safe-authd
    #[structopt(name = "stop")]
    Stop {},
    /// Restart a running safe-authd
    #[structopt(name = "restart")]
    Restart {
        /// Address to listen on
        #[structopt(long = "listen", default_value = "https://localhost:33000")]
        listen: String,
    },
    /// Update the application to the latest available version
    #[structopt(name = "update")]
    Update {},
}

pub fn run() -> Result<(), String> {
    // Let's first get all the arguments passed in
    let opt = CmdArgs::from_args();
    debug!("Running authd with options: {:?}", opt);

    let decorator = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let drain = slog_term::FullFormat::new(decorator)
        .use_original_order()
        .build()
        .fuse();

    match opt {
        CmdArgs::Update {} => {
            update_commander().map_err(|err| format!("Error performing update: {}", err))
        }
        CmdArgs::Start { listen, .. } => {
            let url = Url::parse(&listen).map_err(|_| "Invalid end point address".to_string())?;
            let endpoint = url
                .to_socket_addrs()
                .map_err(|_| "Invalid end point address".to_string())?
                .next()
                .ok_or("The end point is an invalid address".to_string())?;
            if let Err(e) = start_authd(Logger::root(drain, o!()), endpoint) {
                Err(format!("{}", e.pretty()))
            } else {
                Ok(())
            }
        }
        CmdArgs::Stop {} => {
            if let Err(e) = stop_authd(Logger::root(drain, o!())) {
                Err(format!("{}", e.pretty()))
            } else {
                Ok(())
            }
        }
        CmdArgs::Restart { listen } => {
            let url = Url::parse(&listen).map_err(|_| "Invalid end point address".to_string())?;
            let endpoint = url
                .to_socket_addrs()
                .map_err(|_| "Invalid end point address".to_string())?
                .next()
                .ok_or("The end point is an invalid address".to_string())?;
            if let Err(e) = restart_authd(Logger::root(drain, o!()), endpoint) {
                Err(format!("{}", e.pretty()))
            } else {
                Ok(())
            }
        }
    }
}

fn start_authd(log: Logger, listen: SocketAddr) -> Result<(), Error> {
    println!("Starting SAFE Authenticator daemon...");
    let server_config = quinn::ServerConfig {
        transport: Arc::new(quinn::TransportConfig {
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
    let dirs = directories::ProjectDirs::from("org", "quinn", "quinn-examples").unwrap();
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

    let stdout = File::create(SAFE_AUTHD_STDOUT_FILE).unwrap();
    let stderr = File::create(SAFE_AUTHD_STDERR_FILE).unwrap();

    let daemonize = Daemonize::new()
        .pid_file(SAFE_AUTHD_PID_FILE) // Every method except `new` and `start`
        //.chown_pid_file(true)      // is optional, see `Daemonize` documentation
        .working_directory("/tmp") // for default behaviour.
        //.user("nobody")
        //.group("daemon") // Group name
        //.group(2)        // or group id.
        .umask(0o777) // Set umask, `0o027` by default.
        .stdout(stdout) // Redirect stdout to `/tmp/safe-authd.out`.
        .stderr(stderr) // Redirect stderr to `/tmp/safe-authd.err`.
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            println!("Success, SAFE Authenticator daemonised!");

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

            runtime.block_on(endpoint_driver)?;
        }
        Err(e) => eprintln!("Error, {}", e),
    }

    Ok(())
}

fn stop_authd(_log: Logger) -> Result<(), Error> {
    println!("Stopping SAFE Authenticator daemon...");
    let mut file = File::open(SAFE_AUTHD_PID_FILE)?;
    let mut pid = String::new();
    file.read_to_string(&mut pid)?;
    let output = Command::new("kill").arg("-9").arg(&pid).output()?;

    if output.status.success() {
        io::stdout().write_all(&output.stdout)?;
        println!("Success, safe-authd stopped!");
        Ok(())
    } else {
        io::stdout().write_all(&output.stderr)?;
        bail!("Failed to stop safe-authd daemon");
    }
}

fn restart_authd(log: Logger, listen: SocketAddr) -> Result<(), Error> {
    stop_authd(log.clone())?;
    start_authd(log, listen)?;
    println!("Success, safe-authd restarted!");
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
    info!(log, "got connection";
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

fn process_request(
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
    req: &[u8],
) -> ProcessRequest {
    ProcessRequest::HandleRequest {
        safe_auth_handle,
        auth_reqs_handle,
        notif_endpoints_handle,
        req: req.to_vec(),
    }
}

enum ProcessRequest {
    HandleRequest {
        safe_auth_handle: SharedSafeAuthenticatorHandle,
        auth_reqs_handle: SharedAuthReqsHandle,
        notif_endpoints_handle: SharedNotifEndpointsHandle,
        req: Vec<u8>,
    },
    ProcessingResponse {
        safe_auth_handle: SharedSafeAuthenticatorHandle,
        auth_reqs_handle: SharedAuthReqsHandle,
        rx: mpsc::Receiver<bool>,
        req_id: u32,
        auth_req_str: String,
    },
}

impl Future for ProcessRequest {
    type Item = Box<[u8]>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::ProcessRequest::*;

        // TODO: implement JSON-RPC rather.
        // Temporarily prefix message with "[AUTHD_ERROR]" to signal error to the caller,
        // once we have JSON-RPC we can adhere to its format for errors.
        let future_err = |str: String| -> Poll<Self::Item, Self::Error> {
            Ok(Async::Ready(
                format!("[AUTHD_ERROR]:SAFE Authenticator: {}", str)
                    .into_bytes()
                    .into(),
            ))
        };

        let future_ok = |str: String| -> Poll<Self::Item, Self::Error> {
            Ok(Async::Ready(str.into_bytes().into()))
        };

        loop {
            match self {
                ProcessingResponse {
                    safe_auth_handle,
                    auth_reqs_handle,
                    rx,
                    req_id,
                    auth_req_str,
                } => {
                    match rx.poll() {
                        Ok(Async::Ready(Some(is_allowed))) => {
                            rx.close();
                            if is_allowed {
                                let safe_auth: &mut SafeAuthenticator =
                                    &mut *(safe_auth_handle.lock().unwrap());
                                match safe_auth.authorise_app(auth_req_str) {
                                    Ok(resp) => {
                                        println!("Authorisation request ({}) was allowed and response sent back to the application", req_id);
                                        return future_ok(resp);
                                    }
                                    Err(err) => {
                                        println!("Failed to authorise application: {}", err);
                                        return future_err(err.to_string());
                                    }
                                }
                            } else {
                                let msg = format!("Authorisation request ({}) was denied", req_id);
                                println!("{}", msg);
                                return future_err(msg);
                            }
                        }
                        Ok(Async::NotReady) => {
                            return Ok(Async::NotReady);
                        }
                        Ok(Async::Ready(None)) | Err(_) => {
                            rx.close();
                            // We didn't get a response in a timely manner, we cannot allow the list
                            // to grow infinitelly, so let's remove the request from it
                            let auth_reqs_list: &mut AuthReqsList =
                                &mut *(auth_reqs_handle.lock().unwrap());
                            auth_reqs_list.remove(&req_id);
                            let msg = "Failed to get authorision response";
                            println!("{}", msg);
                            return future_err(msg.to_string());
                        }
                    }
                }
                HandleRequest {
                    safe_auth_handle,
                    auth_reqs_handle,
                    notif_endpoints_handle,
                    req,
                } => {
                    if req.len() < 4 || &req[0..4] != b"GET " {
                        return future_err("Missing GET".to_string());
                    }
                    if req[4..].len() < 2 || &req[req.len() - 2..] != b"\r\n" {
                        return future_err("Missing \\r\\n".to_string());
                    }
                    let req = &req[4..req.len() - 2];
                    let end = req
                        .iter()
                        .position(|&c| c == b' ')
                        .unwrap_or_else(|| req.len());
                    let path = match str::from_utf8(&req[..end]).context("Path is malformed UTF-8")
                    {
                        Ok(path) => path,
                        Err(err) => return future_err(err.to_string()),
                    };
                    let req_args: Vec<&str> = path.split("/").collect();

                    let safe_auth_handle = safe_auth_handle.clone();
                    let safe_auth: &mut SafeAuthenticator =
                        &mut *(safe_auth_handle.lock().unwrap());

                    println!("Processing new incoming request: '{}'", req_args[1]);
                    match req_args[1] {
                        "login" => {
                            if req_args.len() != 4 {
                                return future_err(
                                    "Incorrect number of arguments for 'login' action".to_string(),
                                );
                            } else {
                                println!("Logging in to SAFE account...");
                                let secret = req_args[2];
                                let password = req_args[3];

                                match safe_auth.log_in(secret, password) {
                                    Ok(_) => {
                                        let msg = "Logged in successfully!";
                                        println!("{}", msg);
                                        return future_ok(msg.to_string());
                                    }
                                    Err(err) => {
                                        let msg = format!(
                                            "Error occurred when trying to log in: {}",
                                            err
                                        );
                                        println!("{}", msg);
                                        return future_err(err.to_string());
                                    }
                                }
                            }
                        }
                        "logout" => {
                            if req_args.len() != 2 {
                                return future_err(
                                    "Incorrect number of arguments for 'logout' action".to_string(),
                                );
                            } else {
                                println!("Logging out...");
                                match safe_auth.log_out() {
                                    Ok(()) => {
                                        let msg = "Logged out successfully";
                                        println!("{}", msg);
                                        return future_ok(msg.to_string());
                                    }
                                    Err(err) => {
                                        let msg = format!("Failed to log out: {}", err);
                                        println!("{}", msg);
                                        return future_err(msg);
                                    }
                                }
                            }
                        }
                        "create" => {
                            if req_args.len() != 5 {
                                return future_err(
                                    "Incorrect number of arguments for 'create' action".to_string(),
                                );
                            } else {
                                println!("Creating an account in SAFE...");
                                let secret = req_args[2];
                                let password = req_args[3];
                                let sk = req_args[4];

                                match safe_auth.create_acc(sk, secret, password) {
                                    Ok(_) => {
                                        let msg = "Account created successfully";
                                        println!("{}", msg);
                                        return future_ok(msg.to_string());
                                    }
                                    Err(err) => {
                                        println!(
                                            "Error occurred when trying to create SAFE account: {}",
                                            err
                                        );
                                        return future_err(err.to_string());
                                    }
                                }
                            }
                        }
                        "authorise" => {
                            if req_args.len() != 3 {
                                return future_err(
                                    "Incorrect number of arguments for 'authorise' action"
                                        .to_string(),
                                );
                            } else {
                                println!("Authorising application...");
                                // TODO: automatically reject if there are too many pending auth reqs
                                let auth_req_str = req_args[2];
                                match safe_auth.decode_req(auth_req_str) {
                                    Ok((req_id, safe_auth_req)) => {
                                        println!(
                                            "Sending request to user to allow/deny request..."
                                        );

                                        let rx = enqueue_auth_req(
                                            req_id,
                                            safe_auth_req,
                                            auth_reqs_handle,
                                        );

                                        *self = ProcessingResponse {
                                            safe_auth_handle: safe_auth_handle.clone(),
                                            auth_reqs_handle: auth_reqs_handle.clone(),
                                            rx,
                                            req_id,
                                            auth_req_str: auth_req_str.to_string(),
                                        };
                                    }
                                    Err(err) => {
                                        println!("{}", err);
                                        return future_err(err.to_string());
                                    }
                                }
                            }
                        }
                        "authed-apps" => {
                            if req_args.len() != 2 {
                                return future_err(
                                    "Incorrect number of arguments for 'authed-apps' action"
                                        .to_string(),
                                );
                            } else {
                                println!("Obtaining list of authorised applications...");
                                match safe_auth.authed_apps() {
                                    Ok(resp) => {
                                        println!("List of authorised apps sent");
                                        return future_ok(format!("{:?}", resp));
                                    }
                                    Err(err) => {
                                        println!("Failed to get list of authorised apps: {}", err);
                                        return future_err(err.to_string());
                                    }
                                }
                            }
                        }
                        "revoke" => {
                            if req_args.len() != 3 {
                                return future_err(
                                    "Incorrect number of arguments for 'revoke' action".to_string(),
                                );
                            } else {
                                println!("Revoking application...");
                                let app_id = req_args[2];

                                match safe_auth.revoke_app(app_id) {
                                    Ok(()) => {
                                        let msg = "Application revoked successfully";
                                        println!("{}", msg);
                                        return future_ok(msg.to_string());
                                    }
                                    Err(err) => {
                                        println!(
                                            "Failed to revoke application '{}': {}",
                                            app_id, err
                                        );
                                        return future_err(err.to_string());
                                    }
                                }
                            }
                        }
                        "auth-reqs" => {
                            if req_args.len() != 2 {
                                return future_err(
                                    "Incorrect number of arguments for 'auth-reqs' action"
                                        .to_string(),
                                );
                            } else {
                                println!("Obtaining list of pending authorisation requests...");
                                let auth_reqs_list: &mut AuthReqsList =
                                    &mut *(auth_reqs_handle.lock().unwrap());
                                let resp: BTreeSet<String> = auth_reqs_list
                                    .iter()
                                    .map(|(req_id, auth_req)| {
                                        format!("Req ID: {} - App ID: {}", req_id, auth_req.app_id)
                                    })
                                    .collect();

                                println!("List of pending authorisation requests sent");
                                return future_ok(format!("{:?}", resp));
                            }
                        }
                        "allow" => {
                            if req_args.len() != 3 {
                                return future_err(
                                    "Incorrect number of arguments for 'allow' action".to_string(),
                                );
                            } else {
                                println!("Allowing authorisation request...");
                                let auth_req_id = req_args[2];
                                let auth_reqs_list: &mut AuthReqsList =
                                    &mut *(auth_reqs_handle.lock().unwrap());
                                let req_id = match auth_req_id.parse::<u32>() {
                                    Ok(id) => id,
                                    Err(err) => return future_err(err.to_string()),
                                };
                                match auth_reqs_list.remove(&req_id) {
                                    Some(mut auth_req) => match auth_req.tx.try_send(true) {
                                        Ok(_) => {
                                            let msg = format!(
                                                "Authorisation request ({}) allowed successfully",
                                                auth_req_id
                                            );
                                            println!("{}", msg);
                                            return future_ok(msg);
                                        }
                                        Err(_) => {
                                            let msg = format!("Failed to allow authorisation request '{}' since the response couldn't be sent to the requesting application", auth_req_id);
                                            println!("{}", msg);
                                            return future_err(msg);
                                        }
                                    },
                                    None => {
                                        let msg = format!(
                                            "No pending authorisation request found with id '{}'",
                                            auth_req_id
                                        );
                                        println!("{}", msg);
                                        return future_err(msg);
                                    }
                                }
                            }
                        }
                        "deny" => {
                            if req_args.len() != 3 {
                                return future_err(
                                    "Incorrect number of arguments for 'deny' action".to_string(),
                                );
                            } else {
                                println!("Denying authorisation request...");
                                let auth_req_id = req_args[2];
                                let auth_reqs_list: &mut AuthReqsList =
                                    &mut *(auth_reqs_handle.lock().unwrap());
                                let req_id = match auth_req_id.parse::<u32>() {
                                    Ok(id) => id,
                                    Err(err) => return future_err(err.to_string()),
                                };
                                match auth_reqs_list.remove(&req_id) {
                                    Some(mut auth_req) => match auth_req.tx.try_send(false) {
                                        Ok(_) => {
                                            let msg = format!(
                                                "Authorisation request ({}) denied successfully",
                                                auth_req_id
                                            );
                                            println!("{}", msg);
                                            return future_ok(msg);
                                        }
                                        Err(_) => {
                                            let msg = format!("Authorisation request '{}' was already denied since the response couldn't be sent to the requesting application", auth_req_id);
                                            println!("{}", msg);
                                            return future_err(msg);
                                        }
                                    },
                                    None => {
                                        let msg = format!(
                                            "No pending authorisation request found with id '{}'",
                                            auth_req_id
                                        );
                                        println!("{}", msg);
                                        return future_err(msg);
                                    }
                                }
                            }
                        }
                        "subscribe" => {
                            if req_args.len() != 3 {
                                return future_err(
                                    "Incorrect number of arguments for 'subscribe' action"
                                        .to_string(),
                                );
                            } else {
                                println!("Subscribing to authorisation requests notifications...");
                                let mut notif_endpoint = match urlencoding::decode(req_args[2]) {
                                    Ok(url) => url,
                                    Err(err) => {
                                        let msg = format!(
                                        "Subscription rejected, the endpoint URL ('{}') is invalid: {:?}",
                                        req_args[2], err
                                    );
                                        println!("{}", msg);
                                        return future_err(msg);
                                    }
                                };
                                let notif_endpoints_list: &mut BTreeSet<String> =
                                    &mut *(notif_endpoints_handle.lock().unwrap());
                                if notif_endpoints_list.len() >= MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS {
                                    let msg = format!("Subscription rejected. Maximum number of subscriptions ({}) has been already reached", MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS);
                                    println!("{}", msg);
                                    return future_err(msg);
                                } else {
                                    if notif_endpoint.ends_with('/') {
                                        notif_endpoint.pop();
                                    }
                                    notif_endpoints_list.insert(notif_endpoint.clone());
                                    let msg = format!(
                                    "Subscription successful. Endpoint '{}' will receive authorisation requests notifications",
                                    notif_endpoint
                                );
                                    println!("{}", msg);
                                    return future_ok(msg);
                                }
                            }
                        }
                        "unsubscribe" => {
                            if req_args.len() != 3 {
                                return future_err(
                                    "Incorrect number of arguments for 'unsubscribe' action"
                                        .to_string(),
                                );
                            } else {
                                println!(
                                    "Unsubscribing from authorisation requests notifications..."
                                );
                                let notif_endpoint = match urlencoding::decode(req_args[2]) {
                                    Ok(url) => url,
                                    Err(err) => {
                                        let msg = format!(
                                        "Unsubscription request rejected, the endpoint URL ('{}') is invalid: {:?}",
                                        req_args[2], err
                                    );
                                        println!("{}", msg);
                                        return future_err(msg);
                                    }
                                };
                                let notif_endpoints_list: &mut BTreeSet<String> =
                                    &mut *(notif_endpoints_handle.lock().unwrap());

                                if notif_endpoints_list.remove(&notif_endpoint) {
                                    let msg = format!(
                                    "Unsubscription successful. Endpoint '{}' will no longer receive authorisation requests notifications",
                                    notif_endpoint
                                );
                                    println!("{}", msg);
                                    return future_ok(msg);
                                } else {
                                    let msg = format!(
                                    "Unsubscription request ignored, no such the endpoint URL ('{}') was found to be subscribed",
                                    notif_endpoint
                                );
                                    println!("{}", msg);
                                    return future_err(msg);
                                }
                            }
                        }
                        other => {
                            println!(
                                "Action '{}' not supported or unknown by the Authenticator daemon",
                                other
                            );
                            return future_err("Action not supported or unknown".to_string());
                        }
                    }
                }
            }
        }
    }
}

fn enqueue_auth_req(
    req_id: u32,
    req: SafeAuthReq,
    auth_reqs_handle: &SharedAuthReqsHandle,
) -> mpsc::Receiver<bool> {
    let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel(32);
    match req {
        SafeAuthReq::Auth(app_auth_req) => {
            println!("The following application authorisation request was received:");
            println!("{:?}", app_auth_req);

            // Let's add it to the list of pending authorisation requests
            let auth_req = AuthReq {
                app_id: app_auth_req.app.id,
                tx,
            };
            let auth_reqs_list: &mut AuthReqsList = &mut *(auth_reqs_handle.lock().unwrap());
            auth_reqs_list.insert(req_id, auth_req);
        }
        SafeAuthReq::Containers(cont_req) => {
            println!("The following authorisation request for containers was received:");
            println!("{:?}", cont_req);
        }
        SafeAuthReq::ShareMData(share_mdata_req) => {
            println!("The following authorisation request to share a MutableData was received:");
            println!("{:?}", share_mdata_req);
        }
        SafeAuthReq::Unregistered(_) => {
            // we simply allow unregistered authorisation requests
        }
    }
    rx
}

fn monitor_pending_auth_reqs(
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) {
    thread::spawn(move || loop {
        {
            let mut reqs_to_process: Option<AuthReqsList> = None;
            {
                let auth_reqs_list: &mut AuthReqsList = &mut *(auth_reqs_handle.lock().unwrap());
                if !auth_reqs_list.is_empty() {
                    let notif_endpoints_list: &mut BTreeSet<String> =
                        &mut *(notif_endpoints_handle.lock().unwrap());
                    if !notif_endpoints_list.is_empty() {
                        reqs_to_process = Some(auth_reqs_list.clone());
                        auth_reqs_list.clear();
                    }
                };
            }

            // TODO: send a "keep subscription?" notif/request to subscriptors periodically,
            // and remove them if they don't respond or their response is positive.
            match reqs_to_process {
                None => {}
                Some(mut reqs) => {
                    let notif_endpoints_list: &mut BTreeSet<String> =
                        &mut *(notif_endpoints_handle.lock().unwrap());
                    for (req_id, auth_req) in reqs.iter_mut() {
                        let mut is_allow = false;
                        for endpoint in notif_endpoints_list.iter() {
                            println!("ASKING SUBSCRIPTOR: {}", endpoint);
                            match quic_send(
                                &format!("{}/{}", endpoint, auth_req.app_id),
                                false,
                                None,
                                None,
                                false,
                            ) {
                                Ok(allow) => {
                                    is_allow = allow.starts_with("true");
                                    break;
                                }
                                Err(err) => {
                                    println!(
                                        "Skipping subscriptor '{}' since it didn't respond: {}",
                                        endpoint, err
                                    );
                                }
                            }
                        }
                        println!(
                            "ALLOW FOR Req ID: {} - App ID: {} ??: {}",
                            req_id, auth_req.app_id, is_allow
                        );
                        match auth_req.tx.try_send(is_allow) {
                            Ok(_) => println!("Auth req decision ready to be sent to application"),
                            Err(_) => println!(
                                "Auth req decision couldn't be sent, and therefore already denied"
                            ),
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(AUTH_REQS_CHECK_FREQ));
    });
}
