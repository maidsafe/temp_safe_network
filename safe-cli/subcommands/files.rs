// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{
        gen_processed_files_table, get_from_arg_or_stdin, get_from_stdin, notice_dry_run,
        parse_stdin_arg, serialise_output,
    },
    OutputFmt,
};
use prettytable::{format::FormatBuilder, Table};
use safe_api::{
    files::FilesMap,
    xorurl::{XorUrl, XorUrlEncoder},
    Safe,
};
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
    #[structopt(name = "rm")]
    /// Remove a file from an existing FilesContainer on the network
    Rm {
        /// The full URL of the file to remove from its FilesContainer
        target: String,
        /// Automatically update the NRS name to link to the new version of the FilesContainer. This is only allowed if an NRS URL was provided, and if the NRS name is currently linked to a specific version of the FilesContainer
        #[structopt(short = "u", long = "update-nrs")]
        update_nrs: bool,
        /// Recursively remove files found in the target path
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
    },
    #[structopt(name = "ls")]
    /// List files found in an existing FilesContainer on the network
    Ls {
        /// The target FilesContainer to list files from, optionally including a path (default is '/')
        target: Option<String>,
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
        FilesSubCommands::Rm {
            target,
            update_nrs,
            recursive,
        } => {
            let target_url =
                get_from_arg_or_stdin(Some(target), Some("...awaiting target URl from STDIN"))?;

            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }

            // Update the FilesContainer on the Network
            let (version, processed_files, _files_map) =
                safe.files_container_remove_path(&target_url, recursive, update_nrs, dry_run)?;

            // Now let's just print out a list of the files removed
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
        FilesSubCommands::Ls { target } => {
            let target_url =
                get_from_arg_or_stdin(target, Some("...awaiting target URl from STDIN"))?;

            let (version, files_map) = safe
                .files_container_get(&target_url)
                .map_err(|err| format!("Make sure the URL targets a FilesContainer.\n{}", err))?;

            let (total, filtered_filesmap) = filter_files_map(&files_map, &target_url)?;

            // Render FilesContainer
            if OutputFmt::Pretty == output_fmt {
                println!(
                    "Files of FilesContainer (version {}) at \"{}\":",
                    version, target_url
                );
                println!("Total: {}", total);
                let mut table = Table::new();
                let format = FormatBuilder::new()
                    .column_separator(' ')
                    .padding(0, 1)
                    .build();
                table.set_format(format);

                // Columns in output:
                // 1. file/directory size,
                // 2. created timestamp,
                // 3. modified timestamp,
                // 4. file/directory name
                table.add_row(row!["SIZE", "CREATED", "MODIFIED", "NAME"]);
                filtered_filesmap.iter().for_each(|(name, file_item)| {
                    if name.ends_with('/') {
                        table.add_row(row![
                            &file_item["size"],
                            file_item["created"],
                            file_item["modified"],
                            Fbb->name
                        ]);
                    } else {
                        table.add_row(row![
                            &file_item["size"],
                            file_item["created"],
                            file_item["modified"],
                            name
                        ]);
                    }
                });
                table.printstd();
            } else {
                println!(
                    "{}",
                    serialise_output(&(target_url, filtered_filesmap), output_fmt)
                );
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

fn filter_files_map(files_map: &FilesMap, target_url: &str) -> Result<(u64, FilesMap), String> {
    let mut filtered_filesmap = FilesMap::default();
    let mut xorurl_encoder = Safe::parse_url(target_url)?;
    let path = xorurl_encoder.path();

    let folder_path = if !path.ends_with('/') {
        format!("{}/", path)
    } else {
        path.to_string()
    };

    let mut total = 0;
    files_map.iter().for_each(|(filepath, fileitem)| {
        // let's first filter out file items not belonging to the provided path
        if filepath.starts_with(&folder_path) {
            total += 1;
            let mut relative_path = filepath.clone();
            relative_path.replace_range(..folder_path.len(), "");
            let subdirs = relative_path.split('/').collect::<Vec<&str>>();
            if !subdirs.is_empty() {
                // let's get base path of current file item
                let mut is_folder = false;
                let base_path = if subdirs.len() > 1 {
                    is_folder = true;
                    format!("{}/", subdirs[0])
                } else {
                    subdirs[0].to_string()
                };

                // insert or merge current file item into the filtered list
                match filtered_filesmap.get_mut(&base_path) {
                    None => {
                        let mut fileitem = fileitem.clone();
                        if is_folder {
                            // then set link to xorurl with path current subfolder
                            let subfolder_path = format!("{}{}", folder_path, subdirs[0]);
                            xorurl_encoder.set_path(&subfolder_path);
                            let link = xorurl_encoder
                                .to_string()
                                .unwrap_or_else(|_| subfolder_path);
                            fileitem.insert("link".to_string(), link);
                            fileitem.insert("type".to_string(), "".to_string());
                        }

                        filtered_filesmap.insert(base_path.to_string(), fileitem);
                    }
                    Some(item) => {
                        // current file item belongs to same base path as other files,
                        // we need to merge them together into the filtered list

                        // Add up files sizes
                        let current_dir_size = (*item["size"]).parse::<u32>().unwrap_or_else(|_| 0);
                        let additional_dir_size =
                            fileitem["size"].parse::<u32>().unwrap_or_else(|_| 0);
                        (*item).insert(
                            "size".to_string(),
                            format!("{}", current_dir_size + additional_dir_size),
                        );

                        // If current file item's modified date is more recent
                        // set it as the folder's modififed date
                        if fileitem["modified"] > item["modified"] {
                            (*item).insert("modified".to_string(), fileitem["modified"].clone());
                        }

                        // If current file item's creation date is older than others
                        // set it as the folder's created date
                        if fileitem["created"] > item["created"] {
                            (*item).insert("created".to_string(), fileitem["created"].clone());
                        }
                    }
                }
            }
        }
    });

    Ok((total, filtered_filesmap))
}
