// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{metadata::get_metadata, ProcessedFiles};
use crate::{app::consts::*, Error, Result, Safe, XorUrl};
use log::info;
use std::{collections::BTreeMap, fs, path::Path};
use walkdir::{DirEntry, WalkDir};

const MAX_RECURSIVE_DEPTH: usize = 10_000;

// Upload a files to the Network as a Public Blob
pub(crate) async fn upload_file_to_net(
    safe: &mut Safe,
    path: &Path,
    dry_run: bool,
) -> Result<XorUrl> {
    let data = fs::read(path).map_err(|err| {
        Error::InvalidInput(format!("Failed to read file from local location: {}", err))
    })?;

    let mime_type = mime_guess::from_path(&path);
    match safe
        .files_store_public_blob(&data, mime_type.first_raw(), dry_run)
        .await
    {
        Ok(xorurl) => Ok(xorurl),
        Err(err) => {
            // Let's then upload it and set media-type to be simply raw content
            if let Error::InvalidMediaType(_) = err {
                safe.files_store_public_blob(&data, None, dry_run).await
            } else {
                Err(err)
            }
        }
    }
}

// Simply change Windows style path separator into `/`
pub(crate) fn normalise_path_separator(from: &str) -> String {
    str::replace(from, "\\", "/")
}

// Walk the local filesystem starting from `location`, creating a list of files paths,
// and if not requested as a `dry_run` upload the files to the network filling up
// the list of files with their corresponding XOR-URLs
pub(crate) async fn file_system_dir_walk(
    safe: &mut Safe,
    location: &str,
    recursive: bool,
    follow_links: bool,
    dry_run: bool,
) -> Result<ProcessedFiles> {
    let file_path = Path::new(location);
    info!("Reading files from {}", file_path.display());
    let (metadata, _) = get_metadata(file_path, follow_links)?;
    if metadata.is_dir() || !recursive {
        // TODO: option to enable following symlinks?
        // We now compare both FilesMaps to upload the missing files
        let max_depth = if recursive { MAX_RECURSIVE_DEPTH } else { 1 };
        let mut processed_files = BTreeMap::new();
        let children_to_process = WalkDir::new(file_path)
            .follow_links(follow_links)
            .into_iter()
            .filter_entry(|e| valid_depth(e, max_depth))
            .filter_map(|v| v.ok());

        for (idx, child) in children_to_process.enumerate() {
            let current_file_path = child.path();
            let current_path_str = current_file_path.to_str().unwrap_or("").to_string();
            info!("Processing {}...", current_path_str);
            let normalised_path = normalise_path_separator(&current_path_str);

            let result = get_metadata(current_file_path, follow_links);
            match result {
                Ok((metadata, _)) => {
                    if metadata.file_type().is_dir() {
                        if idx == 0 && normalised_path.ends_with('/') {
                            // If the first directory ends with '/' then it is
                            // the root, and we are only interested in the children,
                            // so we skip it.
                            continue;
                        }
                        if !recursive {
                            // We do not include sub-dirs unless recursing.
                            continue;
                        }
                        // Everything is in the iter. We dont need to recurse.
                        //
                        // so what do we do with dirs? We don't upload them as immutable data.
                        // They are only a type of metadata in the FileContainer.
                        // Empty dirs are not reflected in the paths of uploaded files.
                        // We include dirs with an empty xorurl.
                        // Callers can inspect the file's metadata.
                        processed_files.insert(
                            normalised_path.clone(),
                            (CONTENT_ADDED_SIGN.to_string(), String::default()),
                        );
                    }
                    if metadata.file_type().is_symlink() {
                        processed_files.insert(
                            normalised_path.clone(),
                            (CONTENT_ADDED_SIGN.to_string(), String::default()),
                        );
                    }
                    if metadata.file_type().is_file() {
                        match upload_file_to_net(safe, current_file_path, dry_run).await {
                            Ok(xorurl) => {
                                processed_files.insert(
                                    normalised_path,
                                    (CONTENT_ADDED_SIGN.to_string(), xorurl),
                                );
                            }
                            Err(err) => {
                                processed_files.insert(
                                    normalised_path.clone(),
                                    (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)),
                                );
                                info!("Skipping file \"{}\". {}", normalised_path, err);
                            }
                        }
                    }
                }
                Err(err) => {
                    processed_files.insert(
                        normalised_path.clone(),
                        (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)),
                    );
                    info!(
                        "Skipping file \"{}\" since no metadata could be read from local location: {:?}",
                        normalised_path, err);
                }
            }
        }

        Ok(processed_files)
    } else {
        // Recursive only works on a dir path. Let's error as the user may be making a mistake
        // so it's better for the user to double check and either provide the correct path
        // or remove the 'recursive' flag from the args
        Err(Error::InvalidInput(format!(
            "'{}' is not a directory. The \"recursive\" arg is only supported for folders.",
            location
        )))
    }
}

// Checks if the depth in the dir hierarchy is under a threshold
fn valid_depth(entry: &DirEntry, max_depth: usize) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|_| entry.depth() <= max_depth)
        .unwrap_or(false)
}

// Read the local filesystem at `location`, creating a list of one single file's path,
// and if not as a `dry_run` upload the file to the network and putting
// the obtained XOR-URL in the single file list returned
pub(crate) async fn file_system_single_file(
    safe: &mut Safe,
    location: &str,
    dry_run: bool,
) -> Result<ProcessedFiles> {
    let file_path = Path::new(location);
    info!("Reading file {}", file_path.display());
    let (metadata, _) = get_metadata(file_path, true)?; // follows symlinks.

    // We now compare both FilesMaps to upload the missing files
    let mut processed_files = BTreeMap::new();
    let normalised_path = normalise_path_separator(file_path.to_str().unwrap_or(""));
    if metadata.is_dir() {
        Err(Error::InvalidInput(format!(
            "'{}' is a directory, only individual files can be added. Use files sync operation for uploading folders",
            location
        )))
    } else {
        match upload_file_to_net(safe, file_path, dry_run).await {
            Ok(xorurl) => {
                processed_files.insert(normalised_path, (CONTENT_ADDED_SIGN.to_string(), xorurl));
            }
            Err(err) => {
                processed_files.insert(
                    normalised_path.clone(),
                    (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)),
                );
                info!("Skipping file \"{}\". {}", normalised_path, err);
            }
        };
        Ok(processed_files)
    }
}
