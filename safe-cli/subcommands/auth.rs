// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::get_from_arg_or_stdin;
use dirs;
use log::debug;
use prettytable::Table;
use safe_api::{Safe, SafeAuthdClient};
use std::fs::{DirBuilder, File};
use std::io::{Read, Write};
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
    #[structopt(name = "create")]
    /// Send request to a remote Authenticator daemon to create a new SAFE account
    Create {},
    #[structopt(name = "apps")]
    /// Send request to a remote Authenticator daemon to retrieve the list of the authorised applications
    Apps {},
    #[structopt(name = "revoke")]
    /// Send request to a remote Authenticator daemon to revoke permissions from a previously authorised application
    Revoke {
        /// The application ID
        app_id: String,
    },
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
        Some(AuthSubCommands::Create {}) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            let secret = get_from_arg_or_stdin(None, Some("Secret:"))?;
            let password = get_from_arg_or_stdin(None, Some("Password:"))?;
            let sk = get_from_arg_or_stdin(None, Some("Enter SafeKey's secret key to pay with:"))?;
            println!("Sending account creation request to authd...");
            safe_authd.create_acc(&sk, &secret, &password)?;
            println!("Account created successfully");
            Ok(())
        }
        Some(AuthSubCommands::Login {}) => {
            let mut safe_authd = SafeAuthdClient::new(None);
            let secret = get_from_arg_or_stdin(None, Some("Secret:"))?;
            let password = get_from_arg_or_stdin(None, Some("Password:"))?;
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
    table.add_row(row![bFg->"Id", bFg->"Name", bFg->"Vendor", bFg->"Permissions"]);
    table.add_row(row![]);
    /*
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
