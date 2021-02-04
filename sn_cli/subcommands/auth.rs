// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{operations::auth_daemon::*, operations::safe_net::*, APP_ID, APP_NAME, APP_VENDOR};
use anyhow::{bail, Result};
use log::debug;
use sn_api::{AuthReq, Safe, SafeAuthdClient};
use structopt::StructOpt;

const AUTH_REQ_NOTIFS_ENDPOINT: &str = "https://localhost:33002";

#[derive(StructOpt, Debug)]
pub enum AuthSubCommands {
    #[structopt(name = "clear")]
    /// Clear Safe CLI authorisation credentials from local file
    Clear {},
    #[structopt(name = "unlock")]
    /// Send request to a remote Authenticator daemon to unlock a Safe
    Unlock {
        /// A config file to read passphrase/password from. This is a convenience function, which is not recommended (storing this information unencrypted is not secure). Json file format = { "passphrase": "mypassphrase", "password": "mypassword" }
        #[structopt(short = "c", long = "config")]
        config_file_str: Option<String>,
        /// Automatically self authorise the CLI application adding its keys to the Safe that is being unlocked
        #[structopt(long = "self-auth")]
        self_auth: bool,
    },
    #[structopt(name = "lock")]
    /// Send request to a remote Authenticator daemon to lock any currently unlocked Safe
    Lock {},
    #[structopt(name = "status")]
    /// Send request to a remote Authenticator daemon to obtain a status report
    Status {},
    #[structopt(name = "create")]
    /// Send request to a remote Authenticator daemon to create a new Safe
    Create {
        /// A config file to read passphrase/password from. This is a convenience function, which is not recommended (storing this information unencrypted is not secure). Json file format = { "passphrase": "mypassphrase", "password": "mypassword" }
        #[structopt(short = "c", long = "config")]
        config_file_str: Option<String>,
        /// The SafeKey's secret key to pay for the cost of Safe creation, and to be set as the owner of the newly created Safe
        #[structopt(long = "sk")]
        sk: Option<String>,
        /// Request the creation of a SafeKey with test-coins automatically to use it to pay for the cost of Safe creation
        #[structopt(long = "test-coins")]
        test_coins: bool,
    },
    #[structopt(name = "apps")]
    /// Send request to a remote Authenticator daemon to retrieve the list of the authorised applications
    Apps {},
    #[structopt(name = "revoke")]
    /// Send request to a remote Authenticator daemon to revoke permissions from a previously authorised application
    Revoke {
        /// The application ID
        app_id: String,
    },
    #[structopt(name = "reqs")]
    /// Send request to a remote Authenticator daemon to retrieve the list of the pending authorisation requests
    Reqs {},
    #[structopt(name = "allow")]
    /// Send request to a remote Authenticator daemon to allow an authorisation request
    Allow {
        /// The authorisation request ID. You can use the 'auth auth-reqs' command to obtain the list of pending authorisation requests and their corresponding IDs.
        req_id: u32,
    },
    #[structopt(name = "deny")]
    /// Send request to a remote Authenticator daemon to deny an authorisation request
    Deny {
        /// The authorisation request ID. You can use the 'auth auth-reqs' command to obtain the list of pending authorisation requests and their corresponding IDs.
        req_id: u32,
    },
    #[structopt(name = "subscribe")]
    /// Send request to a remote Authenticator daemon to subscribe an endpoint URL to receive authorisation requests notifications
    Subscribe {
        /// The endpoint URL to subscribe
        notifs_endpoint: Option<String>,
    },
    #[structopt(name = "unsubscribe")]
    /// Send request to a remote Authenticator daemon to unsubscribe an endpoint URL from authorisation requests notifications
    Unsubscribe {
        /// The endpoint URL to unsubscribe
        notifs_endpoint: Option<String>,
    },
    #[structopt(name = "install")]
    /// Install latest sn_authd released version in the system
    Install {
        #[structopt(long = "authd-path")]
        /// Path where to install sn_authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "update")]
    /// Update sn_authd binary to a new available released version
    Update {
        #[structopt(long = "authd-path")]
        /// Path of sn_authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "start")]
    /// Starts the Authenticator daemon if it's not running already
    Start {
        #[structopt(long = "authd-path")]
        /// Path of sn_authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "stop")]
    /// Stops the Authenticator daemon if it's running
    Stop {
        #[structopt(long = "authd-path")]
        /// Path of sn_authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "restart")]
    /// Restarts the Authenticator daemon if it's running already
    Restart {
        #[structopt(long = "authd-path")]
        /// Path of sn_authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
}

pub async fn auth_commander(
    cmd: Option<AuthSubCommands>,
    endpoint: Option<String>,
    safe: &mut Safe,
) -> Result<()> {
    match cmd {
        Some(AuthSubCommands::Create {
            config_file_str,
            sk,
            test_coins,
        }) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_create(safe, &sn_authd, config_file_str, sk, test_coins).await
        }
        Some(AuthSubCommands::Unlock {
            config_file_str,
            self_auth,
        }) => {
            let mut sn_authd = SafeAuthdClient::new(endpoint.clone());
            authd_unlock(&mut sn_authd, config_file_str).await?;
            if self_auth {
                self_authorise(endpoint, sn_authd).await?;
            }
            Ok(())
        }
        Some(AuthSubCommands::Lock {}) => {
            let mut sn_authd = SafeAuthdClient::new(endpoint);
            authd_lock(&mut sn_authd).await
        }
        Some(AuthSubCommands::Status {}) => {
            let mut sn_authd = SafeAuthdClient::new(endpoint);
            authd_status(&mut sn_authd).await
        }
        Some(AuthSubCommands::Apps {}) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_apps(&sn_authd).await
        }
        Some(AuthSubCommands::Clear {}) => clear_credentials(),
        Some(AuthSubCommands::Revoke { app_id }) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_revoke(&sn_authd, app_id).await
        }
        Some(AuthSubCommands::Reqs {}) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_auth_reqs(&sn_authd).await
        }
        Some(AuthSubCommands::Allow { req_id }) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_allow(&sn_authd, req_id).await
        }
        Some(AuthSubCommands::Deny { req_id }) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_deny(&sn_authd, req_id).await
        }
        Some(AuthSubCommands::Subscribe { notifs_endpoint }) => match notifs_endpoint {
            None => bail!("The endpoint URL needs to be provided. If you subscribe within the interactive shell the URL is then optional"),
            Some(notif_endpoint) => {
                let sn_authd = SafeAuthdClient::new(endpoint);
                authd_subscribe_url(&sn_authd, notif_endpoint).await
            }
        },
        Some(AuthSubCommands::Unsubscribe { notifs_endpoint }) => {
            let mut sn_authd = SafeAuthdClient::new(endpoint);
            authd_unsubscribe(&mut sn_authd, notifs_endpoint).await
        }
        Some(AuthSubCommands::Install {authd_path}) => {
            authd_install(authd_path)
        }
        Some(AuthSubCommands::Update {authd_path}) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_update(&sn_authd, authd_path)
        }
        Some(AuthSubCommands::Start {authd_path}) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_start(&sn_authd, authd_path)
        }
        Some(AuthSubCommands::Stop {authd_path}) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_stop(&sn_authd, authd_path)
        }
        Some(AuthSubCommands::Restart {authd_path}) => {
            let sn_authd = SafeAuthdClient::new(endpoint);
            authd_restart(&sn_authd, authd_path)
        }
        None => authorise_cli(endpoint, false).await,
    }
}

async fn self_authorise(endpoint: Option<String>, mut sn_authd: SafeAuthdClient) -> Result<()> {
    debug!("Let's subscribe so we can automatically allow our own auth request...");
    sn_authd
        .subscribe(
            AUTH_REQ_NOTIFS_ENDPOINT,
            APP_ID,
            &move |auth_req: AuthReq| {
                debug!("We received an auth req notification, let's allow it if it's CLI's one...");
                if auth_req.app_id == APP_ID
                    && auth_req.app_name == APP_NAME
                    && auth_req.app_vendor == APP_VENDOR
                {
                    Some(true)
                } else {
                    None
                }
            },
        )
        .await?;

    debug!("Send the authorisation request for CLI app now...");
    authorise_cli(endpoint, true).await?;
    Ok(())
}
