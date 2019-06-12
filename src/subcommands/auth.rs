// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use dirs;
use log::debug;
use safe_cli::Safe;
use std::fs::{DirBuilder, File};
use std::io::{Read, Write};
use std::path::Path;
use structopt::StructOpt;

static APP_ID: &str = "net.maidsafe.cli";
static APP_NAME: &str = "SAFE CLI";
static APP_VENDOR: &str = "MaidSafe.net Ltd";
static AUTH_CREDENTIALS_FOLDER: &str = ".safe";
static AUTH_CREDENTIALS_FILENAME: &str = "credentials";

#[derive(StructOpt, Debug)]
pub enum AuthSubCommands {
    #[structopt(name = "clear")]
    /// Clear authorisation credentials from local file
    Clear {},
}

pub fn auth_commander(cmd: Option<AuthSubCommands>, safe: &mut Safe) -> Result<(), String> {
    let file_path = credentials_file_path()?;
    let mut file = File::create(&file_path)
        .map_err(|_| format!("Unable to create credentials file at {}", file_path))?;

    match cmd {
        Some(AuthSubCommands::Clear {}) => {
            file.set_len(0).map_err(|err| {
                format!("Unable to clear credentials from {}: {}", file_path, err)
            })?;

            println!("Credentials were succesfully cleared from {}", file_path);
            Ok(())
        }
        None => {
            println!("Authorising CLI application...");

            let auth_credentials = safe
                .auth_app(APP_ID, APP_NAME, APP_VENDOR)
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

    safe.connect(APP_ID, &auth_credentials).map_err(|err| {
        format!(
            "You need to authorise the safe CLI first with 'auth' command: {}",
            err
        )
    })
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
