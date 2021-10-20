// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{
    operations::config::Config,
    operations::safe_net::connect,
    shell,
    subcommands::{
        cat::cat_commander, config::config_commander, dog::dog_commander, files::files_commander,
        keys::key_commander, networks::networks_commander, node::node_commander,
        nrs::nrs_commander, setup::setup_commander, update::update_commander,
        xorurl::xorurl_commander, OutputFmt, SubCommands,
    },
};
use color_eyre::{eyre::eyre, Result};
use sn_api::{Safe, XorUrlBase};
use std::env;
use std::time::Duration;
use structopt::{clap::AppSettings::ColoredHelp, StructOpt};
use tracing::debug;

const DEFAULT_TIMEOUT_SECS: u64 = 60 * 10; //10 mins

#[derive(StructOpt, Debug)]
/// Interact with the Safe Network
#[structopt(global_settings(&[ColoredHelp]))]
pub struct CmdArgs {
    /// subcommands
    #[structopt(subcommand)]
    pub cmd: Option<SubCommands>,
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
    /// Endpoint of the Authenticator daemon where to send requests to. If not provided, https://localhost:33000 is assumed.
    #[structopt(long = "endpoint", global(true))]
    pub endpoint: Option<String>,
}

pub async fn run() -> Result<()> {
    let cli_timeout: u64 = match env::var("SN_CLI_QUERY_TIMEOUT") {
        Ok(timeout) => timeout
            .parse::<u64>()
            .map_err(|_| eyre!("Could not parse \'SN_CLI_QUERY_TIMEOUT\' env var"))?,
        Err(_) => DEFAULT_TIMEOUT_SECS,
    };

    let mut safe = Safe::new(None, Duration::from_secs(cli_timeout));
    run_with(None, &mut safe).await
}

pub async fn run_with(cmd_args: Option<&[&str]>, safe: &mut Safe) -> Result<()> {
    // Let's first get all the arguments passed in, either as function's args, or CLI args
    let args = match cmd_args {
        None => CmdArgs::from_args(),
        Some(cmd_args) => CmdArgs::from_iter_safe(cmd_args)?,
    };

    let prev_base = safe.xorurl_base;
    if let Some(base) = args.xorurl_base {
        safe.xorurl_base = base;
    }

    let output_fmt = if args.output_json {
        OutputFmt::Json
    } else {
        match args.output_fmt {
            Some(fmt) => fmt,
            None => OutputFmt::Pretty,
        }
    };

    debug!("Processing command: {:?}", args);

    let result = match args.cmd {
        Some(SubCommands::Config { cmd }) => config_commander(cmd, &mut get_config()?).await,
        Some(SubCommands::Networks { cmd }) => networks_commander(cmd, &mut get_config()?).await,
        Some(SubCommands::Update { no_confirm }) => {
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
        Some(SubCommands::Setup(cmd)) => setup_commander(cmd, output_fmt),
        Some(SubCommands::Xorurl {
            cmd,
            location,
            recursive,
            follow_links,
        }) => xorurl_commander(cmd, location, recursive, follow_links, output_fmt, safe).await,
        Some(SubCommands::Node { cmd }) => node_commander(cmd, &mut get_config()?).await,
        Some(other) => {
            // We treat these commands separatelly since we use the credentials if they are
            // available to connect to the network with them (unless dry-run was set),
            // otherwise the connection created  will be with read-only access and some
            // of these commands will fail if they require write access.
            if !args.dry {
                connect(safe, get_config()?).await?;
            }

            match other {
                SubCommands::Keys(cmd) => key_commander(cmd, output_fmt, safe).await,
                SubCommands::Cat(cmd) => cat_commander(cmd, output_fmt, safe).await,
                SubCommands::Dog(cmd) => dog_commander(cmd, output_fmt, safe).await,
                SubCommands::Files(cmd) => files_commander(cmd, output_fmt, args.dry, safe).await,
                SubCommands::Nrs(cmd) => nrs_commander(cmd, output_fmt, args.dry, safe).await,
                _ => Err(eyre!("Unknown safe subcommand")),
            }
        }
        None => shell::shell_run(), // then enter in interactive shell
    };

    safe.xorurl_base = prev_base;
    result
}

fn get_config() -> Result<Config> {
    let mut config_path =
        dirs_next::home_dir().ok_or_else(|| eyre!("Couldn't find user's home directory"))?;
    config_path.push(".safe");
    config_path.push("cli");
    config_path.push("config.json");
    let mut node_config_path =
        dirs_next::home_dir().ok_or_else(|| eyre!("Failed to obtain user's home path"))?;
    node_config_path.push(".safe");
    node_config_path.push("node");
    node_config_path.push("node_connection_info.config");
    Config::new(config_path, node_config_path)
}
