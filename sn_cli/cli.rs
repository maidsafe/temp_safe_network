// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use log::debug;
use structopt::StructOpt;

use crate::{
    operations::safe_net::connect,
    shell,
    subcommands::{
        auth::auth_commander, cat::cat_commander, config::config_commander, dog::dog_commander,
        files::files_commander, keys::key_commander, networks::networks_commander,
        node::node_commander, nrs::nrs_commander, seq::seq_commander, setup::setup_commander,
        update::update_commander, wallet::wallet_commander, xorurl::xorurl_commander, OutputFmt,
        SubCommands,
    },
};
use sn_api::{xorurl::XorUrlBase, Safe};

#[derive(StructOpt, Debug)]
/// Interact with the Safe Network
#[structopt(global_settings(&[structopt::clap::AppSettings::ColoredHelp]))]
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

pub async fn run() -> Result<(), String> {
    let mut safe = Safe::default();
    run_with(None, &mut safe).await
}

pub async fn run_with(cmd_args: Option<&[&str]>, safe: &mut Safe) -> Result<(), String> {
    // Let's first get all the arguments passed in, either as function's args, or CLI args
    let args = match cmd_args {
        None => CmdArgs::from_args(),
        Some(cmd_args) => CmdArgs::from_iter_safe(cmd_args).map_err(|err| err.to_string())?,
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
        Some(SubCommands::Config { cmd }) => config_commander(cmd),
        Some(SubCommands::Networks { cmd }) => networks_commander(cmd),
        Some(SubCommands::Keypair {}) => {
            let key_pair = safe.keypair().await?;
            if OutputFmt::Pretty == output_fmt {
                println!("Key pair generated:");
            }
            let sk = key_pair
                .secret_key()
                .map_err(|e| format!("{:?}", e))?
                .to_string();

            println!("Public Key = {}", key_pair.public_key());
            println!("Secret Key = {}", sk);
            Ok(())
        }
        Some(SubCommands::Update {}) => {
            // We run this command in a separate thread to overcome a conflict with
            // the self_update crate as it seems to be creating its own runtime.
            let handler = std::thread::spawn(|| {
                update_commander().map_err(|err| format!("Error performing update: {}", err))
            });
            handler
                .join()
                .map_err(|err| format!("Failed to run self update: {:?}", err))?
        }
        Some(SubCommands::Setup(cmd)) => setup_commander(cmd, output_fmt),
        Some(SubCommands::Xorurl {
            cmd,
            location,
            recursive,
            follow_links,
        }) => xorurl_commander(cmd, location, recursive, follow_links, output_fmt, safe).await,
        Some(SubCommands::Node { cmd }) => node_commander(cmd),
        Some(SubCommands::Auth { cmd }) => auth_commander(cmd, args.endpoint, safe).await,
        Some(other) => {
            // We treat these separatelly since we use the credentials if they are available to
            // connect to the network with them, otherwise the connection created will be with
            // read-only access and some of these commands will fail if they require write access
            connect(safe).await?;
            match other {
                SubCommands::Keys(cmd) => key_commander(cmd, output_fmt, safe).await,
                SubCommands::Cat(cmd) => cat_commander(cmd, output_fmt, safe).await,
                SubCommands::Dog(cmd) => dog_commander(cmd, output_fmt, safe).await,
                SubCommands::Wallet(cmd) => wallet_commander(cmd, output_fmt, safe).await,
                SubCommands::Files(cmd) => files_commander(cmd, output_fmt, args.dry, safe).await,
                SubCommands::Nrs(cmd) => nrs_commander(cmd, output_fmt, args.dry, safe).await,
                SubCommands::Seq(cmd) => seq_commander(cmd, output_fmt, safe).await,
                _ => Err("Unknown safe subcommand".to_string()),
            }
        }
        None => shell::shell_run(), // then enter in interactive shell
    };

    safe.xorurl_base = prev_base;
    result
}
