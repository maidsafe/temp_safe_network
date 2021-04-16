// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    file_system::{normalise_path_separator, upload_file_to_net},
    metadata::FileMeta,
    ProcessedFiles,
};
use crate::{api::app::consts::*, Error, Result, Safe};
use log::{debug, info};
use std::{collections::BTreeMap, fs, path::Path};

// To use for mapping files names (with path in a flattened hierarchy) to FileItems
pub type FilesMap = BTreeMap<String, FileItem>;

// Each FileItem contains file metadata and the link to the file's Blob XOR-URL
pub type FileItem = BTreeMap<String, String>;

// A trait to get an key attr and return an API Result
pub trait GetAttr {
    fn getattr(&self, key: &str) -> Result<&str>;
}

impl GetAttr for FileItem {
    // Makes it more readable to conditionally get an attribute from a FileItem
    // because we can call it in API funcs like fileitem.getattr("key")?;
    fn getattr(&self, key: &str) -> Result<&str> {
        match self.get(key) {
            Some(v) => Ok(v),
            None => Err(Error::EntryNotFound(format!("key not found: {}", key))),
        }
    }
}

// Helper function to add or update a FileItem in a FilesMap
#[allow(clippy::too_many_arguments)]
pub(crate) async fn add_or_update_file_item(
    safe: &mut Safe,
    file_name: &str,
    file_name_for_map: &str,
    file_path: &Path,
    file_meta: &FileMeta,
    file_link: Option<&str>,
    name_exists: bool,
    dry_run: bool,
    files_map: &mut FilesMap,
    processed_files: &mut ProcessedFiles,
) -> bool {
    // We need to add a new FileItem, let's generate the FileItem first
    match gen_new_file_item(safe, file_path, file_meta, file_link, dry_run).await {
        Ok(new_file_item) => {
            let content_added_sign = if name_exists {
                CONTENT_UPDATED_SIGN.to_string()
            } else {
                CONTENT_ADDED_SIGN.to_string()
            };

            debug!("New FileItem item: {:?}", new_file_item);
            debug!("New FileItem item inserted as {:?}", file_name);
            files_map.insert(file_name_for_map.to_string(), new_file_item.clone());

            processed_files.insert(
                file_name.to_string(),
                (
                    content_added_sign,
                    // note: files have link property,
                    //       dirs and symlinks do not
                    new_file_item
                        .get(PREDICATE_LINK)
                        .unwrap_or(&String::default())
                        .to_string(),
                ),
            );

            true
        }
        Err(err) => {
            processed_files.insert(
                file_name.to_string(),
                (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)),
            );
            info!("Skipping file \"{}\": {:?}", file_link.unwrap_or(""), err);

            false
        }
    }
}

// Generate a FileItem for a file which can then be added to a FilesMap
// This is now a pseudo-RDF but will eventually be converted to be an RDF graph
async fn gen_new_file_item(
    safe: &mut Safe,
    file_path: &Path,
    file_meta: &FileMeta,
    link: Option<&str>, // must be symlink target or None if FileMeta::is_symlink() is true.
    dry_run: bool,
) -> Result<FileItem> {
    let mut file_item = file_meta.to_file_item();
    if file_meta.is_file() {
        let xorurl = match link {
            None => upload_file_to_net(safe, file_path, dry_run).await?,
            Some(link) => link.to_string(),
        };
        file_item.insert(PREDICATE_LINK.to_string(), xorurl);
    } else if file_meta.is_symlink() {
        // get metadata, with any symlinks resolved.
        let result = fs::metadata(&file_path);
        let symlink_target_type = match result {
            Ok(meta) => {
                if meta.is_dir() {
                    "dir"
                } else {
                    "file"
                }
            }
            Err(_) => "unknown", // this occurs for a broken link.  on windows, this would be fixed by: https://github.com/rust-lang/rust/pull/47956
                                 // on unix, there is no way to know if broken link points to file or dir, though we could guess, based on if it has an extension or not.
        };
        let target_path = match link {
            Some(target) => target.to_string(),
            None => {
                let target_path = fs::read_link(&file_path).map_err(|e| {
                    Error::FileSystemError(format!(
                        "Unable to read link: {}.  {:#?}",
                        file_path.display(),
                        e
                    ))
                })?;
                normalise_path_separator(&target_path.display().to_string())
            }
        };
        file_item.insert("symlink_target".to_string(), target_path);
        // This is a hint for windows-platform clients to be able to call
        //   symlink_dir() or symlink_file().  on unix, there's no need.
        file_item.insert(
            "symlink_target_type".to_string(),
            symlink_target_type.to_string(),
        );
    }

    Ok(file_item)
}
