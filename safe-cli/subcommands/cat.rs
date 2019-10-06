// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::dog::print_resolved_from;
use super::helpers::{get_from_arg_or_stdin, xorname_to_hex};
use super::OutputFmt;
use crate::subcommands::auth::auth_connect;
use log::debug;
use prettytable::Table;
use safe_api::{Safe, SafeData};
use std::io::{self, Write};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CatCommands {
    /// The safe:// location to retrieve
    location: Option<String>,
    /// Display additional information about the content being retrieved. Different levels of details can be obtained by passing this flag several times, i.e. `-i`, `-ii`, or `-iii`
    #[structopt(short = "i", long = "info", parse(from_occurrences))]
    info: u8,
}

pub fn cat_commander(
    cmd: CatCommands,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    let url = get_from_arg_or_stdin(cmd.location, None)?;
    debug!("Running cat for: {:?}", &url);

    // TODO: pending: https://github.com/maidsafe/safe_client_libs/issues/899
    // switch to connect_without_authL: connect_without_auth(safe)?;
    auth_connect(safe)?;
    let content = safe.fetch(&url)?;
    match &content {
        SafeData::FilesContainer {
            xorurl,
            version,
            files_map,
            type_tag,
            xorname,
            data_type,
            resolved_from,
        } => {
            // Render FilesContainer
            if OutputFmt::Pretty == output_fmt {
                if cmd.info > 0 {
                    println!("Native data type: {}", data_type);
                    println!("Type tag: {}", type_tag);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("XOR-URL: {}", xorurl);
                    print_resolved_from(cmd.info, resolved_from);
                    println!();
                }

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
            } else if resolved_from.is_some() && cmd.info > 1 {
                println!(
                    "{}",
                    serde_json::to_string(&(&url, content))
                        .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
                );
            } else if cmd.info > 0 {
                println!(
                        "[{}, {{ \"data_type\": \"{}\", \"version\": \"{}\", \"type_tag\": \"{}\", \"xorname\": \"{}\" }}, {:?}]",
                        url,
                        data_type,
                        version,
                        type_tag,
                        xorname_to_hex(xorname),
                        files_map,
                    );
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&(url, files_map))
                        .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
                );
            }
        }
        SafeData::PublishedImmutableData {
            xorurl,
            data,
            xorname,
            resolved_from,
            media_type,
        } => {
            if cmd.info > 0 {
                println!("Native data type: ImmutableData (published)");
                println!("XOR name: 0x{}", xorname_to_hex(xorname));
                println!("XOR-URL: {}", xorurl);
                println!(
                    "Media type: {}",
                    media_type.clone().unwrap_or_else(|| "Unknown".to_string())
                );
                print_resolved_from(cmd.info, resolved_from);
                println!("Raw content of the file:");
            }

            // Render ImmutableData file
            io::stdout()
                .write_all(data)
                .map_err(|err| format!("Failed to print out the content of the file: {}", err))?
        }
        SafeData::Wallet {
            xorurl,
            xorname,
            type_tag,
            balances,
            data_type,
            resolved_from,
        } => {
            // Render Wallet
            if OutputFmt::Pretty == output_fmt {
                if cmd.info > 0 {
                    println!("Native data type: {}", data_type);
                    println!("Type tag: {}", type_tag);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("XOR-URL: {}", xorurl);
                    print_resolved_from(cmd.info, resolved_from);
                    println!();
                }

                println!("Spendable balances of Wallet at \"{}\":", url);
                let mut table = Table::new();
                table.add_row(row![bFg->"Default", bFg->"Friendly Name", bFg->"SafeKey URL"]);
                balances.iter().for_each(|(name, (default, balance))| {
                    let def = if *default { "*" } else { "" };
                    table.add_row(row![def, name, balance.xorurl]);
                });
                table.printstd();
            } else if resolved_from.is_some() && cmd.info > 1 {
                println!(
                    "{}",
                    serde_json::to_string(&(&url, content))
                        .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
                );
            } else if cmd.info > 0 {
                println!(
                        "[{}, {{ \"data_type\": \"{}\", \"type_tag\": \"{}\", \"xorname\": \"{}\" }}, {:?}]",
                        url,
                        data_type,
                        type_tag,
                        xorname_to_hex(xorname),
                        balances,
                    );
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&(url, balances))
                        .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
                );
            }
        }
        SafeData::SafeKey {
            xorurl,
            xorname,
            resolved_from,
        } => {
            if OutputFmt::Pretty == output_fmt {
                if cmd.info > 0 {
                    println!("Native data type: SafeKey");
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("XOR-URL: {}", xorurl);
                    print_resolved_from(cmd.info, resolved_from);
                } else {
                    println!("No content to show since the URL targets a SafeKey. Use -i / --info flag to obtain additional information about the targeted SafeKey.");
                }
            } else if resolved_from.is_some() && cmd.info > 1 {
                println!(
                    "{}",
                    serde_json::to_string(&(&url, content))
                        .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
                );
            } else {
                println!(
                    "[{}, {{ \"data_type\": \"SafeKey\", \"xorname\": \"{}\" }}]",
                    url,
                    xorname_to_hex(xorname),
                );
            }
        }
    }

    Ok(())
}
