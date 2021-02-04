// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::operations::config::{
    read_config_settings, read_current_network_conn_info, retrieve_conn_info,
};
use crate::operations::node::*;
use anyhow::{anyhow, Context, Result};
use log::debug;
use std::path::PathBuf;
use structopt::StructOpt;

const NODES_DATA_FOLDER: &str = "baby-fleming-nodes";

const LOCAL_NODE_DIR: &str = "local-node";

#[derive(StructOpt, Debug)]
pub enum NodeSubCommands {
    #[structopt(name = "install")]
    /// Install latest sn_node released version in the system
    Install {
        #[structopt(long = "node-path")]
        /// Path where to install sn_node executable (default ~/.safe/node/). The SN_NODE_PATH env var can also be used to set the path
        #[structopt(long = "node-path", env = "SN_NODE_PATH")]
        node_path: Option<PathBuf>,
    },
    #[structopt(name = "join")]
    /// Join an already running network
    Join {
        /// Network to have the node to join to
        network_name: Option<String>,
        #[structopt(long = "node-path")]
        /// Path where to run sn_node executable from (default ~/.safe/node/). The SN_NODE_PATH env var can also be used to set the path
        #[structopt(long = "node-path", env = "SN_NODE_PATH")]
        node_path: Option<PathBuf>,
        /// Vebosity level for nodes logs
        #[structopt(short = "y", parse(from_occurrences))]
        verbosity: u8,
        /// Hardcoded contacts (endpoints) to be used to bootstrap to an already running network (this overrides any value passed as 'network_name').
        #[structopt(short = "h", long = "hcc")]
        hard_coded_contacts: Option<String>,
    },
    #[structopt(name = "run-baby-fleming")]
    /// Run nodes to form a local single-section Safe network
    Run {
        /// Path where to run sn_node executable from (default ~/.safe/node/). The SN_NODE_PATH env var can also be used to set the path
        #[structopt(long = "node-path", env = "SN_NODE_PATH")]
        node_path: Option<PathBuf>,
        /// Vebosity level for nodes logs (default = INFO, -y = DEBUG, -yy = TRACE)
        #[structopt(short = "y", parse(from_occurrences))]
        verbosity: u8,
        /// Interval in seconds between launching each of the nodes
        #[structopt(short = "i", long, default_value = "1")]
        interval: u64,
        /// Number of nodes to be launched
        #[structopt(long = "nodes", default_value = "11")]
        num_of_nodes: u8,
        /// IP to be used to launch the local nodes.
        #[structopt(long = "ip")]
        ip: Option<String>,
        /// Start authd and log in with
        #[structopt(short = "t", long = "testing")]
        test: bool,
    },
    /// Shutdown all running nodes processes
    #[structopt(name = "killall")]
    Killall {
        /// Path of the sn_node executable used to launch the processes with (default ~/.safe/node/sn_node). The SN_NODE_PATH env var can be also used to set this path
        #[structopt(long = "node-path", env = "SN_NODE_PATH")]
        node_path: Option<PathBuf>,
    },
    #[structopt(name = "update")]
    /// Update to latest sn_node released version
    Update {
        #[structopt(long = "node-path")]
        /// Path of the sn_node executable to update (default ~/.safe/node/). The SN_NODE_PATH env var can be also used to set the path
        #[structopt(long = "node-path", env = "SN_NODE_PATH")]
        node_path: Option<PathBuf>,
    },
}

pub fn node_commander(cmd: Option<NodeSubCommands>) -> Result<()> {
    match cmd {
        Some(NodeSubCommands::Install { node_path }) => {
            // We run this command in a separate thread to overcome a conflict with
            // the self_update crate as it seems to be creating its own runtime.
            let handler = std::thread::spawn(|| node_install(node_path));
            handler
                .join()
                .map_err(|err| anyhow!("Failed to run self update: {:?}", err))?
        }
        Some(NodeSubCommands::Join {
            network_name,
            node_path,
            verbosity,
            hard_coded_contacts,
        }) => {
            let network_contacts: Result<String> = if let Some(contacts) = hard_coded_contacts {
                let msg = format!("Joining network with contacts {}...", contacts);
                debug!("{}", msg);
                println!("{}", msg);
                Ok(contacts)
            } else {
                let contacts = if let Some(name) = network_name {
                    let (settings, _) = read_config_settings()?;
                    let msg = format!("Joining the '{}' network...", name);
                    debug!("{}", msg);
                    println!("{}", msg);
                    match settings.networks.get(&name) {
                        Some(config_location) => retrieve_conn_info(&name, config_location),
                        None => Err(anyhow!("No network with name '{}' was found in the config. Please use the 'networks add' command to add it", name))
                    }
                } else {
                    let (_, contacts) = read_current_network_conn_info()?;
                    Ok(contacts)
                }?;

                let mut contacts_str = std::str::from_utf8(&contacts)
                    .context("Failed to parse network contact information from the config")?
                    .to_string();

                contacts_str = contacts_str.replace("\"", "");
                let len_withoutcrlf = contacts_str.trim_end().len();
                contacts_str.truncate(len_withoutcrlf);
                debug!("Joining network with contacts {}...", contacts_str);

                Ok(contacts_str)
            };

            node_join(node_path, LOCAL_NODE_DIR, verbosity, &network_contacts?)
        }
        Some(NodeSubCommands::Run {
            node_path,
            verbosity,
            interval,
            num_of_nodes,
            ip,
            test,
        }) => node_run(
            node_path,
            NODES_DATA_FOLDER,
            verbosity,
            &interval.to_string(),
            &num_of_nodes.to_string(),
            ip,
            test,
        ),
        Some(NodeSubCommands::Killall { node_path }) => node_shutdown(node_path),
        Some(NodeSubCommands::Update { node_path }) => node_update(node_path),
        None => Err(anyhow!("Missing node subcommand")),
    }
}
