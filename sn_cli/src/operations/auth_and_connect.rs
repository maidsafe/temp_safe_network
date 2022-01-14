// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::config::Config;
use crate::{APP_ID, APP_NAME, APP_VENDOR};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use sn_api::{Keypair, Safe, XorUrlBase};
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
    time::Duration,
};
use tracing::{debug, info, warn};

const AUTH_CREDENTIALS_FILENAME: &str = "credentials";

#[allow(dead_code)]
pub async fn authorise_cli(endpoint: Option<String>, is_self_authing: bool) -> Result<()> {
    let (mut file, file_path) = create_credentials_file()?;
    println!("Authorising CLI application...");
    if !is_self_authing {
        println!("Note you can use this CLI from another console to authorise it with 'auth allow' command. Alternativelly, you can also use '--self-auth' flag with 'auth unlock' command to automatically self authorise the CLI app.");
    }
    println!("Waiting for authorising response from authd...");
    let app_keypair = Safe::auth_app(APP_ID, APP_NAME, APP_VENDOR, endpoint.as_deref())
        .await
        .wrap_err("Application authorisation failed")?;

    let serialised_keypair = serde_json::to_string(&app_keypair)
        .wrap_err("Unable to serialise the credentials obtained")?;

    file.write_all(serialised_keypair.as_bytes())
        .wrap_err_with(|| format!("Unable to write credentials in {}", file_path.display(),))?;

    println!("Safe CLI app was successfully authorised");
    println!("Credentials were stored in {}", file_path.display());
    Ok(())
}

// Attempt to connect with credentials if found and valid,
// otherwise it creates a read only connection.
// Returns the app's keypair if connection was succesfully made with credentials,
// otherwise it returns 'None' if conneciton is read only.
pub async fn connect(
    config: Config,
    xorurl_base: Option<XorUrlBase>,
    timeout: Option<Duration>,
    dry_run: bool,
) -> Result<Safe> {
    debug!("Connecting...");

    let app_keypair = if let Ok((_, keypair)) = read_credentials() {
        keypair
    } else {
        None
    };

    let found_app_keypair = app_keypair.is_some();
    if !found_app_keypair {
        info!("No credentials found for CLI, connecting with read-only access...");
    }

    let (_, bootstrap_contacts) = config.read_current_node_config().await?;
    let client_cfg = client_config_path();

    let safe = if dry_run {
        Safe::dry_runner(
            bootstrap_contacts,
            app_keypair,
            client_cfg.as_deref(),
            xorurl_base,
            timeout,
        )
        .await?
    } else {
        match Safe::connect(
            bootstrap_contacts.clone(),
            app_keypair.clone(),
            client_cfg.as_deref(),
            xorurl_base,
            timeout,
        )
        .await
        {
            Ok(safe) => safe,
            Err(_) if found_app_keypair => {
                warn!("Credentials found for CLI are invalid, connecting with read-only access...");
                Safe::connect(bootstrap_contacts, None, None, xorurl_base, timeout)
                    .await
                    .wrap_err("Failed to connect with read-only access")?
            }
            Err(err) => return Err(eyre!("Failed to connect: {}", err)),
        }
    };

    Ok(safe)
}

pub fn create_credentials_file() -> Result<(File, PathBuf)> {
    let (credentials_folder, file_path) = get_credentials_file_path()?;
    if !credentials_folder.exists() {
        println!("Creating '{}' folder", credentials_folder.display());
        create_dir_all(credentials_folder)
            .context("Couldn't create project's local data folder")?;
    }
    let file = File::create(&file_path)
        .with_context(|| format!("Unable to open credentials file at {}", file_path.display()))?;

    Ok((file, file_path))
}

pub fn read_credentials() -> Result<(PathBuf, Option<Keypair>)> {
    let (_, file_path) = get_credentials_file_path()?;

    let keypair = if let Ok(mut file) = File::open(&file_path) {
        let mut credentials = String::new();
        match file.read_to_string(&mut credentials) {
            Ok(_) if credentials.is_empty() => None,
            Ok(_) => {
                let keypair = serde_json::from_str(&credentials).with_context(|| {
                    format!(
                        "Unable to parse the credentials read from {}",
                        file_path.display(),
                    )
                })?;
                Some(keypair)
            }
            Err(err) => {
                debug!(
                    "Unable to read credentials from {}: {}",
                    file_path.display(),
                    err
                );
                None
            }
        }
    } else {
        None
    };

    Ok((file_path, keypair))
}

#[allow(dead_code)]
pub fn clear_credentials() -> Result<()> {
    let (_, file_path) = create_credentials_file().context("Failed to clear credentials")?;

    println!(
        "Credentials were succesfully cleared from {}",
        file_path.display()
    );
    Ok(())
}

// Private helpers

fn get_credentials_file_path() -> Result<(PathBuf, PathBuf)> {
    let mut project_data_path =
        dirs_next::home_dir().ok_or_else(|| eyre!("Failed to obtain user's home path"))?;

    project_data_path.push(".safe");
    project_data_path.push("cli");

    let credentials_folder = project_data_path;

    let file_path = credentials_folder.join(AUTH_CREDENTIALS_FILENAME);
    Ok((credentials_folder, file_path))
}

fn client_config_path() -> Option<PathBuf> {
    let mut client_cfg_path = dirs_next::home_dir()?;
    client_cfg_path.push(".safe");
    client_cfg_path.push("client");
    client_cfg_path.push("sn_client.config");

    Some(client_cfg_path)
}
