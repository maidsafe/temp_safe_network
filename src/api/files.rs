// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::xorurl::SafeContentType;
use super::{Safe, XorUrl, XorUrlEncoder};
use chrono::Utc;
use common_path::common_path_all;
use log::{debug, info};
use relative_path::RelativePath;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

// Each FileItem contains file metadata and the link to the file's ImmutableData XOR-URL
pub type FileItem = BTreeMap<String, String>;

// To use for mapping files names (with path in a flattened hierarchy) to FileItems
pub type FilesMap = BTreeMap<String, FileItem>;

// List of files uploaded with details if they were added, updated or deleted from FilesContainer
type ContentMap = BTreeMap<String, (String, String)>;

// Type tag to use for the FilesContainer stored on AppendOnlyData
const FILES_CONTAINER_TYPE_TAG: u64 = 10_100;

const FILE_ADDED_SIGN: &str = "+";
const FILE_UPDATED_SIGN: &str = "*";
const FILE_DELETED_SIGN: &str = "-";

#[allow(dead_code)]
impl Safe {
    /// # Create versioned data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::new("base32".to_string());
    /// let (xor_url, _) = safe.files_container_create("tests/testfolder", true, None).unwrap();
    /// assert!(xor_url.contains("safe://"))
    /// ```
    pub fn files_container_create(
        &mut self,
        location: &str,
        recursive: bool,
        set_root: Option<String>,
    ) -> Result<(XorUrl, ContentMap), String> {
        // TODO: Enable source for funds / ownership
        // Warn about ownership?
        let content_map = upload_dir_contents(self, location, recursive)?;

        // The FilesContainer is created as a AppendOnlyData with a single entry containing the
        // timestamp as the entry's key, and the serialised FilesMap as the entry's value
        // TODO: use RDF format
        let files_map = files_map_create(&content_map, set_root)?;
        let serialised_files_map = serde_json::to_string(&files_map)
            .map_err(|err| format!("Couldn't serialise the FilesMap generated: {:?}", err))?;
        let now = Utc::now().to_string().to_string();
        let files_container_data = vec![(
            now.into_bytes().to_vec(),
            serialised_files_map.as_bytes().to_vec(),
        )];

        // Store the FilesContainer in a Published AppendOnlyData
        let xorname = self.safe_app.put_seq_appendable_data(
            files_container_data,
            None,
            FILES_CONTAINER_TYPE_TAG,
            None,
        )?;

        let xorurl = XorUrlEncoder::encode(
            xorname,
            FILES_CONTAINER_TYPE_TAG,
            SafeContentType::FilesContainer,
            &self.xorurl_base,
        )?;

        Ok((xorurl, content_map))
    }

    pub fn files_container_get_latest(&self, xorurl: &str) -> Result<(u64, FilesMap), String> {
        let xorurl_encoder = XorUrlEncoder::from_url(xorurl)?;
        match self
            .safe_app
            .get_seq_appendable_latest(xorurl_encoder.xorname(), FILES_CONTAINER_TYPE_TAG)
        {
            Ok((version, (_key, value))) => {
                // TODO: use RDF format and deserialise it
                let files_map = serde_json::from_str(&String::from_utf8_lossy(&value.as_slice()))
                    .map_err(|err| {
                    format!(
                        "Couldn't deserialise the FilesMap stored in the FilesContainer: {:?}",
                        err
                    )
                })?;
                Ok((version, files_map))
            }
            Err("SeqAppendOnlyDataEmpty") => Ok((0, FilesMap::default())),
            Err("SeqAppendOnlyDataNotFound") | Err(_) => {
                Err("No FilesContainer found at this address".to_string())
            }
        }
    }

    pub fn files_container_sync(
        &mut self,
        location: &str,
        xorurl: &str,
        recursive: bool,
        set_root: Option<String>,
        delete: bool,
    ) -> Result<(u64, ContentMap), String> {
        let (mut version, current_files_map): (u64, FilesMap) =
            self.files_container_get_latest(xorurl)?;

        let (content_map, new_files_map): (ContentMap, FilesMap) = sync_dir_contents(
            self,
            location,
            current_files_map,
            recursive,
            set_root,
            delete,
        )?;

        if !content_map.is_empty() {
            // The FilesContainer is updated adding an entry containing the timestamp as the
            // entry's key, and the serialised new version of the FilesMap as the entry's value
            let serialised_files_map = serde_json::to_string(&new_files_map)
                .map_err(|err| format!("Couldn't serialise the FilesMap generated: {:?}", err))?;
            let now = Utc::now().to_string().to_string();
            let files_container_data = (
                now.into_bytes().to_vec(),
                serialised_files_map.as_bytes().to_vec(),
            );

            let xorurl_encoder = XorUrlEncoder::from_url(xorurl)?;

            // Append new entry in the FilesContainer, which is a Published AppendOnlyData
            version = self.safe_app.append_seq_appendable_data(
                files_container_data,
                xorurl_encoder.xorname(),
                xorurl_encoder.type_tag(),
            )?;
        }

        Ok((version, content_map))
    }

    // TODO:
    // Upload files as ImmutableData
    // Check if file or dir
    // if dir, grab and do many.
    // upload individual file
    // get file metadata?
    // if not now... when?

    /// # Put Published ImmutableData
    /// Put data blobs onto the network.
    ///
    /// ## Example
    /// ```
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// # let mut safe = Safe::new("base32".to_string());
    /// let data = b"Something super good";
    /// let xorurl = safe.files_put_published_immutable(data).unwrap();
    /// # let received_data = safe.files_get_published_immutable(&xorurl).unwrap();
    /// # assert_eq!(received_data, data);
    /// ```
    pub fn files_put_published_immutable(&mut self, data: &[u8]) -> Result<XorUrl, String> {
        // TODO: do we want ownership from other PKs yet?
        let xorname = self.safe_app.files_put_published_immutable(&data)?;

        XorUrlEncoder::encode(
            xorname,
            0,
            SafeContentType::ImmutableData,
            &self.xorurl_base,
        )
    }

    /// # Get Published ImmutableData
    /// Put data blobs onto the network.
    ///
    /// ## Example
    /// ```
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// # let mut safe = Safe::new("base32".to_string());
    /// # let data = b"Something super good";
    /// let xorurl = safe.files_put_published_immutable(data).unwrap();
    /// let received_data = safe.files_get_published_immutable(&xorurl).unwrap();
    /// # assert_eq!(received_data, data);
    /// ```
    pub fn files_get_published_immutable(&self, xorurl: &str) -> Result<Vec<u8>, String> {
        // TODO: do we want ownership from other PKs yet?
        let xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        self.safe_app
            .files_get_published_immutable(xorurl_encoder.xorname())
    }
}

// Helper functions

fn gen_normalised_paths(new_content: &ContentMap, set_root: Option<String>) -> (String, String) {
    let replacement_root = set_root.unwrap_or_else(|| "".to_string());
    // Let's normalise the path to use '/' (instead of '\' as on Windows)
    let mut base_path = str::replace(&replacement_root, "\\", "/").to_string();

    if !base_path.starts_with('/') {
        base_path = format!("/{}", base_path)
    }

    let normalised_prefix = if new_content.len() > 1 {
        let mut paths: Vec<&Path> = vec![];
        new_content.keys().for_each(|key| {
            paths.push(Path::new(key));
        });
        let prefix = common_path_all(paths).unwrap_or_else(PathBuf::new);
        let normalised = &str::replace(&prefix.to_str().unwrap(), "\\", "/");
        normalised.clone()
    } else {
        "/".to_string()
    };

    (base_path, normalised_prefix)
}

fn gen_new_file_item(
    safe: &mut Safe,
    file_path: &Path,
    file_type: &str,
    file_size: &str,
    file_created: Option<&str>,
) -> Result<FileItem, String> {
    let now = Utc::now().to_string().to_string();
    let mut file_item = FileItem::new();
    let xorurl = upload_file(safe, file_path)?;
    file_item.insert("link".to_string(), xorurl.to_string());
    file_item.insert("type".to_string(), file_type.to_string());
    file_item.insert("size".to_string(), file_size.to_string());
    file_item.insert("modified".to_string(), now.clone());
    let created = file_created.unwrap_or_else(|| &now);
    file_item.insert("created".to_string(), created.to_string());

    Ok(file_item)
}

fn files_map_sync(
    safe: &mut Safe,
    mut current_files_map: FilesMap,
    new_content: ContentMap,
    set_root: Option<String>,
    delete: bool,
) -> Result<(ContentMap, FilesMap), String> {
    let (base_path, normalised_prefix) = gen_normalised_paths(&new_content, set_root);
    let mut updated_files_map = FilesMap::new();
    let mut content_map = ContentMap::new();

    for (key, _value) in new_content.iter() {
        let metadata = fs::metadata(&key).map_err(|err| {
            format!(
                "Couldn't obtain metadata information for local file: {:?}",
                err,
            )
        })?;

        let file_path = Path::new(&key);
        let file_type = file_path
            .extension()
            .ok_or("unknown")?
            .to_str()
            .unwrap_or_else(|| "unknown")
            .to_string();
        let file_size = metadata.len().to_string();

        let file_name =
            RelativePath::new(&key.to_string().replace(&normalised_prefix, &base_path)).normalize();
        // Above normalize removes initial slash, and uses '\' if it's on Windows
        let normalised_file_name = format!("/{}", str::replace(file_name.as_str(), "\\", "/"));

        // Let's update FileItem if there is a change or it doesn't exist in current_files_map
        match current_files_map.get(&normalised_file_name) {
            None => {
                // We need to add a new FileItem, let's upload it first
                match gen_new_file_item(safe, &file_path, &file_type, &file_size, None) {
                    Ok(new_file_item) => {
                        debug!("New FileItem item: {:?}", new_file_item);
                        debug!("New FileItem item inserted as {:?}", &file_name);
                        updated_files_map.insert(normalised_file_name, new_file_item.clone());
                        content_map.insert(
                            key.to_string(),
                            (FILE_ADDED_SIGN.to_string(), new_file_item["link"].clone()),
                        );
                    }
                    Err(err) => eprintln!( // TODO: add it to the report with "E" as change string
                        "Skipping file \"{}\" since it couldn't be uploaded to the network: {:?}",
                        normalised_file_name, err
                    ),
                };
            }
            Some(file_item) => {
                // TODO: we don't record the original creation/modified timestamp from the,
                // filesystem thus we cannot compare to see if they changed
                if file_item["size"] != file_size || file_item["type"] != file_type {
                    // We need to update the current FileItem, let's upload it first
                    match gen_new_file_item(safe, &file_path, &file_type, &file_size, Some(&file_item["created"])) {
                        Ok(new_file_item) => {
                            debug!("Updated FileItem item: {:?}", new_file_item);
                            debug!("Updated FileItem item inserted as {:?}", &file_name);
                            updated_files_map.insert(normalised_file_name.to_string(), new_file_item.clone());
                            content_map.insert(key.to_string(), (FILE_UPDATED_SIGN.to_string(), new_file_item["link"].clone()));
                        },
                        Err(err) => eprintln!( // TODO: add to the report with "E" as the change string
                            "Skipping file \"{}\" since it couldn't be uploaded to the network: {:?}",
                            &normalised_file_name, err
                        )
                    };
                } else {
                    // No need to update FileItem just copy the existing one
                    updated_files_map.insert(normalised_file_name.to_string(), file_item.clone());
                }

                // let's now remove it form the current list so we now it has been processed
                current_files_map.remove(&normalised_file_name);
            }
        }
    }

    // Finally, unless 'delete' was set keep the files that are currently
    // in FilesContainer but not in source location
    current_files_map.iter().for_each(|(file_name, file_item)| {
        if !delete {
            updated_files_map.insert(file_name.to_string(), file_item.clone());
        } else {
            content_map.insert(
                file_name.to_string(),
                (FILE_DELETED_SIGN.to_string(), file_item["link"].clone()),
            );
        }
    });

    Ok((content_map, updated_files_map))
}

fn sync_dir_contents(
    safe: &mut Safe,
    location: &str,
    current_files_map: FilesMap,
    recursive: bool,
    set_root: Option<String>,
    delete: bool,
) -> Result<(ContentMap, FilesMap), String> {
    let path = Path::new(location);
    info!("Reading files from {}", &path.display());
    let metadata = fs::metadata(&path).map_err(|err| {
        format!(
            "Couldn't read metadata from source path ('{}'): {}",
            location, err
        )
    })?;

    debug!("Metadata for location: {:?}", metadata);

    let mut new_content_map = BTreeMap::new();
    if recursive {
        // TODO: option to enable following symlinks and hidden files?
        // We now compare both FilesMaps to upload the missing files
        WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| is_not_hidden(e))
            .filter_map(|v| v.ok())
            .for_each(|child| {
                info!("{}", child.path().display());
                let current_file_path = child.path();
                let current_path_str = current_file_path.to_str().unwrap_or_else(|| "").to_string();
                // Let's normalise the path to use '/' (instead of '\' as on Windows)
                //let normalised_path = str::replace(&current_path_str, "\\", "/");
                match fs::metadata(&current_file_path) {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            // Everything is in the iter. We dont need to recurse.
                            // so what do we do with dirs? decide if we want to support empty dirs also
                        } else {
                            new_content_map.insert(current_path_str, ("".to_string(), "SYNC".to_string()));
                        }
                    },
                    Err(err) => eprintln!(
                        "Skipping file \"{}\" since no metadata could be read from local location: {:?}",
                        current_path_str, err
                    )
                }
            });
    } else {
        if metadata.is_dir() {
            return Err(format!(
                "{:?} is a directory. Use \"-r\" to recursively upload folders.",
                location
            ));
        }
        new_content_map.insert(location.to_string(), ("".to_string(), "SYNC".to_string()));
    }

    files_map_sync(safe, current_files_map, new_content_map, set_root, delete)
}

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with('.'))
        .unwrap_or(false)
}

fn upload_dir_contents(
    safe: &mut Safe,
    location: &str,
    recursive: bool,
) -> Result<ContentMap, String> {
    let path = Path::new(location);
    info!("Reading files from {}", &path.display());
    let metadata = fs::metadata(&path).map_err(|err| {
        format!(
            "Couldn't read metadata from source path ('{}'): {}",
            location, err
        )
    })?;

    debug!("Metadata for location: {:?}", metadata);

    let mut content_map = BTreeMap::new();
    if recursive {
        // TODO: option to enable following symlinks and hidden files?
        WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| is_not_hidden(e))
            .filter_map(|v| v.ok())
            .for_each(|child| {
                info!("{}", child.path().display());
                let current_path = child.path();
                let current_path_str = current_path.to_str().unwrap_or_else(|| "").to_string();
                // Let's normalise the path to use '/' (instead of '\' as on Windows)
                let normalised_path = str::replace(&current_path_str, "\\", "/");
                match fs::metadata(&current_path) {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            // Everything is in the iter. We dont need to recurse.
                            // so what do we do with dirs? decide if we want to support empty dirs also
                        } else {
                            match upload_file(safe, &current_path) {
                                Ok(xorurl) => {
                                    content_map.insert(normalised_path, (FILE_ADDED_SIGN.to_string(), xorurl));
                                }
                                Err(err) => eprintln!(
                                    "Skipping file \"{}\" since it couldn't be uploaded to the network: {:?}",
                                    normalised_path, err
                                ),
                            };
                        }
                    },
                    Err(err) => eprintln!(
                        "Skipping file \"{}\" since no metadata could be read from local location: {:?}",
                        normalised_path, err
                    )
                }
            });
    } else {
        if metadata.is_dir() {
            return Err(format!(
                "{:?} is a directory. Use \"-r\" to recursively upload folders.",
                location
            ));
        }
        let xorurl = upload_file(safe, &path)?;
        content_map.insert(location.to_string(), (FILE_ADDED_SIGN.to_string(), xorurl));
    }

    Ok(content_map)
}

fn upload_file(safe: &mut Safe, path: &Path) -> Result<XorUrl, String> {
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(err) => return Err(err.to_string()),
    };
    safe.files_put_published_immutable(&data)
}

fn files_map_create(content: &ContentMap, set_root: Option<String>) -> Result<FilesMap, String> {
    let mut files_map = FilesMap::default();
    let now = Utc::now().to_string().to_string();

    let (base_path, normalised_prefix) = gen_normalised_paths(content, set_root);

    for (file_name, (_change, link)) in content.iter() {
        let mut file_item = FileItem::new();
        let metadata = fs::metadata(&file_name).map_err(|err| {
            format!(
                "Couldn't obtain metadata information for local file: {:?}",
                err,
            )
        })?;

        file_item.insert("link".to_string(), link.to_string());

        let file_type = Path::new(&file_name).extension().ok_or("unknown")?;
        file_item.insert(
            "type".to_string(),
            file_type.to_str().unwrap_or_else(|| "unknown").to_string(),
        );

        let file_size = &metadata.len().to_string();
        file_item.insert("size".to_string(), file_size.to_string());

        // file_item.insert("permissions", metadata.permissions().to_string());
        file_item.insert("modified".to_string(), now.clone());
        file_item.insert("created".to_string(), now.clone());

        debug!("FileItem item: {:?}", file_item);
        let new_file_name = RelativePath::new(
            &file_name
                .to_string()
                .replace(&normalised_prefix, &base_path),
        )
        .normalize();

        // Above normalize removes initial slash, and uses '\' if it's on Windows
        let final_name = format!("/{}", str::replace(new_file_name.as_str(), "\\", "/"));

        debug!("FileItem item inserted as {:?}", &final_name);
        files_map.insert(final_name.to_string(), file_item);
    }

    Ok(files_map)
}

// Unit Tests

#[test]
fn test_keys_create_preload_test_coins() {}

// # use safe_cli::Safe;
// # use unwrap::unwrap;
// # use std::collections::BTreeMap;
// # let mut safe = Safe::new("base32".to_string());
// let top = b"Something top level";
// let top_xorurl = safe.files_put_published_immutable(top).unwrap();
// let second = b"Something second level";
// let second_xorurl = safe.files_put_published_immutable(second).unwrap();
// let mut content_map = BTreeMap::new();
// content_map.insert("./tests/testfolder/test.md".to_string(), top_xorurl);
// content_map.insert("./tests/testfolder/subfolder/subexists.md".to_string(), second_xorurl);
// let file_map = safe.files_map_create( &content_map, None ).unwrap();
// # assert!(file_map.contains("\"md\""));
// # assert!(file_map.contains("\"/test.md\""));
// # assert!(file_map.contains("\"/subfolder/subexists.md\""));
// # assert!(!file_map.contains("tests/testfolder"));
// ```
