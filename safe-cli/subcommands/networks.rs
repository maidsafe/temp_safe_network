// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::config::{
    print_networks_settings, read_config_settings, retrieve_conn_info,
};
use log::debug;
use std::fs;

pub fn networks_commander(network_name: Option<String>) -> Result<(), String> {
    match network_name {
        Some(name) => {
            let (settings, _) = read_config_settings()?;
            let msg = format!("Switching to '{}' network...", name);
            debug!("{}", msg);
            println!("{}", msg);
            match settings
            .networks
            .get(&name) {
                Some(config_location) => {
                    let conn_info = retrieve_conn_info(&name, config_location)?;
                    let file_path = match directories::ProjectDirs::from("net", "maidsafe", "safe_vault") {
                        Some(dirs) => dirs.config_dir().join("vault_connection_info.config"),
                        None => {
                            return Err(
                                "Failed to obtain local home directory where to set network connection info"
                                    .to_string(),
                            )
                        }
                    };

                    fs::write(&file_path, conn_info)
                        .map_err(|err| format!("Unable to write config in {}: {}", file_path.display(), err))?;
                    println!("Succesfully switched to '{}' network in your system!", name);
                    println!("You'll need to re-authorise the CLI if you need write access to the '{}' network", name);
                },
                None => return Err(format!("No network with name '{}' was found in the config. Please use the 'config add network' command to add it", name))
            }
        }
        None => print_networks_settings()?,
    }

    Ok(())
}
