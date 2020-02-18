// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{operations::auth_daemon::*, operations::safe_net::*, APP_ID};
use safe_api::{AuthReq, Safe, SafeAuthdClient};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum AuthSubCommands {
    #[structopt(name = "clear")]
    /// Clear SAFE CLI authorisation credentials from local file
    Clear {},
    #[structopt(name = "login")]
    /// Send request to a remote Authenticator daemon to login to a SAFE account
    Login {
        /// A config file to read passphrase/password from. This is a convenience function, which is not recommended (storing login information unencrypted is not secure)
        #[structopt(short = "c", long = "config")]
        config_file_str: Option<String>,
        /// Automatically self authorise the CLI application using the account is being logged in with
        #[structopt(long = "self-auth")]
        self_auth: bool,
    },
    #[structopt(name = "logout")]
    /// Send request to a remote Authenticator daemon to logout from currently logged in SAFE account
    Logout {},
    #[structopt(name = "status")]
    /// Send request to a remote Authenticator daemon to obtain a status report
    Status {},
    #[structopt(name = "create-acc")]
    /// Send request to a remote Authenticator daemon to create a new SAFE account
    Create {
        /// A config file to read passphrase/password from. This is a convenience function, which is not recommended (storing login information unencrypted is not secure)
        #[structopt(short = "c", long = "config")]
        config_file_str: Option<String>,
        /// The SafeKey's secret key to pay for the account creation, and to be set as the default spendable balance in the newly created SAFE account
        #[structopt(long = "sk")]
        sk: Option<String>,
        /// Request the creation of a SafeKey with test-coins automatically to use it to pay for the account creation
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
    /// Install latest safe-authd released version in the system. On Windows platforms it also installs it as a service
    Install {
        #[structopt(long = "authd-path")]
        /// Path where to install safe-authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "uninstall")]
    /// Uninstall safe-authd service. Only for Windows platforms
    Uninstall {
        #[structopt(long = "authd-path")]
        /// Path where to uninstall safe-authd executable from (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "update")]
    /// Update safe-authd binary to a new available released version
    Update {
        #[structopt(long = "authd-path")]
        /// Path of safe-authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "start")]
    /// Starts the Authenticator daemon if it's not running already
    Start {
        #[structopt(long = "authd-path")]
        /// Path of safe-authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "stop")]
    /// Stops the Authenticator daemon if it's running
    Stop {
        #[structopt(long = "authd-path")]
        /// Path of safe-authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
    #[structopt(name = "restart")]
    /// Restarts the Authenticator daemon if it's running already
    Restart {
        #[structopt(long = "authd-path")]
        /// Path of safe-authd executable (default ~/.safe/authd/)
        authd_path: Option<String>,
    },
}

pub fn auth_commander(
    cmd: Option<AuthSubCommands>,
    endpoint: Option<String>,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(AuthSubCommands::Create {
            config_file_str,
            sk,
            test_coins,
        }) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_create(safe, &safe_authd, config_file_str, sk, test_coins)
        }
        Some(AuthSubCommands::Login {
            config_file_str,
            self_auth,
        }) => {
            let mut safe_authd = SafeAuthdClient::new(endpoint.clone());
            authd_login(&mut safe_authd, config_file_str)?;
            if self_auth {
                // Let's subscribe so we can automatically allow our own auth request
                safe_authd.subscribe(
                    "https://localhost:33002",
                    APP_ID,
                    &move |auth_req: AuthReq| {
                        // TODO: pass the endpoint
                        let safe_authd = SafeAuthdClient::new(None);
                        match safe_authd.allow(auth_req.req_id) {
                            Ok(()) => {}
                            Err(err) => println!("Failed to self authorise: {}", err),
                        }
                        None
                    },
                )?;
                authorise_cli(safe, endpoint, true)?;
            }
            Ok(())
        }
        Some(AuthSubCommands::Logout {}) => {
            let mut safe_authd = SafeAuthdClient::new(endpoint);
            authd_logout(&mut safe_authd)
        }
        Some(AuthSubCommands::Status {}) => {
            let mut safe_authd = SafeAuthdClient::new(endpoint);
            authd_status(&mut safe_authd)
        }
        Some(AuthSubCommands::Apps {}) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_apps(&safe_authd)
        }
        Some(AuthSubCommands::Clear {}) => clear_credentials(),
        Some(AuthSubCommands::Revoke { app_id }) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_revoke(&safe_authd, app_id)
        }
        Some(AuthSubCommands::Reqs {}) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_auth_reqs(&safe_authd)
        }
        Some(AuthSubCommands::Allow { req_id }) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_allow(&safe_authd, req_id)
        }
        Some(AuthSubCommands::Deny { req_id }) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_deny(&safe_authd, req_id)
        }
        Some(AuthSubCommands::Subscribe { notifs_endpoint }) => match notifs_endpoint {
            None => Err("The endpoint URL needs to be provided. If you subscribe within the interactive shell the URL is then optional".to_string()),
            Some(notif_endpoint) => {
                let safe_authd = SafeAuthdClient::new(endpoint);
                authd_subscribe_url(&safe_authd, notif_endpoint)
            }
        },
        Some(AuthSubCommands::Unsubscribe { notifs_endpoint }) => {
            let mut safe_authd = SafeAuthdClient::new(endpoint);
            authd_unsubscribe(&mut safe_authd, notifs_endpoint)
        }
        Some(AuthSubCommands::Install {authd_path}) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_install(&safe_authd, authd_path)
        }
        Some(AuthSubCommands::Uninstall {authd_path}) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_uninstall(&safe_authd, authd_path)
        }
        Some(AuthSubCommands::Update {authd_path}) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_update(&safe_authd, authd_path)
        }
        Some(AuthSubCommands::Start {authd_path}) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_start(&safe_authd, authd_path)
        }
        Some(AuthSubCommands::Stop {authd_path}) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_stop(&safe_authd, authd_path)
        }
        Some(AuthSubCommands::Restart {authd_path}) => {
            let safe_authd = SafeAuthdClient::new(endpoint);
            authd_restart(&safe_authd, authd_path)
        }
        None => authorise_cli(safe, endpoint, false),
    }
}
