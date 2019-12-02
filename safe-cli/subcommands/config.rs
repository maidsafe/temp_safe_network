// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::config::{
    config_file_path, print_networks_settings, read_config_settings, write_config_settings,
    ConfigSettings,
};
use log::debug;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ConfigSubCommands {
    #[structopt(name = "add")]
    /// Add a config setting
    Add(SettingAddCmd),
    #[structopt(name = "remove")]
    /// Remove a config setting
    Remove(SettingRemoveCmd),
    #[structopt(name = "clear")]
    /// Remove all config settings
    Clear,
}

#[derive(StructOpt, Debug)]
pub enum SettingAddCmd {
    #[structopt(name = "network")]
    Network {
        /// Network name
        network_name: String,
        /// Location of the network connection information
        config_location: String,
    },
    // #[structopt(name = "contact")]
    // Contact {
    //    /// Contact friendly name
    //    name: String,
    //    /// SafeId of the contact
    //    safeid: String,
    // },
}

#[derive(StructOpt, Debug)]
pub enum SettingRemoveCmd {
    #[structopt(name = "network")]
    Network {
        /// Network to remove
        network_name: String,
    },
    // #[structopt(name = "contact")]
    // Contact {
    //    /// Name of the contact to remove
    //    name: String,
    // },
}

pub fn config_commander(cmd: Option<ConfigSubCommands>) -> Result<(), String> {
    match cmd {
        Some(ConfigSubCommands::Add(SettingAddCmd::Network {
            network_name,
            config_location,
        })) => {
            let (mut settings, file_path) = read_config_settings()?;
            settings
                .networks
                .insert(network_name.clone(), config_location.clone());
            write_config_settings(&file_path, settings)?;
            debug!(
                "Network {} - {} added to settings",
                network_name, config_location
            );
            println!("Network '{}' was added to the list", network_name);
        }
        // Some(ConfigSubCommands::Add(SettingAddCmd::Contact { name, safeid })) => {}
        Some(ConfigSubCommands::Remove(SettingRemoveCmd::Network { network_name })) => {
            let (mut settings, file_path) = read_config_settings()?;
            settings.networks.remove(&network_name);
            write_config_settings(&file_path, settings)?;
            debug!("Network {} removed from settings", network_name);
            println!("Network '{}' was removed from the list", network_name);
        }
        // Some(ConfigSubCommands::Remove(SettingRemoveCmd::Contact { name })) => {}
        Some(ConfigSubCommands::Clear) => {
            let file_path = config_file_path()?;
            let empty_settings = ConfigSettings::default();
            write_config_settings(&file_path, empty_settings)?;
            debug!("Config settings cleared out");
        }
        None => print_networks_settings()?,
    }

    Ok(())
}
