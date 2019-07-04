// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::get_target_location;
use super::OutputFmt;
use prettytable::{format::FormatBuilder, Table};
use safe_cli::Safe;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum FilesSubCommands {
    // #[structopt(name = "add")]
    /// Add a file to a network document / container
    // Add {
    //     /// The soure file location
    //     #[structopt(short = "s", long = "source")]
    //     source: String,
    //     /// desired file name
    //     #[structopt(long = "name")]
    //     name: String,
    //     /// desired file name
    //     #[structopt(short = "l", long = "link")]
    //     link: String,
    // },
    #[structopt(name = "put")]
    /// Put a file or folder's files onto the network
    Put {
        /// The source file/folder local path
        location: String,
        /// Recursively upload folders and files found in the source location
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
        #[structopt(long = "set-root")]
        set_root: Option<String>,
    },
    #[structopt(name = "sync")]
    /// Sync files to the network
    Sync {
        /// The soure location
        location: String,
        /// The target FilesContainer to sync up source files with
        target: Option<String>,
        /// Recursively sync folders and files found in the source location
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
        #[structopt(long = "set-root")]
        set_root: Option<String>,
        /// Delete files found at the target FilesContainer that are not in the source location
        #[structopt(short = "d", long = "delete")]
        delete: bool,
    },
}

pub fn files_commander(
    cmd: Option<FilesSubCommands>,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(FilesSubCommands::Put {
            location,
            recursive,
            set_root,
        }) => {
            // create FilesContainer from a given path to local files/folders
            let (files_container_xorurl, content_map) =
                safe.files_container_create(&location, recursive, set_root)?;

            // Now let's just print out the content of the FilesMap
            if OutputFmt::Pretty == output_fmt {
                println!("FilesContainer created at: \"{}\"", files_container_xorurl);
                let mut table = Table::new();
                let format = FormatBuilder::new()
                    .column_separator(' ')
                    .padding(0, 1)
                    .build();
                table.set_format(format);
                for (file_name, (change, link)) in content_map.iter() {
                    table.add_row(row![change, file_name, link]);
                }
                table.printstd();
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&(files_container_xorurl, content_map))
                        .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
                );
            }

            Ok(())
        }
        Some(FilesSubCommands::Sync {
            location,
            target,
            recursive,
            set_root,
            delete,
        }) => {
            let target = get_target_location(target)?;

            // Update the FilesContainer on the Network
            let (version, content_map) =
                safe.files_container_sync(&location, &target, recursive, set_root, delete)?;

            // Now let's just print out the content of the FilesMap
            if OutputFmt::Pretty == output_fmt {
                if content_map.is_empty() {
                    println!("No changes were required, source location is already in sync with FilesContainer (version {}) at: \"{}\"", version, target);
                } else {
                    println!(
                        "FilesContainer synced up (version {}): \"{}\"",
                        version, target
                    );
                    let mut table = Table::new();
                    let format = FormatBuilder::new()
                        .column_separator(' ')
                        .padding(0, 1)
                        .build();
                    table.set_format(format);
                    for (file_name, (change, link)) in content_map.iter() {
                        table.add_row(row![change, file_name, link]);
                    }
                    table.printstd();
                }
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&(target, content_map))
                        .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
                );
            }
            Ok(())
        }
        None => Err("Missing keys sub-command. Use --help for details.".to_string()),
    }
}
