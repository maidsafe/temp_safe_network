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
        gen_processed_files_table, get_from_arg_or_stdin, get_from_stdin, if_tty, notice_dry_run,
        parse_stdin_arg, pluralize, serialise_output,
    },
    OutputFmt,
};
use ansi_term::Colour;
use prettytable::{format::FormatBuilder, Table};
use safe_api::{
    files::{FilesMap, ProcessedFiles},
    xorurl::{XorUrl, XorUrlEncoder},
    Safe,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use structopt::StructOpt;

type FileDetails = BTreeMap<String, String>;

// Differentiates between nodes in a file system.
#[derive(Debug, Serialize, PartialEq)]
enum FileTreeNodeType {
    File,
    Directory,
}

// A recursive type to represent a directory tree.
// used by `safe files tree`
#[derive(Debug, Serialize)]
struct FileTreeNode {
    name: String,

    // This field could be useful in json output, because presently json
    // consumer cannot differentiate between an empty sub-directory and
    // a file. Though also at present, SAFE does not appear to store and
    // retrieve empty subdirectories.
    #[serde(skip)]
    fs_type: FileTreeNodeType,

    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<FileDetails>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    //    #[allow(clippy::vec_box)]
    sub: Vec<FileTreeNode>,
}

impl FileTreeNode {
    // create a new FileTreeNode (either a Directory or File)
    fn new(name: &str, fs_type: FileTreeNodeType, details: Option<FileDetails>) -> FileTreeNode {
        Self {
            name: name.to_string(),
            fs_type,
            details,
            sub: Vec::<FileTreeNode>::new(),
        }
    }

    // find's a (mutable) child node matching `name`
    fn find_child(&mut self, name: &str) -> Option<&mut FileTreeNode> {
        for c in self.sub.iter_mut() {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    // adds a child node
    // warning: does not enforce unique `name` between child nodes.
    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<FileTreeNode>,
    {
        self.sub.push(leaf.into());
        self
    }
}

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
    #[structopt(name = "tree")]
    /// Recursively list files found in an existing FilesContainer on the network
    Tree {
        /// The target FilesContainer to list files from, optionally including a path (default is '/')
        target: Option<String>,
        /// Include file details
        #[structopt(short = "d", long = "details")]
        details: bool,
    },
}

pub async fn files_commander(
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
                )
                .await?;

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
                .files_container_sync(&location, &target, recursive, delete, update_nrs, dry_run)
                .await?;

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
                    safe.files_container_add_from_raw(&file_content, &target_url, force, update_nrs, dry_run).await?
                } else {
                    // Update the FilesContainer on the Network
                    safe.files_container_add(&location, &target_url, force, update_nrs, dry_run).await?
                };

            // Now let's just print out a list of the files synced/processed
            output_processed_files_list(output_fmt, processed_files, version, target_url)?;
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
            let (version, processed_files, _files_map) = safe
                .files_container_remove_path(&target_url, recursive, update_nrs, dry_run)
                .await?;

            // Now let's just print out a list of the files removed
            output_processed_files_list(output_fmt, processed_files, version, target_url)?;
            Ok(())
        }
        FilesSubCommands::Ls { target } => {
            let target_url =
                get_from_arg_or_stdin(target, Some("...awaiting target URl from STDIN"))?;

            let (version, files_map) = safe
                .files_container_get(&target_url)
                .await
                .map_err(|err| format!("Make sure the URL targets a FilesContainer.\n{}", err))?;
            let (total, filtered_filesmap) = filter_files_map(&files_map, &target_url)?;

            if OutputFmt::Pretty == output_fmt {
                print_files_map(&filtered_filesmap, total, version, &target_url);
            } else {
                println!(
                    "{}",
                    serialise_output(&(target_url, filtered_filesmap), output_fmt)
                );
            }

            Ok(())
        }
        FilesSubCommands::Tree { target, details } => {
            process_tree_command(safe, target, details, output_fmt).await
        }
    }
}

// processes the `safe files tree` command.
async fn process_tree_command(
    safe: &mut Safe,
    target: Option<XorUrl>,
    details: bool,
    output_fmt: OutputFmt,
) -> Result<(), String> {
    let target_url = get_from_arg_or_stdin(target, Some("...awaiting target URl from STDIN"))?;

    let (_version, files_map) = safe
        .files_container_get(&target_url)
        .await
        .map_err(|err| format!("Make sure the URL targets a FilesContainer.\n{}", err))?;

    let filtered_filesmap = filter_files_map_by_xorurl_path(&files_map, &target_url)?;

    // Create a top/root node representing `target_url`.
    let mut top = FileTreeNode::new(&target_url, FileTreeNodeType::Directory, Option::None);
    // Transform flat list in `files_map` to a hierarchy in `top`
    let mut files: u64 = 0;
    let mut dirs: u64 = 0;
    for (name, file_details) in filtered_filesmap.iter() {
        let path_parts: Vec<String> = name
            .to_string()
            .trim_matches('/')
            .split('/')
            .map(|s| s.to_string())
            .collect();
        let (d, f) = build_tree(&mut top, &path_parts, file_details, details, 0);
        files += f;
        dirs += d;
    }
    // Display.  with or without details.
    if OutputFmt::Pretty == output_fmt {
        if details {
            print_file_system_node_details(&top, dirs, files);
        } else {
            print_file_system_node(&top, dirs, files);
        }
    } else {
        println!("{}", serialise_output(&top, output_fmt));
    }

    Ok(())
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

fn output_processed_files_list(
    output_fmt: OutputFmt,
    processed_files: ProcessedFiles,
    version: u64,
    target_url: String,
) -> Result<(), String> {
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

// Builds a file-system tree (hierarchy) from a single file path, split into its parts.
// May be called multiple times to expand the tree.
fn build_tree(
    node: &mut FileTreeNode,
    path_parts: &[String],
    details: &FileDetails,
    show_details: bool,
    depth: usize,
) -> (u64, u64) {
    let mut dirs: u64 = 0;
    let mut files: u64 = 0;
    if depth < path_parts.len() {
        let item = &path_parts[depth];

        let mut node = match node.find_child(&item) {
            Some(n) => n,
            None => {
                // Note: fs_type assignment relies on the fact that input
                // is a path to a file, not a directory.
                let (fs_type, d, di, fi) = if depth == path_parts.len() - 1 {
                    (
                        FileTreeNodeType::File,
                        if show_details {
                            Some(details.clone())
                        } else {
                            None
                        },
                        0, // dirs increment
                        1, // files increment
                    )
                } else {
                    (FileTreeNodeType::Directory, None, 1, 0)
                };
                dirs += di;
                files += fi;
                let n = FileTreeNode::new(&item, fs_type, d);
                node.add_child(n);
                // Very gross, but it works.
                // if this can be done in a better way,
                // please show me. We just need to return the node
                // that was added via add_child().  I tried modifying
                // add_child() to return it instead of &self, but couldn't
                // get it to work.  Also, using `n` does not work.

                match node.find_child(&item) {
                    Some(n2) => n2,
                    None => panic!("But that's impossible!"),
                }
            }
        };
        let (di, fi) = build_tree(&mut node, path_parts, details, show_details, depth + 1);
        dirs += di;
        files += fi;
    }
    (dirs, files)
}

// A function to print a FileTreeNode in format similar to unix `tree` command.
// prints a summary row below the main tree body.
fn print_file_system_node(dir: &FileTreeNode, dirs: u64, files: u64) {
    let mut siblings = HashMap::new();
    print_file_system_node_body(dir, 0, &mut siblings);

    // print summary row
    println!(
        "\n{} {}, {} {}",
        dirs,
        pluralize("directory", "directories", dirs),
        files,
        pluralize("file", "files", files),
    );
}

// generates tree body for print_file_system_node()
// operates recursively on `dir`
fn print_file_system_node_body(dir: &FileTreeNode, depth: u32, siblings: &mut HashMap<u32, bool>) {
    println!("{}", format_file_system_node_line(dir, depth, siblings));

    // And now, for some recursion...
    for (idx, child) in dir.sub.iter().enumerate() {
        let is_last = idx == dir.sub.len() - 1;
        siblings.insert(depth, !is_last);
        print_file_system_node_body(child, depth + 1, siblings);
    }
}

// A function to print a FileTreeNode in format similar to unix `tree` command.
// File details are displayed in a table to the left of the tree.
// prints a summary row below the main body.
fn print_file_system_node_details(dir: &FileTreeNode, dirs: u64, files: u64) {
    let mut siblings = HashMap::new();
    let mut table = Table::new();
    let format = FormatBuilder::new()
        .column_separator(' ')
        .padding(0, 1)
        .build();
    table.set_format(format);
    table.add_row(row!["SIZE", "CREATED", "MODIFIED", "NAME"]);

    print_file_system_node_details_body(dir, 0, &mut siblings, &mut table);

    table.printstd();

    // print summary row
    println!(
        "\n{} {}, {} {}",
        dirs,
        pluralize("directory", "directories", dirs),
        files,
        pluralize("file", "files", files),
    );
}

// generates table body for print_file_system_node_details()
// operates recursively on `dir`
fn print_file_system_node_details_body(
    dir: &FileTreeNode,
    depth: u32,
    siblings: &mut HashMap<u32, bool>,
    table: &mut Table,
) {
    let name = format_file_system_node_line(dir, depth, siblings);

    match dir.fs_type {
        FileTreeNodeType::File => {
            if let Some(d) = &dir.details {
                table.add_row(row![d["size"], d["created"], d["modified"], name]);
            }
        }
        FileTreeNodeType::Directory => {
            table.add_row(row!["", "", "", name]);
        }
    }

    // And now, for some recursion...
    for (idx, child) in dir.sub.iter().enumerate() {
        let is_last = idx == dir.sub.len() - 1;
        siblings.insert(depth, !is_last);
        print_file_system_node_details_body(child, depth + 1, siblings, table);
    }
}

// Generates a single line when printing a FileTreeNode
// in unix `tree` format.
fn format_file_system_node_line(
    dir: &FileTreeNode,
    depth: u32,
    siblings: &mut HashMap<u32, bool>,
) -> String {
    if depth == 0 {
        siblings.insert(depth, false);
        if_tty(&dir.name, Colour::Blue.bold())
    } else {
        let is_last = !siblings[&(depth - 1)];
        let conn = if is_last { "└──" } else { "├──" };

        let mut buf: String = "".to_owned();
        for x in 0..depth - 1 {
            if siblings[&(x)] {
                buf.push_str("│   ");
            } else {
                buf.push_str("    ");
            }
        }
        let name = if dir.fs_type == FileTreeNodeType::Directory {
            if_tty(&dir.name, Colour::Blue.bold())
        } else {
            dir.name.clone()
        };
        format!("{}{} {}", buf, conn, name)
    }
}

// A function to print a FilesMap in human-friendly table format.
fn print_files_map(files_map: &FilesMap, total_files: u64, version: u64, target_url: &str) {
    println!(
        "Files of FilesContainer (version {}) at \"{}\":",
        version, target_url
    );
    let mut table = Table::new();
    let format = FormatBuilder::new()
        .column_separator(' ')
        .padding(0, 1)
        .build();
    table.set_format(format);
    let mut total_bytes = 0;
    let mut cwd_files = 0;
    let mut cwd_size = 0;

    // Columns in output:
    // 1. file/directory size,
    // 2. created timestamp,
    // 3. modified timestamp,
    // 4. file/directory name
    table.add_row(row!["SIZE", "CREATED", "MODIFIED", "NAME"]);
    files_map.iter().for_each(|(name, file_item)| {
        total_bytes += file_item["size"].parse().unwrap_or(0);
        if name.ends_with('/') {
            table.add_row(row![
                &file_item["size"],
                file_item["created"],
                file_item["modified"],
                Fbb->name
            ]);
        } else {
            if None == name.trim_matches('/').find('/') {
                cwd_size += file_item["size"].parse().unwrap_or(0);
                cwd_files += 1;
            }
            table.add_row(row![
                &file_item["size"],
                file_item["created"],
                file_item["modified"],
                name
            ]);
        }
    });
    println!(
        "Files: {}   Size: {}   Total Files: {}   Total Size: {}",
        cwd_files, cwd_size, total_files, total_bytes
    );
    table.printstd();
}

// filters out file items not belonging to the xorurl path
// note: maybe should be moved into api/app/files.rs
//       and optionally called by files_container_get()
//       or make a files_container_get_matching() API.
fn filter_files_map_by_xorurl_path(
    files_map: &FilesMap,
    target_url: &str,
) -> Result<FilesMap, String> {
    let xorurl_encoder = Safe::parse_url(target_url)?;
    let path = xorurl_encoder.path();

    Ok(filter_files_map_by_path(files_map, path))
}

// filters out file items not belonging to the path
// note: maybe should be moved into api/app/files.rs
fn filter_files_map_by_path(files_map: &FilesMap, path: &str) -> FilesMap {
    let mut filtered_filesmap = FilesMap::default();

    files_map.iter().for_each(|(filepath, fileitem)| {
        if filepath
            .trim_matches('/')
            .starts_with(&path.trim_matches('/'))
        {
            let mut relative_path = filepath.clone();
            relative_path.replace_range(..path.len(), "");
            filtered_filesmap.insert(relative_path, fileitem.clone());
        }
    });
    filtered_filesmap
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
        if filepath
            .trim_matches('/')
            .starts_with(&folder_path.trim_matches('/'))
        {
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
