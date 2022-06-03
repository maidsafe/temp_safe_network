// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::config::Config;
use crate::{APP_ID, APP_NAME, APP_VENDOR};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use sn_api::{Keypair, Safe};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};
use tracing::{debug, info, warn};

const AUTH_CREDENTIALS_FILENAME: &str = "credentials";

#[allow(dead_code)]
pub async fn authorise_cli(
    endpoint: Option<String>,
    is_self_authing: bool,
    authd_cert_path: &Path,
    config: &Config,
) -> Result<()> {
    let (mut file, file_path) = create_credentials_file(config)?;
    println!("Authorising CLI application...");
    if !is_self_authing {
        println!("Note you can use this CLI from another console to authorise it with 'auth allow' command. Alternativelly, you can also use '--self-auth' flag with 'auth unlock' command to automatically self authorise the CLI app.");
    }
    println!("Waiting for authorising response from authd...");
    let app_keypair = Safe::auth_app(
        APP_ID,
        APP_NAME,
        APP_VENDOR,
        endpoint.as_deref(),
        authd_cert_path,
    )
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
pub async fn connect(safe: &mut Safe, config: &Config, timeout: Duration) -> Result<()> {
    debug!("Connecting...");

    let app_keypair = if let Ok((_, keypair)) = read_credentials(safe, config) {
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

    match safe
        .connect(
            bootstrap_contacts.clone(),
            app_keypair.clone(),
            client_cfg.as_deref(),
            Some(timeout),
            config.dbc_owner.clone(),
        )
        .await
    {
        Ok(()) => Ok(()),
        Err(_) if found_app_keypair => {
            warn!("Credentials found for CLI are invalid, connecting with read-only access...");
            safe.connect(
                bootstrap_contacts,
                None,
                None,
                Some(timeout),
                config.dbc_owner.clone(),
            )
            .await
            .wrap_err("Failed to connect with read-only access")
        }
        Err(err) => return Err(eyre!("Failed to connect: {}", err)),
    }
}

pub fn create_credentials_file(config: &Config) -> Result<(File, PathBuf)> {
    let (credentials_folder, file_path) = get_credentials_file_path(config)?;
    if !credentials_folder.exists() {
        println!("Creating '{}' folder", credentials_folder.display());
        create_dir_all(credentials_folder)
            .context("Couldn't create project's local data folder")?;
    }
    let file = File::create(&file_path)
        .with_context(|| format!("Unable to open credentials file at {}", file_path.display()))?;

    Ok((file, file_path))
}

pub fn read_credentials(safe: &Safe, config: &Config) -> Result<(PathBuf, Option<Keypair>)> {
    let (_, path) = get_credentials_file_path(config)?;
    let keypair = match safe.deserialize_keypair(&path) {
        Ok(kp) => Some(kp),
        Err(e) => {
            debug!("Unable to read credentials from {}: {}", path.display(), e);
            None
        }
    };
    Ok((path, keypair))
}

#[allow(dead_code)]
pub fn clear_credentials(config: &Config) -> Result<()> {
    let (_, file_path) = create_credentials_file(config).context("Failed to clear credentials")?;

    println!(
        "Credentials were succesfully cleared from {}",
        file_path.display()
    );
    Ok(())
}

pub fn get_credentials_file_path(config: &Config) -> Result<(PathBuf, PathBuf)> {
    let mut pb = config.cli_config_path.clone();
    pb.pop();
    let credentials_folder = pb;
    let file_path = credentials_folder.join(AUTH_CREDENTIALS_FILENAME);
    Ok((credentials_folder, file_path))
}

///
/// Private helpers
///

fn client_config_path() -> Option<PathBuf> {
    let mut client_cfg_path = dirs_next::home_dir()?;
    client_cfg_path.push(".safe");
    client_cfg_path.push("client");
    client_cfg_path.push("sn_client.config");

    Some(client_cfg_path)
}
