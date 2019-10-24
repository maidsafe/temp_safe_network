// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::auth_daemon::*;
use crate::operations::safe_net::*;
use safe_api::{Safe, SafeAuthdClient};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum AuthSubCommands {
    #[structopt(name = "clear")]
    /// Clear SAFE CLI authorisation credentials from local file
    Clear {},
    #[structopt(name = "login")]
    /// Send request to a remote Authenticator daemon to login to a SAFE account
    Login {},
    #[structopt(name = "logout")]
    /// Send request to a remote Authenticator daemon to logout from currently logged in SAFE account
    Logout {},
    #[structopt(name = "status")]
    /// Send request to a remote Authenticator daemon to obtain an status report
    Status {},
    #[structopt(name = "create-acc")]
    /// Send request to a remote Authenticator daemon to create a new SAFE account
    Create {
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
        notifs_endpoint: String,
    },
    #[structopt(name = "unsubscribe")]
    /// Send request to a remote Authenticator daemon to unsubscribe an endpoint URL from authorisation requests notifications
    Unsubscribe {
        /// The endpoint URL to unsubscribe
        notifs_endpoint: String,
    },
    #[structopt(name = "start")]
    /// Starts the Authenticator daemon if it's not running already
    Start {},
    #[structopt(name = "stop")]
    /// Stops the Authenticator daemon if it's running
    Stop {},
    #[structopt(name = "restart")]
    /// Restarts the Authenticator daemon if it's running already
    Restart {},
}

pub fn auth_commander(
    cmd: Option<AuthSubCommands>,
    port: Option<u16>,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(AuthSubCommands::Create { sk, test_coins }) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_create(safe, &safe_authd, sk, test_coins)
        }
        Some(AuthSubCommands::Login {}) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            authd_login(&mut safe_authd)
        }
        Some(AuthSubCommands::Logout {}) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            authd_logout(&mut safe_authd)
        }
        Some(AuthSubCommands::Status {}) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            authd_status(&mut safe_authd)
        }
        Some(AuthSubCommands::Apps {}) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_apps(&safe_authd)
        }
        Some(AuthSubCommands::Clear {}) => clear_credentials(),
        Some(AuthSubCommands::Revoke { app_id }) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_revoke(&safe_authd, app_id)
        }
        Some(AuthSubCommands::Reqs {}) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_auth_reqs(&safe_authd)
        }
        Some(AuthSubCommands::Allow { req_id }) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_allow(&safe_authd, req_id)
        }
        Some(AuthSubCommands::Deny { req_id }) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_deny(&safe_authd, req_id)
        }
        Some(AuthSubCommands::Subscribe { notifs_endpoint }) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_subscribe_url(&safe_authd, notifs_endpoint)
        }
        Some(AuthSubCommands::Unsubscribe { notifs_endpoint }) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_unsubscribe(&safe_authd, notifs_endpoint)
        }
        Some(AuthSubCommands::Start {}) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_start(&safe_authd)
        }
        Some(AuthSubCommands::Stop {}) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_stop(&safe_authd)
        }
        Some(AuthSubCommands::Restart {}) => {
            let safe_authd = SafeAuthdClient::new(None);
            authd_restart(&safe_authd)
        }
        None => authorise_cli(safe, port),
    }
}
