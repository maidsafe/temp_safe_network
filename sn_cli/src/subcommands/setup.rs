// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{helpers::serialise_output, OutputFmt};
use crate::cli::CmdArgs;
use clap::{CommandFactory, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use color_eyre::{eyre::bail, eyre::WrapErr, Result};
use std::io::Write;

// Defines subcommands of 'setup'
#[derive(Subcommand, Debug)]
pub enum SetupSubCommands {
    /// Dump shell completions.
    #[clap(name = "completions")]
    Completions {
        /// one of: [bash, fish, zsh, powershell, elvish]  default = all shells
        shell: Option<Shell>,
    },
}

// handles 'setup <cmd>' commands.
pub fn setup_commander(cmd: SetupSubCommands, output_fmt: OutputFmt) -> Result<()> {
    // Let's keep this clean and place each cmd handler in its own func.
    match cmd {
        SetupSubCommands::Completions { shell } => setup_completions(shell, output_fmt),
    }
}

// differentiates between 'setup completions' and 'setup completions <shell>'
fn setup_completions(shell: Option<Shell>, output_fmt: OutputFmt) -> Result<()> {
    match shell {
        Some(shell_id) => setup_completions_dumpone(shell_id, output_fmt),
        None => setup_completions_dumpall(output_fmt),
    }
}

// handles 'setup completions <shell>' command.  dumps completions for single shell.
fn setup_completions_dumpone(shell: Shell, output_fmt: OutputFmt) -> Result<()> {
    let buf = gen_completions_for_shell(shell)?;

    if OutputFmt::Pretty == output_fmt {
        // Pretty format just writes the shell completion to stdout
        std::io::stdout()
            .write_all(&buf)
            .wrap_err("Failed to print shell completions")?;
        println!();
    } else {
        // will be serialized as a string.  no object container.
        match std::str::from_utf8(&buf) {
            Ok(v) => println!("{}", serialise_output(v, output_fmt)),
            Err(e) => println!("Invalid UTF-8 sequence: {}", e),
        };
    }

    Ok(())
}

// handles 'setup completions' command.  dumps completions for all shells.
fn setup_completions_dumpall(output_fmt: OutputFmt) -> Result<()> {
    // get names of available shells and sort them.
    let shells = Vec::from_iter(Shell::value_variants());

    if OutputFmt::Pretty == output_fmt {
        // Pretty format outputs shell completions with header --- <shellname> --- above each
        // Only useful for human readability/review.  Installers should use --json
        for shell in shells {
            let buf = gen_completions_for_shell(*shell)?;

            println!("--- {} ---", shell);

            std::io::stdout()
                .write_all(&buf)
                .wrap_err("Failed to print shell completions")?
        }
        println!();
    } else {
        // To serialise, we first need to build a json object dynamically. looks like:
        // { "bash": "completion_buf", "powershell": "completion_buf", ... }
        let mut map = serde_json::map::Map::new();

        for shell in shells {
            let buf = gen_completions_for_shell(*shell)?;
            match std::str::from_utf8(&buf) {
                Ok(v) => {
                    map.insert(shell.to_string(), serde_json::json!(v));
                }
                Err(e) => println!("Invalid UTF-8 sequence: {}", e),
            };
        }

        let jsonv = serde_json::json!(map);

        println!("{}", serialise_output(&jsonv, output_fmt));
    }

    Ok(())
}

// generates completions for a given shell, eg bash.
fn gen_completions_for_shell(shell: Shell) -> Result<Vec<u8>> {
    // Get exe path
    let exe_path = std::env::current_exe().wrap_err("Can't get the exec path")?;

    // get filename without preceding path as std::ffi::OsStr (C string)
    let exec_name_ffi = match exe_path.file_name() {
        Some(v) => v,
        None => bail!("Can't extract file_name of executable"),
    };

    // Convert OsStr to string.  Can fail if OsStr contains any invalid unicode.
    let exec_name = match exec_name_ffi.to_str() {
        Some(v) => v.to_string(),
        None => bail!("Can't decode unicode in executable name"),
    };

    // Generates shell completions for <shell> and prints to stdout
    let mut buf: Vec<u8> = vec![];
    generate(shell, &mut CmdArgs::command(), exec_name, &mut buf);

    Ok(buf)
}
