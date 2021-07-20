// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{get_from_arg_or_stdin, print_nrs_map, serialise_output},
    OutputFmt,
};
use anyhow::{Context, Result};
use log::{debug, trace};
use prettytable::Table;
use sn_api::{fetch::SafeData, Safe};
use std::io::{self, Write};
use structopt::StructOpt;
use tokio::time::{sleep, Duration};

const MAX_RETRY_ATTEMPTS: usize = 5;

#[derive(StructOpt, Debug)]
pub struct CatCommands {
    /// The safe:// location to retrieve
    location: Option<String>,
    /// Renders file output as hex
    #[structopt(short = "x", long = "hexdump")]
    hexdump: bool,
}

pub async fn cat_commander(cmd: CatCommands, output_fmt: OutputFmt, safe: &mut Safe) -> Result<()> {
    let url = get_from_arg_or_stdin(cmd.location, None)?;
    debug!("Running cat for: {:?}", &url);

    let mut attempts = 0;

    let mut content = safe.fetch(&url, None).await;

    while content.is_err() && attempts < MAX_RETRY_ATTEMPTS {
        trace!("cat attempt #{:?}", attempts);
        sleep(Duration::from_secs(1)).await;
        content = safe.fetch(&url, None).await;

        attempts += 1;
    }

    let content = content?;

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
                    row![bFg->"Name", bFg->"Type", bFg->"Size", bFg->"Created", bFg->"Modified", bFg->"Link"],
                );
                files_map.iter().for_each(|(name, file_item)| {
                    table.add_row(row![
                        name,
                        file_item["type"],
                        file_item["size"],
                        file_item["created"],
                        file_item["modified"],
                        file_item.get("link").unwrap_or(&String::default()),
                    ]);
                });
                table.printstd();
            } else {
                println!("{}", serialise_output(&(url, files_map), output_fmt));
            }
        }
        SafeData::PublicBlob { data, .. } => {
            if cmd.hexdump {
                // Render hex representation of Blob file
                println!("{}", pretty_hex::pretty_hex(data));
            } else {
                // Render Blob file
                io::stdout()
                    .write_all(data)
                    .context("Failed to print out the content of the file")?;
            }
        }
        SafeData::NrsMapContainer {
            public_name,
            version,
            nrs_map,
            ..
        } => {
            // Render NRS Map Container
            if OutputFmt::Pretty == output_fmt {
                println!("NRS Map Container (version {}) at \"{}\":", version, url);
                print_nrs_map(&nrs_map, public_name);
            } else {
                println!("{}", serialise_output(&(url, nrs_map), output_fmt));
            }
        }
        SafeData::SafeKey { .. } => {
            println!("No content to show since the URL targets a SafeKey. Use the 'dog' command to obtain additional information about the targeted SafeKey.");
        }
        SafeData::PublicSequence { data, version, .. } => {
            if OutputFmt::Pretty == output_fmt {
                println!("Public Sequence (version {}) at \"{}\":", version, url);
                if cmd.hexdump {
                    // Render hex representation of Sequence content
                    println!("{}", pretty_hex::pretty_hex(data));
                } else {
                    // Render Sequence content
                    io::stdout()
                        .write_all(data)
                        .context("Failed to print out the content of the file")?;
                }
            } else {
                println!("{}", serialise_output(&(url, data), output_fmt));
            }
        }
        SafeData::PrivateSequence { data, version, .. } => {
            if OutputFmt::Pretty == output_fmt {
                println!("Private Sequence (version {}) at \"{}\":", version, url);
                if cmd.hexdump {
                    // Render hex representation of Sequence content
                    println!("{}", pretty_hex::pretty_hex(data));
                } else {
                    // Render Sequence content
                    io::stdout()
                        .write_all(data)
                        .context("Failed to print out the content of the file")?
                }
            } else {
                println!("{}", serialise_output(&(url, data), output_fmt));
            }
        }
        SafeData::Multimap { .. }
        | SafeData::PrivateRegister { .. }
        | SafeData::PublicRegister { .. } => unimplemented!(),
    }

    Ok(())
}
