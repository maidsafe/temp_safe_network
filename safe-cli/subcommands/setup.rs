// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::serialise_output;
use super::OutputFmt;
use crate::cli::CmdArgs;
use std::io::Write;
use structopt::{clap, StructOpt};

// Defines subcommands of 'setup'
#[derive(StructOpt, Debug)]
pub enum SetupSubCommands {
    /// Dump shell completions.
    #[structopt(name = "completions")]
    Completions {
        /// one of: [bash, fish, zsh, powershell, elvish]  default = all shells
        shell: Option<clap::Shell>,
    },
}

// handles 'setup <cmd>' commands.
pub fn setup_commander(cmd: SetupSubCommands, output_fmt: OutputFmt) -> Result<(), String> {
    // Let's keep this clean and place each cmd handler in its own func.
    match cmd {
        SetupSubCommands::Completions { shell } => setup_completions(shell, output_fmt),
    }
}

// differentiates between 'setup completions' and 'setup completions <shell>'
fn setup_completions(shell: Option<clap::Shell>, output_fmt: OutputFmt) -> Result<(), String> {
    match shell {
        Some(shell_id) => setup_completions_dumpone(shell_id, output_fmt),
        None => setup_completions_dumpall(output_fmt),
    }
}

// handles 'setup completions <shell>' command.  dumps completions for single shell.
fn setup_completions_dumpone(shell: clap::Shell, output_fmt: OutputFmt) -> Result<(), String> {
    let buf = gen_completions_for_shell(shell)?;

    if OutputFmt::Pretty == output_fmt {
        // Pretty format just writes the shell completion to stdout
        std::io::stdout()
            .write_all(&buf)
            .map_err(|err| format!("Failed to print shell completions. {}", err))?;
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
fn setup_completions_dumpall(output_fmt: OutputFmt) -> Result<(), String> {
    // get names of available shells and sort them.
    let mut shellnames = clap::Shell::variants();
    shellnames.sort();

    if OutputFmt::Pretty == output_fmt {
        // Pretty format outputs shell completions with header --- <shellname> --- above each
        // Only useful for human readability/review.  Installers should use --json
        for shellname in shellnames.iter() {
            let shell = shellname.parse::<clap::Shell>()?;
            let buf = gen_completions_for_shell(shell)?;

            println!("--- {} ---", shellname);
            std::io::stdout()
                .write_all(&buf)
                .map_err(|err| format!("Failed to print shell completions. {}", err))?
        }
        println!();
    } else {
        // To serialise, we first need to build a json object dynamically. looks like:
        // { "bash": "completion_buf", "powershell": "completion_buf", ... }
        let mut map = serde_json::map::Map::new();

        for shellname in shellnames.iter() {
            let shell = shellname.parse::<clap::Shell>()?;
            let buf = gen_completions_for_shell(shell)?;
            match std::str::from_utf8(&buf) {
                Ok(v) => {
                    map.insert((*shellname).to_string(), serde_json::json!(v));
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
fn gen_completions_for_shell(shell: clap::Shell) -> Result<Vec<u8>, String> {
    // Get exe path
    let exe_path =
        std::env::current_exe().map_err(|err| format!("Can't get the exec path: {}", err))?;

    // get filename without preceding path as std::ffi::OsStr (C string)
    let exec_name_ffi = match exe_path.file_name() {
        Some(v) => v,
        None => return Err("Can't extract file_name of executable".to_string()),
    };

    // Convert OsStr to string.  Can fail if OsStr contains any invalid unicode.
    let exec_name = match exec_name_ffi.to_str() {
        Some(v) => v.to_string(),
        None => return Err("Can't decode unicode in executable name".to_string()),
    };

    // Generates shell completions for <shell> and prints to stdout
    let mut buf: Vec<u8> = vec![];
    CmdArgs::clap().gen_completions_to(exec_name, shell, &mut buf);

    Ok(buf)
}
