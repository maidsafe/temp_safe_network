// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::constants::{
    CONTENT_ADDED_SIGN, CONTENT_DELETED_SIGN, CONTENT_ERROR_SIGN, CONTENT_UPDATED_SIGN,
    FAKE_RDF_PREDICATE_CREATED, FAKE_RDF_PREDICATE_LINK, FAKE_RDF_PREDICATE_MODIFIED,
    FAKE_RDF_PREDICATE_SIZE, FAKE_RDF_PREDICATE_TYPE,
};
use super::helpers::{gen_timestamp_nanos, gen_timestamp_secs};
use super::xorurl::{SafeContentType, SafeDataType};
use super::{Error, ResultReturn, Safe, SafeApp, XorUrl, XorUrlEncoder};
use log::{debug, info, warn};
use mime_guess;
use relative_path::RelativePath;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

// Each FileItem contains file metadata and the link to the file's ImmutableData XOR-URL
pub type FileItem = BTreeMap<String, String>;

// To use for mapping files names (with path in a flattened hierarchy) to FileItems
pub type FilesMap = BTreeMap<String, FileItem>;

// List of files uploaded with details if they were added, updated or deleted from FilesContainer
type ProcessedFiles = BTreeMap<String, (String, String)>;

// Type tag to use for the FilesContainer stored on AppendOnlyData
const FILES_CONTAINER_TYPE_TAG: u64 = 1_100;

const ERROR_MSG_NO_FILES_CONTAINER_FOUND: &str = "No FilesContainer found at this address";

const MAX_RECURSIVE_DEPTH: usize = 10_000;

#[allow(dead_code)]
impl Safe {
    /// # Create a FilesContaier.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use safe_cli::Safe;
    /// # let mut safe = Safe::new("base32z");
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// let (xorurl, _processed_files, _files_map) = safe.files_container_create("tests/testfolder", None, true, false).unwrap();
    /// assert!(xorurl.contains("safe://"))
    /// ```
    pub fn files_container_create(
        &mut self,
        location: &str,
        dest: Option<String>,
        recursive: bool,
        dry_run: bool,
    ) -> ResultReturn<(XorUrl, ProcessedFiles, FilesMap)> {
        // TODO: Enable source for funds / ownership
        // Warn about ownership?

        // Let's upload the files and generate the list of local files paths
        let processed_files = file_system_dir_walk(self, location, recursive, !dry_run)?;

        // The FilesContainer is created as a AppendOnlyData with a single entry containing the
        // timestamp as the entry's key, and the serialised FilesMap as the entry's value
        // TODO: use RDF format
        let files_map = files_map_create(&processed_files, location, dest)?;

        let xorurl = if dry_run {
            "".to_string()
        } else {
            let serialised_files_map = serde_json::to_string(&files_map).map_err(|err| {
                Error::Unexpected(format!(
                    "Couldn't serialise the FilesMap generated: {:?}",
                    err
                ))
            })?;

            let now = gen_timestamp_nanos();
            let files_container_data = vec![(
                now.into_bytes().to_vec(),
                serialised_files_map.as_bytes().to_vec(),
            )];

            // Store the FilesContainer in a Published AppendOnlyData
            let xorname = self.safe_app.put_seq_append_only_data(
                files_container_data,
                None,
                FILES_CONTAINER_TYPE_TAG,
                None,
            )?;

            XorUrlEncoder::encode(
                xorname,
                FILES_CONTAINER_TYPE_TAG,
                SafeDataType::PublishedSeqAppendOnlyData,
                SafeContentType::FilesContainer,
                None,
                None,
                None,
                &self.xorurl_base,
            )?
        };

        Ok((xorurl, processed_files, files_map))
    }

    /// # Fetch an existing FilesContaier.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use safe_cli::Safe;
    /// # let mut safe = Safe::new("base32z");
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// let (xorurl, _processed_files, _files_map) = safe.files_container_create("tests/testfolder", None, true, false).unwrap();
    /// let (version, files_map) = safe.files_container_get(&xorurl).unwrap();
    /// println!("FilesContainer fetched is at version: {}", version);
    /// println!("FilesMap of fetched version is: {:?}", files_map);
    /// ```
    pub fn files_container_get(&self, url: &str) -> ResultReturn<(u64, FilesMap)> {
        debug!("Getting files container from: {:?}", url);
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url)?;

        // Check if the URL specifies a specific version of the content or simply the latest available
        let data = xorurl_encoder.content_version().map_or_else(
            || {
                self.safe_app.get_latest_seq_append_only_data(
                    xorurl_encoder.xorname(),
                    FILES_CONTAINER_TYPE_TAG,
                )
            },
            |content_version| {
                let (key, value) = self
                    .safe_app
                    .get_seq_append_only_data(
                        xorurl_encoder.xorname(),
                        FILES_CONTAINER_TYPE_TAG,
                        content_version,
                    )
                    .map_err(|_| {
                        Error::VersionNotFound(format!(
                            "Version '{}' is invalid for FilesContainer found at \"{}\"",
                            content_version, url,
                        ))
                    })?;
                Ok((content_version, (key, value)))
            },
        );

        match data {
            Ok((version, (_key, value))) => {
                debug!("Files map retrieved.... v{:?}", &version);
                // TODO: use RDF format and deserialise it
                let files_map = serde_json::from_str(&String::from_utf8_lossy(&value.as_slice()))
                    .map_err(|err| {
                    Error::ContentError(format!(
                        "Couldn't deserialise the FilesMap stored in the FilesContainer: {:?}",
                        err
                    ))
                })?;
                Ok((version, files_map))
            }
            Err(Error::EmptyContent(_)) => {
                warn!("FilesContainer found at \"{:?}\" was empty", url);
                Ok((0, FilesMap::default()))
            }
            Err(Error::ContentNotFound(_)) => Err(Error::ContentNotFound(
                ERROR_MSG_NO_FILES_CONTAINER_FOUND.to_string(),
            )),
            Err(Error::VersionNotFound(msg)) => Err(Error::VersionNotFound(msg)),
            Err(err) => Err(Error::NetDataError(format!(
                "Failed to get current version: {}",
                err
            ))),
        }
    }

    /// # Sync up local folder with the content on a FilesContaier.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use safe_cli::Safe;
    /// # let mut safe = Safe::new("base32z");
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// let (xorurl, _processed_files, _files_map) = safe.files_container_create("tests/testfolder", None, true, false).unwrap();
    /// let (version, new_processed_files, new_files_map) = safe.files_container_sync("tests/testfolder", &xorurl, true, false, false, false).unwrap();
    /// println!("FilesContainer fetched is at version: {}", version);
    /// println!("The local files that were synced up are: {:?}", new_processed_files);
    /// println!("The FilesMap of the updated FilesContainer now is: {:?}", new_files_map);
    /// ```
    pub fn files_container_sync(
        &mut self,
        location: &str,
        url: &str,
        recursive: bool,
        delete: bool,
        update_nrs: bool,
        dry_run: bool,
    ) -> ResultReturn<(u64, ProcessedFiles, FilesMap)> {
        if delete && !recursive {
            return Err(Error::InvalidInput(
                "'delete' is not allowed if --recursive is not set".to_string(),
            ));
        }

        let xorurl_encoder = Safe::parse_url(url)?;
        if xorurl_encoder.content_version().is_some() {
            return Err(Error::InvalidInput(format!(
                "The target URL cannot cannot contain a version: {}",
                url
            )));
        };

        let (mut xorurl_encoder, is_nrs_resolved) = self.parse_and_resolve_url(url)?;
        // If NRS name shall be updated then the URL has to be an NRS-URL
        if update_nrs && !is_nrs_resolved {
            return Err(Error::InvalidInput(
                "'update-nrs' is not allowed since the URL provided is not an NRS URL".to_string(),
            ));
        }

        // If the FilesContainer URL was resolved from an NRS name we need to remove
        // the version from it so we can fetch latest version of it for sync-ing
        if is_nrs_resolved {
            xorurl_encoder.set_content_version(None);
        }

        let (current_version, current_files_map): (u64, FilesMap) =
            self.files_container_get(&xorurl_encoder.to_string()?)?;

        // Let's generate the list of local files paths, without uploading any new file yet
        let processed_files = file_system_dir_walk(self, location, recursive, false)?;

        let dest_path = Some(xorurl_encoder.path().to_string());
        let (processed_files, new_files_map, success_count): (ProcessedFiles, FilesMap, u64) =
            files_map_sync(
                self,
                current_files_map,
                location,
                processed_files,
                dest_path,
                delete,
                !dry_run,
            )?;

        let version = if success_count == 0 {
            current_version
        } else if dry_run {
            current_version + 1
        } else {
            // The FilesContainer is updated by adding an entry containing the timestamp as the
            // entry's key, and the serialised new version of the FilesMap as the entry's value
            let serialised_files_map = serde_json::to_string(&new_files_map).map_err(|err| {
                Error::Unexpected(format!(
                    "Couldn't serialise the FilesMap generated: {:?}",
                    err
                ))
            })?;

            let now = gen_timestamp_nanos();
            let files_container_data = vec![(
                now.into_bytes().to_vec(),
                serialised_files_map.as_bytes().to_vec(),
            )];

            let xorname = xorurl_encoder.xorname();
            let type_tag = xorurl_encoder.type_tag();
            let new_version = self.safe_app.append_seq_append_only_data(
                files_container_data,
                current_version + 1,
                xorname,
                type_tag,
            )?;

            if update_nrs {
                // We need to update the link in the NRS container as well,
                // to link it to the new new_version of the FilesContainer we just generated
                xorurl_encoder.set_content_version(Some(new_version));
                let new_link_for_nrs = xorurl_encoder.to_string()?;
                let _ = self.nrs_map_container_add(url, &new_link_for_nrs, false, true, false)?;
            }

            new_version
        };

        Ok((version, processed_files, new_files_map))
    }

    /// # Put Published ImmutableData
    /// Put data blobs onto the network.
    ///
    /// ## Example
    /// ```
    /// # use safe_cli::Safe;
    /// # let mut safe = Safe::new("base32z");
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// let data = b"Something super good";
    /// let xorurl = safe.files_put_published_immutable(data, Some("text/plain")).unwrap();
    /// # let received_data = safe.files_get_published_immutable(&xorurl).unwrap();
    /// # assert_eq!(received_data, data);
    /// ```
    pub fn files_put_published_immutable(
        &mut self,
        data: &[u8],
        media_type: Option<&str>,
    ) -> ResultReturn<XorUrl> {
        // TODO: do we want ownership from other PKs yet?
        let xorname = self.safe_app.files_put_published_immutable(&data)?;
        let content_type = media_type.map_or_else(
            || SafeContentType::Raw,
            |mime_str| SafeContentType::MediaType(mime_str.to_string()),
        );

        XorUrlEncoder::encode(
            xorname,
            0,
            SafeDataType::PublishedImmutableData,
            content_type,
            None,
            None,
            None,
            &self.xorurl_base,
        )
    }

    /// # Get Published ImmutableData
    /// Put data blobs onto the network.
    ///
    /// ## Example
    /// ```
    /// # use safe_cli::Safe;
    /// # let mut safe = Safe::new("base32z");
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// # let data = b"Something super good";
    /// let xorurl = safe.files_put_published_immutable(data, None).unwrap();
    /// let received_data = safe.files_get_published_immutable(&xorurl).unwrap();
    /// # assert_eq!(received_data, data);
    /// ```
    pub fn files_get_published_immutable(&self, url: &str) -> ResultReturn<Vec<u8>> {
        // TODO: do we want ownership from other PKs yet?
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url)?;
        self.safe_app
            .files_get_published_immutable(xorurl_encoder.xorname())
    }
}

// Helper functions

// Simply change Windows style path separator into `/`
fn normalise_path_separator(from: &str) -> String {
    str::replace(&from, "\\", "/").to_string()
}

// From the location path and the destination path chosen by the user, calculate
// the destination path considering ending '/' in both the  location and dest path
fn get_base_paths(location: &str, dest_path: Option<String>) -> ResultReturn<(String, String)> {
    // Let's normalise the path to use '/' (instead of '\' as on Windows)
    let location_base_path = if location == "." {
        "./".to_string()
    } else {
        normalise_path_separator(location)
    };

    let new_dest_path = match dest_path {
        Some(path) => {
            if path.is_empty() {
                "/".to_string()
            } else {
                path
            }
        }
        None => "/".to_string(),
    };

    // Let's first check if it ends with '/'
    let dest_base_path = if new_dest_path.ends_with('/') {
        if location_base_path.ends_with('/') {
            new_dest_path
        } else {
            // Location is a folder, then append it to dest path
            let parts_vec: Vec<&str> = location_base_path.split('/').collect();
            let dir_name = parts_vec[parts_vec.len() - 1];
            format!("{}{}", new_dest_path, dir_name)
        }
    } else {
        // Then just append an ending '/'
        format!("{}/", new_dest_path)
    };

    Ok((location_base_path, dest_base_path))
}

// Generate a FileItem for a file which can then be added to a FilesMap
// This is now a pseudo-RDF but will eventually be converted to be an RDF graph
fn gen_new_file_item(
    safe: &mut Safe,
    file_path: &Path,
    file_type: &str,
    file_size: &str,
    file_created: Option<&str>,
    upload_file: bool,
) -> Result<FileItem, String> {
    let now = gen_timestamp_secs();
    let mut file_item = FileItem::new();
    let xorurl = if upload_file {
        upload_file_to_net(safe, file_path)?
    } else {
        "".to_string()
    };
    file_item.insert(FAKE_RDF_PREDICATE_LINK.to_string(), xorurl.to_string());
    file_item.insert(FAKE_RDF_PREDICATE_TYPE.to_string(), file_type.to_string());
    file_item.insert(FAKE_RDF_PREDICATE_SIZE.to_string(), file_size.to_string());
    file_item.insert(FAKE_RDF_PREDICATE_MODIFIED.to_string(), now.clone());
    let created = file_created.unwrap_or_else(|| &now);
    file_item.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), created.to_string());

    Ok(file_item)
}

// From the provided list of local files paths, find the local changes made in comparison with the
// target FilesContainer, uploading new files as necessary, and creating a new FilesMap with file's
// metadata and their corresponding links, as well as generating the report of processed files
fn files_map_sync(
    safe: &mut Safe,
    mut current_files_map: FilesMap,
    location: &str,
    new_content: ProcessedFiles,
    dest_path: Option<String>,
    delete: bool,
    upload_files: bool,
) -> ResultReturn<(ProcessedFiles, FilesMap, u64)> {
    let (location_base_path, dest_base_path) = get_base_paths(location, dest_path)?;
    let mut updated_files_map = FilesMap::new();
    let mut processed_files = ProcessedFiles::new();
    let mut success_count = 0;

    for (key, _value) in new_content
        .iter()
        .filter(|(_, (change, _))| change != CONTENT_ERROR_SIGN)
    {
        let file_path = Path::new(&key);
        let (metadata, file_type) = get_metadata(&file_path)?;
        let file_size = metadata.len().to_string();

        let file_name = RelativePath::new(
            &key.to_string()
                .replace(&location_base_path, &dest_base_path),
        )
        .normalize();
        // Above normalize removes initial slash, and uses '\' if it's on Windows
        let normalised_file_name = format!("/{}", normalise_path_separator(file_name.as_str()));

        // Let's update FileItem if there is a change or it doesn't exist in current_files_map
        match current_files_map.get(&normalised_file_name) {
            None => {
                // We need to add a new FileItem, let's upload it first
                match gen_new_file_item(
                    safe,
                    &file_path,
                    &file_type,
                    &file_size,
                    None,
                    upload_files,
                ) {
                    Ok(new_file_item) => {
                        debug!("New FileItem item: {:?}", new_file_item);
                        debug!("New FileItem item inserted as {:?}", &file_name);
                        updated_files_map.insert(normalised_file_name, new_file_item.clone());
                        processed_files.insert(
                            key.to_string(),
                            (
                                CONTENT_ADDED_SIGN.to_string(),
                                new_file_item[FAKE_RDF_PREDICATE_LINK].clone(),
                            ),
                        );
                        success_count += 1;
                    }
                    Err(err) => {
                        processed_files.insert(
                            key.to_string(),
                            (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)),
                        );
                        info!(
                        "Skipping file \"{}\" since it couldn't be uploaded to the network: {:?}",
                        normalised_file_name, err);
                    }
                };
            }
            Some(file_item) => {
                // TODO: we don't record the original creation/modified timestamp from the,
                // filesystem thus we cannot compare to see if they changed
                if file_item[FAKE_RDF_PREDICATE_SIZE] != file_size
                    || file_item[FAKE_RDF_PREDICATE_TYPE] != file_type
                {
                    // We need to update the current FileItem, let's upload it first
                    match gen_new_file_item(
                        safe,
                        &file_path,
                        &file_type,
                        &file_size,
                        Some(&file_item[FAKE_RDF_PREDICATE_CREATED]),
                        upload_files,
                    ) {
                        Ok(new_file_item) => {
                            debug!("Updated FileItem item: {:?}", new_file_item);
                            debug!("Updated FileItem item inserted as {:?}", &file_name);
                            updated_files_map
                                .insert(normalised_file_name.to_string(), new_file_item.clone());
                            processed_files.insert(
                                key.to_string(),
                                (
                                    CONTENT_UPDATED_SIGN.to_string(),
                                    new_file_item[FAKE_RDF_PREDICATE_LINK].clone(),
                                ),
                            );
                            success_count += 1;
                        }
                        Err(err) => {
                            processed_files.insert(
                                key.to_string(),
                                (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)),
                            );
                            info!("Skipping file \"{}\": {}", &normalised_file_name, err);
                        }
                    };
                } else {
                    // No need to update FileItem just copy the existing one
                    updated_files_map.insert(normalised_file_name.to_string(), file_item.clone());
                }

                // let's now remove it from the current list so we now it has been processed
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
            processed_files.insert(
                file_name.to_string(),
                (
                    CONTENT_DELETED_SIGN.to_string(),
                    file_item[FAKE_RDF_PREDICATE_LINK].clone(),
                ),
            );
            success_count += 1;
        }
    });

    Ok((processed_files, updated_files_map, success_count))
}

// Upload a files to the Network as a Published-ImmutableData
fn upload_file_to_net(safe: &mut Safe, path: &Path) -> ResultReturn<XorUrl> {
    let data = fs::read(path).map_err(|err| {
        Error::InvalidInput(format!("Failed to read file from local location: {}", err))
    })?;
    let mime_type = mime_guess::from_path(&path);
    safe.files_put_published_immutable(&data, mime_type.first_raw())
}

// Get file metadata from local filesystem
fn get_metadata(path: &Path) -> ResultReturn<(fs::Metadata, String)> {
    let metadata = fs::metadata(path).map_err(|err| {
        Error::FilesSystemError(format!(
            "Couldn't read metadata from source path ('{}'): {}",
            path.display(),
            err
        ))
    })?;
    debug!("Metadata for location: {:?}", metadata);

    let extension = match path.extension() {
        Some(ext) => ext
            .to_str()
            .ok_or("unknown")
            .map_err(|err| Error::Unexpected(err.to_string()))?,
        None => "unknown",
    };

    Ok((metadata, extension.to_string()))
}

// Walk the local filesystem starting from `location`, creating a list of files paths,
// and if requested with `upload_files` arg, upload the files to the network filling up
// the list of files with their corresponding XOR-URLs
fn file_system_dir_walk(
    safe: &mut Safe,
    location: &str,
    recursive: bool,
    upload_files: bool,
) -> ResultReturn<ProcessedFiles> {
    let file_path = Path::new(location);
    let (metadata, _) = get_metadata(&file_path)?;
    info!("Reading files from {}", file_path.display());
    if metadata.is_dir() || !recursive {
        // TODO: option to enable following symlinks and hidden files?
        // We now compare both FilesMaps to upload the missing files
        let max_depth = if recursive { MAX_RECURSIVE_DEPTH } else { 1 };
        let mut processed_files = BTreeMap::new();
        WalkDir::new(file_path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| not_hidden_and_valid_depth(e, max_depth))
            .filter_map(|v| v.ok())
            .for_each(|child| {
                let current_file_path = child.path();
                let current_path_str = current_file_path.to_str().unwrap_or_else(|| "").to_string();
                info!("Processing {}...", current_path_str);
                let normalised_path = normalise_path_separator(&current_path_str);
                match fs::metadata(&current_file_path) {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            // Everything is in the iter. We dont need to recurse.
                            // so what do we do with dirs? decide if we want to support empty dirs also
                        } else if upload_files {
                            match upload_file_to_net(safe, &current_file_path) {
                                Ok(xorurl) => {
                                    processed_files.insert(normalised_path, (CONTENT_ADDED_SIGN.to_string(), xorurl));
                                }
                                Err(err) => {
                                    processed_files.insert(normalised_path.clone(), (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)));
                                    info!(
                                    "Skipping file \"{}\". {}",
                                    normalised_path, err);
                                },
                            };
                        } else {
                            processed_files.insert(normalised_path.clone(), (CONTENT_ADDED_SIGN.to_string(), "".to_string()));
                        }
                    },
                    Err(err) => {
                        processed_files.insert(normalised_path.clone(), (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)));
                        info!(
                        "Skipping file \"{}\" since no metadata could be read from local location: {:?}",
                        normalised_path, err);
                    }
                }
            });

        Ok(processed_files)
    } else {
        // Recursive only works on a dir path. Let's error as the user may be making a mistake
        // so it's better for the user to double check and either provide the correct path
        // or remove the `--recursive` from the args
        Err(Error::InvalidInput(format!(
            "'{}' is not a directory. The \"--recursive\" arg is only supported for folders.",
            location
        )))
    }
}

// Checks if a path is not a hidden file or if the depth in the dir hierarchy is under a threshold
fn not_hidden_and_valid_depth(entry: &DirEntry, max_depth: usize) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() <= max_depth && (entry.depth() == 0 || !s.starts_with('.')))
        .unwrap_or(false)
}

// From the provided list of local files paths and corresponding files XOR-URLs,
// create a FilesMap with file's metadata and their corresponding links
fn files_map_create(
    content: &ProcessedFiles,
    location: &str,
    dest_path: Option<String>,
) -> ResultReturn<FilesMap> {
    let mut files_map = FilesMap::default();
    let now = gen_timestamp_secs();

    let (location_base_path, dest_base_path) = get_base_paths(location, dest_path)?;
    for (file_name, (_change, link)) in content
        .iter()
        .filter(|(_, (change, _))| change != CONTENT_ERROR_SIGN)
    {
        debug!("FileItem item name:{:?}", &file_name);
        let mut file_item = FileItem::new();
        let file_path = Path::new(&file_name);
        let (metadata, file_type) = get_metadata(&file_path)?;

        file_item.insert(FAKE_RDF_PREDICATE_LINK.to_string(), link.to_string());

        file_item.insert(FAKE_RDF_PREDICATE_TYPE.to_string(), file_type);

        let file_size = &metadata.len().to_string();
        file_item.insert(FAKE_RDF_PREDICATE_SIZE.to_string(), file_size.to_string());

        // file_item.insert("permissions", metadata.permissions().to_string());
        file_item.insert(FAKE_RDF_PREDICATE_MODIFIED.to_string(), now.clone());
        file_item.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), now.clone());

        debug!("FileItem item: {:?}", file_item);
        let new_file_name = RelativePath::new(
            &file_name
                .to_string()
                .replace(&location_base_path, &dest_base_path),
        )
        .normalize();

        // Above normalize removes initial slash, and uses '\' if it's on Windows
        let final_name = format!("/{}", normalise_path_separator(new_file_name.as_str()));

        debug!("FileItem item inserted with filename {:?}", &final_name);
        files_map.insert(final_name.to_string(), file_item);
    }

    Ok(files_map)
}

// Unit Tests

#[test]
fn test_files_map_create() {
    use unwrap::unwrap;
    let mut processed_files = ProcessedFiles::new();
    processed_files.insert(
        "./tests/testfolder/test.md".to_string(),
        (
            CONTENT_ADDED_SIGN.to_string(),
            "safe://top_xorurl".to_string(),
        ),
    );
    processed_files.insert(
        "./tests/testfolder/subfolder/subexists.md".to_string(),
        (
            CONTENT_ADDED_SIGN.to_string(),
            "safe://second_xorurl".to_string(),
        ),
    );
    let files_map = unwrap!(files_map_create(
        &processed_files,
        "./tests/testfolder",
        Some("".to_string())
    ));
    assert_eq!(files_map.len(), 2);
    let file_item1 = &files_map["/testfolder/test.md"];
    assert_eq!(file_item1[FAKE_RDF_PREDICATE_LINK], "safe://top_xorurl");
    assert_eq!(file_item1[FAKE_RDF_PREDICATE_TYPE], "md");
    assert_eq!(file_item1[FAKE_RDF_PREDICATE_SIZE], "12");

    let file_item2 = &files_map["/testfolder/subfolder/subexists.md"];
    assert_eq!(file_item2[FAKE_RDF_PREDICATE_LINK], "safe://second_xorurl");
    assert_eq!(file_item2[FAKE_RDF_PREDICATE_TYPE], "md");
    assert_eq!(file_item2[FAKE_RDF_PREDICATE_SIZE], "23");
}

#[test]
fn test_files_container_create_file() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let filename = "tests/testfolder/test.md";
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create(filename, None, false, false));

    assert!(xorurl.starts_with("safe://"));
    assert_eq!(processed_files.len(), 1);
    assert_eq!(files_map.len(), 1);
    let file_path = "/test.md";
    assert_eq!(processed_files[filename].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename].1,
        files_map[file_path][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_create_dry_run() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let filename = "./tests/testfolder/";
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create(filename, None, true, true));

    assert!(xorurl.is_empty());
    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);

    let filename1 = "./tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert!(processed_files[filename1].1.is_empty());
    assert_eq!(
        processed_files[filename1].1,
        files_map["/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "./tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert!(processed_files[filename2].1.is_empty());
    assert_eq!(
        processed_files[filename2].1,
        files_map["/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert!(processed_files[filename3].1.is_empty());
    assert_eq!(
        processed_files[filename3].1,
        files_map["/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "./tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert!(processed_files[filename4].1.is_empty());
    assert_eq!(
        processed_files[filename4].1,
        files_map["/noextension"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_create_folder_without_trailing_slash() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create("tests/testfolder", None, true, false));

    assert!(xorurl.starts_with("safe://"));
    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);

    let filename1 = "tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename1].1,
        files_map["/testfolder/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename2].1,
        files_map["/testfolder/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename3].1,
        files_map["/testfolder/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename4].1,
        files_map["/testfolder/noextension"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_create_folder_with_trailing_slash() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    assert!(xorurl.starts_with("safe://"));
    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);

    let filename1 = "./tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename1].1,
        files_map["/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "./tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename2].1,
        files_map["/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename3].1,
        files_map["/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "./tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename4].1,
        files_map["/noextension"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_create_dest_path_without_trailing_slash() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) = unwrap!(safe.files_container_create(
        "./tests/testfolder",
        Some("/myroot".to_string()),
        true,
        false
    ));

    assert!(xorurl.starts_with("safe://"));
    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);

    let filename1 = "./tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename1].1,
        files_map["/myroot/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "./tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename2].1,
        files_map["/myroot/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename3].1,
        files_map["/myroot/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "./tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename4].1,
        files_map["/myroot/noextension"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_create_dest_path_with_trailing_slash() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) = unwrap!(safe.files_container_create(
        "./tests/testfolder",
        Some("/myroot/".to_string()),
        true,
        false
    ));

    assert!(xorurl.starts_with("safe://"));
    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);

    let filename1 = "./tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename1].1,
        files_map["/myroot/testfolder/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "./tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename2].1,
        files_map["/myroot/testfolder/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename3].1,
        files_map["/myroot/testfolder/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "./tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename4].1,
        files_map["/myroot/testfolder/noextension"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_sync() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);

    let (version, new_processed_files, new_files_map) = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &xorurl,
        true,
        false,
        false,
        false
    ));

    assert_eq!(version, 1);
    assert_eq!(new_processed_files.len(), 2);
    assert_eq!(new_files_map.len(), 7);

    let filename1 = "./tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename1].1,
        new_files_map["/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "./tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename2].1,
        new_files_map["/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename3].1,
        new_files_map["/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "./tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename4].1,
        new_files_map["/noextension"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename5 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        new_processed_files[filename5].1,
        new_files_map["/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename6 = "./tests/testfolder/subfolder/sub2.md";
    assert_eq!(new_processed_files[filename6].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        new_processed_files[filename6].1,
        new_files_map["/sub2.md"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_sync_dry_run() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);

    let (version, new_processed_files, new_files_map) = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &xorurl,
        true,
        false,
        false,
        true // set dry_run flag on
    ));

    assert_eq!(version, 1);
    assert_eq!(new_processed_files.len(), 2);
    assert_eq!(new_files_map.len(), 7);

    let filename1 = "./tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename1].1,
        new_files_map["/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "./tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename2].1,
        new_files_map["/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename3].1,
        new_files_map["/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "./tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename4].1,
        new_files_map["/noextension"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename5 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
    assert!(new_processed_files[filename5].1.is_empty());
    assert_eq!(
        new_processed_files[filename5].1,
        new_files_map["/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename6 = "./tests/testfolder/subfolder/sub2.md";
    assert_eq!(new_processed_files[filename6].0, CONTENT_ADDED_SIGN);
    assert!(new_processed_files[filename6].1.is_empty());
    assert_eq!(
        new_processed_files[filename6].1,
        new_files_map["/sub2.md"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_sync_with_versioned_target() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));

    let (xorurl, _, _) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    let versioned_xorurl = format!("{}?v=5", xorurl);
    match safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &versioned_xorurl,
        false,
        false,
        true, // this flag requests the update-nrs
        false,
    ) {
        Ok(_) => panic!("Sync was unexpectdly successful"),
        Err(err) => assert_eq!(
            err,
            Error::InvalidInput(format!(
                "The target URL cannot cannot contain a version: {}",
                versioned_xorurl
            ))
        ),
    };
}

#[test]
fn test_files_container_sync_with_delete() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);

    let (version, new_processed_files, new_files_map) = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &xorurl,
        true,
        true, // this sets the delete flag
        false,
        false
    ));

    assert_eq!(version, 1);
    assert_eq!(new_processed_files.len(), 7);
    assert_eq!(new_files_map.len(), 2);

    // first check all previous files were removed
    let file_path1 = "/test.md";
    assert_eq!(new_processed_files[file_path1].0, CONTENT_DELETED_SIGN);
    assert_eq!(
        new_processed_files[file_path1].1,
        files_map[file_path1][FAKE_RDF_PREDICATE_LINK]
    );

    let file_path2 = "/another.md";
    assert_eq!(new_processed_files[file_path2].0, CONTENT_DELETED_SIGN);
    assert_eq!(
        new_processed_files[file_path2].1,
        files_map[file_path2][FAKE_RDF_PREDICATE_LINK]
    );

    let file_path3 = "/subfolder/subexists.md";
    assert_eq!(new_processed_files[file_path3].0, CONTENT_DELETED_SIGN);
    assert_eq!(
        new_processed_files[file_path3].1,
        files_map[file_path3][FAKE_RDF_PREDICATE_LINK]
    );

    let file_path4 = "/noextension";
    assert_eq!(new_processed_files[file_path4].0, CONTENT_DELETED_SIGN);
    assert_eq!(
        new_processed_files[file_path4].1,
        files_map[file_path4][FAKE_RDF_PREDICATE_LINK]
    );

    // and finally check the synced file was added
    let filename5 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        new_processed_files[filename5].1,
        new_files_map["/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_sync_delete_without_recursive() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    match safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        "some-url",
        false, // this sets the recursive flag to off
        true,  // this sets the delete flag
        false,
        false,
    ) {
        Ok(_) => panic!("Sync was unexpectdly successful"),
        Err(err) => assert_eq!(
            err,
            Error::InvalidInput("'delete' is not allowed if --recursive is not set".to_string())
        ),
    };
}

#[test]
fn test_files_container_sync_update_nrs_unversioned_link() {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, _, _) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    let nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_content_version(None);
    let unversioned_link = unwrap!(xorurl_encoder.to_string());
    match safe.nrs_map_container_create(&nrsurl, &unversioned_link, false, true, false) {
        Ok(_) => panic!("NRS create was unexpectdly successful"),
        Err(err) => assert_eq!(
            err,
            Error::InvalidInput(format!(
                "The linked content (FilesContainer) is versionable, therefore NRS requires the link to specify a version: \"{}\"",
                unversioned_link
            ))
        ),
    };
}

#[test]
fn test_files_container_sync_update_nrs_with_xorurl() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));

    let (xorurl, _, _) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    match safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &xorurl,
        false,
        false,
        true, // this flag requests the update-nrs
        false,
    ) {
        Ok(_) => panic!("Sync was unexpectdly successful"),
        Err(err) => assert_eq!(
            err,
            Error::InvalidInput(
                "'update-nrs' is not allowed since the URL provided is not an NRS URL".to_string()
            )
        ),
    };
}

#[test]
fn test_files_container_sync_update_nrs_versioned_link() {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, _, _) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    let nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_content_version(Some(0));
    let _ = unwrap!(safe.nrs_map_container_create(
        &nrsurl,
        &unwrap!(xorurl_encoder.to_string()),
        false,
        true,
        false
    ));

    let _ = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &nrsurl,
        false,
        false,
        true, // this flag requests the update-nrs
        false,
    ));

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_content_version(Some(1));
    let (new_link, _) = unwrap!(safe.parse_and_resolve_url(&nrsurl));
    assert_eq!(
        unwrap!(new_link.to_string()),
        unwrap!(xorurl_encoder.to_string())
    );
}

#[test]
fn test_files_container_sync_target_path_without_trailing_slash() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_path("path/when/sync");
    let (version, new_processed_files, new_files_map) = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder",
        &unwrap!(xorurl_encoder.to_string()),
        true,
        false,
        false,
        false
    ));

    assert_eq!(version, 1);
    assert_eq!(new_processed_files.len(), 2);
    assert_eq!(new_files_map.len(), 7);

    let filename1 = "./tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename1].1,
        new_files_map["/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "./tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename2].1,
        new_files_map["/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename3].1,
        new_files_map["/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "./tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename4].1,
        new_files_map["/noextension"][FAKE_RDF_PREDICATE_LINK]
    );

    // and finally check the synced file is there
    let filename5 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        new_processed_files[filename5].1,
        new_files_map["/path/when/sync/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_sync_target_path_with_trailing_slash() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, processed_files, files_map) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    assert_eq!(processed_files.len(), 5);
    assert_eq!(files_map.len(), 5);
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    //let xorurl_with_path = format!("{}/path/when/sync/", xorurl);
    xorurl_encoder.set_path("/path/when/sync/");
    let (version, new_processed_files, new_files_map) = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder",
        &unwrap!(xorurl_encoder.to_string()),
        true,
        false,
        false,
        false,
    ));

    assert_eq!(version, 1);
    assert_eq!(new_processed_files.len(), 2);
    assert_eq!(new_files_map.len(), 7);

    let filename1 = "./tests/testfolder/test.md";
    assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename1].1,
        new_files_map["/test.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename2 = "./tests/testfolder/another.md";
    assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename2].1,
        new_files_map["/another.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename3 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename3].1,
        new_files_map["/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );

    let filename4 = "./tests/testfolder/noextension";
    assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        processed_files[filename4].1,
        new_files_map["/noextension"][FAKE_RDF_PREDICATE_LINK]
    );

    // and finally check the synced file is there
    let filename5 = "./tests/testfolder/subfolder/subexists.md";
    assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
    assert_eq!(
        new_processed_files[filename5].1,
        new_files_map["/path/when/sync/subfolder/subexists.md"][FAKE_RDF_PREDICATE_LINK]
    );
}

#[test]
fn test_files_container_get() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, _processed_files, files_map) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    let (version, fetched_files_map) = unwrap!(safe.files_container_get(&xorurl));

    assert_eq!(version, 0);
    assert_eq!(fetched_files_map.len(), 5);
    assert_eq!(files_map.len(), fetched_files_map.len());
    assert_eq!(files_map["/test.md"], fetched_files_map["/test.md"]);
    assert_eq!(files_map["/another.md"], fetched_files_map["/another.md"]);
    assert_eq!(
        files_map["/subfolder/subexists.md"],
        fetched_files_map["/subfolder/subexists.md"]
    );
    assert_eq!(files_map["/noextension"], fetched_files_map["/noextension"]);
}

#[test]
fn test_files_container_version() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, _, _) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    let (version, _) = unwrap!(safe.files_container_get(&xorurl));
    assert_eq!(version, 0);

    let (version, _, _) = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &xorurl,
        true,
        true, // this sets the delete flag,
        false,
        false,
    ));
    assert_eq!(version, 1);

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_content_version(None);
    let (version, _) = unwrap!(safe.files_container_get(&unwrap!(xorurl_encoder.to_string())));
    assert_eq!(version, 1);
}

#[test]
fn test_files_container_get_with_version() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, _processed_files, files_map) =
        unwrap!(safe.files_container_create("./tests/testfolder/", None, true, false));

    // let's create a new version of the files container
    let (_version, _new_processed_files, new_files_map) = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &xorurl,
        true,
        true, // this sets the delete flag
        false,
        false
    ));

    // let's fetch version 0
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_content_version(Some(0));
    let (version, v0_files_map) =
        unwrap!(safe.files_container_get(&unwrap!(xorurl_encoder.to_string())));

    assert_eq!(version, 0);
    assert_eq!(files_map, v0_files_map);
    // let's check that one of the files in v1 is still there
    let file_path1 = "/test.md";
    assert_eq!(
        files_map[file_path1][FAKE_RDF_PREDICATE_LINK],
        v0_files_map[file_path1][FAKE_RDF_PREDICATE_LINK]
    );

    // let's fetch version 1
    xorurl_encoder.set_content_version(Some(1));
    let (version, v1_files_map) =
        unwrap!(safe.files_container_get(&unwrap!(xorurl_encoder.to_string())));

    assert_eq!(version, 1);
    assert_eq!(new_files_map, v1_files_map);
    // let's check that some of the files are no in v2 anymore
    let file_path2 = "/another.md";
    let file_path3 = "/subfolder/subexists.md";
    let file_path4 = "/noextension";
    assert!(v1_files_map.get(file_path1).is_none());
    assert!(v1_files_map.get(file_path2).is_none());
    assert!(v1_files_map.get(file_path3).is_none());
    assert!(v1_files_map.get(file_path4).is_none());

    // let's fetch version 2 (invalid)
    xorurl_encoder.set_content_version(Some(2));
    match safe.files_container_get(&unwrap!(xorurl_encoder.to_string())) {
        Ok(_) => panic!("unexpectdly retrieved verion 3 of container"),
        Err(Error::VersionNotFound(msg)) => assert_eq!(
            msg,
            format!(
                "Version '2' is invalid for FilesContainer found at \"{}\"",
                xorurl_encoder
            )
        ),
        other => panic!(format!(
            "error returned is not the expected one: {:?}",
            other
        )),
    };
}

#[test]
fn test_files_container_sync_with_nrs_url() {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, _, _) =
        unwrap!(safe.files_container_create("./tests/testfolder/test.md", None, false, false));

    let nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_content_version(Some(0));
    let _ = unwrap!(safe.nrs_map_container_create(
        &nrsurl,
        &unwrap!(xorurl_encoder.to_string()),
        false,
        true,
        false
    ));

    let _ = unwrap!(safe.files_container_sync(
        "./tests/testfolder/subfolder/",
        &xorurl,
        false,
        false,
        false,
        false,
    ));

    let _ = unwrap!(safe.files_container_sync(
        "./tests/testfolder/",
        &nrsurl,
        false,
        false,
        true, // this flag requests the update-nrs
        false,
    ));

    let (version, fetched_files_map) = unwrap!(safe.files_container_get(&xorurl));
    assert_eq!(version, 2);
    assert_eq!(fetched_files_map.len(), 5);
}
