// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    helpers::{get_from_arg_or_stdin, get_target_url, print_nrs_map, serialise_output},
    OutputFmt,
};
use color_eyre::{eyre::WrapErr, Result};
use prettytable::Table;
use sn_api::{resolver::SafeData, Safe};
use std::io::{self, Write};
use structopt::StructOpt;
use tokio::time::{sleep, Duration};
// use tracing::{debug, trace};

const MAX_RETRY_ATTEMPTS: usize = 5;

#[derive(StructOpt, Debug)]
pub struct CatCommands {
    /// The safe:// location to retrieve
    location: Option<String>,
    /// Renders file output as hex
    #[structopt(short = "x", long = "hexdump")]
    hexdump: bool,
}

pub async fn cat_commander(cmd: CatCommands, output_fmt: OutputFmt, safe: &Safe) -> Result<()> {
    let link = get_from_arg_or_stdin(cmd.location, None)?;
    let url = get_target_url(&link)?;
    // debug!("Running cat for: {}", &url.to_string());

    let mut attempts = 0;

    let mut content = safe.fetch(&url.to_string(), None).await;

    while content.is_err() && attempts < MAX_RETRY_ATTEMPTS {
        // trace!("cat attempt #{:?}", attempts);
        sleep(Duration::from_secs(1)).await;
        content = safe.fetch(&url.to_string(), None).await;

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
                    "Files of FilesContainer ({}) at \"{}\":",
                    version.map_or("empty".to_string(), |v| format!("version {}", v)),
                    url
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
                println!(
                    "{}",
                    serialise_output(&(url.to_string(), files_map), output_fmt)
                );
            }
        }
        SafeData::PublicFile { data, .. } => {
            if cmd.hexdump {
                // Render hex representation of file
                println!("{}", pretty_hex::pretty_hex(data));
            } else {
                // Render file
                io::stdout()
                    .write_all(data)
                    .context("Failed to print out the content of the file")?;
            }
        }
        SafeData::NrsMapContainer { nrs_map, .. } => {
            if OutputFmt::Pretty == output_fmt {
                println!("NRS Map Container at {}", url);
                print_nrs_map(nrs_map);
            } else {
                println!(
                    "{}",
                    serialise_output(&(url.to_string(), nrs_map), output_fmt)
                );
            }
        }
        SafeData::SafeKey { .. } => {
            println!("No content to show since the URL targets a SafeKey. Use the 'dog' command to obtain additional information about the targeted SafeKey.");
        }
        SafeData::Multimap { .. }
        | SafeData::NrsEntry { .. }
        | SafeData::PrivateRegister { .. }
        | SafeData::PublicRegister { .. } => unimplemented!(),
    }

    Ok(())
}
