// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::{debug, info};
use prettytable::{format::FormatBuilder, Table};
use safe_cli::{Safe, XorUrl};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

const FILE_ADDED_SIGN: &str = "+";

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
        /// The recursively upload folders and files found in the source location
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
    },
    #[structopt(name = "sync")]
    /// Sync files to the network
    Sync {
        /// The soure location
        location: String,
        /// The recursively upload folders?
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
    },
}

pub fn files_commander(
    cmd: Option<FilesSubCommands>,
    pretty: bool,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(FilesSubCommands::Put {
            location,
            recursive,
        }) => {
            let path = Path::new(&location);
            info!("Reading files from {}", &path.display());
            let metadata =
                fs::metadata(&path).map_err(|_| "Couldn't read metadata from source path")?;

            debug!("Metadata for location: {:?}", metadata);

            // TODO: Enable source for funds / ownership
            // Warn about ownership?
            let content_map = if recursive {
                upload_dir_contents(safe, &path)?
            } else {
                if metadata.is_dir() {
                    return Err(format!(
                        "{:?} is a directory. Use \"-r\" to recursively upload folders.",
                        &location
                    ));
                }
                let xorurl = upload_file(safe, &path)?;
                let mut content_map = BTreeMap::new();
                content_map.insert(location, xorurl);
                content_map
            };

            // create FilesContainer with the content of content_map
            let serialised_files_map = safe.files_map_create(&content_map)?;
            let files_container_xorurl =
                safe.files_container_create(serialised_files_map.as_bytes().to_vec())?;

            if pretty {
                println!("FilesContainer created at: \"{}\"", files_container_xorurl);
                let mut table = Table::new();
                let format = FormatBuilder::new()
                    .column_separator(' ')
                    .padding(0, 1)
                    .build();
                table.set_format(format);
                for (key, value) in content_map.iter() {
                    table.add_row(row![FILE_ADDED_SIGN, key, value]);
                }
                table.printstd();
            } else {
                println!("[\"{}\", {:?}]", files_container_xorurl, &content_map);
            }

            Ok(())
        }
        Some(FilesSubCommands::Sync { .. }) => {
            // TODO: pull a given dir / file.
            // Get metadatas.
            // Check dates / sizes.
            // if newer upload new.
            // update FilesMap
            Ok(())
        }
        None => Err("Missing keys sub-command. Use --help for details.".to_string()),
    }
}

// TODO: Decide at what point does this functinality go into our lib/apis?

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with('.'))
        .unwrap_or(false)
}

fn upload_dir_contents(safe: &mut Safe, path: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut content_map = BTreeMap::new();

    // TODO: option to enable following symlinks and hidden files?
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .for_each(|child| {
            info!("{}", child.path().display());
            let the_path = child.path();
            let the_path_str = the_path.to_str().unwrap_or_else(|| "").to_string();
            match fs::metadata(&the_path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        // Everything is in the iter. We dont need to recurse.
                        // so what do we do with dirs? decide if we want to support empty dirs also
                    } else {
                        match upload_file(safe, &the_path) {
                            Ok(xorurl) => {
                                content_map.insert(the_path_str, xorurl);
                            }
                            Err(err) => eprintln!(
                                "Skipping file \"{}\" since it couldn't be uploaded to the network: {:?}",
                                the_path_str, err
                            ),
                        };
                    }
                },
                Err(err) => eprintln!(
                    "Skipping file \"{}\" since no metadata could be read from local location: {:?}",
                    the_path_str, err
                )
            }
        });

    Ok(content_map)
}

fn upload_file(safe: &mut Safe, path: &Path) -> Result<XorUrl, String> {
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(e) => return Err(format!("{}", e)),
    };
    safe.files_put_published_immutable(&data)
}
