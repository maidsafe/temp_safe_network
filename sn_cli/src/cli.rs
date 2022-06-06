// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    operations::auth_and_connect::connect,
    operations::config::{Config, SnLaunchToolNetworkLauncher},
    subcommands::{
        cat::cat_commander,
        config::config_commander,
        dog::dog_commander,
        files::files_commander,
        keys::key_commander,
        networks::networks_commander,
        node::node_commander,
        nrs::nrs_commander,
        setup::setup_commander,
        update::update_commander,
        wallet::wallet_commander,
        xorurl::{xorurl_commander, xorurl_of_files},
        OutputFmt, SubCommands,
    },
};
use color_eyre::{eyre::eyre, Result};
use sn_api::{Safe, XorUrlBase};
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use structopt::{clap::AppSettings::ColoredHelp, StructOpt};
use tracing::debug;

const DEFAULT_OPERATION_TIMEOUT_SECS: u64 = 120; // 2mins
const DEFAULT_PREFIX_SYMLINK_NAME: &str = "default";
const SN_CLI_QUERY_TIMEOUT: &str = "SN_CLI_QUERY_TIMEOUT";

#[derive(StructOpt, Debug)]
/// Interact with the Safe Network
#[structopt(global_settings(&[ColoredHelp]))]
pub struct CmdArgs {
    /// subcommands
    #[structopt(subcommand)]
    pub cmd: SubCommands,
    /// Output data serialisation: [json, jsoncompact, yaml]
    #[structopt(short = "o", long = "output", global(true))]
    output_fmt: Option<OutputFmt>,
    /// Sets JSON as output serialisation format (alias of '--output json')
    #[structopt(long = "json", global(true))]
    output_json: bool,
    // /// Increase output verbosity. (More logs!)
    // #[structopt(short = "v", long = "verbose", global(true))]
    // verbose: bool,
    /// Dry run of command. No data will be written. No coins spent
    #[structopt(short = "n", long = "dry-run", global(true))]
    dry: bool,
    /// Base encoding to be used for XOR-URLs generated. Currently supported: base32z (default), base32 and base64
    #[structopt(long = "xorurl", global(true))]
    xorurl_base: Option<XorUrlBase>,
}

pub async fn run() -> Result<()> {
    // Let's first get all the arguments passed in, either as function's args, or CLI args
    let args = CmdArgs::from_args();

    let mut safe = Safe::dry_runner(None);
    if let Some(base) = args.xorurl_base {
        safe.xorurl_base = base;
    }

    let output_fmt = if args.output_json {
        OutputFmt::Json
    } else if let Some(fmt) = args.output_fmt {
        fmt
    } else {
        OutputFmt::Pretty
    };

    debug!("Processing command: {:?}", args);

    let result = match args.cmd {
        SubCommands::Config { cmd } => config_commander(cmd, &mut get_config().await?).await,
        SubCommands::Networks { cmd } => networks_commander(cmd, &mut get_config().await?).await,
        SubCommands::Update { no_confirm } => {
            // We run this command in a separate thread to overcome a conflict with
            // the self_update crate as it seems to be creating its own runtime.
            // The use of the move keyword is required for the closure to take ownership of
            // the no_confirm flag.
            let handler = std::thread::spawn(move || {
                update_commander(no_confirm)
                    .map_err(|err| eyre!("Error performing update: {}", err))
            });
            handler
                .join()
                .map_err(|err| eyre!("Failed to run self update: {:?}", err))?
        }
        SubCommands::Setup(cmd) => setup_commander(cmd, output_fmt),
        SubCommands::Node { cmd } => {
            let mut launcher = Box::new(SnLaunchToolNetworkLauncher::default());
            node_commander(cmd, &mut get_config().await?, &mut launcher).await
        }
        SubCommands::Keys(cmd) => key_commander(cmd, output_fmt, &get_config().await?),
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
        other => {
            // Set dry run mode in Safe instance as per arg provide
            safe.dry_run_mode = args.dry;
            // We treat these commands separatelly since we use the credentials if they are
            // available to connect to the network with them (unless dry-run was set),
            // otherwise the connection created will be with read-only access and some
            // of these commands will fail if they require write access.
            if !safe.dry_run_mode {
                let timeout_secs: u64 = match env::var(SN_CLI_QUERY_TIMEOUT) {
                    Ok(timeout) => timeout.parse::<u64>().map_err(|_| {
                        eyre!(
                            "Could not parse {} env var value: {}",
                            SN_CLI_QUERY_TIMEOUT,
                            timeout
                        )
                    })?,
                    Err(_) => DEFAULT_OPERATION_TIMEOUT_SECS,
                };

                connect(
                    &mut safe,
                    &get_config().await?,
                    Duration::from_secs(timeout_secs),
                )
                .await?;
            }

            match other {
                SubCommands::Cat(cmd) => cat_commander(cmd, output_fmt, &safe).await,
                SubCommands::Dog(cmd) => dog_commander(cmd, output_fmt, &safe).await,
                SubCommands::Files(cmd) => files_commander(cmd, output_fmt, &safe).await,
                SubCommands::Nrs(cmd) => nrs_commander(cmd, output_fmt, &safe).await,
                SubCommands::Wallet(cmd) => {
                    wallet_commander(cmd, output_fmt, &safe, &get_config().await?).await
                }
                _ => Err(eyre!("Unknown safe subcommand")),
            }
        }
    };

    result
}

/// Gets the configuration, which is used by various parts of the application.
///
/// The `SN_CLI_CONFIG_PATH` allows the user to define a custom location as an alternative to
/// ~/.safe, but this has mainly been added to enable integration tests to use a temporary location
/// for the config files, and you can then use `assert_fs` to to assert against those temp files.
/// Using a temporary location also means the test suites don't manipulate the current user's home
/// directory.
async fn get_config() -> Result<Config> {
    let mut default_config_path =
        dirs_next::home_dir().ok_or_else(|| eyre!("Couldn't find user's home directory"))?;
    default_config_path.push(".safe");
    let config_path =
        std::env::var("SN_CLI_CONFIG_PATH").map_or(default_config_path.clone(), PathBuf::from);

    let mut cli_config_path = config_path.clone();
    cli_config_path.push("cli");
    cli_config_path.push("config.json");
    let mut prefix_maps_dir = default_config_path;
    prefix_maps_dir.push("prefix_maps");
    let mut config = Config::new(
        cli_config_path,
        prefix_maps_dir,
        DEFAULT_PREFIX_SYMLINK_NAME.to_string(),
    )
    .await?;
    config.sync().await?;
    Ok(config)
}
