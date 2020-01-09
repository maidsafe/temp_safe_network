// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::helpers::{get_from_arg_or_stdin, serialise_output};
use super::OutputFmt;
use log::debug;
use pretty_hex;
use prettytable::Table;
use safe_api::{Safe, SafeData};
use std::io::{self, Write};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CatCommands {
    /// The safe:// location to retrieve
    location: Option<String>,
    /// Renders file output as hex
    #[structopt(short = "x", long = "hexdump")]
    hexdump: bool,
}

pub fn cat_commander(
    cmd: CatCommands,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    let url = get_from_arg_or_stdin(cmd.location, None)?;
    debug!("Running cat for: {:?}", &url);

    let content = safe.fetch(&url)?;
    match &content {
        SafeData::FilesContainer {
            version, files_map, ..
        } => {
            // Render FilesContainer
            if OutputFmt::Pretty == output_fmt {
                println!(
                    "Files of FilesContainer (version {}) at \"{}\":",
                    version, url
                );
                let mut table = Table::new();
                table.add_row(
                    row![bFg->"Name", bFg->"Size", bFg->"Created", bFg->"Modified", bFg->"Link"],
                );
                files_map.iter().for_each(|(name, file_item)| {
                    table.add_row(row![
                        name,
                        file_item["size"],
                        file_item["created"],
                        file_item["modified"],
                        file_item["link"],
                    ]);
                });
                table.printstd();
            } else {
                println!("{}", serialise_output(&(url, files_map), output_fmt));
            }
        }
        SafeData::PublishedImmutableData { data, .. } => {
            if cmd.hexdump {
                // Render hex representation of ImmutableData file
                println!("{}", pretty_hex::pretty_hex(data));
            } else {
                // Render ImmutableData file
                io::stdout().write_all(data).map_err(|err| {
                    format!("Failed to print out the content of the file: {}", err)
                })?
            }
        }
        SafeData::Wallet { balances, .. } => {
            // Render Wallet
            if OutputFmt::Pretty == output_fmt {
                println!("Spendable balances of Wallet at \"{}\":", url);
                let mut table = Table::new();
                table.add_row(row![bFg->"Default", bFg->"Friendly Name", bFg->"SafeKey URL"]);
                balances.iter().for_each(|(name, (default, balance))| {
                    let def = if *default { "*" } else { "" };
                    table.add_row(row![def, name, balance.xorurl]);
                });
                table.printstd();
            } else {
                println!("{}", serialise_output(&(url, balances), output_fmt));
            }
        }
        SafeData::SafeKey { .. } => {
            println!("No content to show since the URL targets a SafeKey. Use the 'dog' command to obtain additional information about the targeted SafeKey.");
        }
    }

    Ok(())
}
