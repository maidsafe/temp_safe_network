// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[cfg(feature = "self-update")]
use super::helpers::download_from_s3_and_install_bin;
use crate::APP_ID;
use envy::from_env;
use log::info;
use prettytable::Table;
use serde::Deserialize;
use sn_api::{
    AuthAllowPrompt, AuthdStatus, AuthedAppsList, PendingAuthReqs, Safe, SafeAuthdClient,
};
use std::convert::From;
use std::fs::File;
#[cfg(feature = "self-update")]
use std::path::PathBuf;

const AUTH_REQS_NOTIFS_ENDPOINT: &str = "https://localhost:33001";
#[cfg(feature = "self-update")]
const ENV_VAR_SN_AUTHD_PATH: &str = "SN_AUTHD_PATH";

#[cfg(all(not(target_os = "windows"), feature = "self-update"))]
const SN_AUTHD_EXECUTABLE: &str = "sn_authd";

#[cfg(all(target_os = "windows", feature = "self-update"))]
const SN_AUTHD_EXECUTABLE: &str = "sn_authd.exe";

#[derive(Deserialize, Debug)]
struct Environment {
    safe_auth_passphrase: Option<String>,
    safe_auth_password: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SafeSeed {
    pub passphrase: String,
    pub password: String,
}

#[cfg(not(feature = "self-update"))]
pub fn authd_install(_authd_path: Option<String>) -> Result<(), String> {
    Err("Self updates are disabled".to_string())
}

#[cfg(feature = "self-update")]
pub fn authd_install(authd_path: Option<String>) -> Result<(), String> {
    let target_path = get_authd_bin_path(authd_path)?;
    download_from_s3_and_install_bin(target_path, "sn-api", "sn_authd", SN_AUTHD_EXECUTABLE, None)?;
    Ok(())
}

pub fn authd_update(sn_authd: &SafeAuthdClient, authd_path: Option<String>) -> Result<(), String> {
    sn_authd
        .update(authd_path.as_deref())
        .map_err(|err| err.to_string())
}

pub fn authd_start(sn_authd: &SafeAuthdClient, authd_path: Option<String>) -> Result<(), String> {
    sn_authd
        .start(authd_path.as_deref())
        .map_err(|err| err.to_string())
}

pub fn authd_stop(sn_authd: &SafeAuthdClient, authd_path: Option<String>) -> Result<(), String> {
    sn_authd
        .stop(authd_path.as_deref())
        .map_err(|err| err.to_string())
}

pub fn authd_restart(sn_authd: &SafeAuthdClient, authd_path: Option<String>) -> Result<(), String> {
    sn_authd
        .restart(authd_path.as_deref())
        .map_err(|err| err.to_string())
}

pub async fn authd_create(
    _safe: &mut Safe,
    sn_authd: &SafeAuthdClient,
    config_file_str: Option<String>,
    _sk: Option<String>,
    test_coins: bool,
) -> Result<(), String> {
    let safe_seed = get_safe_seed(config_file_str)?;

    if test_coins {
        // We then generate a SafeKey with test-coins to use it for the Safe creation
        // TODO: generate a payment proof to send to authd
        // println!("Creating a SafeKey with test-coins...");
        // let (_xorurl, kp) = safe.keys_create_preload_test_coins("1000.11").await?;
        // let kp =
        // key_pair.ok_or("Faild to obtain the secret key of the newly created SafeKey")?;
        // let sk = kp.secret_key().map_err(|e| format!("{:?}", e))?.to_string();
        println!("Sending request to authd to create a Safe...");
        sn_authd
            .create(&safe_seed.passphrase, &safe_seed.password)
            .await?;

        println!("Safe was created successfully!");
        // println!("SafeKey created and preloaded with test-coins. Owner key pair generated:");
        // println!("Public Key = {}", kp.public_key());
        // println!("Secret Key = {}", sk);
        Ok(())
    } else {
        Err("Please use --test-coins option, other options not implemented yet.".to_string())
        // TODO: support generating a payment proof to be sent to authd, either
        // by using the provided SK to sign the payment request,
        // or by obtaining a signed request from the user for the payment request.
        /*
        let sk = prompt_sensitive(sk, "Enter SafeKey's secret key to pay with:")?;
        println!("Sending request to authd to create a Safe...");
        sn_authd
            .create(&safe_seed.passphrase, &safe_seed.password)
            .await?;
        println!("Safe was created successfully!");
        */
    }
}

pub async fn authd_unlock(
    sn_authd: &mut SafeAuthdClient,
    config_file_str: Option<String>,
) -> Result<(), String> {
    let safe_seed = get_safe_seed(config_file_str)?;
    println!("Sending action request to authd to unlock the Safe...");
    sn_authd
        .unlock(&safe_seed.passphrase, &safe_seed.password)
        .await?;
    println!("Safe unlocked successfully");
    Ok(())
}

pub async fn authd_lock(sn_authd: &mut SafeAuthdClient) -> Result<(), String> {
    println!("Sending action request to authd to lock the Safe...");
    sn_authd.lock().await?;
    println!("Safe locked successfully");
    Ok(())
}

pub async fn authd_status(sn_authd: &mut SafeAuthdClient) -> Result<(), String> {
    println!("Sending request to authd to obtain a status report...");
    let status_report = sn_authd.status().await?;
    pretty_print_status_report(status_report);

    Ok(())
}

pub async fn authd_apps(sn_authd: &SafeAuthdClient) -> Result<(), String> {
    println!("Requesting list of authorised apps from authd...");
    let authed_apps = sn_authd.authed_apps().await?;
    pretty_print_authed_apps(authed_apps);

    Ok(())
}

pub async fn authd_revoke(sn_authd: &SafeAuthdClient, app_id: String) -> Result<(), String> {
    println!("Sending application revocation request to authd...");
    sn_authd.revoke_app(&app_id).await?;
    println!("Application revoked successfully");
    Ok(())
}

pub async fn authd_auth_reqs(sn_authd: &SafeAuthdClient) -> Result<(), String> {
    println!("Requesting list of pending authorisation requests from authd...");
    let auth_reqs = sn_authd.auth_reqs().await?;
    pretty_print_auth_reqs(auth_reqs, Some("Pending Authorisation requests"));
    Ok(())
}

pub async fn authd_allow(sn_authd: &SafeAuthdClient, req_id: u32) -> Result<(), String> {
    println!("Sending request to authd to allow an authorisation request...");
    sn_authd.allow(req_id).await?;
    println!("Authorisation request was allowed");
    Ok(())
}

pub async fn authd_deny(sn_authd: &SafeAuthdClient, req_id: u32) -> Result<(), String> {
    println!("Sending request to authd to deny an authorisation request...");
    sn_authd.deny(req_id).await?;
    println!("Authorisation request was denied successfully");
    Ok(())
}

pub async fn authd_subscribe(
    sn_authd: &mut SafeAuthdClient,
    notifs_endpoint: Option<String>,
    auth_allow_prompt: &'static AuthAllowPrompt,
) -> Result<(), String> {
    println!("Sending request to subscribe...");
    let endpoint = notifs_endpoint.unwrap_or_else(|| AUTH_REQS_NOTIFS_ENDPOINT.to_string());
    sn_authd
        .subscribe(&endpoint, APP_ID, auth_allow_prompt)
        .await?;
    println!("Subscribed successfully");
    Ok(())
}

#[allow(dead_code)]
pub async fn authd_subscribe_url(
    sn_authd: &SafeAuthdClient,
    notifs_endpoint: String,
) -> Result<(), String> {
    println!("Sending request to subscribe URL...");
    sn_authd.subscribe_url(&notifs_endpoint).await?;
    println!("URL subscribed successfully");
    Ok(())
}

pub async fn authd_unsubscribe(
    sn_authd: &mut SafeAuthdClient,
    notifs_endpoint: Option<String>,
) -> Result<(), String> {
    println!("Sending request to unsubscribe...");
    let endpoint = notifs_endpoint.unwrap_or_else(|| AUTH_REQS_NOTIFS_ENDPOINT.to_string());
    sn_authd.unsubscribe(&endpoint).await?;
    println!("Unsubscribed successfully");
    Ok(())
}

pub fn pretty_print_authed_apps(authed_apps: AuthedAppsList) {
    let mut table = Table::new();
    table.add_row(row![bFg->"Authorised Applications"]);
    table.add_row(row![bFg->"Id", bFg->"Name", bFg->"Vendor", bFg->"Permissions"]);
    let all_app_iterator = authed_apps.iter();
    for authed_app in all_app_iterator {
        table.add_row(row![authed_app.id, authed_app.name, authed_app.vendor,]);
    }
    table.printstd();
}

pub fn pretty_print_auth_reqs(auth_reqs: PendingAuthReqs, title_msg: Option<&str>) {
    if auth_reqs.is_empty() {
        println!("There are no pending authorisation requests");
    } else {
        let mut table = Table::new();
        if let Some(title) = title_msg {
            table.add_row(row![bFg->title]);
        }
        table.add_row(
            row![bFg->"Request Id", bFg->"App Id", bFg->"Name", bFg->"Vendor", bFg->"Permissions requested"],
        );
        for auth_req in auth_reqs.iter() {
            table.add_row(row![
                auth_req.req_id,
                auth_req.app_id,
                auth_req.app_name,
                auth_req.app_vendor,
            ]);
        }
        table.printstd();
    }
}

pub fn pretty_print_status_report(status_report: AuthdStatus) {
    let mut table = Table::new();
    table.add_row(row![bFg->"Safe Authenticator status"]);
    table.add_row(row![
        "Authenticator daemon version",
        status_report.authd_version
    ]);
    table.add_row(row![
        "Is there a Safe currently unlocked?",
        boolean_to_string(status_report.safe_unlocked),
    ]);
    table.add_row(row![
        "Number of pending authorisation requests",
        status_report.num_auth_reqs,
    ]);
    table.add_row(row![
        "Number of notifications subscribers",
        status_report.num_notif_subs,
    ]);
    table.printstd();
}

// Private helpers

#[inline]
fn boolean_to_string(boolean: bool) -> &'static str {
    if boolean {
        "Yes"
    } else {
        "No"
    }
}

fn prompt_sensitive(arg: Option<String>, msg: &str) -> Result<String, String> {
    if let Some(str) = arg {
        Ok(str)
    } else {
        rpassword::read_password_from_tty(Some(msg))
            .map_err(|err| format!("Failed reading string from input: {}", err))
    }
}

fn get_safe_seed(config_file: Option<String>) -> Result<SafeSeed, String> {
    let environment_details = from_env::<Environment>().map_err(|err| {
        format!(
            "Failed when attempting to read Safe seed from env vars: {}",
            err
        )
    })?;

    // try to get SafeSeed from file specified by --config flag
    let (the_passphrase, the_password) = if let Some(config_file_str) = config_file {
        let file = match File::open(&config_file_str) {
            Ok(file) => file,
            Err(error) => {
                return Err(format!("Error reading config file: {}", error));
            }
        };

        let json: SafeSeed = serde_json::from_reader(file).map_err(|err| {
            format!(
                "Format of the config file is not valid and couldn't be parsed: {}",
                err
            )
        })?;

        eprintln!("Warning! Storing your passphrase/password in plaintext in a config file is not secure." );

        if json.passphrase.is_empty() {
            return Err("The config files's passphrase field cannot be empty".to_string());
        }
        let the_passphrase = json.passphrase;

        if json.password.is_empty() {
            return Err("The config files's password field cannot be empty".to_string());
        }
        let the_password = json.password;

        (the_passphrase, the_password)
    } else {
        // try to get SafeSeed from environment variables then
        let mut the_passphrase = environment_details
            .safe_auth_passphrase
            .unwrap_or_else(|| String::from(""));
        if !the_passphrase.is_empty() {
            info!("Using passphrase from provided ENV var: SAFE_AUTH_PASSPHRASE")
        }

        let mut the_password = environment_details
            .safe_auth_password
            .unwrap_or_else(|| String::from(""));
        if !the_password.is_empty() {
            info!("Using password from provided ENV var: SAFE_AUTH_PASSWORD")
        }

        if the_passphrase.is_empty() ^ the_password.is_empty() {
            return Err("Both the passphrase (SAFE_AUTH_PASSPHRASE) and password (SAFE_AUTH_PASSWORD) environment variables must be set for creating/unlocking a Safe.".to_string());
        }

        // try to prompt the user to enter the SafeSeed
        // if it was not set in the environment variables
        if the_passphrase.is_empty() || the_password.is_empty() {
            // Prompt the user for the Safe's credentials
            the_passphrase = prompt_sensitive(None, "Passphrase: ")
                .map_err(|err| format!("Failed reading 'passphrase' string from input: {}", err))?;
            the_password = prompt_sensitive(None, "Password: ")
                .map_err(|err| format!("Failed reading 'passphrase' string from input: {}", err))?;
        }

        (the_passphrase, the_password)
    };

    if the_passphrase.is_empty() || the_password.is_empty() {
        return Err(String::from(
            "Neither the passphrase nor password can be empty.",
        ));
    }

    let details = SafeSeed {
        passphrase: the_passphrase,
        password: the_password,
    };

    Ok(details)
}

#[cfg(feature = "self-update")]
#[inline]
fn get_authd_bin_path(authd_path: Option<String>) -> Result<PathBuf, String> {
    match authd_path {
        Some(p) => Ok(PathBuf::from(p)),
        None => {
            // if SN_AUTHD_PATH is set it then overrides default
            if let Ok(authd_path) = std::env::var(ENV_VAR_SN_AUTHD_PATH) {
                Ok(PathBuf::from(authd_path))
            } else {
                let mut path = dirs_next::home_dir()
                    .ok_or_else(|| "Failed to obtain user's home path".to_string())?;

                path.push(".safe");
                path.push("authd");
                Ok(path)
            }
        }
    }
}
