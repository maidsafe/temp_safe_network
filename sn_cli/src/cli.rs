// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[cfg(feature = "node-ctrl")]
use crate::subcommands::node::node_commander;
#[cfg(feature = "data-network")]
use crate::subcommands::{dog::dog_commander, files::files_commander, nrs::nrs_commander};
use crate::{
    operations::auth_and_connect::connect,
    operations::config::Config,
    subcommands::{
        cat::cat_commander,
        config::config_commander,
        keys::key_commander,
        networks::networks_commander,
        setup::setup_commander,
        update::update_commander,
        wallet::wallet_commander,
        xorurl::{xorurl_commander, xorurl_of_files},
        OutputFmt, SubCommands,
    },
};
use clap::{AppSettings::ColoredHelp, Parser};
use color_eyre::{eyre::eyre, Result};
use sn_api::{Safe, XorUrlBase};
use std::path::PathBuf;
use tracing::{debug, warn};

#[derive(clap::StructOpt, Debug)]
/// Interact with the Safe Network
#[clap(global_settings(&[ColoredHelp]), version)]
pub struct CmdArgs {
    /// Available sub commands.
    #[clap(subcommand)]
    pub cmd: SubCommands,
    /// Set the location for the config directory.
    ///
    /// Defaults to $HOME/.safe on Linux/macOS or %USERPROFILE%\.safe on Windows
    #[clap(long = "config-dir-path", env = "SAFE_CONFIG_DIR_PATH", global(true))]
    config_dir_path: Option<PathBuf>,
    /// Perform a dry run of the command. No data will be written.
    #[clap(short = 'n', long = "dry-run", global(true))]
    dry: bool,
    /// Output data serialisation: [json, jsoncompact, yaml]
    #[clap(short = 'o', long = "output", global(true))]
    output_fmt: Option<OutputFmt>,
    /// Use JSON as output serialisation format (alias of '--output json')
    #[clap(long = "json", global(true))]
    output_json: bool,
    /// Base encoding for XOR-URLs. Currently supported: base32z (default), base32 and base64
    #[clap(long = "xorurl", global(true))]
    xorurl_base: Option<XorUrlBase>,
}

pub async fn run() -> Result<()> {
    // Let's first get all the arguments passed in as CLI args
    let args = CmdArgs::from_args();

    let mut safe = Safe::dry_runner(None);
    if let Some(base) = args.xorurl_base {
        safe.xorurl_base = base;
    }

    let mut config = get_config(args.config_dir_path.clone()).await?;

    let result = process_commands(&mut safe, args, &mut config).await;

    // If we were connected to a network, cache the up to date network contacts
    // to disk before exiting
    if safe.is_connected() {
        match safe.section_tree().await {
            Ok(section_tree) => {
                if let Err(err) = config.update_default_network_contacts(&section_tree).await {
                    warn!(
                        "Failed to cache up to date network contacts for genesis key {:?} to '{}': {:?}",
                        section_tree.genesis_key(),
                        config.network_contacts_dir.display(),
                        err
                    );
                } else {
                    debug!(
                        "Up to date network contacts for genesis key {:?} was cached at '{}'",
                        section_tree.genesis_key(),
                        config.network_contacts_dir.display(),
                    );
                }
            }
            Err(err) => {
                let (network_contacts, _) = config.read_default_network_contacts().await?;
                warn!(
                    "Failed to cache updated network contacts for genesis key {:?}: {:?}",
                    network_contacts.genesis_key(),
                    err
                );
            }
        }
    }

    result
}

async fn process_commands(mut safe: &mut Safe, args: CmdArgs, config: &mut Config) -> Result<()> {
    debug!("Processing command: {:?}", args);

    let output_fmt = if args.output_json {
        OutputFmt::Json
    } else if let Some(fmt) = args.output_fmt {
        fmt
    } else {
        OutputFmt::Pretty
    };

    match args.cmd {
        SubCommands::Config { cmd } => config_commander(cmd, config).await,
        SubCommands::Networks { cmd } => networks_commander(cmd, output_fmt, config).await,
        SubCommands::Update { no_confirm } => {
            // We run this command in a separate thread to overcome a conflict with
            // the self_update crate as it seems to be creating its own runtime.
            // The use of the move keyword is required for the closure to take ownership of
            // the no_confirm flag.
            let handler = std::thread::spawn(move || update_commander(no_confirm));
            handler
                .join()
                .map_err(|err| eyre!("Failed to run self update: {:?}", err))?
        }
        SubCommands::Setup(cmd) => setup_commander(cmd, output_fmt),
        SubCommands::Keys(cmd) => key_commander(cmd, output_fmt, config),
        SubCommands::Xorurl {
            cmd,
            location,
            recursive,
            follow_links,
        } => {
            if let Some(cmd) = cmd {
                xorurl_commander(cmd, output_fmt, safe.xorurl_base)
            } else {
                xorurl_of_files(
                    location,
                    recursive,
                    follow_links,
                    output_fmt,
                    safe.xorurl_base,
                )
                .await
            }
        }
        #[cfg(feature = "node-ctrl")]
        SubCommands::Node { cmd, addr } => node_commander(cmd, addr, output_fmt).await,
        other => {
            // Set dry run mode in Safe instance as per arg provide
            safe.dry_run_mode = args.dry;
            // We treat these commands separatelly since we use the credentials if they are
            // available to connect to the network with them (unless dry-run was set),
            // otherwise the connection created will be with read-only access and some
            // of these commands will fail if they require write access.
            if !safe.dry_run_mode {
                connect(safe, config).await?;
            }

            #[cfg(not(feature = "data-network"))]
            return match other {
                SubCommands::Cat(cmd) => cat_commander(cmd, output_fmt, safe).await,
                SubCommands::Wallet(cmd) => wallet_commander(cmd, output_fmt, safe, config).await,
                _ => Err(eyre!("Unknown safe subcommand")),
            };

            #[cfg(feature = "data-network")]
            return match other {
                SubCommands::Cat(cmd) => cat_commander(cmd, output_fmt, safe).await,
                SubCommands::Dog(cmd) => dog_commander(cmd, output_fmt, safe).await,
                SubCommands::Files(cmd) => files_commander(cmd, output_fmt, safe).await,
                SubCommands::Nrs(cmd) => nrs_commander(cmd, output_fmt, safe).await,
                SubCommands::Wallet(cmd) => wallet_commander(cmd, output_fmt, safe, config).await,
                _ => Err(eyre!("Unknown safe subcommand")),
            };
        }
    }
}

/// Gets the configuration, which is used by various parts of the application.
///
/// The location for the configuration directory is either:
/// * The value of the global `--config-dir` argument
/// * The value of the `SAFE_CONFIG_DIR_PATH` environment variable
/// * Defaults to $HOME/.safe on Linux or %USERPROFILE%\.safe on Windows
///
/// The reason for customisation of the location is mainly for integration testing, but users could
/// also apply the customisation if they want to.
async fn get_config(config_dir_path: Option<PathBuf>) -> Result<Config> {
    let config_dir_path = if let Some(config_dir_path) = config_dir_path {
        config_dir_path
    } else {
        let mut default_config_path =
            dirs_next::home_dir().ok_or_else(|| eyre!("Couldn't find user's home directory"))?;
        default_config_path.push(".safe");
        default_config_path
    };

    debug!("Using {config_dir_path:?} for configuration");
    let mut cli_config_path = config_dir_path.clone();
    cli_config_path.push("cli");
    cli_config_path.push("config.json");
    let mut network_contacts_path = config_dir_path;
    network_contacts_path.push("network_contacts");
    let mut config = Config::new(cli_config_path, network_contacts_path).await?;
    config.sync().await?;
    Ok(config)
}
