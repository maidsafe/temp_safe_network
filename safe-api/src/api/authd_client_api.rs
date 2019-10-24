// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::constants::{SAFE_AUTHD_ENDPOINT_HOST, SAFE_AUTHD_ENDPOINT_PORT};
use super::quic_client::quic_send;
use super::quic_endpoint::quic_listen;
use super::{AuthedAppsList, Error, Result, SafeAuthReqId};
use log::{debug, error, info};
use safe_core::ipc::req::ContainerPermissions;
use safe_nd::AppPermissions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::Command;
use std::sync::mpsc;
use std::thread;

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
}

// Type of the list of pending authorisation requests
pub type PendingAuthReqs = Vec<AuthReq>;

// Type of the function/callback invoked for notifying and querying if an authorisation request
// shall be allowed. All the relevant information about the authorisation request is passed as args to the callback.
pub type AuthAllowPrompt = dyn Fn(AuthReq) -> Option<bool> + std::marker::Send + std::marker::Sync;

// Path of authenticator endpoint for getting an status report of the safe-authd
const SAFE_AUTHD_ENDPOINT_STATUS: &str = "status";

// Path of authenticator endpoint for login into a SAFE account
const SAFE_AUTHD_ENDPOINT_LOGIN: &str = "login/";

// Path of authenticator endpoint for loging out from a SAFE account
const SAFE_AUTHD_ENDPOINT_LOGOUT: &str = "logout";

// Path of authenticator endpoint for creating a new SAFE account
const SAFE_AUTHD_ENDPOINT_CREATE: &str = "create/";

// Path of authenticator endpoint for fetching list of authorised apps
const SAFE_AUTHD_ENDPOINT_AUTHED_APPS: &str = "authed-apps";

// Path of authenticator endpoint for revoking applications and/or permissions
const SAFE_AUTHD_ENDPOINT_REVOKE: &str = "revoke/";

// Path of authenticator endpoint for retrieving the list of pending authorisation requests
const SAFE_AUTHD_ENDPOINT_AUTH_REQS: &str = "auth-reqs";

// Path of authenticator endpoint for allowing an authorisation request
const SAFE_AUTHD_ENDPOINT_ALLOW: &str = "allow/";

// Path of authenticator endpoint for denying an authorisation request
const SAFE_AUTHD_ENDPOINT_DENY: &str = "deny/";

// Path of authenticator endpoint for subscribing to authorisation requests notifications
const SAFE_AUTHD_ENDPOINT_SUBSCRIBE: &str = "subscribe/";

// Path of authenticator endpoint for unsubscribing from authorisation requests notifications
const SAFE_AUTHD_ENDPOINT_UNSUBSCRIBE: &str = "unsubscribe/";

// authd subcommand to start the daemon
const SAFE_AUTHD_CMD_START: &str = "start";

// authd subcommand to stop the daemon
const SAFE_AUTHD_CMD_STOP: &str = "stop";

// authd subcommand to restart the daemon
const SAFE_AUTHD_CMD_RESTART: &str = "restart";

// Authd Client API
pub struct SafeAuthdClient {
    port: u16,
    endpoint_thread_handle: Option<thread::JoinHandle<()>>,
    callback_thread_handle: Option<thread::JoinHandle<()>>,
    // TODO: add a session_token field to use for communicating with authd for restricted operations,
    // we should restrict operations like subscribe, or allow/deny, to only be accepted with a valid token
    // session_token: String,
}

impl Drop for SafeAuthdClient {
    fn drop(&mut self) {
        // TODO: send message to terminate thread
        //let _ = self.endpoint_thread_handle.take().unwrap().join();
    }
}

#[allow(dead_code)]
impl SafeAuthdClient {
    pub fn new(port: Option<u16>) -> Self {
        let port_number = port.unwrap_or(SAFE_AUTHD_ENDPOINT_PORT);
        Self {
            port: port_number,
            endpoint_thread_handle: None,
            callback_thread_handle: None,
        }
    }

    // Start the Authenticator daemon
    pub fn start(&self, authd_path: Option<&str>) -> Result<()> {
        let path = authd_path.unwrap_or_else(|| "");
        debug!("Attempting to start authd from '{}' ...", path);
        authd_run_cmd(path, SAFE_AUTHD_CMD_START)
    }

    // Stop the Authenticator daemon
    pub fn stop(&self, authd_path: Option<&str>) -> Result<()> {
        let path = authd_path.unwrap_or_else(|| "");
        debug!("Attempting to stop authd from '{}' ...", path);
        authd_run_cmd(path, SAFE_AUTHD_CMD_STOP)
    }

    // Restart the Authenticator daemon
    pub fn restart(&self, authd_path: Option<&str>) -> Result<()> {
        let path = authd_path.unwrap_or_else(|| "");
        debug!("Attempting to restart authd from '{}' ...", path);
        authd_run_cmd(path, SAFE_AUTHD_CMD_RESTART)
    }

    // Send a request to remote authd endpoint to obtain an status report
    pub fn status(&mut self) -> Result<AuthdStatus> {
        debug!("Attempting to retrieve status report from remote authd...");
        let authd_service_url = format!(
            "{}:{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_STATUS
        );

        info!("Sending status report request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        info!(
            "SAFE status report retrieved successfully: {}",
            authd_response
        );

        let status_report: AuthdStatus = serde_json::from_str(&authd_response).map_err(|err| {
            Error::AuthdClientError(format!("Failed to parse status report: {}", err))
        })?;

        Ok(status_report)
    }

    // Send a login action request to remote authd endpoint
    pub fn log_in(&mut self, secret: &str, password: &str) -> Result<()> {
        debug!("Attempting to log in on remote authd...");
        let authd_service_url = format!(
            "{}:{}/{}{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_LOGIN, secret, password
        );

        info!("Sending login action to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        info!("SAFE login action was successful: {}", authd_response);
        // TODO: store the authd session token, replacing an existing one
        // self.session_token = authd_response;

        Ok(())
    }

    // Sends a logout action request to the SAFE Authenticator
    pub fn log_out(&mut self) -> Result<()> {
        debug!("Dropping logged in session and logging out in remote authd...");

        let authd_service_url = format!(
            "{}:{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_LOGOUT
        );

        info!("Sending logout action to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        info!("SAFE logout action was successful: {}", authd_response);

        // TODO: clean up the stored authd session token
        // self.session_token = "".to_string();

        Ok(())
    }

    // Sends an account creation request to the SAFE Authenticator
    pub fn create_acc(&self, sk: &str, secret: &str, password: &str) -> Result<()> {
        debug!("Attempting to create a SAFE account on remote authd...");
        let authd_service_url = format!(
            "{}:{}/{}{}/{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_CREATE, secret, password, sk
        );

        debug!("Sending account creation request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "SAFE account creation action was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Get the list of applications authorised from remote authd
    pub fn authed_apps(&self) -> Result<AuthedAppsList> {
        debug!("Attempting to fetch list of authorised apps from remote authd...");
        let authd_service_url = format!(
            "{}:{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_AUTHED_APPS
        );

        debug!("Sending request request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "List of applications authorised successfully received: {}",
            authd_response
        );

        let authed_apps_list: AuthedAppsList =
            serde_json::from_str(&authd_response).map_err(|err| {
                Error::AuthdClientError(format!("Failed to parse list of authorised apps: {}", err))
            })?;

        Ok(authed_apps_list)
    }

    // Revoke all permissions from an application
    pub fn revoke_app(&self, app_id: &str) -> Result<()> {
        debug!(
            "Requesting to revoke permissions from application: {}",
            app_id
        );
        let authd_service_url = format!(
            "{}:{}/{}{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_REVOKE, app_id
        );

        debug!("Sending revoke action request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "Application revocation action successful: {}",
            authd_response
        );
        Ok(())
    }

    // Get the list of pending authorisation requests from remote authd
    pub fn auth_reqs(&self) -> Result<PendingAuthReqs> {
        debug!("Attempting to fetch list of pending authorisation requests from remote authd...");
        let authd_service_url = format!(
            "{}:{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_AUTH_REQS
        );

        debug!("Sending request request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "List of pending authorisation requests successfully received: {}",
            authd_response
        );

        let auth_reqs_list: PendingAuthReqs =
            serde_json::from_str(&authd_response).map_err(|err| {
                Error::AuthdClientError(format!("Failed to parse list of auth reqs: {}", err))
            })?;

        Ok(auth_reqs_list)
    }

    // Allow an authorisation request
    pub fn allow(&self, req_id: SafeAuthReqId) -> Result<()> {
        debug!("Requesting to allow authorisation request: {}", req_id);
        let authd_service_url = format!(
            "{}:{}/{}{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_ALLOW, req_id
        );

        debug!("Sending allow action request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "Action to allow authorisation request was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Deny an authorisation request
    pub fn deny(&self, req_id: SafeAuthReqId) -> Result<()> {
        debug!("Requesting to deny authorisation request: {}", req_id);
        let authd_service_url = format!(
            "{}:{}/{}{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_DENY, req_id
        );

        debug!("Sending deny action request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "Action to deny authorisation request was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Subscribe a callback to receive notifications to allow/deny authorisation requests
    pub fn subscribe<
        CB: 'static + Fn(AuthReq) -> Option<bool> + std::marker::Send + std::marker::Sync,
    >(
        &mut self,
        endpoint_url: &str,
        allow_cb: CB,
    ) -> Result<()> {
        debug!("Subscribing to receive authorisation requests notifications...",);

        let url_encoded = urlencoding::encode(endpoint_url);
        let authd_service_url = format!(
            "{}:{}/{}{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_SUBSCRIBE, url_encoded
        );

        debug!("Sending subscribe action request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "Successfully subscribed to receive authorisation requests notifications: {}",
            authd_response
        );

        // Start listening first
        let listen = endpoint_url.to_string();

        // We need a channel to receive auth req notifications from the thread running the QUIC endpoint
        let (tx, rx): (mpsc::Sender<AuthReq>, mpsc::Receiver<AuthReq>) = mpsc::channel();

        // TODO: use Tokio futures with singled-threaded tasks and mpsc channel to receive reqs callbacks
        let endpoint_thread_join_handle = thread::spawn(move || match quic_listen(&listen, tx) {
            Ok(_) => {
                info!("Endpoint successfully launched for receiving auth req notifications");
            }
            Err(err) => {
                error!(
                    "Failed to launch endpoint for receiving auth req notifications: {}",
                    err
                );
            }
        });
        self.endpoint_thread_handle = Some(endpoint_thread_join_handle);

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
                    cb(auth_req);
                }
                Err(err) => {
                    debug!("Failed to receive message: {}", err);
                }
            }
        });
        self.callback_thread_handle = Some(cb_thread_join_handle);

        Ok(())
    }

    // Subscribe an endpoint URL where notifications to allow/deny authorisation requests shall be sent
    pub fn subscribe_url(&self, endpoint_url: &str) -> Result<()> {
        debug!(
            "Subscribing '{}' as endpoint for authorisation requests notifications...",
            endpoint_url
        );
        let url_encoded = urlencoding::encode(endpoint_url);
        let authd_service_url = format!(
            "{}:{}/{}{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_SUBSCRIBE, url_encoded
        );

        debug!("Sending subscribe action request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "Successfully subscribed a URL for authorisation requests notifications: {}",
            authd_response
        );
        Ok(())
    }

    // Unsubscribe from notifications to allow/deny authorisation requests
    pub fn unsubscribe(&self, endpoint_url: &str) -> Result<()> {
        debug!("Unsubscribing from authorisation requests notifications...",);
        let url_encoded = urlencoding::encode(endpoint_url);
        let authd_service_url = format!(
            "{}:{}/{}{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_UNSUBSCRIBE, url_encoded
        );

        debug!("Sending unsubscribe action request to SAFE Authenticator...");
        let authd_response = send_request(&authd_service_url)?;

        debug!(
            "Successfully unsubscribed from authorisation requests notifications: {}",
            authd_response
        );

        // TODO: terminate endpoint_thread_handle thread

        Ok(())
    }
}

fn send_request(url_str: &str) -> Result<String> {
    quic_send(&url_str, false, None, None, false)
}

pub fn authd_run_cmd(authd_path: &str, command: &str) -> Result<()> {
    let output = Command::new(&authd_path)
        .arg(command)
        .output()
        .map_err(|err| {
            Error::AuthdClientError(format!(
                "Failed to start authd from '{}': {}",
                authd_path, err
            ))
        })?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| Error::AuthdClientError(format!("Failed to output stdout: {}", err)))?;
        Ok(())
    } else {
        io::stderr()
            .write_all(&output.stderr)
            .map_err(|err| Error::AuthdClientError(format!("Failed to output stderr: {}", err)))?;
        Err(Error::AuthdClientError(
            "Failed to invoke safe-authd executable".to_string(),
        ))
    }
}
