// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use dirs;
use log::debug;
use prettytable::Table;
use rpassword;
use safe_api::{Safe, SafeAuthdClient};
use std::fs::{DirBuilder, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

const APP_ID: &str = "net.maidsafe.cli";
const APP_NAME: &str = "SAFE CLI";
const APP_VENDOR: &str = "MaidSafe.net Ltd";
const AUTH_CREDENTIALS_FILENAME: &str = "credentials";
const PROJECT_DATA_DIR_QUALIFIER: &str = "net";
const PROJECT_DATA_DIR_ORGANISATION: &str = "MaidSafe";
const PROJECT_DATA_DIR_APPLICATION: &str = "safe-cli";

#[derive(StructOpt, Debug)]
pub enum AuthSubCommands {
    #[structopt(name = "clear")]
    /// Clear authorisation credentials from local file
    Clear {},
    #[structopt(name = "login")]
    /// Send request to a remote Authenticator daemon to login to a SAFE account
    Login {},
    #[structopt(name = "logout")]
    /// Send request to a remote Authenticator daemon to logout from currently logged in SAFE account
    Logout {},
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
    #[structopt(name = "auth-reqs")]
    /// Send request to a remote Authenticator daemon to retrieve the list of the pending authorisation requests
    AuthReqs {},
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
    #[structopt(name = "start-authd")]
    /// Starts the Authenticator daemon if it's not running already
    StartAuthd {},
    #[structopt(name = "stop-authd")]
    /// Stops the Authenticator daemon if it's running
    StopAuthd {},
    #[structopt(name = "restart-authd")]
    /// Restarts the Authenticator daemon if it's running already
    RestartAuthd {},
}

pub fn auth_commander(
    cmd: Option<AuthSubCommands>,
    port: Option<u16>,
    safe: &mut Safe,
) -> Result<(), String> {
    let file_path = credentials_file_path()?;
    let mut file = File::create(&file_path)
        .map_err(|_| format!("Unable to create credentials file at {}", file_path))?;

    match cmd {
        Some(AuthSubCommands::Create { sk, test_coins }) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            let secret = prompt_sensitive(None, "Secret:")?;
            let password = prompt_sensitive(None, "Password:")?;
            if test_coins {
                // We then generate a SafeKey with test-coins to use it for the account creation
                println!("Creating a SafeKey with test-coins...");
                let (_xorurl, key_pair) = safe.keys_create_preload_test_coins("1000.11")?;
                let kp = key_pair
                    .ok_or("Faild to obtain the secret key of the newly created SafeKey")?;
                println!("Sending account creation request to authd...");
                safe_authd.create_acc(&kp.sk, &secret, &password)?;
                println!("Account was created successfully!");
                println!(
                    "SafeKey created and preloaded with test-coins. Owner key pair generated:"
                );
                println!("Public Key = {}", kp.pk);
                println!("Secret Key = {}", kp.sk);
            } else {
                let sk = prompt_sensitive(sk, "Enter SafeKey's secret key to pay with:")?;
                println!("Sending account creation request to authd...");
                safe_authd.create_acc(&sk, &secret, &password)?;
                println!("Account was created successfully!");
            };
            Ok(())
        }
        Some(AuthSubCommands::Login {}) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            let secret = prompt_sensitive(None, "Secret:")?;
            let password = prompt_sensitive(None, "Password:")?;
            println!("Sending login action request to authd...");
            safe_authd.log_in(&secret, &password)?;
            println!("Logged in successfully");
            Ok(())
        }
        Some(AuthSubCommands::Logout {}) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            println!("Sending logout action request to authd...");
            safe_authd.log_out()?;
            println!("Logged out successfully");
            Ok(())
        }
        Some(AuthSubCommands::Apps {}) => {
            let safe_authd = SafeAuthdClient::new(None);
            println!("Requesting list of authorised apps from authd...");
            let authed_apps = safe_authd.authed_apps()?;
            pretty_print_authed_apps(authed_apps);

            Ok(())
        }
        Some(AuthSubCommands::Clear {}) => {
            file.set_len(0).map_err(|err| {
                format!("Unable to clear credentials from {}: {}", file_path, err)
            })?;

            println!("Credentials were succesfully cleared from {}", file_path);
            Ok(())
        }
        Some(AuthSubCommands::Revoke { app_id }) => {
            let safe_authd = SafeAuthdClient::new(None);
            println!("Sending application revocation request to authd...");
            safe_authd.revoke_app(&app_id)?;
            println!("Application revoked successfully");
            Ok(())
        }
        Some(AuthSubCommands::AuthReqs {}) => {
            let safe_authd = SafeAuthdClient::new(None);
            println!("Requesting list of pending authorisation requests from authd...");
            let auth_reqs = safe_authd.auth_reqs()?;
            //pretty_print_authed_apps(auth_reqs);
            println!("{}", auth_reqs);
            Ok(())
        }
        Some(AuthSubCommands::Allow { req_id }) => {
            let safe_authd = SafeAuthdClient::new(None);
            println!("Sending request to authd to allow an authorisation request...");
            safe_authd.allow(req_id)?;
            println!("Authorisation request was allowed successfully");
            Ok(())
        }
        Some(AuthSubCommands::Deny { req_id }) => {
            let safe_authd = SafeAuthdClient::new(None);
            println!("Sending request to authd to deny an authorisation request...");
            safe_authd.deny(req_id)?;
            println!("Authorisation request was denied successfully");
            Ok(())
        }
        Some(AuthSubCommands::Subscribe { notifs_endpoint }) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            println!("Sending request to subscribe...");
            //safe_authd.subscribe_url(&notifs_endpoint)?;
            safe_authd.subscribe(&notifs_endpoint, &|app_id| {
                println!("Allowing app {}", app_id);
                true
            })?;
            println!("Subscribed successfully");
            thread::sleep(Duration::from_millis(20000));
            Ok(())
        }
        Some(AuthSubCommands::Unsubscribe { notifs_endpoint }) => {
            let safe_authd = SafeAuthdClient::new(None);
            println!("Sending request to unsubscribe...");
            safe_authd.unsubscribe(&notifs_endpoint)?;
            println!("Unsubscribed successfully");
            Ok(())
        }
        Some(AuthSubCommands::StartAuthd {}) => run_authd_cmd("start"),
        Some(AuthSubCommands::StopAuthd {}) => run_authd_cmd("stop"),
        Some(AuthSubCommands::RestartAuthd {}) => run_authd_cmd("restart"),
        None => {
            println!("Authorising CLI application...");

            let auth_credentials = safe
                .auth_app(APP_ID, APP_NAME, APP_VENDOR, port)
                .map_err(|err| format!("Application authorisation failed: {}", err))?;

            file.write_all(auth_credentials.as_bytes())
                .map_err(|err| format!("Unable to write credentials in {}: {}", file_path, err))?;

            println!("SAFE CLI app was successfully authorised");
            println!("Credentials were stored in {}", file_path);
            Ok(())
        }
    }
}

// TODO: use a different crate than rpassword as it has problems with some Windows shells including PowerShell
fn prompt_sensitive(arg: Option<String>, msg: &str) -> Result<String, String> {
    if let Some(str) = arg {
        Ok(str)
    } else {
        rpassword::read_password_from_tty(Some(msg))
            .map_err(|err| format!("Failed reading string from input: {}", err))
    }
}

fn run_authd_cmd(command: &str) -> Result<(), String> {
    let authd_bin_path = get_authd_bin_path();
    let output = Command::new(&authd_bin_path)
        .arg(command)
        .output()
        .map_err(|err| format!("Failed to start authd from '{}': {}", authd_bin_path, err))?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| format!("Failed to output stdout: {}", err))?;
        Ok(())
    } else {
        io::stderr()
            .write_all(&output.stderr)
            .map_err(|err| format!("Failed to output stderr: {}", err))?;
        Err("Failed to run safe-authd".to_string())
    }
}

fn get_authd_bin_path() -> String {
    let target_dir = match std::env::var("CARGO_TARGET_DIR") {
        Ok(target_dir) => target_dir,
        Err(_) => "target".to_string(),
    };

    if cfg!(debug_assertions) {
        format!("{}{}", target_dir, "/debug/safe-authd")
    } else {
        format!("{}{}", target_dir, "/release/safe-authd")
    }
}

pub fn auth_connect(safe: &mut Safe) -> Result<(), String> {
    debug!("Connecting...");

    let file_path = credentials_file_path()?;
    let mut file = File::open(&file_path)
        .map_err(|_| "You need to authorise the safe CLI first with 'auth' command")?;

    let mut auth_credentials = String::new();
    file.read_to_string(&mut auth_credentials)
        .map_err(|_| format!("Unable to read credentials from {}", file_path))?;

    safe.connect(APP_ID, Some(&auth_credentials))
        .map_err(|err| {
            format!(
                "You need to authorise the safe CLI first with 'auth' command: {}",
                err
            )
        })
}

#[allow(dead_code)]
pub fn connect_without_auth(safe: &mut Safe) -> Result<(), String> {
    debug!("Connecting...");

    safe.connect(APP_ID, None)?;

    Ok(())
}

fn credentials_file_path() -> Result<String, String> {
    let project_data_path = ProjectDirs::from(
        PROJECT_DATA_DIR_QUALIFIER,
        PROJECT_DATA_DIR_ORGANISATION,
        PROJECT_DATA_DIR_APPLICATION,
    )
    .ok_or_else(|| "Couldn't find user's home directory".to_string())?;

    let data_local_path = project_data_path.data_local_dir();

    if !data_local_path.exists() {
        println!("Creating '{}' folder", data_local_path.display());
        create_dir_all(data_local_path)
            .map_err(|err| format!("Couldn't create project's local data folder: {}", err))?;
    }

    let path = data_local_path.join(AUTH_CREDENTIALS_FILENAME);
    Ok(path.display().to_string())
}

pub fn pretty_print_authed_apps(authed_apps: /*Vec<AuthedAppsList>*/ String) {
    let mut table = Table::new();
    table.add_row(row![bFg->"Authorised Applications"]);
    /*table.add_row(row![bFg->"Id", bFg->"Name", bFg->"Vendor", bFg->"Permissions"]);
    table.add_row(row![]);
    let all_app_iterator = authed_apps.iter();
    for app_info in all_app_iterator {
        let mut row = String::from("");
        for (cont, perms) in app_info.perms.iter() {
            row += &format!("{}: {:?}\n", cont, perms);
        }
        table.add_row(row![
            app_info.app.id,
            app_info.app.name,
            // app_info.app.scope || "",
            app_info.app.vendor,
            row,
        ]);
    }*/
    table.printstd();
    println!("{}", authed_apps);
}
