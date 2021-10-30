// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::operations::{config::Config, config::NetworkLauncher, node::*};
use color_eyre::{eyre::eyre, Result};
use sn_api::PublicKey;
use std::{collections::BTreeSet, net::SocketAddr, path::PathBuf};
use structopt::StructOpt;
use tracing::debug;

const NODES_DATA_DIR_PATH: &str = "baby-fleming-nodes";

const LOCAL_NODE_DIR: &str = "local-node";

#[derive(StructOpt, Debug)]
pub enum NodeSubCommands {
    /// Gets the version of `sn_node` binary
    BinVersion {
        #[structopt(long = "node-path", env = "SN_NODE_PATH")]
        node_path: Option<PathBuf>,
    },
    #[structopt(name = "install")]
    /// Install latest sn_node released version in the system
    Install {
        #[structopt(long = "node-path")]
        /// Path where to install sn_node executable (default ~/.safe/node/). The SN_NODE_PATH env var can also be used to set the path
        #[structopt(long = "node-path", env = "SN_NODE_PATH")]
        node_path: Option<PathBuf>,
        /// Specify the version of sn_node to install. If not supplied, the latest version will be
        /// installed. Note: just the version number should be supplied, with no 'v' prefix.
        #[structopt(short = "v", long)]
        version: Option<String>,
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
        hard_coded_contacts: Vec<SocketAddr>,
        /// Internal address provided for the node
        #[structopt(short = "l", long)]
        local_addr: Option<SocketAddr>,
        #[structopt(short = "p", long)]
        /// External address provided for the node
        public_addr: Option<SocketAddr>,
        /// Delete all data from a previous node running on the same PC
        #[structopt(long = "clear-data")]
        clear_data: bool,
    },
    #[structopt(name = "run-baby-fleming")]
    /// Run nodes to form a local single-section Safe network
    Run {
        /// Path of the directory where sn_node is located (default is ~/.safe/node/). The SN_NODE_PATH env var can also be used to set the path
        #[structopt(long = "node-dir-path", env = "SN_NODE_PATH")]
        node_dir_path: Option<PathBuf>,
        /// Interval in seconds between launching each of the nodes
        #[structopt(short = "i", long, default_value = "1")]
        interval: u64,
        /// Number of nodes to be launched
        #[structopt(long = "nodes", default_value = "11")]
        num_of_nodes: u8,
        /// IP to be used to launch the local nodes.
        #[structopt(long = "ip")]
        ip: Option<String>,
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

pub async fn node_commander(
    cmd: Option<NodeSubCommands>,
    config: &mut Config,
    network_launcher: &mut Box<impl NetworkLauncher>,
) -> Result<()> {
    match cmd {
        Some(NodeSubCommands::BinVersion { node_path }) => node_version(node_path),
        Some(NodeSubCommands::Install { node_path, version }) => {
            // We run this command in a separate thread to overcome a conflict with
            // the self_update crate as it seems to be creating its own runtime.
            let handler = std::thread::spawn(|| node_install(node_path, version));
            handler
                .join()
                .map_err(|err| eyre!("Failed to run self update: {:?}", err))?
        }
        Some(NodeSubCommands::Join {
            network_name,
            node_path,
            verbosity,
            hard_coded_contacts,
            local_addr,
            public_addr,
            clear_data,
        }) => {
            let network_contacts = if hard_coded_contacts.is_empty() {
                if let Some(name) = network_name {
                    let msg = format!("Joining the '{}' network...", name);
                    debug!("{}", msg);
                    println!("{}", msg);
                    config.get_network_info(&name).await?
                } else {
                    let (_, contacts) = config.read_current_node_config()?;
                    contacts
                }
            } else {
                let genesis_key = PublicKey::bls_from_hex("8640e62cc44e75cf4fadc8ee91b74b4cf0fd2c0984fb0e3ab40f026806857d8c41f01d3725223c55b1ef87d669f5e2cc")?
                    .bls()
                    .ok_or_else(|| eyre!("Unexpectedly failed to obtain (BLS) genesis key."))?;
                let mut set: BTreeSet<SocketAddr> = BTreeSet::new();
                for contact in hard_coded_contacts {
                    set.insert(contact);
                }
                (genesis_key, set)
            };

            let msg = format!("Joining network with contacts {:?} ...", network_contacts);
            debug!("{}", msg);
            println!("{}", msg);

            node_join(
                node_path,
                LOCAL_NODE_DIR,
                verbosity,
                &network_contacts.1,
                local_addr,
                public_addr,
                clear_data,
            )
        }
        Some(NodeSubCommands::Run {
            node_dir_path,
            interval,
            num_of_nodes,
            ip,
        }) => {
            let node_directory_path = node_dir_path.unwrap_or_else(|| {
                let mut default_node_dir_path = config.node_config_path.clone();
                default_node_dir_path.pop();
                default_node_dir_path
            });
            node_run(
                network_launcher,
                node_directory_path,
                NODES_DATA_DIR_PATH,
                interval,
                &num_of_nodes.to_string(),
                ip,
            )?;
            config.add_network("baby-fleming", None)?;
            Ok(())
        }
        Some(NodeSubCommands::Killall { node_path }) => node_shutdown(node_path),
        Some(NodeSubCommands::Update { node_path }) => node_update(node_path),
        None => Err(eyre!("Missing node subcommand")),
    }
}

#[cfg(test)]
mod run_command {
    use super::{node_commander, NodeSubCommands, NODES_DATA_DIR_PATH};
    use crate::operations::config::{Config, NetworkInfo, NetworkLauncher};
    use crate::operations::node::SN_NODE_EXECUTABLE;
    use assert_fs::prelude::*;
    use color_eyre::{eyre::eyre, Report, Result};
    use std::path::PathBuf;

    // Each of these tests will assume the launch tool runs successfully and a node config is
    // written out as a result of the running network. This dummy config will be read when the
    // baby-fleming network is added to the networks list.
    const SERIALIZED_NODE_CONFIG: &str = r#"
    [
        "89505bbfcac9335a7639a1dca9ed027b98be46b03953e946e53695f678c827f18f6fc22dc888de2bce9078f3fce55095",
        [
            "127.0.0.1:33314",
            "127.0.0.1:38932",
            "127.0.0.1:39132",
            "127.0.0.1:47795",
            "127.0.0.1:49976",
            "127.0.0.1:53018",
            "127.0.0.1:53421",
            "127.0.0.1:54002",
            "127.0.0.1:54386",
            "127.0.0.1:55890",
            "127.0.0.1:57956"
        ]
    ]"#;

    pub struct FakeNetworkLauncher {
        pub launch_args: Vec<String>,
    }

    impl NetworkLauncher for FakeNetworkLauncher {
        fn launch(&mut self, args: Vec<&str>, interval: u64) -> Result<(), Report> {
            self.launch_args.extend(args.iter().map(|s| s.to_string()));
            println!("Sleep for {} seconds", interval);
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_use_optionally_supplied_node_directory_path() -> Result<()> {
        let custom_node_dir = assert_fs::TempDir::new()?;
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: Some(PathBuf::from(custom_node_dir.path())),
            interval: 1,
            num_of_nodes: 11,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());

        assert_eq!(launcher.launch_args[1], "--node-path");
        assert_eq!(
            launcher.launch_args[2],
            custom_node_dir
                .path()
                .join(SN_NODE_EXECUTABLE)
                .to_str()
                .unwrap()
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_use_default_node_directory_path() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let node_dir = tmp_dir.child(".safe/node");
        node_dir.create_dir_all()?;

        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = node_dir.child("node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: None,
            interval: 1,
            num_of_nodes: 11,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        assert_eq!(launcher.launch_args[1], "--node-path");
        assert_eq!(
            launcher.launch_args[2],
            node_dir.path().join(SN_NODE_EXECUTABLE).to_str().unwrap()
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_use_default_node_data_directory_path() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let node_dir = tmp_dir.child(".safe/node");
        node_dir.create_dir_all()?;

        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = node_dir.child("node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: None,
            interval: 1,
            num_of_nodes: 11,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        assert_eq!(launcher.launch_args[3], "--nodes-dir");
        assert_eq!(
            launcher.launch_args[4],
            node_dir.path().join(NODES_DATA_DIR_PATH).to_str().unwrap()
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_use_custom_node_data_directory_path() -> Result<()> {
        let custom_node_dir = assert_fs::TempDir::new()?;
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: Some(PathBuf::from(custom_node_dir.path())),
            interval: 1,
            num_of_nodes: 11,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        assert_eq!(launcher.launch_args[3], "--nodes-dir");
        assert_eq!(
            launcher.launch_args[4],
            custom_node_dir
                .path()
                .join(NODES_DATA_DIR_PATH)
                .to_str()
                .unwrap()
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_create_the_node_data_directory_if_it_does_not_exist() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let node_dir = tmp_dir.child(".safe/node");
        node_dir.create_dir_all()?;
        let node_data_dir = node_dir.child(NODES_DATA_DIR_PATH);

        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = node_dir.child("node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: None,
            interval: 1,
            num_of_nodes: 11,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        node_data_dir.assert(predicates::path::is_dir());

        Ok(())
    }

    #[tokio::test]
    async fn should_use_optionally_supplied_interval_value() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let node_dir = tmp_dir.child(".safe/node");
        node_dir.create_dir_all()?;

        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = node_dir.child("node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: None,
            interval: 10,
            num_of_nodes: 11,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        assert_eq!(launcher.launch_args[5], "--interval");
        assert_eq!(launcher.launch_args[6], "10");

        Ok(())
    }

    #[tokio::test]
    async fn should_use_optionally_supplied_num_of_nodes_value() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let node_dir = tmp_dir.child(".safe/node");
        node_dir.create_dir_all()?;

        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = node_dir.child("node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: None,
            interval: 1,
            num_of_nodes: 15,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        assert_eq!(launcher.launch_args[7], "--num-nodes");
        assert_eq!(launcher.launch_args[8], "15");

        Ok(())
    }

    #[tokio::test]
    async fn should_use_optionally_supplied_ip_address() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let node_dir = tmp_dir.child(".safe/node");
        node_dir.create_dir_all()?;

        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = node_dir.child("node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: None,
            interval: 1,
            num_of_nodes: 11,
            ip: Some("10.10.0.1".to_string()),
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        assert_eq!(launcher.launch_args[9], "--ip");
        assert_eq!(launcher.launch_args[10], "10.10.0.1");

        Ok(())
    }

    #[tokio::test]
    async fn should_use_local_flag_if_no_ip_is_supplied() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let node_dir = tmp_dir.child(".safe/node");
        node_dir.create_dir_all()?;

        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = node_dir.child("node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: None,
            interval: 1,
            num_of_nodes: 11,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        assert_eq!(launcher.launch_args[9], "--local");

        Ok(())
    }

    #[tokio::test]
    async fn should_add_baby_fleming_to_networks_list() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let node_dir = tmp_dir.child(".safe/node");
        node_dir.create_dir_all()?;

        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let baby_fleming_config_file =
            tmp_dir.child(".safe/cli/networks/baby-fleming_node_connection_info.config");
        let node_config_file = node_dir.child("node_connection_info.config");
        node_config_file.write_str(SERIALIZED_NODE_CONFIG)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let mut launcher = Box::new(FakeNetworkLauncher {
            launch_args: Vec::new(),
        });

        let cmd = NodeSubCommands::Run {
            node_dir_path: None,
            interval: 1,
            num_of_nodes: 11,
            ip: None,
        };

        let result = node_commander(Some(cmd), &mut config, &mut launcher).await;

        assert!(result.is_ok());
        assert_eq!(config.networks_iter().count(), 1);
        baby_fleming_config_file.assert(predicates::path::is_file());

        let (network_name, network_info) = config.networks_iter().next().unwrap();
        assert_eq!(network_name, "baby-fleming");
        match network_info {
            NetworkInfo::NodeConfig(_) => {
                return Err(eyre!("node config doesn't apply to this test"));
            }
            NetworkInfo::ConnInfoLocation(path) => {
                assert_eq!(
                    *path,
                    String::from(baby_fleming_config_file.path().to_str().unwrap())
                );
            }
        }

        Ok(())
    }
}
