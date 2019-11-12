// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{APP_ID, APP_NAME, APP_VENDOR};
use directories::ProjectDirs;
use log::debug;
use safe_api::Safe;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};

const AUTH_CREDENTIALS_FILENAME: &str = "credentials";
const PROJECT_DATA_DIR_QUALIFIER: &str = "net";
const PROJECT_DATA_DIR_ORGANISATION: &str = "MaidSafe";
const PROJECT_DATA_DIR_APPLICATION: &str = "safe-cli";

pub fn authorise_cli(safe: &mut Safe, endpoint: Option<String>) -> Result<(), String> {
    println!("Authorising CLI application...");
    let (mut file, file_path) = get_credentials_file()?;
    let auth_credentials = safe
        .auth_app(
            APP_ID,
            APP_NAME,
            APP_VENDOR,
            endpoint.as_ref().map(String::as_str),
        )
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

pub fn connect(safe: &mut Safe) -> Result<(), String> {
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

// Private helpers

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

fn get_credentials_file() -> Result<(File, String), String> {
    let file_path = credentials_file_path()?;
    let file = File::create(&file_path)
        .map_err(|_| format!("Unable to open credentials file at {}", file_path))?;

    Ok((file, file_path))
}
