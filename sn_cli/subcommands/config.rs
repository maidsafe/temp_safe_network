// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::operations::config::{Config, NetworkInfo};
use anyhow::Result;
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
        /// Location of the network connection information. If this argument is not passed, it takes current network connection information and caches it
        config_location: Option<String>,
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

pub fn config_commander(cmd: Option<ConfigSubCommands>) -> Result<()> {
    let mut config = Config::read()?;
    match cmd {
        Some(ConfigSubCommands::Add(SettingAddCmd::Network {
            network_name,
            config_location,
        })) => {
            config.add_network(&network_name, config_location.map(NetworkInfo::ConnInfoUrl))?;
        }
        // Some(ConfigSubCommands::Add(SettingAddCmd::Contact { name, safeid })) => {}
        Some(ConfigSubCommands::Remove(SettingRemoveCmd::Network { network_name })) => {
            config.remove_network(&network_name)?
        }
        // Some(ConfigSubCommands::Remove(SettingRemoveCmd::Contact { name })) => {}
        Some(ConfigSubCommands::Clear) => {
            config.clear()?;
            debug!("Config settings cleared out");
        }
        None => config.print_networks(),
    }

    Ok(())
}
