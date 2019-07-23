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
use safe_cli::{Safe, SafeData};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CatCommands {
    /// The safe:// location to retrieve
    location: Option<String>,
    /// Version of the resource to retrieve
    #[structopt(long = "version")]
    version: Option<String>,
    /// Display additional information about the content being retrieved
    #[structopt(short = "i", long = "info")]
    info: bool,
}

pub fn cat_commander(
    cmd: CatCommands,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    // TODO: Get specific versions.
    let xorurl = get_from_arg_or_stdin(cmd.location, None)?;
    debug!("Running cat for: {:?}", &xorurl);

    // TODO: pending: https://github.com/maidsafe/safe_client_libs/issues/899
    // switch to connect_without_auth
    auth_connect(safe)?;
    // connect_without_auth(safe)?;
    let content = safe.fetch(&xorurl)?;
    match content {
        SafeData::FilesContainer {
            version,
            files_map,
            type_tag,
            xorname,
            data_type,
        } => {
            // Render FilesContainer
            if OutputFmt::Pretty == output_fmt {
                if cmd.info {
                    println!("Native data type: {}", data_type);
                    println!("Type tag: {}", type_tag,);
                    println!("XOR name: 0x{}", xorname_to_hex(&xorname));
                    println!();
                }

                println!(
                    "Files of FilesContainer (version {}) at \"{}\":",
                    version, xorurl
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
            } else if cmd.info {
                println!(
                        "[{}, {{ \"data_type\": \"{}\", \"type_tag\": \"{}\", \"xorname\": \"{}\" }}, {:?}]",
                        xorurl,
                        data_type,
                        type_tag,
                        xorname_to_hex(&xorname),
                        files_map
                    );
            } else {
                println!("[{}, {:?}]", xorurl, files_map);
            }
        }
        SafeData::PublishedImmutableData { data, xorname } => {
            if cmd.info {
                println!("Native data type: ImmutableData (published)");
                println!("XOR name: 0x{}", xorname_to_hex(&xorname));
                println!();
                println!("Raw content of the file:");
            }

            // Render ImmutableData file
            let data_string = match String::from_utf8(data) {
                Ok(string) => string,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };

            // data always has \n at end?
            println!("{}", data_string);
        }
        other => println!(
            "Content type '{:?}' not supported yet by 'cat' command",
            other
        ),
    }

    Ok(())
}
