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
use safe_api::{AuthAllowPrompt, Safe, SafeAuthdClient};
use std::fs::{DirBuilder, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;

const APP_ID: &str = "net.maidsafe.cli";
const APP_NAME: &str = "SAFE CLI";
const APP_VENDOR: &str = "MaidSafe.net Ltd";
const AUTH_CREDENTIALS_FOLDER: &str = ".safe";
const AUTH_CREDENTIALS_FILENAME: &str = "credentials";

pub fn authd_run_cmd(command: &str) -> Result<(), String> {
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
        Err("Failed to invoke safe-authd executable".to_string())
    }
}

pub fn authd_create(
    safe: &mut Safe,
    safe_authd: &SafeAuthdClient,
    sk: Option<String>,
    test_coins: bool,
) -> Result<(), String> {
    let secret = prompt_sensitive(None, "Secret:")?;
    let password = prompt_sensitive(None, "Password:")?;
    if test_coins {
        // We then generate a SafeKey with test-coins to use it for the account creation
        println!("Creating a SafeKey with test-coins...");
        let (_xorurl, key_pair) = safe.keys_create_preload_test_coins("1000.11")?;
        let kp = key_pair.ok_or("Faild to obtain the secret key of the newly created SafeKey")?;
        println!("Sending account creation request to authd...");
        safe_authd.create_acc(&kp.sk, &secret, &password)?;
        println!("Account was created successfully!");
        println!("SafeKey created and preloaded with test-coins. Owner key pair generated:");
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

pub fn authd_login(safe_authd: &mut SafeAuthdClient) -> Result<(), String> {
    let secret = prompt_sensitive(None, "Secret:")?;
    let password = prompt_sensitive(None, "Password:")?;
    println!("Sending login action request to authd...");
    safe_authd.log_in(&secret, &password)?;
    println!("Logged in successfully");
    Ok(())
}

pub fn authd_logout(safe_authd: &mut SafeAuthdClient) -> Result<(), String> {
    println!("Sending logout action request to authd...");
    safe_authd.log_out()?;
    println!("Logged out successfully");
    Ok(())
}

pub fn authd_apps(safe_authd: &SafeAuthdClient) -> Result<(), String> {
    println!("Requesting list of authorised apps from authd...");
    let authed_apps = safe_authd.authed_apps()?;
    pretty_print_authed_apps(authed_apps);

    Ok(())
}

pub fn authd_revoke(safe_authd: &SafeAuthdClient, app_id: String) -> Result<(), String> {
    println!("Sending application revocation request to authd...");
    safe_authd.revoke_app(&app_id)?;
    println!("Application revoked successfully");
    Ok(())
}

pub fn authd_auth_reqs(safe_authd: &SafeAuthdClient) -> Result<(), String> {
    println!("Requesting list of pending authorisation requests from authd...");
    let auth_reqs = safe_authd.auth_reqs()?;
    //pretty_print_authed_apps(auth_reqs);
    println!("{}", auth_reqs);
    Ok(())
}

pub fn authd_allow(safe_authd: &SafeAuthdClient, req_id: u32) -> Result<(), String> {
    println!("Sending request to authd to allow an authorisation request...");
    safe_authd.allow(req_id)?;
    println!("Authorisation request was allowed successfully");
    Ok(())
}

pub fn authd_deny(safe_authd: &SafeAuthdClient, req_id: u32) -> Result<(), String> {
    println!("Sending request to authd to deny an authorisation request...");
    safe_authd.deny(req_id)?;
    println!("Authorisation request was denied successfully");
    Ok(())
}

pub fn authd_subscribe(
    safe_authd: &mut SafeAuthdClient,
    notifs_endpoint: String,
    auth_allow_prompt: &'static AuthAllowPrompt,
) -> Result<(), String> {
    println!("Sending request to subscribe...");
    safe_authd.subscribe(&notifs_endpoint, auth_allow_prompt)?;
    println!("Subscribed successfully");
    Ok(())
}

pub fn authd_subscribe_url(
    safe_authd: &SafeAuthdClient,
    notifs_endpoint: String,
) -> Result<(), String> {
    println!("Sending request to subscribe URL...");
    safe_authd.subscribe_url(&notifs_endpoint)?;
    println!("URL subscribed successfully");
    Ok(())
}

pub fn authd_unsubscribe(
    safe_authd: &SafeAuthdClient,
    notifs_endpoint: String,
) -> Result<(), String> {
    println!("Sending request to unsubscribe...");
    safe_authd.unsubscribe(&notifs_endpoint)?;
    println!("Unsubscribed successfully");
    Ok(())
}

pub fn authorise_cli(safe: &mut Safe, port: Option<u16>) -> Result<(), String> {
    println!("Authorising CLI application...");
    let (mut file, file_path) = get_credentials_file()?;
    let auth_credentials = safe
        .auth_app(APP_ID, APP_NAME, APP_VENDOR, port)
        .map_err(|err| format!("Application authorisation failed: {}", err))?;

    file.write_all(auth_credentials.as_bytes())
        .map_err(|err| format!("Unable to write credentials in {}: {}", file_path, err))?;

    println!("SAFE CLI app was successfully authorised");
    println!("Credentials were stored in {}", file_path);
    Ok(())
}

pub fn clear_credentials() -> Result<(), String> {
    let (_file, file_path) =
        get_credentials_file().map_err(|err| format!("Failed to clear credentials. {}", err))?;

    println!("Credentials were succesfully cleared from {}", file_path);
    Ok(())
}

pub fn safe_connect(safe: &mut Safe) -> Result<(), String> {
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
pub fn safe_connect_without_auth(safe: &mut Safe) -> Result<(), String> {
    debug!("Connecting...");

    safe.connect(APP_ID, None)?;

    Ok(())
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

// Private helpers

// TODO: use a different crate than rpassword as it has problems with some Windows shells including PowerShell
fn prompt_sensitive(arg: Option<String>, msg: &str) -> Result<String, String> {
    if let Some(str) = arg {
        Ok(str)
    } else {
        rpassword::read_password_from_tty(Some(msg))
            .map_err(|err| format!("Failed reading string from input: {}", err))
    }
}

fn credentials_file_path() -> Result<String, String> {
    let home_path =
        dirs::home_dir().ok_or_else(|| "Couldn't find user's home directory".to_string())?;

    let path = Path::new(&home_path).join(AUTH_CREDENTIALS_FOLDER);
    if !Path::new(&path).exists() {
        println!("Creating ~/{} folder", AUTH_CREDENTIALS_FOLDER);
        DirBuilder::new().recursive(false).create(&path).unwrap();
    }

    let path = Path::new(&path).join(AUTH_CREDENTIALS_FILENAME);
    Ok(path.display().to_string())
}

fn get_credentials_file() -> Result<(File, String), String> {
    let file_path = credentials_file_path()?;
    let file = File::create(&file_path)
        .map_err(|_| format!("Unable to open credentials file at {}", file_path))?;

    Ok((file, file_path))
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
