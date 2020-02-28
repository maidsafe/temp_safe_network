// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    common::send_authd_request,
    constants::{SAFE_AUTHD_ENDPOINT_HOST, SAFE_AUTHD_ENDPOINT_PORT},
    notifs_endpoint::jsonrpc_listen,
};
use crate::{AuthedAppsList, Error, Result, SafeAuthReqId};
use directories::BaseDirs;
use log::{debug, error, info, trace};
use safe_core::ipc::req::ContainerPermissions;
use safe_nd::AppPermissions;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    io::{self, Write},
    path::PathBuf,
    process::Command,
    sync::mpsc,
    thread,
};

#[cfg(not(target_os = "windows"))]
const SAFE_AUTHD_EXECUTABLE: &str = "safe-authd";

#[cfg(target_os = "windows")]
const SAFE_AUTHD_EXECUTABLE: &str = "safe-authd.exe";

const ENV_VAR_SAFE_AUTHD_PATH: &str = "SAFE_AUTHD_PATH";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthReq {
    /// The authorisation request ID
    pub req_id: SafeAuthReqId,
    /// The App ID. It must be unique.
    pub app_id: String,
    /// The application friendly-name.
    pub app_name: String,
    /// The application provider/vendor (e.g. MaidSafe)
    pub app_vendor: String,
    /// Permissions requested, e.g. allowing to work with the user's coin balance.
    pub app_permissions: AppPermissions,
    /// The permissions requested by the app for named containers
    // TODO: ContainerPermissions will/shall be refactored to expose a struct defined in this crate
    pub containers: HashMap<String, ContainerPermissions>,
    /// If the app requested a dedicated named container for itself
    pub own_container: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthdStatus {
    pub logged_in: bool,
    pub num_auth_reqs: u32,
    pub num_notif_subs: u32,
    pub authd_version: String,
}

// Type of the list of pending authorisation requests
pub type PendingAuthReqs = Vec<AuthReq>;

// Type of the function/callback invoked for notifying and querying if an authorisation request
// shall be allowed. All the relevant information about the authorisation request is passed as args to the callback.
pub type AuthAllowPrompt = dyn Fn(AuthReq) -> Option<bool> + std::marker::Send + std::marker::Sync;

// Path of authenticator endpoint for getting a status report of the safe-authd
const SAFE_AUTHD_METHOD_STATUS: &str = "status";

// Path of authenticator endpoint for login into a SAFE account
const SAFE_AUTHD_METHOD_LOGIN: &str = "login";

// Path of authenticator endpoint for loging out from a SAFE account
const SAFE_AUTHD_METHOD_LOGOUT: &str = "logout";

// Path of authenticator endpoint for creating a new SAFE account
const SAFE_AUTHD_METHOD_CREATE: &str = "create-acc";

// Path of authenticator endpoint for fetching list of authorised apps
const SAFE_AUTHD_METHOD_AUTHED_APPS: &str = "authed-apps";

// Path of authenticator endpoint for revoking applications and/or permissions
const SAFE_AUTHD_METHOD_REVOKE: &str = "revoke";

// Path of authenticator endpoint for retrieving the list of pending authorisation requests
const SAFE_AUTHD_METHOD_AUTH_REQS: &str = "auth-reqs";

// Path of authenticator endpoint for allowing an authorisation request
const SAFE_AUTHD_METHOD_ALLOW: &str = "allow";

// Path of authenticator endpoint for denying an authorisation request
const SAFE_AUTHD_METHOD_DENY: &str = "deny";

// Path of authenticator endpoint for subscribing to authorisation requests notifications
const SAFE_AUTHD_METHOD_SUBSCRIBE: &str = "subscribe";

// Path of authenticator endpoint for unsubscribing from authorisation requests notifications
const SAFE_AUTHD_METHOD_UNSUBSCRIBE: &str = "unsubscribe";

// authd subcommand to install the daemon
const SAFE_AUTHD_CMD_INSTALL: &str = "install";

// authd subcommand to uninstall the daemon
const SAFE_AUTHD_CMD_UNINSTALL: &str = "uninstall";

// authd subcommand to update the binary to new available released version
const SAFE_AUTHD_CMD_UPDATE: &str = "update";

// authd subcommand to start the daemon
const SAFE_AUTHD_CMD_START: &str = "start";

// authd subcommand to stop the daemon
const SAFE_AUTHD_CMD_STOP: &str = "stop";

// authd subcommand to restart the daemon
const SAFE_AUTHD_CMD_RESTART: &str = "restart";

// Authd Client API
pub struct SafeAuthdClient {
    // authd endpoint
    pub authd_endpoint: String,
    // keep track of (endpoint URL, join handle for the listening thread, join handle of callback thread)
    subscribed_endpoint: Option<(String, thread::JoinHandle<()>, thread::JoinHandle<()>)>,
    // TODO: add a session_token field to use for communicating with authd for restricted operations,
    // we should restrict operations like subscribe, or allow/deny, to only be accepted with a valid token
    // session_token: String,
}

impl Drop for SafeAuthdClient {
    fn drop(&mut self) {
        trace!("SafeAuthdClient instance being dropped...");
        // Let's try to unsubscribe if we had a subscribed endpoint
        match &self.subscribed_endpoint {
            None => {}
            Some((url, _, _)) => {
                match send_unsubscribe(url, &self.authd_endpoint) {
                    Ok(msg) => {
                        debug!("{}", msg);
                    }
                    Err(err) => {
                        // We are still ok, it was just us trying to be nice and unsubscribe if possible
                        // It could be the case we were already unsubscribe automatically by authd before
                        // we were attempting to do it now, which can happend due to our endpoint
                        // being unresponsive, so it's all ok
                        debug!("Failed to unsubscribe endpoint from authd: {}", err);
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
impl SafeAuthdClient {
    pub fn new(endpoint: Option<String>) -> Self {
        let endpoint = match endpoint {
            None => format!("{}:{}", SAFE_AUTHD_ENDPOINT_HOST, SAFE_AUTHD_ENDPOINT_PORT),
            Some(endpoint) => endpoint,
        };
        debug!("Creating new authd client for endpoint {}", endpoint);
        Self {
            authd_endpoint: endpoint,
            subscribed_endpoint: None,
        }
    }

    // Install the Authenticator daemon/service
    pub fn install(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(authd_path, &[SAFE_AUTHD_CMD_INSTALL])
    }

    // Uninstall the Authenticator daemon/service
    pub fn uninstall(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(authd_path, &[SAFE_AUTHD_CMD_UNINSTALL])
    }

    // Update the Authenticator binary to a new released version
    pub fn update(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(authd_path, &[SAFE_AUTHD_CMD_UPDATE])
    }

    // Start the Authenticator daemon
    pub fn start(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(
            authd_path,
            &[SAFE_AUTHD_CMD_START, "--listen", &self.authd_endpoint],
        )
    }

    // Stop the Authenticator daemon
    pub fn stop(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(authd_path, &[SAFE_AUTHD_CMD_STOP])
    }

    // Restart the Authenticator daemon
    pub fn restart(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(
            authd_path,
            &[SAFE_AUTHD_CMD_RESTART, "--listen", &self.authd_endpoint],
        )
    }

    // Send a request to remote authd endpoint to obtain a status report
    pub fn status(&mut self) -> Result<AuthdStatus> {
        debug!("Attempting to retrieve status report from remote authd...");
        info!("Sending status report request to SAFE Authenticator...");
        let status_report = send_authd_request::<AuthdStatus>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_STATUS,
            serde_json::Value::Null,
        )?;

        info!(
            "SAFE status report retrieved successfully: {:?}",
            status_report
        );
        Ok(status_report)
    }

    // Send a login action request to remote authd endpoint
    pub fn log_in(&mut self, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to log in on remote authd...");
        info!(
            "Sending login action to SAFE Authenticator ({})...",
            self.authd_endpoint
        );
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_LOGIN,
            json!(vec![passphrase, password]),
        )?;

        info!("SAFE login action was successful: {}", authd_response);
        // TODO: store the authd session token, replacing an existing one
        // self.session_token = authd_response;

        Ok(())
    }

    // Sends a logout action request to the SAFE Authenticator
    pub fn log_out(&mut self) -> Result<()> {
        debug!("Dropping logged in session and logging out in remote authd...");
        info!("Sending logout action to SAFE Authenticator...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_LOGOUT,
            serde_json::Value::Null,
        )?;

        info!("SAFE logout action was successful: {}", authd_response);

        // TODO: clean up the stored authd session token
        // self.session_token = "".to_string();

        Ok(())
    }

    // Sends an account creation request to the SAFE Authenticator
    pub fn create_acc(&self, sk: &str, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to create a SAFE account on remote authd...");
        debug!("Sending account creation request to SAFE Authenticator...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_CREATE,
            json!(vec![passphrase, password, sk]),
        )?;

        debug!(
            "SAFE account creation action was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Get the list of applications authorised from remote authd
    pub fn authed_apps(&self) -> Result<AuthedAppsList> {
        debug!("Attempting to fetch list of authorised apps from remote authd...");
        debug!("Sending request request to SAFE Authenticator...");
        let authed_apps_list = send_authd_request::<AuthedAppsList>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_AUTHED_APPS,
            serde_json::Value::Null,
        )?;

        debug!(
            "List of applications authorised successfully received: {:?}",
            authed_apps_list
        );
        Ok(authed_apps_list)
    }

    // Revoke all permissions from an application
    pub fn revoke_app(&self, app_id: &str) -> Result<()> {
        debug!(
            "Requesting to revoke permissions from application: {}",
            app_id
        );
        debug!("Sending revoke action request to SAFE Authenticator...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_REVOKE,
            json!(app_id),
        )?;

        debug!(
            "Application revocation action successful: {}",
            authd_response
        );
        Ok(())
    }

    // Get the list of pending authorisation requests from remote authd
    pub fn auth_reqs(&self) -> Result<PendingAuthReqs> {
        debug!("Attempting to fetch list of pending authorisation requests from remote authd...");
        debug!("Sending request request to SAFE Authenticator...");
        let auth_reqs_list = send_authd_request::<PendingAuthReqs>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_AUTH_REQS,
            serde_json::Value::Null,
        )?;

        debug!(
            "List of pending authorisation requests successfully received: {:?}",
            auth_reqs_list
        );
        Ok(auth_reqs_list)
    }

    // Allow an authorisation request
    pub fn allow(&self, req_id: SafeAuthReqId) -> Result<()> {
        debug!("Requesting to allow authorisation request: {}", req_id);
        debug!("Sending allow action request to SAFE Authenticator...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_ALLOW,
            json!(req_id.to_string()),
        )?;

        debug!(
            "Action to allow authorisation request was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Deny an authorisation request
    pub fn deny(&self, req_id: SafeAuthReqId) -> Result<()> {
        debug!("Requesting to deny authorisation request: {}", req_id);
        debug!("Sending deny action request to SAFE Authenticator...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_DENY,
            json!(req_id.to_string()),
        )?;

        debug!(
            "Action to deny authorisation request was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Subscribe a callback to receive notifications to allow/deny authorisation requests
    // We support having only one subscripton at a time, a previous subscription will be dropped
    pub fn subscribe<
        CB: 'static + Fn(AuthReq) -> Option<bool> + std::marker::Send + std::marker::Sync,
    >(
        &mut self,
        endpoint_url: &str,
        app_id: &str,
        allow_cb: CB,
    ) -> Result<()> {
        debug!("Subscribing to receive authorisation requests notifications...",);

        // Generate a path which is where we will store the endpoint certificates that authd will
        // need to read to be able to create a secure channel to send us the notifications with
        let dirs = directories::ProjectDirs::from("net", "maidsafe", "safe-authd-client")
            .ok_or_else(|| {
                Error::AuthdClientError(
                    "Failed to obtain local home directory where to store endpoint certificates to"
                        .to_string(),
                )
            })?;

        // Let's postfix the path with the app id so we avoid clashes with other
        // endpoints subscribed from within the same local box
        let cert_base_path = dirs.config_dir().join(app_id.to_string());

        debug!("Sending subscribe action request to SAFE Authenticator...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_SUBSCRIBE,
            json!(vec![endpoint_url, &cert_base_path.display().to_string()]),
        ).map_err(|err| Error::AuthdClientError(format!("Failed when trying to subscribe endpoint URL ({}) to receive authorisation request for self-auth: {}", endpoint_url, err)))?;

        debug!(
            "Successfully subscribed to receive authorisation requests notifications: {}",
            authd_response
        );

        // Start listening first
        // We need a channel to receive auth req notifications from the thread running the QUIC endpoint
        let (tx, rx): (mpsc::Sender<AuthReq>, mpsc::Receiver<AuthReq>) = mpsc::channel();

        let listen = endpoint_url.to_string();
        // TODO: use Tokio futures with singled-threaded tasks and mpsc channel to receive reqs callbacks
        // TODO: if there was a previous subscription, make sure we kill the previously created threads
        let endpoint_thread_join_handle = thread::spawn(move || {
            match jsonrpc_listen(&listen, &cert_base_path.display().to_string(), tx) {
                Ok(()) => {
                    info!("Endpoint successfully launched for receiving auth req notifications");
                }
                Err(err) => {
                    error!(
                        "Failed to launch endpoint for receiving auth req notifications: {}",
                        err
                    );
                }
            }
        });

        let cb = Box::new(allow_cb);
        // TODO: use Tokio futures with singled-threaded tasks and mpsc channel to receive reqs callbacks
        // TODO: we may be also able to merge this logic into the endpoint thread
        let cb_thread_join_handle = thread::spawn(move || loop {
            match rx.recv() {
                Ok(auth_req) => {
                    debug!(
                        "Notification for authorisation request ({}) from app ID '{}' received",
                        auth_req.req_id, auth_req.app_id
                    );
                    let _user_decision = cb(auth_req);
                    // TODO: send the callback return value back to authd
                    /*match auth_req.tx.try_send(user_decision) {
                        Ok(_) => println!("Auth req decision made"),
                        Err(_) => println!(
                            "Auth req decision couldn't be obtained from user callback"
                        ),
                    };*/
                }
                Err(err) => {
                    debug!("Failed to receive message: {}", err);
                }
            }
        });
        self.subscribed_endpoint = Some((
            endpoint_url.to_string(),
            endpoint_thread_join_handle,
            cb_thread_join_handle,
        ));

        Ok(())
    }

    // Subscribe an endpoint URL where notifications to allow/deny authorisation requests shall be sent
    pub fn subscribe_url(&self, endpoint_url: &str) -> Result<()> {
        debug!(
            "Subscribing '{}' as endpoint for authorisation requests notifications...",
            endpoint_url
        );

        debug!("Sending subscribe action request to SAFE Authenticator...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SAFE_AUTHD_METHOD_SUBSCRIBE,
            json!(vec![endpoint_url]),
        )?;

        debug!(
            "Successfully subscribed a URL for authorisation requests notifications: {}",
            authd_response
        );
        Ok(())
    }

    // Unsubscribe from notifications to allow/deny authorisation requests
    pub fn unsubscribe(&mut self, endpoint_url: &str) -> Result<()> {
        debug!("Unsubscribing from authorisation requests notifications...",);
        let authd_response = send_unsubscribe(endpoint_url, &self.authd_endpoint)?;
        debug!(
            "Successfully unsubscribed from authorisation requests notifications: {}",
            authd_response
        );

        // If the URL is the same as the endpoint locally launched, terminate the thread
        if let Some((url, _, _)) = &self.subscribed_endpoint {
            if endpoint_url == url {
                // TODO: send signal to stop/kill threads
                self.subscribed_endpoint = None;
            }
        }

        Ok(())
    }
}

fn send_unsubscribe(endpoint_url: &str, authd_endpoint: &str) -> Result<String> {
    debug!(
        "Sending unsubscribe action request to SAFE Authenticator on {}...",
        authd_endpoint
    );
    send_authd_request::<String>(
        authd_endpoint,
        SAFE_AUTHD_METHOD_UNSUBSCRIBE,
        json!(endpoint_url),
    )
}

fn authd_run_cmd(authd_path: Option<&str>, args: &[&str]) -> Result<()> {
    let mut path = get_authd_bin_path(authd_path)?;
    path.push(SAFE_AUTHD_EXECUTABLE);
    let path_str = path.display().to_string();
    debug!("Attempting to {} authd from '{}' ...", args[0], path_str);

    let child = Command::new(&path_str).args(args).spawn().map_err(|err| {
        Error::AuthdClientError(format!(
            "Failed to execute authd from '{}': {}",
            path_str, err
        ))
    })?;

    let output = child.wait_with_output().map_err(|err| {
        Error::AuthdClientError(format!(
            "Failed to execute authd from '{}': {}",
            path_str, err
        ))
    })?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| Error::AuthdClientError(format!("Failed to output stdout: {}", err)))?;
        Ok(())
    } else {
        match output.status.code() {
            Some(10) => {
                // safe-authd exit code 10 is safe-authd::errors::Error::AuthdAlreadyStarted
                Err(Error::AuthdAlreadyStarted(format!(
                       "Failed to start safe-authd daemon '{}' as an instance seems to be already running",
                       path_str,
                   )))
            }
            Some(_) | None => Err(Error::AuthdError(format!(
                "Failed when invoking safe-authd executable from '{}':\n{}",
                path_str,
                String::from_utf8_lossy(&output.stderr)
            ))),
        }
    }
}

fn get_authd_bin_path(authd_path: Option<&str>) -> Result<PathBuf> {
    match authd_path {
        Some(p) => Ok(PathBuf::from(p)),
        None => {
            // if SAFE_AUTHD_PATH is set it then overrides default
            if let Ok(authd_path) = std::env::var(ENV_VAR_SAFE_AUTHD_PATH) {
                Ok(PathBuf::from(authd_path))
            } else {
                let base_dirs = BaseDirs::new().ok_or_else(|| {
                    Error::AuthdClientError("Failed to obtain user's home path".to_string())
                })?;

                let mut path = PathBuf::from(base_dirs.home_dir());
                path.push(".safe");
                path.push("authd");
                Ok(path)
            }
        }
    }
}
