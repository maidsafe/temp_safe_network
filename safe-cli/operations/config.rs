// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    PROJECT_DATA_DIR_APPLICATION, PROJECT_DATA_DIR_ORGANISATION, PROJECT_DATA_DIR_QUALIFIER,
};
use directories::ProjectDirs;
use log::debug;
use prettytable::Table;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, create_dir_all};
use std::path::PathBuf;

const CONFIG_FILENAME: &str = "config.json";

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct ConfigSettings {
    pub networks: BTreeMap<String, String>,
    // pub contacts: BTreeMap<String, String>,
}

pub fn read_config_settings() -> Result<(ConfigSettings, PathBuf), String> {
    let file_path = config_file_path()?;
    let file = match fs::File::open(&file_path) {
        Ok(file) => file,
        Err(error) => {
            return Err(format!(
                "Error reading config file from '{}': {}",
                file_path.display(),
                error
            ));
        }
    };
    let settings: ConfigSettings = serde_json::from_reader(file).map_err(|err| {
        format!(
            "Format of the config file is not valid and couldn't be parsed: {:?}",
            err
        )
    })?;
    debug!(
        "Config settings retrieved from {}: {:?}",
        file_path.display(),
        settings
    );
    Ok((settings, file_path))
}

pub fn write_config_settings(file_path: &PathBuf, settings: ConfigSettings) -> Result<(), String> {
    let serialised_settings = serde_json::to_string(&settings)
        .map_err(|err| format!("Failed to add config to file: {}", err))?;

    fs::write(&file_path, serialised_settings.as_bytes())
        .map_err(|err| format!("Unable to write config in {}: {}", file_path.display(), err))?;

    debug!(
        "Config settings at {} updated with: {:?}",
        file_path.display(),
        settings
    );

    Ok(())
}

pub fn config_file_path() -> Result<PathBuf, String> {
    let project_data_path = ProjectDirs::from(
        PROJECT_DATA_DIR_QUALIFIER,
        PROJECT_DATA_DIR_ORGANISATION,
        PROJECT_DATA_DIR_APPLICATION,
    )
    .ok_or_else(|| "Couldn't find user's home directory".to_string())?;

    let config_local_path = project_data_path.config_dir();

    let file_path = config_local_path.join(CONFIG_FILENAME);
    if !config_local_path.exists() {
        println!(
            "Creating '{}' folder for config file",
            config_local_path.display()
        );
        create_dir_all(config_local_path)
            .map_err(|err| format!("Couldn't create project's local config folder: {}", err))?;
    }

    if !file_path.exists() {
        let empty_settings = ConfigSettings::default();
        write_config_settings(&file_path, empty_settings).map_err(|err| {
            format!(
                "Unable to create config in {}: {}",
                file_path.display(),
                err
            )
        })?;
    }

    Ok(file_path)
}

pub fn print_networks_settings() -> Result<(), String> {
    let mut table = Table::new();
    table.add_row(row![bFg->"Networks"]);
    table.add_row(row![bFg->"Network name", bFg->"Connection info location"]);

    let (settings, _) = read_config_settings()?;
    settings
        .networks
        .iter()
        .for_each(|(network_name, config_location)| {
            table.add_row(row![network_name, config_location,]);
        });
    table.printstd();
    Ok(())
}

pub fn retrieve_conn_info(name: &str, location: &str) -> Result<Vec<u8>, String> {
    println!(
        "Fetching network connection information from '{}' ...",
        location
    );
    if location.starts_with("http") {
        // Fetch info from an HTTP/s location
        let mut resp = reqwest::get(location).map_err(|err| {
            format!(
                "Failed to fetch connection information for network '{}' from '{}': {}",
                name, location, err
            )
        })?;

        let conn_info = resp.text().map_err(|err| {
            format!(
                "Failed to fetch connection information for network '{}' from '{}': {}",
                name, location, err
            )
        })?;
        Ok(conn_info.as_bytes().to_vec())
    } else {
        // Then fetch it from a local file
        let conn_info = fs::read(location).map_err(|err| {
            format!(
                "Unable to read connection information from '{}': {}",
                location, err
            )
        })?;
        Ok(conn_info)
    }
}
