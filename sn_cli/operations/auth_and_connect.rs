// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{APP_ID, APP_NAME, APP_VENDOR};
use log::{debug, info, warn};
use sn_api::Safe;
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
};

const AUTH_CREDENTIALS_FILENAME: &str = "credentials";

pub async fn authorise_cli(
    _safe: &mut Safe,
    endpoint: Option<String>,
    is_self_authing: bool,
) -> Result<(), String> {
    let (mut file, file_path) = create_credentials_file()?;
    println!("Authorising CLI application...");
    if !is_self_authing {
        println!("Note you can use this CLI from another console to authorise it with 'auth allow' command. Alternativelly, you can also use '--self-auth' flag with 'auth unlock' command to automatically self authorise the CLI app.");
    }
    println!("Waiting for authorising response from authd...");
    let auth_credentials = Safe::auth_app(APP_ID, APP_NAME, APP_VENDOR, endpoint.as_deref())
        .await
        .map_err(|err| format!("Application authorisation failed: {}", err))?;

    file.write_all(auth_credentials.as_bytes()).map_err(|err| {
        format!(
            "Unable to write credentials in {}: {}",
            file_path.display(),
            err
        )
    })?;

    println!("Safe CLI app was successfully authorised");
    println!("Credentials were stored in {}", file_path.display());
    Ok(())
}

pub fn clear_credentials() -> Result<(), String> {
    let (_, file_path) =
        create_credentials_file().map_err(|err| format!("Failed to clear credentials. {}", err))?;

    println!(
        "Credentials were succesfully cleared from {}",
        file_path.display()
    );
    Ok(())
}

pub async fn connect(safe: &mut Safe) -> Result<(), String> {
    debug!("Connecting...");

    let auth_credentials = match get_credentials_file_path() {
        Ok((_, file_path)) => {
            if let Ok(mut file) = File::open(&file_path) {
                let mut credentials = String::new();
                match file.read_to_string(&mut credentials) {
                    Err(err) => {
                        debug!(
                            "Unable to read credentials from {}: {}",
                            file_path.display(),
                            err
                        );
                        None
                    }
                    Ok(_) if credentials.is_empty() => None,
                    Ok(_) => Some(credentials),
                }
            } else {
                None
            }
        }
        Err(_) => None,
    };

    if auth_credentials.is_none() {
        info!("No credentials found for CLI, connecting with read-only access...");
    }

    match safe.connect(APP_ID, auth_credentials.as_deref()).await {
        Err(_err) if auth_credentials.is_some() => {
            warn!("Credentials found for CLI are invalid, connecting with read-only access...");
            safe.connect(APP_ID, None).await
        }
        other => other,
    }
    .map_err(|err| format!("Failed to connect: {}", err))
}

// Private helpers

fn get_credentials_file_path() -> Result<(PathBuf, PathBuf), String> {
    let mut project_data_path =
        dirs_next::home_dir().ok_or_else(|| "Failed to obtain user's home path".to_string())?;

    project_data_path.push(".safe");
    project_data_path.push("cli");

    let credentials_folder = project_data_path;

    let file_path = credentials_folder.join(AUTH_CREDENTIALS_FILENAME);
    Ok((credentials_folder, file_path))
}

fn create_credentials_file() -> Result<(File, PathBuf), String> {
    let (credentials_folder, file_path) = get_credentials_file_path()?;
    if !credentials_folder.exists() {
        println!("Creating '{}' folder", credentials_folder.display());
        create_dir_all(credentials_folder)
            .map_err(|err| format!("Couldn't create project's local data folder: {}", err))?;
    }
    let file = File::create(&file_path)
        .map_err(|_| format!("Unable to open credentials file at {}", file_path.display()))?;

    Ok((file, file_path))
}
