// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{
    gen_processed_files_table, get_from_arg_or_stdin, get_from_stdin, notice_dry_run,
    parse_stdin_arg, serialise_output,
};
use super::OutputFmt;
use prettytable::{format::FormatBuilder, Table};
use safe_api::{Safe, XorUrl, XorUrlEncoder};
use std::collections::BTreeMap;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum FilesSubCommands {
    #[structopt(name = "put")]
    /// Put a file or folder's files onto the SAFE Network
    Put {
        /// The source file/folder local path
        location: String,
        /// The destination path (in the FilesContainer) for the uploaded files and folders (default is '/')
        dest: Option<String>,
        /// Recursively upload folders and files found in the source location
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
    },
    #[structopt(name = "sync")]
    /// Sync files to the SAFE Network
    Sync {
        /// The source location
        location: String,
        /// The target FilesContainer to sync up source files with, optionally including the destination path (default is '/')
        target: Option<String>,
        /// Recursively sync folders and files found in the source location
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
        /// Delete files found at the target FilesContainer that are not in the source location. This is only allowed when --recursive is passed as well
        #[structopt(short = "d", long = "delete")]
        delete: bool,
        /// Automatically update the NRS name to link to the new version of the FilesContainer. This is only allowed if an NRS URL was provided, and if the NRS name is currently linked to a specific version of the FilesContainer
        #[structopt(short = "u", long = "update-nrs")]
        update_nrs: bool,
    },
    #[structopt(name = "add")]
    /// Add a file to an existing FilesContainer on the network
    Add {
        /// The source file location.  Specify '-' to read from stdin
        #[structopt(
            parse(from_str = parse_stdin_arg),
            requires_if("", "target"),
            requires_if("-", "target")
        )]
        location: String,
        /// The target FilesContainer to add the source file to, optionally including the destination path (default is '/') and new file name
        #[structopt(parse(from_str = parse_stdin_arg))]
        target: Option<String>,
        /// Automatically update the NRS name to link to the new version of the FilesContainer. This is only allowed if an NRS URL was provided, and if the NRS name is currently linked to a specific version of the FilesContainer
        #[structopt(short = "u", long = "update-nrs")]
        update_nrs: bool,
        /// Overwrite the file on the FilesContainer if there already exists a file with the same name
        #[structopt(short = "f", long = "force")]
        force: bool,
    },
}

pub fn files_commander(
    cmd: FilesSubCommands,
    output_fmt: OutputFmt,
    dry_run: bool,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        FilesSubCommands::Put {
            location,
            dest,
            recursive,
        } => {
            // create FilesContainer from a given path to local files/folders
            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }
            let (files_container_xorurl, processed_files, _files_map) = safe
                .files_container_create(
                    &location,
                    dest.as_ref().map(String::as_str),
                    recursive,
                    dry_run,
                )?;

            // Now let's just print out a list of the files uploaded/processed
            if OutputFmt::Pretty == output_fmt {
                if dry_run {
                    println!("FilesContainer not created since running in dry-run mode");
                } else {
                    println!("FilesContainer created at: \"{}\"", files_container_xorurl);
                }
                let mut table = Table::new();
                let format = FormatBuilder::new()
                    .column_separator(' ')
                    .padding(0, 1)
                    .build();
                table.set_format(format);
                for (file_name, (change, link)) in processed_files.iter() {
                    table.add_row(row![change, file_name, link]);
                }
                table.printstd();
            } else {
                print_serialized_output(files_container_xorurl, 0, processed_files, output_fmt)?;
            }

            Ok(())
        }
        FilesSubCommands::Sync {
            location,
            target,
            recursive,
            delete,
            update_nrs,
        } => {
            let target = get_from_arg_or_stdin(target, None)?;
            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }
            // Update the FilesContainer on the Network
            let (version, processed_files, _files_map) = safe
                .files_container_sync(&location, &target, recursive, delete, update_nrs, dry_run)?;

            // Now let's just print out a list of the files synced/processed
            if OutputFmt::Pretty == output_fmt {
                let (table, success_count) = gen_processed_files_table(&processed_files, true);
                if success_count > 0 {
                    let url = match XorUrlEncoder::from_url(&target) {
                        Ok(mut xorurl_encoder) => {
                            xorurl_encoder.set_content_version(Some(version));
                            xorurl_encoder.set_path("");
                            xorurl_encoder.to_string()?
                        }
                        Err(_) => target,
                    };

                    println!(
                        "FilesContainer synced up (version {}): \"{}\"",
                        version, url
                    );
                    table.printstd();
                } else if !processed_files.is_empty() {
                    println!(
                        "No changes were made to FilesContainer (version {}) at \"{}\"",
                        version, target
                    );
                    table.printstd();
                } else {
                    println!("No changes were required, source location is already in sync with FilesContainer (version {}) at: \"{}\"", version, target);
                }
            } else {
                print_serialized_output(target, version, processed_files, output_fmt)?;
            }
            Ok(())
        }
        FilesSubCommands::Add {
            location,
            target,
            update_nrs,
            force,
        } => {
            // Validate that location and target are not both "", ie stdin.
            let target_url = target.unwrap_or_else(|| "".to_string());
            if target_url.is_empty() && location.is_empty() {
                return Err("Cannot read both <location> and <target> from stdin.".to_string());
            }

            let target_url =
                get_from_arg_or_stdin(Some(target_url), Some("...awaiting target URl from STDIN"))?;
            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }

            let (version, processed_files, _files_map) =
                // If location is empty then we read arg from STDIN, which can still be a safe:// URL
                if location.is_empty() {
                    let file_content = get_from_stdin(Some("...awaiting file's content to add from STDIN"))?;
                    // Update the FilesContainer on the Network
                    safe.files_container_add_from_raw(&file_content, &target_url, force, update_nrs, dry_run)?
                } else {
                    // Update the FilesContainer on the Network
                    safe.files_container_add(&location, &target_url, force, update_nrs, dry_run)?
                };

            // Now let's just print out a list of the files synced/processed
            if OutputFmt::Pretty == output_fmt {
                let (table, success_count) = gen_processed_files_table(&processed_files, true);
                if success_count > 0 {
                    let url = match XorUrlEncoder::from_url(&target_url) {
                        Ok(mut xorurl_encoder) => {
                            xorurl_encoder.set_content_version(Some(version));
                            xorurl_encoder.set_path("");
                            xorurl_encoder.to_string()?
                        }
                        Err(_) => target_url,
                    };

                    println!("FilesContainer updated (version {}): \"{}\"", version, url);
                    table.printstd();
                } else if !processed_files.is_empty() {
                    println!(
                        "No changes were made to FilesContainer (version {}) at \"{}\"",
                        version, target_url
                    );
                    table.printstd();
                } else {
                    println!(
                        "No changes were made to the FilesContainer (version {}) at: \"{}\"",
                        version, target_url
                    );
                }
            } else {
                print_serialized_output(target_url, version, processed_files, output_fmt)?;
            }
            Ok(())
        }
    }
}

fn print_serialized_output(
    xorurl: XorUrl,
    version: u64,
    processed_files: BTreeMap<String, (String, String)>,
    output_fmt: OutputFmt,
) -> Result<(), String> {
    let url = match XorUrlEncoder::from_url(&xorurl) {
        Ok(mut xorurl_encoder) => {
            xorurl_encoder.set_content_version(Some(version));
            xorurl_encoder.to_string()?
        }
        Err(_) => xorurl,
    };
    println!("{}", serialise_output(&(url, processed_files), output_fmt));

    Ok(())
}
