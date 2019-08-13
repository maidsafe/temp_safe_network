// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{get_from_arg_or_stdin, xorname_to_hex};
use super::OutputFmt;
use crate::subcommands::auth::auth_connect;
use log::debug;
use prettytable::Table;
use safe_cli::{NrsMapContainerInfo, Safe, SafeData};
use std::io::{self, Write};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CatCommands {
    /// The safe:// location to retrieve
    location: Option<String>,
    /// Display additional information about the content being retrieved
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
    match content {
        SafeData::FilesContainer {
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
                    println!("XOR name: 0x{}", xorname_to_hex(&xorname));
                    println!();
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
            } else if cmd.info > 0 {
                // TODO: print out the resolved_from info if -ii was passed (i.e. --info level 2)
                println!(
                        "[{}, {{ \"data_type\": \"{}\", \"type_tag\": \"{}\", \"xorname\": \"{}\" }}, {:?}]",
                        url,
                        data_type,
                        type_tag,
                        xorname_to_hex(&xorname),
                        files_map
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
            data,
            xorname,
            resolved_from,
        } => {
            if cmd.info > 0 {
                println!("Native data type: ImmutableData (published)");
                println!("XOR name: 0x{}", xorname_to_hex(&xorname));
                println!();
                print_resolved_from(cmd.info, resolved_from);
                println!("Raw content of the file:");
            }

            // Render ImmutableData file
            io::stdout()
                .write_all(&data)
                .map_err(|err| format!("Failed to print out the content of the file: {}", err))?
        }
        SafeData::Key { .. } => println!("Content type 'Key' not supported yet by 'cat' command"),
        SafeData::Wallet { .. } => {
            println!("Content type 'Wallet' not supported yet by 'cat' command")
        }
    }

    Ok(())
}

fn print_resolved_from(info_level: u8, resolved_from: Option<NrsMapContainerInfo>) {
    if info_level > 1 {
        if let Some(nrs_map_container) = resolved_from {
            // print out the resolved_from info since it's --info level 2
            println!("Resolved using NRS Map:");
            println!("PublicName: \"{}\"", nrs_map_container.public_name);
            println!("Container XOR-URL: {}", nrs_map_container.xorurl);
            println!("Native data type: {}", nrs_map_container.data_type);
            println!("Type tag: {}", nrs_map_container.type_tag);
            println!("XOR name: 0x{}", xorname_to_hex(&nrs_map_container.xorname));
            println!("Version: {}", nrs_map_container.version);

            if info_level > 2 {
                let mut table = Table::new();
                table.add_row(
                    row![bFg->"NRS name/subname", bFg->"Created", bFg->"Modified", bFg->"Link"],
                );

                let summary = nrs_map_container.nrs_map.get_map_summary();
                summary.iter().for_each(|(name, rdf_info)| {
                    table.add_row(row![
                        format!("{}{}", name, nrs_map_container.public_name),
                        rdf_info["created"],
                        rdf_info["modified"],
                        rdf_info["link"],
                    ]);
                });
                table.printstd();
                println!();
            }
        }
    }
}
