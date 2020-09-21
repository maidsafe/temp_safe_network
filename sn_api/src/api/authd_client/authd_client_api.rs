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
    constants::{SN_AUTHD_ENDPOINT_HOST, SN_AUTHD_ENDPOINT_PORT},
    notifs_endpoint::jsonrpc_listen,
};
use crate::{AuthedAppsList, Error, Result, SafeAuthReqId};
use directories::BaseDirs;
use log::{debug, error, info, trace};
use safe_core::ipc::req::ContainerPermissions;
use sn_data_types::AppPermissions;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

#[cfg(not(target_os = "windows"))]
const SN_AUTHD_EXECUTABLE: &str = "sn_authd";

#[cfg(target_os = "windows")]
const SN_AUTHD_EXECUTABLE: &str = "sn_authd.exe";

const ENV_VAR_SN_AUTHD_PATH: &str = "SN_AUTHD_PATH";

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

// Authenticator method for getting a status report of the sn_authd
const SN_AUTHD_METHOD_STATUS: &str = "status";

// Authenticator method for logging into a SAFE account
const SN_AUTHD_METHOD_LOGIN: &str = "login";

// Authenticator method for logging out from a SAFE account
const SN_AUTHD_METHOD_LOGOUT: &str = "logout";

// Authenticator method for creating a new SAFE account
const SN_AUTHD_METHOD_CREATE: &str = "create-acc";

// Authenticator method for fetching list of authorised apps
const SN_AUTHD_METHOD_AUTHED_APPS: &str = "authed-apps";

// Authenticator method for revoking applications and/or permissions
const SN_AUTHD_METHOD_REVOKE: &str = "revoke";

// Authenticator method for retrieving the list of pending authorisation requests
const SN_AUTHD_METHOD_AUTH_REQS: &str = "auth-reqs";

// Authenticator method for allowing an authorisation request
const SN_AUTHD_METHOD_ALLOW: &str = "allow";

// Authenticator method for denying an authorisation request
const SN_AUTHD_METHOD_DENY: &str = "deny";

// Authenticator method for subscribing to authorisation requests notifications
const SN_AUTHD_METHOD_SUBSCRIBE: &str = "subscribe";

// Authenticator method for unsubscribing from authorisation requests notifications
const SN_AUTHD_METHOD_UNSUBSCRIBE: &str = "unsubscribe";

// authd subcommand to update the binary to new available released version
const SN_AUTHD_CMD_UPDATE: &str = "update";

// authd subcommand to start the daemon
const SN_AUTHD_CMD_START: &str = "start";

// authd subcommand to stop the daemon
const SN_AUTHD_CMD_STOP: &str = "stop";

// authd subcommand to restart the daemon
const SN_AUTHD_CMD_RESTART: &str = "restart";

// Authd Client API
pub struct SafeAuthdClient {
    // authd endpoint
    pub authd_endpoint: String,
    // keep track of (endpoint URL, join handle for the listening thread, join handle of callback thread)
    subscribed_endpoint: Option<(String, task::JoinHandle<()>, task::JoinHandle<()>)>,
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
                match futures::executor::block_on(send_unsubscribe(url, &self.authd_endpoint)) {
                    Ok(msg) => {
                        debug!("{}", msg);
                    }
                    Err(err) => {
                        // We are still ok, it was just us trying to be nice and unsubscribe if possible
                        // It could be the case we were already unsubscribe automatically by authd before
                        // we were attempting to do it now, which can happen due to our endpoint
                        // being unresponsive, so it's all ok
                        debug!("Failed to unsubscribe endpoint from authd: {}", err);
                    }
                }
            }
        }
    }
}

impl SafeAuthdClient {
    pub fn new(endpoint: Option<String>) -> Self {
        let endpoint = match endpoint {
            None => format!("{}:{}", SN_AUTHD_ENDPOINT_HOST, SN_AUTHD_ENDPOINT_PORT),
            Some(endpoint) => endpoint,
        };
        debug!("Creating new authd client for endpoint {}", endpoint);
        Self {
            authd_endpoint: endpoint,
            subscribed_endpoint: None,
        }
    }

    // Update the Authenticator binary to a new released version
    pub fn update(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(authd_path, &[SN_AUTHD_CMD_UPDATE])
    }

    // Start the Authenticator daemon
    pub fn start(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(
            authd_path,
            &[SN_AUTHD_CMD_START, "--listen", &self.authd_endpoint],
        )
    }

    // Stop the Authenticator daemon
    pub fn stop(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(authd_path, &[SN_AUTHD_CMD_STOP])
    }

    // Restart the Authenticator daemon
    pub fn restart(&self, authd_path: Option<&str>) -> Result<()> {
        authd_run_cmd(
            authd_path,
            &[SN_AUTHD_CMD_RESTART, "--listen", &self.authd_endpoint],
        )
    }

    // Send a request to remote authd endpoint to obtain a status report
    pub async fn status(&mut self) -> Result<AuthdStatus> {
        debug!("Attempting to retrieve status report from remote authd...");
        let status_report = send_authd_request::<AuthdStatus>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_STATUS,
            serde_json::Value::Null,
        )
        .await?;

        info!(
            "SAFE status report retrieved successfully: {:?}",
            status_report
        );
        Ok(status_report)
    }

    // Send a login action request to remote authd endpoint
    pub async fn log_in(&mut self, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to log in on remote authd...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_LOGIN,
            json!(vec![passphrase, password]),
        )
        .await?;

        info!("SAFE login action was successful: {}", authd_response);
        // TODO: store the authd session token, replacing an existing one
        // self.session_token = authd_response;

        Ok(())
    }

    // Sends a logout action request to the SAFE Authenticator
    pub async fn log_out(&mut self) -> Result<()> {
        debug!("Dropping logged in session and logging out in remote authd...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_LOGOUT,
            serde_json::Value::Null,
        )
        .await?;

        info!("SAFE logout action was successful: {}", authd_response);

        // TODO: clean up the stored authd session token
        // self.session_token = "".to_string();

        Ok(())
    }

    // Sends an account creation request to the SAFE Authenticator
    pub async fn create_acc(&self, sk: &str, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to create a SAFE account on remote authd...");
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_CREATE,
            json!(vec![passphrase, password, sk]),
        )
        .await?;

        debug!(
            "SAFE account creation action was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Get the list of applications authorised from remote authd
    pub async fn authed_apps(&self) -> Result<AuthedAppsList> {
        debug!("Attempting to fetch list of authorised apps from remote authd...");
        let authed_apps_list = send_authd_request::<AuthedAppsList>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_AUTHED_APPS,
            serde_json::Value::Null,
        )
        .await?;

        debug!(
            "List of applications authorised successfully received: {:?}",
            authed_apps_list
        );
        Ok(authed_apps_list)
    }

    // Revoke all permissions from an application
    pub async fn revoke_app(&self, app_id: &str) -> Result<()> {
        debug!(
            "Requesting to revoke permissions from application: {}",
            app_id
        );
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_REVOKE,
            json!(app_id),
        )
        .await?;

        debug!(
            "Application revocation action successful: {}",
            authd_response
        );
        Ok(())
    }

    // Get the list of pending authorisation requests from remote authd
    pub async fn auth_reqs(&self) -> Result<PendingAuthReqs> {
        debug!("Attempting to fetch list of pending authorisation requests from remote authd...");
        let auth_reqs_list = send_authd_request::<PendingAuthReqs>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_AUTH_REQS,
            serde_json::Value::Null,
        )
        .await?;

        debug!(
            "List of pending authorisation requests successfully received: {:?}",
            auth_reqs_list
        );
        Ok(auth_reqs_list)
    }

    // Allow an authorisation request
    pub async fn allow(&self, req_id: SafeAuthReqId) -> Result<()> {
        debug!("Requesting to allow authorisation request: {}", req_id);
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_ALLOW,
            json!(req_id.to_string()),
        )
        .await?;

        debug!(
            "Action to allow authorisation request was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Deny an authorisation request
    pub async fn deny(&self, req_id: SafeAuthReqId) -> Result<()> {
        debug!("Requesting to deny authorisation request: {}", req_id);
        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_DENY,
            json!(req_id.to_string()),
        )
        .await?;

        debug!(
            "Action to deny authorisation request was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Subscribe a callback to receive notifications to allow/deny authorisation requests
    // We support having only one subscripton at a time, a previous subscription will be dropped
    pub async fn subscribe<
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
        let dirs = directories::ProjectDirs::from("net", "maidsafe", "sn_authd-client")
            .ok_or_else(|| {
                Error::AuthdClientError(
                    "Failed to obtain local home directory where to store endpoint certificates to"
                        .to_string(),
                )
            })?;

        // Let's postfix the path with the app id so we avoid clashes with other
        // endpoints subscribed from within the same local box
        let cert_base_path = dirs.config_dir().join(app_id.to_string());

        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_SUBSCRIBE,
            json!(vec![endpoint_url, &cert_base_path.display().to_string()]),
        ).await.map_err(|err| Error::AuthdClientError(format!("Failed when trying to subscribe endpoint URL ({}) to receive authorisation request for self-auth: {}", endpoint_url, err)))?;

        debug!(
            "Successfully subscribed to receive authorisation requests notifications: {}",
            authd_response
        );

        // Start listening first
        // We need a channel to receive auth req notifications from the thread running the QUIC endpoint
        let (tx, mut rx) = mpsc::unbounded_channel::<(AuthReq, oneshot::Sender<Option<bool>>)>();

        let listen = endpoint_url.to_string();
        // TODO: if there was a previous subscription,
        // make sure we kill/stop the previously created tasks
        let endpoint_thread_join_handle = tokio::spawn(async move {
            match jsonrpc_listen(&listen, &cert_base_path.display().to_string(), tx).await {
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
        let cb_thread_join_handle = tokio::spawn(async move {
            while let Some((auth_req, decision_tx)) = rx.recv().await {
                debug!(
                    "Notification for authorisation request ({}) from app ID '{}' received",
                    auth_req.req_id, auth_req.app_id
                );

                // Let's get the decision from the user by invoking the callback provided
                let user_decision = cb(auth_req);

                // Send the decision received back to authd-client so it
                // can in turn send it to authd
                match decision_tx.send(user_decision) {
                    Ok(_) => debug!("Auth req decision sent to authd"),
                    Err(_) => error!("Auth req decision couldn't be sent back to authd"),
                };
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
    pub async fn subscribe_url(&self, endpoint_url: &str) -> Result<()> {
        debug!(
            "Subscribing '{}' as endpoint for authorisation requests notifications...",
            endpoint_url
        );

        let authd_response = send_authd_request::<String>(
            &self.authd_endpoint,
            SN_AUTHD_METHOD_SUBSCRIBE,
            json!(vec![endpoint_url]),
        )
        .await?;

        debug!(
            "Successfully subscribed a URL for authorisation requests notifications: {}",
            authd_response
        );
        Ok(())
    }

    // Unsubscribe from notifications to allow/deny authorisation requests
    pub async fn unsubscribe(&mut self, endpoint_url: &str) -> Result<()> {
        debug!("Unsubscribing from authorisation requests notifications...",);
        let authd_response = send_unsubscribe(endpoint_url, &self.authd_endpoint).await?;
        debug!(
            "Successfully unsubscribed from authorisation requests notifications: {}",
            authd_response
        );

        // If the URL is the same as the endpoint locally launched, terminate the thread
        if let Some((url, _, _)) = &self.subscribed_endpoint {
            if endpoint_url == url {
                // TODO: send signal to stop the currently running tasks
                self.subscribed_endpoint = None;
            }
        }

        Ok(())
    }
}

async fn send_unsubscribe(endpoint_url: &str, authd_endpoint: &str) -> Result<String> {
    send_authd_request::<String>(
        authd_endpoint,
        SN_AUTHD_METHOD_UNSUBSCRIBE,
        json!(endpoint_url),
    )
    .await
}

fn authd_run_cmd(authd_path: Option<&str>, args: &[&str]) -> Result<()> {
    let mut path = get_authd_bin_path(authd_path)?;
    path.push(SN_AUTHD_EXECUTABLE);
    let path_str = path.display().to_string();
    debug!("Attempting to {} authd from '{}' ...", args[0], path_str);

    let output = Command::new(&path_str)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| {
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
                // sn_authd exit code 10 is sn_authd::errors::Error::AuthdAlreadyStarted
                Err(Error::AuthdAlreadyStarted(format!(
                       "Failed to start sn_authd daemon '{}' as an instance seems to be already running",
                       path_str,
                   )))
            }
            Some(_) | None => Err(Error::AuthdError(format!(
                "Failed when invoking sn_authd executable from '{}'",
                path_str,
            ))),
        }
    }
}

fn get_authd_bin_path(authd_path: Option<&str>) -> Result<PathBuf> {
    match authd_path {
        Some(p) => Ok(PathBuf::from(p)),
        None => {
            // if SN_AUTHD_PATH is set it then overrides default
            if let Ok(authd_path) = std::env::var(ENV_VAR_SN_AUTHD_PATH) {
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
