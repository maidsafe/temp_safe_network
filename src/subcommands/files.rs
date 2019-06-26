// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::{debug, error, info};
use prettytable::{format::FormatBuilder, Table};
use safe_cli::{ContentMap, Safe, XorUrl};
use std::fs;
use std::path::Path;
use structopt::StructOpt;
use unwrap::unwrap;

use walkdir::{DirEntry, WalkDir};

static FILE_ADDED_SIGN: &str = "+";

// TODO: Decide at what point does this functinality go into our lib/apis?

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}

fn upload_dir_contents(path: &Path, safe: &mut Safe) -> Result<ContentMap, String> {
    let mut content_map: ContentMap = Default::default();

    // TODO: option to enable following symlinks?
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .for_each(|child| {
            info!("{}", child.path().display());
            let the_path = child.path();
            let metadata = unwrap!(fs::metadata(&the_path));
            if metadata.is_dir() {
                // Everything is in the iter. We dont need to recurse.
                // so what do we do with dirs?

            } else {
                let xorurl = unwrap!(upload_file(&the_path, safe));

                content_map.insert(unwrap!(the_path.to_str()).to_string(), xorurl);
            }
        });
    Ok(content_map)
}

fn upload_file(path: &Path, safe: &mut Safe) -> Result<XorUrl, String> {
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(e) => return Err(format!("{}", e)),
    };

    // TODO: For each file:
    // add metadata....?
    // add FilesMap style thing.
    // Do we create ONE per file?

    safe.put_published_immutable(&data)
}

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
            let mut content_map: ContentMap = Default::default();

            // TODO: Enable source for funds / ownership
            // Warn about ownership?
            if recursive {
                content_map = upload_dir_contents(&path, safe)?;
            } else {
                if metadata.is_dir() {
                    return Err(format!(
                        "{:?} is a directory. Use \"-r\" to recursively upload folders.",
                        &location
                    ));
                }

                let xorurl = upload_file(&path, safe)?;

                content_map.insert(location, xorurl);
            }

            // TODO: create FilesContainer with the content of content_map
            let files_container_xorurl = "safe://<FilesContainer XOR-URL>".to_string();

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
        Some(FilesSubCommands::Sync {
            location,
            recursive,
        }) => {
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
