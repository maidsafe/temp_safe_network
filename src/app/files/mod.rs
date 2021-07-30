// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod file_system;
mod files_map;
mod metadata;
mod realpath;

use crate::{
    app::consts::*, app::nrs::VersionHash, fetch::Range, Error, Result, Safe, SafeContentType,
    SafeDataType, SafeUrl, XorUrl,
};
use file_system::{file_system_dir_walk, file_system_single_file, normalise_path_separator};
use files_map::add_or_update_file_item;
use log::{debug, info, warn};
use relative_path::RelativePath;
use std::collections::BTreeMap;
use std::{fs, path::Path};

pub(crate) use metadata::FileMeta;
pub(crate) use realpath::RealPath;

pub use files_map::{FileItem, FilesMap, GetAttr};

// List of files uploaded with details if they were added, updated or deleted from FilesContainer
pub type ProcessedFiles = BTreeMap<String, (String, String)>;

// Type tag to use for the FilesContainer stored on Register
const FILES_CONTAINER_TYPE_TAG: u64 = 1_100;

const ERROR_MSG_NO_FILES_CONTAINER_FOUND: &str = "No FilesContainer found at this address";

impl Safe {
    /// # Create a FilesContainer.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    ///     safe.connect(None, None, None).await.unwrap();
    ///     let (xorurl, _processed_files, _files_map) = safe.files_container_create(Some("../testdata"), None, true, true, false).await.unwrap();
    ///     assert!(xorurl.contains("safe://"))
    /// # });
    /// ```
    pub async fn files_container_create(
        &mut self,
        location: Option<&str>,
        dest: Option<&str>,
        recursive: bool,
        follow_links: bool,
        dry_run: bool,
    ) -> Result<(XorUrl, ProcessedFiles, FilesMap)> {
        // TODO: Enable source for funds / ownership
        // Warn about ownership?

        // Let's upload the files and generate the list of local files paths
        let (processed_files, files_map) = match location {
            Some(path) => {
                let mut processed_files =
                    file_system_dir_walk(self, path, recursive, follow_links, dry_run).await?;

                // The FilesContainer is stored on a Register
                // and the link to the serialised FilesMap as the entry's value
                // TODO: use RDF format
                let files_map = files_map_create(
                    self,
                    &mut processed_files,
                    path,
                    dest,
                    follow_links,
                    dry_run,
                )
                .await?;
                (processed_files, files_map)
            }
            None => (ProcessedFiles::default(), FilesMap::default()),
        };

        let xorurl = if dry_run {
            "".to_string()
        } else {
            // Store the serialised FilesMap in a Public Blob
            let files_map_xorurl = self.store_files_map(&files_map).await?;

            // Store the serialised FilesMap XOR-URL as the first entry value in the Register
            let xorname = self
                .safe_client
                .store_register(None, FILES_CONTAINER_TYPE_TAG, None, false)
                .await?;

            let xor_url = SafeUrl::encode_register(
                xorname,
                FILES_CONTAINER_TYPE_TAG,
                SafeContentType::FilesContainer,
                self.xorurl_base,
                false,
            )?;

            let _ = self
                .write_to_register(
                    &xor_url,
                    files_map_xorurl.as_bytes().to_vec(),
                    Default::default(),
                )
                .await?;

            xor_url
        };

        Ok((xorurl, processed_files, files_map))
    }

    /// # Fetch an existing FilesContainer.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let (xorurl, _processed_files, _files_map) = safe.files_container_create(Some("../testdata"), None, true, true, false).await.unwrap();
    ///     let (version, files_map) = safe.files_container_get(&xorurl).await.unwrap();
    ///     println!("FilesContainer fetched is at version: {}", version);
    ///     println!("FilesMap of fetched version is: {:?}", files_map);
    /// # });
    /// ```
    pub async fn files_container_get(&mut self, url: &str) -> Result<(VersionHash, FilesMap)> {
        debug!("Getting files container from: {:?}", url);
        let (safe_url, _) = self.parse_and_resolve_url(url).await?;

        self.fetch_files_container(&safe_url).await
    }

    /// Fetch a FilesContainer from a SafeUrl without performing any type of URL resolution
    pub(crate) async fn fetch_files_container(
        &self,
        safe_url: &SafeUrl,
    ) -> Result<(VersionHash, FilesMap)> {
        // fetch register entries and wrap errors
        let entries = self
            .fetch_register_entries(&safe_url)
            .await
            .map_err(|e| match e {
                Error::ContentNotFound(_) => {
                    Error::ContentNotFound(ERROR_MSG_NO_FILES_CONTAINER_FOUND.to_string())
                }
                Error::HashNotFound(_) => Error::VersionNotFound(format!(
                    "Version '{}' is invalid for FilesContainer found at \"{}\"",
                    match safe_url.content_version() {
                        Some(v) => v.to_string(),
                        None => "None".to_owned(),
                    },
                    safe_url
                )),
                err => Error::NetDataError(format!("Failed to get current version: {}", err)),
            })?;

        // take the 1st entry (TODO Multiple entries)
        if entries.len() > 1 {
            unimplemented!("Multiple file container entries not managed, this happends when 2 clients write concurrently to a file container");
        }
        let first_entry = entries.iter().next();
        let (version, curr_serialised_files_map);
        if let Some((v, m)) = first_entry {
            version = v.into();
            curr_serialised_files_map = m.to_owned();
        } else {
            warn!("FilesContainer found at \"{:?}\" was empty", safe_url);
            return Ok((VersionHash::default(), FilesMap::default()));
        }

        debug!("Files map retrieved.... v{:?}", &version);
        // TODO: use RDF format and deserialise it
        // We first obtain the FilesMap XOR-URL from the Sequence
        let files_map_xorurl = SafeUrl::from_url(
            &String::from_utf8(curr_serialised_files_map).map_err(|err| {
                Error::ContentError(format!(
                    "Couldn't parse the FilesMap link stored in the FilesContainer: {:?}",
                    err
                ))
            })?,
        )?;

        // Using the FilesMap XOR-URL we can now fetch the FilesMap and deserialise it
        let serialised_files_map = self.fetch_public_blob(&files_map_xorurl, None).await?;
        let files_map = serde_json::from_slice(serialised_files_map.as_slice()).map_err(|err| {
            Error::ContentError(format!(
                "Couldn't deserialise the FilesMap stored in the FilesContainer: {:?}",
                err
            ))
        })?;

        Ok((version, files_map))
    }

    /// # Sync up local folder with the content on a FilesContainer.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let (xorurl, _processed_files, _files_map) = safe.files_container_create(Some("../testdata"), None, true, false, false).await.unwrap();
    ///     let (version, new_processed_files, new_files_map) = safe.files_container_sync("../testdata", &xorurl, true, true, false, false, false).await.unwrap();
    ///     println!("FilesContainer synced up is at version: {}", version);
    ///     println!("The local files that were synced up are: {:?}", new_processed_files);
    ///     println!("The FilesMap of the updated FilesContainer now is: {:?}", new_files_map);
    /// # });
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub async fn files_container_sync(
        &mut self,
        location: &str,
        url: &str,
        recursive: bool,
        follow_links: bool,
        delete: bool,
        update_nrs: bool,
        dry_run: bool,
    ) -> Result<(VersionHash, ProcessedFiles, FilesMap)> {
        if delete && !recursive {
            return Err(Error::InvalidInput(
                "'delete' is not allowed if 'recursive' is not set".to_string(),
            ));
        }

        let safe_url = Safe::parse_url(url)?;
        if safe_url.content_version().is_some() {
            return Err(Error::InvalidInput(format!(
                "The target URL cannot contain a version: {}",
                url
            )));
        };

        // If NRS name shall be updated then the URL has to be an NRS-URL
        if update_nrs && safe_url.content_type() != SafeContentType::NrsMapContainer {
            return Err(Error::InvalidInput(
                "'update-nrs' is not allowed since the URL provided is not an NRS URL".to_string(),
            ));
        }

        let (mut safe_url, _) = self.parse_and_resolve_url(url).await?;

        // If the FilesContainer URL was resolved from an NRS name we need to remove
        // the version from it so we can fetch latest version of it for sync-ing
        safe_url.set_content_version(None);

        let (current_version, current_files_map): (VersionHash, FilesMap) =
            self.fetch_files_container(&safe_url).await?;

        // Let's generate the list of local files paths, without uploading any new file yet
        let processed_files =
            file_system_dir_walk(self, location, recursive, follow_links, true).await?;

        let dest_path = Some(safe_url.path());

        let (processed_files, new_files_map, success_count): (ProcessedFiles, FilesMap, u64) =
            files_map_sync(
                self,
                current_files_map,
                location,
                processed_files,
                dest_path,
                delete,
                dry_run,
                false,
                true,
                follow_links,
            )
            .await?;

        let version = self
            .append_version_to_files_container(
                success_count,
                current_version,
                &new_files_map,
                url,
                safe_url,
                dry_run,
                update_nrs,
            )
            .await?;

        Ok((version, processed_files, new_files_map))
    }

    /// # Add a file, either a local path or an already uploaded file, on an existing FilesContainer.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let (xorurl, _processed_files, _files_map) = safe.files_container_create(Some("../testdata"), None, true, true, false).await.unwrap();
    ///     let new_file_name = format!("{}/new_name_test.md", xorurl);
    ///     let (version, new_processed_files, new_files_map) = safe.files_container_add("../testdata/test.md", &new_file_name, false, false, true, false).await.unwrap();
    ///     println!("FilesContainer is now at version: {}", version);
    ///     println!("The local files that were synced up are: {:?}", new_processed_files);
    ///     println!("The FilesMap of the updated FilesContainer now is: {:?}", new_files_map);
    /// # });
    /// ```
    pub async fn files_container_add(
        &mut self,
        source_file: &str,
        url: &str,
        force: bool,
        update_nrs: bool,
        follow_links: bool,
        dry_run: bool,
    ) -> Result<(VersionHash, ProcessedFiles, FilesMap)> {
        let (safe_url, current_version, current_files_map) =
            validate_files_add_params(self, source_file, url, update_nrs).await?;

        let dest_path = safe_url.path();

        // Let's act according to if it's a local file path or a safe:// location
        let (processed_files, new_files_map, success_count) = if source_file.starts_with("safe://")
        {
            files_map_add_link(self, current_files_map, source_file, dest_path, force).await?
        } else {
            // Let's generate the list of local files paths, without uploading any new file yet
            let processed_files = file_system_single_file(self, source_file, true).await?;

            files_map_sync(
                self,
                current_files_map,
                source_file,
                processed_files,
                Some(dest_path),
                false,
                dry_run,
                force,
                false,
                follow_links,
            )
            .await?
        };

        let version = self
            .append_version_to_files_container(
                success_count,
                current_version,
                &new_files_map,
                url,
                safe_url,
                dry_run,
                update_nrs,
            )
            .await?;

        Ok((version, processed_files, new_files_map))
    }

    /// # Add a file, from raw bytes, on an existing FilesContainer.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let (xorurl, _processed_files, _files_map) = safe.files_container_create(Some("../testdata"), None, true, true, false).await.unwrap();
    ///     let new_file_name = format!("{}/new_name_test.md", xorurl);
    ///     let (version, new_processed_files, new_files_map) = safe.files_container_add_from_raw(b"0123456789", &new_file_name, false, false, false).await.unwrap();
    ///     println!("FilesContainer is now at version: {}", version);
    ///     println!("The local files that were synced up are: {:?}", new_processed_files);
    ///     println!("The FilesMap of the updated FilesContainer now is: {:?}", new_files_map);
    /// # });
    /// ```
    pub async fn files_container_add_from_raw(
        &mut self,
        data: &[u8],
        url: &str,
        force: bool,
        update_nrs: bool,
        dry_run: bool,
    ) -> Result<(VersionHash, ProcessedFiles, FilesMap)> {
        let (safe_url, current_version, current_files_map) =
            validate_files_add_params(self, "", url, update_nrs).await?;

        let dest_path = safe_url.path();
        let new_file_xorurl = self.files_store_public_blob(data, None, false).await?;

        // Let's act according to if it's a local file path or a safe:// location
        let (processed_files, new_files_map, success_count) =
            files_map_add_link(self, current_files_map, &new_file_xorurl, dest_path, force).await?;
        let version = self
            .append_version_to_files_container(
                success_count,
                current_version,
                &new_files_map,
                url,
                safe_url,
                dry_run,
                update_nrs,
            )
            .await?;

        Ok((version, processed_files, new_files_map))
    }

    /// # Remove a file from an existing FilesContainer.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let (xorurl, processed_files, files_map) = safe.files_container_create(Some("../testdata/"), None, true, true, false).await.unwrap();
    ///     let remote_file_path = format!("{}/test.md", xorurl);
    ///     let (version, new_processed_files, new_files_map) = safe.files_container_remove_path(&remote_file_path, false, false, false).await.unwrap();
    ///     println!("FilesContainer is now at version: {}", version);
    ///     println!("The files that were removed: {:?}", new_processed_files);
    ///     println!("The FilesMap of the updated FilesContainer now is: {:?}", new_files_map);
    /// # });
    /// ```
    pub async fn files_container_remove_path(
        &mut self,
        url: &str,
        recursive: bool,
        update_nrs: bool,
        dry_run: bool,
    ) -> Result<(VersionHash, ProcessedFiles, FilesMap)> {
        let safe_url = Safe::parse_url(url)?;
        if safe_url.content_version().is_some() {
            return Err(Error::InvalidInput(format!(
                "The target URL cannot contain a version: {}",
                url
            )));
        };

        let dest_path = safe_url.path();
        if dest_path.is_empty() {
            return Err(Error::InvalidInput(
                "The destination URL should include a target file path".to_string(),
            ));
        }

        // If NRS name shall be updated then the URL has to be an NRS-URL
        if update_nrs && safe_url.content_type() != SafeContentType::NrsMapContainer {
            return Err(Error::InvalidInput(
                "'update-nrs' is not allowed since the URL provided is not an NRS URL".to_string(),
            ));
        }

        let (mut safe_url, _) = self.parse_and_resolve_url(url).await?;

        // If the FilesContainer URL was resolved from an NRS name we need to remove
        // the version from it so we can fetch latest version of it
        safe_url.set_content_version(None);

        let (current_version, files_map): (VersionHash, FilesMap) =
            self.fetch_files_container(&safe_url).await?;

        let (processed_files, new_files_map, success_count) =
            files_map_remove_path(dest_path, files_map, recursive)?;

        let version = self
            .append_version_to_files_container(
                success_count,
                current_version,
                &new_files_map,
                url,
                safe_url,
                dry_run,
                update_nrs,
            )
            .await?;

        Ok((version, processed_files, new_files_map))
    }

    // Private helper function to append new version of the FilesMap to the Files Container
    // It flagged with `update_nrs`, it will also update the link in the corresponding NRS Map Container
    #[allow(clippy::too_many_arguments)]
    async fn append_version_to_files_container(
        &mut self,
        success_count: u64,
        current_version: VersionHash,
        new_files_map: &FilesMap,
        url: &str,
        mut safe_url: SafeUrl,
        dry_run: bool,
        update_nrs: bool,
    ) -> Result<VersionHash> {
        let version = if success_count == 0 || dry_run {
            current_version
        } else {
            // The FilesContainer is updated by adding an entry containing the link to
            // the Blob with the serialised new version of the FilesMap.
            let files_map_xorurl = self.store_files_map(new_files_map).await?;

            // append entry to register
            let entry = files_map_xorurl.as_bytes().to_vec();
            let replace = self
                .register_read(&url)
                .await?
                .iter()
                .map(|(h, _)| h.to_owned())
                .collect();
            let entry_hash = &self.write_to_register(&url, entry, replace).await?;
            let new_version: VersionHash = entry_hash.into();

            if update_nrs {
                // We need to update the link in the NRS container as well,
                // to link it to the new new_version of the FilesContainer we just generated
                safe_url.set_content_version(Some(new_version));
                let new_link_for_nrs = safe_url.to_string();
                let _ = self
                    .nrs_map_container_add(url, &new_link_for_nrs, false, true, false)
                    .await?;
            }

            new_version
        };

        Ok(version)
    }

    /// # Put a Public Blob
    /// Put data blobs onto the network.
    ///
    /// ## Example
    /// ```no_run
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let data = b"Something super good";
    ///     let xorurl = safe.files_store_public_blob(data, Some("text/plain"), false).await.unwrap();
    ///     let received_data = safe.files_get_public_blob(&xorurl, None).await.unwrap();
    ///     assert_eq!(received_data, data);
    /// # });
    /// ```
    pub async fn files_store_public_blob(
        &self,
        data: &[u8],
        media_type: Option<&str>,
        dry_run: bool,
    ) -> Result<XorUrl> {
        let content_type = media_type.map_or_else(
            || Ok(SafeContentType::Raw),
            |media_type_str| {
                if SafeUrl::is_media_type_supported(media_type_str) {
                    Ok(SafeContentType::MediaType(media_type_str.to_string()))
                } else {
                    Err(Error::InvalidMediaType(format!(
                        "Media-type '{}' not supported. You can pass 'None' as the 'media_type' for this content to be treated as raw",
                        media_type_str
                    )))
                }
            },
        )?;

        // TODO: do we want ownership from other PKs yet?
        let xorname = self.safe_client.store_public_blob(data, dry_run).await?;

        let xorurl = SafeUrl::encode_blob(xorname, content_type, self.xorurl_base)?;

        Ok(xorurl)
    }

    /// # Get a Public Blob
    /// Get blob from the network.
    ///
    /// ## Example
    /// ```no_run
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let data = b"Something super good";
    ///     let xorurl = safe.files_store_public_blob(data, None, false).await.unwrap();
    ///     let received_data = safe.files_get_public_blob(&xorurl, None).await.unwrap();
    ///     assert_eq!(received_data, data);
    /// # });
    /// ```
    pub async fn files_get_public_blob(&mut self, url: &str, range: Range) -> Result<Vec<u8>> {
        // TODO: do we want ownership from other PKs yet?
        let (safe_url, _) = self.parse_and_resolve_url(url).await?;
        self.fetch_public_blob(&safe_url, range).await
    }

    /// Fetch an Blob from a SafeUrl without performing any type of URL resolution
    pub(crate) async fn fetch_public_blob(
        &self,
        safe_url: &SafeUrl,
        range: Range,
    ) -> Result<Vec<u8>> {
        self.safe_client
            .get_public_blob(safe_url.xorname(), range)
            .await
    }

    // Private helper to serialise a FilesMap and store it in a Public Blob
    async fn store_files_map(&mut self, files_map: &FilesMap) -> Result<String> {
        // The FilesMapContainer is a Register where each NRS Map version is
        // an entry containing the XOR-URL of the Blob that contains the serialised NrsMap.
        // TODO: use RDF format
        let serialised_files_map = serde_json::to_string(&files_map).map_err(|err| {
            Error::Serialisation(format!(
                "Couldn't serialise the FilesMap generated: {:?}",
                err
            ))
        })?;
        let files_map_xorurl = self
            .files_store_public_blob(serialised_files_map.as_bytes(), None, false)
            .await?;

        Ok(files_map_xorurl)
    }
}

// Helper functions

// Make sure the input params are valid for a files_container_add operation
async fn validate_files_add_params(
    safe: &mut Safe,
    source_file: &str,
    url: &str,
    update_nrs: bool,
) -> Result<(SafeUrl, VersionHash, FilesMap)> {
    let safe_url = Safe::parse_url(url)?;
    if safe_url.content_version().is_some() {
        return Err(Error::InvalidInput(format!(
            "The target URL cannot contain a version: {}",
            url
        )));
    };

    // If NRS name shall be updated then the URL has to be an NRS-URL
    if update_nrs && safe_url.content_type() != SafeContentType::NrsMapContainer {
        return Err(Error::InvalidInput(
            "'update-nrs' is not allowed since the URL provided is not an NRS URL".to_string(),
        ));
    }

    let (mut safe_url, _) = safe.parse_and_resolve_url(url).await?;

    // If the FilesContainer URL was resolved from an NRS name we need to remove
    // the version from it so we can fetch latest version of it for sync-ing
    safe_url.set_content_version(None);

    let (current_version, current_files_map): (VersionHash, FilesMap) =
        safe.fetch_files_container(&safe_url).await?;

    let dest_path = safe_url.path().to_string();

    // Let's act according to if it's a local file path or a safe:// location
    if source_file.starts_with("safe://") {
        let source_safe_url = Safe::parse_url(source_file)?;
        if source_safe_url.data_type() != SafeDataType::PublicBlob {
            return Err(Error::InvalidInput(format!(
                "The source URL should target a file ('{}'), but the URL provided targets a '{}'",
                SafeDataType::PublicBlob,
                source_safe_url.content_type()
            )));
        }

        if dest_path.is_empty() {
            return Err(Error::InvalidInput(
                "The destination URL should include a target file path since we are adding a link"
                    .to_string(),
            ));
        }
    }
    Ok((safe_url, current_version, current_files_map))
}

// From the location path and the destination path chosen by the user, calculate
// the destination path considering ending '/' in both the  location and dest path
fn get_base_paths(location: &str, dest_path: Option<&str>) -> (String, String) {
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
                path.to_string()
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

    (location_base_path, dest_base_path)
}

// From the provided list of local files paths, find the local changes made in comparison with the
// target FilesContainer, uploading new files as necessary, and creating a new FilesMap with file's
// metadata and their corresponding links, as well as generating the report of processed files
#[allow(clippy::too_many_arguments)]
async fn files_map_sync(
    safe: &mut Safe,
    mut current_files_map: FilesMap,
    location: &str,
    new_content: ProcessedFiles,
    dest_path: Option<&str>,
    delete: bool,
    dry_run: bool,
    force: bool,
    compare_file_content: bool,
    follow_links: bool,
) -> Result<(ProcessedFiles, FilesMap, u64)> {
    let (location_base_path, dest_base_path) = get_base_paths(location, dest_path);
    let mut updated_files_map = FilesMap::new();
    let mut processed_files = ProcessedFiles::new();
    let mut success_count = 0;

    for (local_file_name, _) in new_content
        .iter()
        .filter(|(_, (change, _))| change != CONTENT_ERROR_SIGN)
    {
        let file_path = Path::new(&local_file_name);

        let file_name = RelativePath::new(
            &local_file_name
                .to_string()
                .replace(&location_base_path, &dest_base_path),
        )
        .normalize();
        // Above normalize removes initial slash, and uses '\' if it's on Windows
        // here, we trim any trailing '/', as it could be a filename.
        let mut normalised_file_name = format!("/{}", normalise_path_separator(file_name.as_str()))
            .trim_end_matches('/')
            .to_string();

        if normalised_file_name.is_empty() {
            normalised_file_name = "/".to_string();
        }

        // Let's update FileItem if there is a change or it doesn't exist in current_files_map
        match current_files_map.get(&normalised_file_name) {
            None => {
                // We need to add a new FileItem
                if add_or_update_file_item(
                    safe,
                    local_file_name,
                    &normalised_file_name,
                    file_path,
                    &FileMeta::from_path(local_file_name, follow_links)?,
                    None, // no xorurl link
                    false,
                    dry_run,
                    &mut updated_files_map,
                    &mut processed_files,
                )
                .await
                {
                    success_count += 1;

                    // We remove self and any parent directories
                    // from the current list so we know it has been processed
                    let mut trail = Vec::<&str>::new();
                    for part in normalised_file_name.split('/') {
                        trail.push(part);
                        let ancestor = if trail.len() > 1 {
                            trail.join("/")
                        } else {
                            "/".to_string()
                        };
                        if ancestor != normalised_file_name {
                            if let Some(fi) = current_files_map.get(&ancestor) {
                                updated_files_map.insert(ancestor.clone(), fi.clone());
                                current_files_map.remove(&ancestor);
                            }
                        }
                    }
                }
            }
            Some(file_item) => {
                let is_modified =
                    is_file_item_modified(safe, Path::new(local_file_name), file_item).await;
                if force || (compare_file_content && is_modified) {
                    // We need to update the current FileItem
                    if add_or_update_file_item(
                        safe,
                        local_file_name,
                        &normalised_file_name,
                        file_path,
                        &FileMeta::from_path(local_file_name, follow_links)?,
                        None, // no xorurl link
                        true,
                        dry_run,
                        &mut updated_files_map,
                        &mut processed_files,
                    )
                    .await
                    {
                        success_count += 1;
                    }
                } else {
                    // No need to update FileItem just copy the existing one
                    updated_files_map.insert(normalised_file_name.to_string(), file_item.clone());

                    if !force && !compare_file_content {
                        let comp_str = if is_modified { "different" } else { "same" };
                        processed_files.insert(
                            local_file_name.to_string(),
                            (
                                CONTENT_ERROR_SIGN.to_string(),
                                format!(
                                    "File named \"{}\" with {} content already exists on target. Use the 'force' flag to replace it",
                                    normalised_file_name, comp_str
                                ),
                            ),
                        );
                        info!("Skipping file \"{}\" since a file named \"{}\" with {} content already exists on target. You can use the 'force' flag to replace the existing file with the new one", local_file_name, normalised_file_name, comp_str);
                    }
                }

                // let's now remove it from the current list so we now it has been processed
                current_files_map.remove(&normalised_file_name);

                // We also remove any parent directories
                // from the current list, so they will not be deleted.
                let mut trail = Vec::<&str>::new();
                for part in normalised_file_name.split('/') {
                    trail.push(part);
                    let ancestor = if trail.len() > 1 {
                        trail.join("/")
                    } else {
                        "/".to_string()
                    };
                    if ancestor != normalised_file_name {
                        if let Some(fi) = current_files_map.get(&ancestor) {
                            updated_files_map.insert(ancestor.clone(), fi.clone());
                            current_files_map.remove(&ancestor);
                        }
                    }
                }
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
                    // note: files have link property,
                    //       dirs and symlinks do not
                    file_item
                        .get(PREDICATE_LINK)
                        .unwrap_or(&String::default())
                        .to_string(),
                ),
            );
            success_count += 1;
        }
    });

    Ok((processed_files, updated_files_map, success_count))
}

async fn is_file_item_modified(
    safe: &mut Safe,
    local_filename: &Path,
    file_item: &FileItem,
) -> bool {
    if FileMeta::filetype_is_file(&file_item[PREDICATE_TYPE]) {
        match upload_file_to_net(safe, local_filename, true /* dry-run */).await {
            Ok(local_xorurl) => file_item[PREDICATE_LINK] != local_xorurl,
            Err(_err) => false,
        }
    } else {
        // for now, we just return false if a symlink or directory.
        // In the future, should check if symlink has been modified.
        // Also, could check if ctime or mtime is different, though that
        // could apply to files as well, and some use-cases would not want to
        // sync remotely if actual content has not changed.  So it should
        // probably be a user flag to enable.
        false
    }
}

async fn files_map_add_link(
    safe: &mut Safe,
    mut files_map: FilesMap,
    file_link: &str,
    file_name: &str,
    force: bool,
) -> Result<(ProcessedFiles, FilesMap, u64)> {
    let mut processed_files = ProcessedFiles::new();
    let mut success_count = 0;
    match SafeUrl::from_url(file_link) {
        Err(err) => {
            processed_files.insert(
                file_link.to_string(),
                (CONTENT_ERROR_SIGN.to_string(), format!("<{}>", err)),
            );
            info!("Skipping file \"{}\". {}", file_link, err);
            Ok((processed_files, files_map, success_count))
        }
        Ok(safe_url) => {
            let file_path = Path::new("");
            let file_type = match safe_url.content_type() {
                SafeContentType::MediaType(media_type) => media_type,
                other => format!("{}", other),
            };
            let file_size = ""; // unknown

            // Let's update FileItem if the link is different or it doesn't exist in the files_map
            match files_map.get(file_name) {
                Some(current_file_item) => {
                    let mut file_meta = FileMeta::from_file_item(current_file_item);
                    file_meta.file_type = file_type;
                    file_meta.file_size = file_size.to_string();

                    let is_modified = if file_meta.is_file() {
                        current_file_item[PREDICATE_LINK] != file_link
                    } else {
                        // directory: nothing to check.
                        // symlink: TODO: check if sym-link path has changed.
                        false
                    };

                    if is_modified {
                        if force {
                            if add_or_update_file_item(
                                safe,
                                file_name,
                                file_name,
                                file_path,
                                &file_meta,
                                Some(file_link),
                                true,
                                true,
                                &mut files_map,
                                &mut processed_files,
                            )
                            .await
                            {
                                success_count += 1;
                            }
                        } else {
                            processed_files.insert(file_name.to_string(), (CONTENT_ERROR_SIGN.to_string(), format!("File named \"{}\" already exists on target. Use the 'force' flag to replace it", file_name)));
                            info!("Skipping file \"{}\" since a file with name \"{}\" already exists on target. You can use the 'force' flag to replace the existing file with the new one", file_link, file_name);
                        }
                    } else {
                        processed_files.insert(
                            file_link.to_string(),
                            (
                                CONTENT_ERROR_SIGN.to_string(),
                                format!(
                                    "File named \"{}\" already exists on target with same link",
                                    file_name
                                ),
                            ),
                        );
                        info!("Skipping file \"{}\" since a file with name \"{}\" already exists on target with the same link", file_link, file_name);
                    }
                }
                None => {
                    if add_or_update_file_item(
                        safe,
                        file_name,
                        file_name,
                        file_path,
                        &FileMeta::from_type_and_size(&file_type, file_size),
                        Some(file_link),
                        false,
                        true,
                        &mut files_map,
                        &mut processed_files,
                    )
                    .await
                    {
                        success_count += 1;
                    }
                }
            };

            Ok((processed_files, files_map, success_count))
        }
    }
}

// Remove a path from the FilesMap provided
fn files_map_remove_path(
    dest_path: &str,
    mut files_map: FilesMap,
    recursive: bool,
) -> Result<(ProcessedFiles, FilesMap, u64)> {
    let mut processed_files = ProcessedFiles::default();
    let (success_count, new_files_map) = if recursive {
        let mut success_count = 0;
        let mut new_files_map = FilesMap::default();
        let folder_path = if !dest_path.ends_with('/') {
            format!("{}/", dest_path)
        } else {
            dest_path.to_string()
        };

        files_map.iter().for_each(|(file_path, file_item)| {
            // if the current file_path is a subfolder we remove it
            if file_path.starts_with(&folder_path) {
                processed_files.insert(
                    file_path.to_string(),
                    (
                        CONTENT_DELETED_SIGN.to_string(),
                        // note: files have link property,
                        //       dirs and symlinks do not
                        file_item
                            .get(PREDICATE_LINK)
                            .unwrap_or(&String::default())
                            .to_string(),
                    ),
                );
                success_count += 1;
            } else {
                new_files_map.insert(file_path.to_string(), file_item.clone());
            }
        });
        (success_count, new_files_map)
    } else {
        let file_item = files_map
            .remove(dest_path)
            .ok_or_else(|| Error::ContentError(format!(
                "No content found matching the \"{}\" path on the target FilesContainer. If you are trying to remove a folder rather than a file, you need to pass the 'recursive' flag",
                dest_path
            )))?;
        processed_files.insert(
            dest_path.to_string(),
            (
                CONTENT_DELETED_SIGN.to_string(),
                // note: files have link property,
                //       dirs and symlinks do not
                file_item
                    .get(PREDICATE_LINK)
                    .unwrap_or(&String::default())
                    .to_string(),
            ),
        );
        (1, files_map)
    };

    Ok((processed_files, new_files_map, success_count))
}

// Upload a files to the Network as a Public Blob
async fn upload_file_to_net(safe: &mut Safe, path: &Path, dry_run: bool) -> Result<XorUrl> {
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

// From the provided list of local files paths and corresponding files XOR-URLs,
// create a FilesMap with file's metadata and their corresponding links
async fn files_map_create(
    safe: &mut Safe,
    mut content: &mut ProcessedFiles,
    location: &str,
    dest_path: Option<&str>,
    follow_links: bool,
    dry_run: bool,
) -> Result<FilesMap> {
    let mut files_map = FilesMap::default();

    let (location_base_path, dest_base_path) = get_base_paths(location, dest_path);

    // We want to iterate over the BTreeMap and also modify it.
    // We DON'T want to clone/dup the whole thing, might be very big.
    // Rust doesn't allow that exactly, but we can get the keys
    // to iterate over instead.  Cloning the keys isn't ideal
    // either, but is much less data.  Is there a more efficient way?
    let keys = content.keys().cloned().collect::<Vec<_>>();
    for file_name in keys {
        let (change, link) = &content[&file_name].clone();

        if change == CONTENT_ERROR_SIGN {
            continue;
        }

        let new_file_name = RelativePath::new(
            &file_name
                .to_string()
                .replace(&location_base_path, &dest_base_path),
        )
        .normalize();

        // Above normalize removes initial slash, and uses '\' if it's on Windows
        // here, we trim any trailing '/', as it could be a filename.
        let final_name = format!("/{}", normalise_path_separator(new_file_name.as_str()))
            .trim_end_matches('/')
            .to_string();

        debug!("FileItem item name: {:?}", &file_name);

        add_or_update_file_item(
            safe,
            &file_name,
            &final_name,
            Path::new(&file_name),
            &FileMeta::from_path(&file_name, follow_links)?,
            if link.is_empty() { None } else { Some(link) },
            false,
            dry_run,
            &mut files_map,
            &mut content,
        )
        .await;
    }
    Ok(files_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::test_helpers::{new_safe_instance, random_nrs_name},
        retry_loop, retry_loop_for_pattern,
    };
    use anyhow::{anyhow, bail, Result};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    // make some constants for these, in case entries in the
    // testdata folder change.
    const TESTDATA_PUT_FILEITEM_COUNT: usize = 11;
    const TESTDATA_NO_SLASH_PUT_FILEITEM_COUNT: usize = 12;
    const SUBFOLDER_PUT_FILEITEM_COUNT: usize = 2;
    const SUBFOLDER_NO_SLASH_PUT_FILEITEM_COUNT: usize = 3;

    #[tokio::test]
    async fn test_files_map_create() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let mut processed_files = ProcessedFiles::new();
        let first_xorurl = SafeUrl::from_url("safe://top_xorurl")?.to_xorurl_string();
        let second_xorurl = SafeUrl::from_url("safe://second_xorurl")?.to_xorurl_string();

        processed_files.insert(
            "../testdata/test.md".to_string(),
            (CONTENT_ADDED_SIGN.to_string(), first_xorurl.clone()),
        );
        processed_files.insert(
            "../testdata/subfolder/subexists.md".to_string(),
            (CONTENT_ADDED_SIGN.to_string(), second_xorurl.clone()),
        );
        let files_map = files_map_create(
            &mut safe,
            &mut processed_files,
            "../testdata",
            Some(""),
            true,
            false,
        )
        .await?;
        assert_eq!(files_map.len(), 2);
        let file_item1 = &files_map["/testdata/test.md"];
        assert_eq!(file_item1[PREDICATE_LINK], first_xorurl);
        assert_eq!(file_item1[PREDICATE_TYPE], "text/markdown");
        assert_eq!(file_item1[PREDICATE_SIZE], "12");

        let file_item2 = &files_map["/testdata/subfolder/subexists.md"];
        assert_eq!(file_item2[PREDICATE_LINK], second_xorurl);
        assert_eq!(file_item2[PREDICATE_TYPE], "text/markdown");
        assert_eq!(file_item2[PREDICATE_SIZE], "23");
        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_create_empty() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(None, None, false, false, false)
            .await?;

        assert!(xorurl.starts_with("safe://"));
        assert_eq!(processed_files.len(), 0);
        assert_eq!(files_map.len(), 0);

        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        // let's add a file
        let (version, new_processed_files, new_files_map) = safe
            .files_container_add("../testdata/test.md", &xorurl, false, false, false, false)
            .await?;

        assert!(version != VersionHash::default());
        assert!(version != version0);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), 1);

        let filename = "../testdata/test.md";
        assert_eq!(new_processed_files[filename].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[filename].1,
            new_files_map["/test.md"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_store_pub_blob() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let random_blob_content: String =
            thread_rng().sample_iter(&Alphanumeric).take(20).collect();

        let file_xorurl = safe
            .files_store_public_blob(random_blob_content.as_bytes(), None, false)
            .await?;

        let retrieved = retry_loop!(safe.files_get_public_blob(&file_xorurl, None));
        assert_eq!(retrieved, random_blob_content.as_bytes());

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_create_file() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let filename = "../testdata/test.md";
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(Some(filename), None, false, false, false)
            .await?;

        assert!(xorurl.starts_with("safe://"));
        assert_eq!(processed_files.len(), 1);
        assert_eq!(files_map.len(), 1);
        let file_path = "/test.md";
        assert_eq!(processed_files[filename].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename].1,
            files_map[file_path][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_create_dry_run() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let filename = "../testdata/";
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(Some(filename), None, true, false, true)
            .await?;

        assert!(xorurl.is_empty());
        assert_eq!(processed_files.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert!(!processed_files[filename1].1.is_empty());
        assert_eq!(
            processed_files[filename1].1,
            files_map["/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert!(!processed_files[filename2].1.is_empty());
        assert_eq!(
            processed_files[filename2].1,
            files_map["/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert!(!processed_files[filename3].1.is_empty());
        assert_eq!(
            processed_files[filename3].1,
            files_map["/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert!(!processed_files[filename4].1.is_empty());
        assert_eq!(
            processed_files[filename4].1,
            files_map["/noextension"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_create_folder_without_trailing_slash() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) =
            retry_loop!(safe.files_container_create(Some("../testdata"), None, true, true, false));

        assert!(xorurl.starts_with("safe://"));
        assert_eq!(processed_files.len(), TESTDATA_NO_SLASH_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_NO_SLASH_PUT_FILEITEM_COUNT);

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            files_map["/testdata/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            files_map["/testdata/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename3].1,
            files_map["/testdata/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename4].1,
            files_map["/testdata/noextension"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_create_folder_with_trailing_slash() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));

        assert!(xorurl.starts_with("safe://"));
        assert_eq!(processed_files.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            files_map["/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            files_map["/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename3].1,
            files_map["/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename4].1,
            files_map["/noextension"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_create_dest_path_without_trailing_slash() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(Some("../testdata"), Some("/myroot"), true, true, false)
            .await?;

        assert!(xorurl.starts_with("safe://"));
        assert_eq!(processed_files.len(), TESTDATA_NO_SLASH_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_NO_SLASH_PUT_FILEITEM_COUNT);

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            files_map["/myroot/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            files_map["/myroot/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename3].1,
            files_map["/myroot/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename4].1,
            files_map["/myroot/noextension"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_create_dest_path_with_trailing_slash() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(Some("../testdata"), Some("/myroot/"), true, true, false)
            .await?;

        assert!(xorurl.starts_with("safe://"));
        assert_eq!(processed_files.len(), TESTDATA_NO_SLASH_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_NO_SLASH_PUT_FILEITEM_COUNT);

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            files_map["/myroot/testdata/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            files_map["/myroot/testdata/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename3].1,
            files_map["/myroot/testdata/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename4].1,
            files_map["/myroot/testdata/noextension"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_sync() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(Some("../testdata/"), None, true, true, false)
            .await?;

        assert_eq!(processed_files.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);

        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        let (version, new_processed_files, new_files_map) = safe
            .files_container_sync(
                "../testdata/subfolder/",
                &xorurl,
                true,
                true,
                false,
                false,
                false,
            )
            .await?;

        assert!(version != version0);
        assert_eq!(new_processed_files.len(), 2);
        assert_eq!(
            new_files_map.len(),
            TESTDATA_PUT_FILEITEM_COUNT + SUBFOLDER_PUT_FILEITEM_COUNT
        );

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            new_files_map["/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            new_files_map["/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename3].1,
            new_files_map["/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename4].1,
            new_files_map["/noextension"][PREDICATE_LINK]
        );

        let filename5 = "../testdata/subfolder/subexists.md";
        assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[filename5].1,
            new_files_map["/subexists.md"][PREDICATE_LINK]
        );

        let filename6 = "../testdata/subfolder/sub2.md";
        assert_eq!(new_processed_files[filename6].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[filename6].1,
            new_files_map["/sub2.md"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_sync_dry_run() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(Some("../testdata/"), None, true, true, false)
            .await?;

        assert_eq!(processed_files.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);

        let _ = retry_loop!(safe.fetch(&xorurl, None));

        let (version, new_processed_files, new_files_map) = safe
            .files_container_sync(
                "../testdata/subfolder/",
                &xorurl,
                true,
                true,
                false,
                false,
                true, // set dry_run flag on
            )
            .await?;

        assert!(version != VersionHash::default());
        assert_eq!(new_processed_files.len(), 2);
        assert_eq!(
            new_files_map.len(),
            TESTDATA_PUT_FILEITEM_COUNT + SUBFOLDER_PUT_FILEITEM_COUNT
        );

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            new_files_map["/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            new_files_map["/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename3].1,
            new_files_map["/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename4].1,
            new_files_map["/noextension"][PREDICATE_LINK]
        );

        let filename5 = "../testdata/subfolder/subexists.md";
        assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
        assert!(!new_processed_files[filename5].1.is_empty());
        assert_eq!(
            new_processed_files[filename5].1,
            new_files_map["/subexists.md"][PREDICATE_LINK]
        );

        let filename6 = "../testdata/subfolder/sub2.md";
        assert_eq!(new_processed_files[filename6].0, CONTENT_ADDED_SIGN);
        assert!(!new_processed_files[filename6].1.is_empty());
        assert_eq!(
            new_processed_files[filename6].1,
            new_files_map["/sub2.md"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_sync_same_size() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(Some("../testdata/test.md"), None, false, false, false)
            .await?;

        assert_eq!(processed_files.len(), 1);
        assert_eq!(files_map.len(), 1);

        let _ = retry_loop!(safe.fetch(&xorurl, None));

        let (version, new_processed_files, new_files_map) = safe
            .files_container_sync(
                "../testdata/.subhidden/test.md",
                &xorurl,
                false,
                false,
                false,
                false,
                false,
            )
            .await?;

        assert!(version != VersionHash::default());
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), 1);

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            files_map["/test.md"][PREDICATE_LINK]
        );
        let filename2 = "../testdata/.subhidden/test.md";
        assert_eq!(new_processed_files[filename2].0, CONTENT_UPDATED_SIGN);
        assert_eq!(
            new_processed_files[filename2].1,
            new_files_map["/test.md"][PREDICATE_LINK]
        );

        // check sizes are the same but links are different
        assert_eq!(
            files_map["/test.md"][PREDICATE_SIZE],
            new_files_map["/test.md"][PREDICATE_SIZE]
        );
        assert_ne!(
            files_map["/test.md"][PREDICATE_LINK],
            new_files_map["/test.md"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_sync_with_versioned_target() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _, _) = safe
            .files_container_create(Some("../testdata/"), None, true, true, false)
            .await?;

        let _ = retry_loop!(safe.fetch(&xorurl, None));

        let versioned_xorurl = format!("{}?v={}", xorurl, VersionHash::default());
        match safe
            .files_container_sync(
                "../testdata/subfolder/",
                &versioned_xorurl,
                false,
                false,
                false,
                true, // this flag requests the update-nrs
                false,
            )
            .await
        {
            Ok(_) => Err(anyhow!("Sync was unexpectedly successful".to_string(),)),
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(
                    msg,
                    format!(
                        "The target URL cannot contain a version: {}",
                        versioned_xorurl
                    )
                );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_files_container_sync_with_delete() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = safe
            .files_container_create(Some("../testdata/"), None, true, true, false)
            .await?;

        assert_eq!(processed_files.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);

        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        let (version1, new_processed_files, new_files_map) = safe
            .files_container_sync(
                "../testdata/subfolder/",
                &xorurl,
                true,
                false,
                true, // this sets the delete flag
                false,
                false,
            )
            .await?;

        assert!(version1 != version0);
        assert_eq!(
            new_processed_files.len(),
            TESTDATA_PUT_FILEITEM_COUNT + SUBFOLDER_PUT_FILEITEM_COUNT
        );
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);

        // first check all previous files were removed
        let file_path1 = "/test.md";
        assert_eq!(new_processed_files[file_path1].0, CONTENT_DELETED_SIGN);
        assert_eq!(
            new_processed_files[file_path1].1,
            files_map[file_path1][PREDICATE_LINK]
        );

        let file_path2 = "/another.md";
        assert_eq!(new_processed_files[file_path2].0, CONTENT_DELETED_SIGN);
        assert_eq!(
            new_processed_files[file_path2].1,
            files_map[file_path2][PREDICATE_LINK]
        );

        let file_path3 = "/subfolder/subexists.md";
        assert_eq!(new_processed_files[file_path3].0, CONTENT_DELETED_SIGN);
        assert_eq!(
            new_processed_files[file_path3].1,
            files_map[file_path3][PREDICATE_LINK]
        );

        let file_path4 = "/noextension";
        assert_eq!(new_processed_files[file_path4].0, CONTENT_DELETED_SIGN);
        assert_eq!(
            new_processed_files[file_path4].1,
            files_map[file_path4][PREDICATE_LINK]
        );

        // and finally check the synced file was added
        let filename5 = "../testdata/subfolder/subexists.md";
        assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[filename5].1,
            new_files_map["/subexists.md"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_sync_delete_without_recursive() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        match safe
            .files_container_sync(
                "../testdata/subfolder/",
                "some-url",
                false, // this sets the recursive flag to off
                false, // do not follow links
                true,  // this sets the delete flag
                false,
                false,
            )
            .await
        {
            Ok(_) => Err(anyhow!("Sync was unexpectedly successful".to_string(),)),
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(
                    msg,
                    "'delete' is not allowed if 'recursive' is not set".to_string()
                );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_files_container_sync_update_nrs_unversioned_link() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _, _) = safe
            .files_container_create(Some("../testdata/"), None, true, true, false)
            .await?;

        let nrsurl = random_nrs_name();
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(None);
        let unversioned_link = safe_url.to_string();
        match safe
            .nrs_map_container_create(&nrsurl, &unversioned_link, false, true, false)
            .await
        {
            Ok(_) => Err(anyhow!(
                "NRS create was unexpectedly successful".to_string(),
            )),
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(
                msg,
                format!(
                    "The linked content (FilesContainer) is versionable, therefore NRS requires the link to specify a hash: \"{}\"",
                    unversioned_link
                )
            );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_files_container_sync_update_nrs_with_xorurl() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _, _) = safe
            .files_container_create(Some("../testdata/"), None, true, true, false)
            .await?;

        let _ = retry_loop!(safe.fetch(&xorurl, None));

        match safe
            .files_container_sync(
                "../testdata/subfolder/",
                &xorurl,
                false,
                false,
                false,
                true, // this flag requests the update-nrs
                false,
            )
            .await
        {
            Ok(_) => Err(anyhow!("Sync was unexpectedly successful".to_string(),)),
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(
                    msg,
                    "'update-nrs' is not allowed since the URL provided is not an NRS URL"
                        .to_string()
                );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_files_container_sync_update_nrs_versioned_link() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _, _) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));

        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        let nrsurl = random_nrs_name();
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(Some(version0));
        let (nrs_xorurl, _, _) = retry_loop!(safe.nrs_map_container_create(
            &nrsurl,
            &safe_url.to_string(),
            false,
            true,
            false
        ));

        let _ = retry_loop!(safe.fetch(&nrs_xorurl, None));

        let (version1, _, _) = retry_loop!(safe.files_container_sync(
            "../testdata/subfolder/",
            &nrsurl,
            false,
            false,
            false,
            true, // this flag requests the update-nrs
            false,
        ));

        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(Some(version1));
        let (new_link, _) = retry_loop!(safe.parse_and_resolve_url(&nrsurl));
        assert_eq!(new_link.to_string(), safe_url.to_string());

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_sync_target_path_without_trailing_slash() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        assert_eq!(processed_files.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_path("path/when/sync");
        let (version, new_processed_files, new_files_map) = retry_loop!(safe.files_container_sync(
            "../testdata/subfolder",
            &safe_url.to_string(),
            true,
            false,
            false,
            false,
            false,
        ));

        assert!(version != VersionHash::default());
        assert_eq!(
            new_processed_files.len(),
            SUBFOLDER_NO_SLASH_PUT_FILEITEM_COUNT
        );
        assert_eq!(
            new_files_map.len(),
            TESTDATA_PUT_FILEITEM_COUNT + SUBFOLDER_NO_SLASH_PUT_FILEITEM_COUNT
        );

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            new_files_map["/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            new_files_map["/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename3].1,
            new_files_map["/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename4].1,
            new_files_map["/noextension"][PREDICATE_LINK]
        );

        // and finally check the synced file is there
        let filename5 = "../testdata/subfolder/subexists.md";
        assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[filename5].1,
            new_files_map["/path/when/sync/subexists.md"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_sync_target_path_with_trailing_slash() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        assert_eq!(processed_files.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_path("/path/when/sync/");
        let (version, new_processed_files, new_files_map) = retry_loop!(safe.files_container_sync(
            "../testdata/subfolder",
            &safe_url.to_string(),
            true,
            false,
            false,
            false,
            false,
        ));

        assert!(version != VersionHash::default());
        assert_eq!(
            new_processed_files.len(),
            SUBFOLDER_NO_SLASH_PUT_FILEITEM_COUNT
        );
        assert_eq!(
            new_files_map.len(),
            TESTDATA_PUT_FILEITEM_COUNT + SUBFOLDER_NO_SLASH_PUT_FILEITEM_COUNT
        );

        let filename1 = "../testdata/test.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            new_files_map["/test.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/another.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            new_files_map["/another.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename3].1,
            new_files_map["/subfolder/subexists.md"][PREDICATE_LINK]
        );

        let filename4 = "../testdata/noextension";
        assert_eq!(processed_files[filename4].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename4].1,
            new_files_map["/noextension"][PREDICATE_LINK]
        );

        // and finally check the synced file is there
        let filename5 = "../testdata/subfolder/subexists.md";
        assert_eq!(new_processed_files[filename5].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[filename5].1,
            new_files_map["/path/when/sync/subfolder/subexists.md"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_get() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _, files_map) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));

        let (version, fetched_files_map) = retry_loop!(safe.files_container_get(&xorurl));

        assert!(version != VersionHash::default());
        assert_eq!(fetched_files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), fetched_files_map.len());
        assert_eq!(files_map["/test.md"], fetched_files_map["/test.md"]);
        assert_eq!(files_map["/another.md"], fetched_files_map["/another.md"]);
        assert_eq!(
            files_map["/subfolder/subexists.md"],
            fetched_files_map["/subfolder/subexists.md"]
        );
        assert_eq!(files_map["/noextension"], fetched_files_map["/noextension"]);

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_version() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _, _) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        let (version1, _, _) = retry_loop!(safe.files_container_sync(
            "../testdata/subfolder/",
            &xorurl,
            true,
            false,
            true, // this sets the delete flag,
            false,
            false,
        ));
        assert!(version1 != version0);

        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(None);
        let (version, _) = retry_loop_for_pattern!(safe
            .files_container_get(&safe_url.to_string()), Ok((version, _)) if *version == version1)?;
        assert_eq!(version, version1);

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_get_with_version() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _processed_files, files_map) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        // let's create a new version of the files container
        let (version1, _new_processed_files, new_files_map) = retry_loop!(safe
            .files_container_sync(
                "../testdata/subfolder/",
                &xorurl,
                true,
                false,
                true, // this sets the delete flag
                false,
                false,
            ));

        // let's fetch version 0
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(Some(version0));
        let (version, v0_files_map) = retry_loop!(safe.files_container_get(&safe_url.to_string()));

        assert_eq!(version, version0);
        assert_eq!(files_map, v0_files_map);
        // let's check that one of the files in v1 is still there
        let file_path1 = "/test.md";
        assert_eq!(
            files_map[file_path1][PREDICATE_LINK],
            v0_files_map[file_path1][PREDICATE_LINK]
        );

        // let's fetch version1
        safe_url.set_content_version(Some(version1));
        let (version, v1_files_map) = retry_loop_for_pattern!(safe
                .files_container_get(&safe_url.to_string()), Ok((version, _)) if *version == version1)?;

        assert_eq!(version, version1);
        assert_eq!(new_files_map, v1_files_map);
        // let's check that some of the files are no in v2 anymore
        let file_path2 = "/another.md";
        let file_path3 = "/subfolder/subexists.md";
        let file_path4 = "/noextension";
        assert!(v1_files_map.get(file_path1).is_none());
        assert!(v1_files_map.get(file_path2).is_none());
        assert!(v1_files_map.get(file_path3).is_none());
        assert!(v1_files_map.get(file_path4).is_none());

        // let's fetch version invalid version
        safe_url.set_content_version(Some(VersionHash::default()));
        match safe.files_container_get(&safe_url.to_string()).await {
            Ok(_) => Err(anyhow!(
                "Unexpectedly retrieved invalid version of container".to_string(),
            )),
            Err(Error::VersionNotFound(msg)) => {
                let default_vh = format!("{}", VersionHash::default());
                assert_eq!(
                    msg,
                    format!(
                        "Version '{}' is invalid for FilesContainer found at \"{}\"",
                        default_vh, safe_url
                    )
                );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_files_container_create_get_empty_folder() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _processed_files, files_map) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));

        let (_, files_map_get) = retry_loop!(safe.files_container_get(&xorurl.to_string()));

        assert_eq!(files_map, files_map_get);
        assert_eq!(files_map_get["/emptyfolder"], files_map["/emptyfolder"]);
        assert_eq!(
            files_map_get["/emptyfolder"]["type"],
            MIMETYPE_FILESYSTEM_DIR
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_sync_with_nrs_url() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _, _) = retry_loop!(safe.files_container_create(
            Some("../testdata/test.md"),
            None,
            false,
            true,
            false
        ));
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        let nrsurl = random_nrs_name();
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(Some(version0));
        let (nrs_xorurl, _, _) = retry_loop!(safe.nrs_map_container_create(
            &nrsurl,
            &safe_url.to_string(),
            false,
            true,
            false
        ));
        let _ = retry_loop!(safe.fetch(&nrs_xorurl, None));

        let _ = retry_loop!(safe.files_container_sync(
            "../testdata/subfolder/",
            &xorurl,
            false,
            false,
            false,
            false,
            false,
        ));

        let (version2, _, _) = retry_loop!(safe.files_container_sync(
            "../testdata/",
            &nrsurl,
            false,
            false,
            false,
            true, // this flag requests the update-nrs
            false,
        ));

        // now it should look like:
        // safe://<nrs>
        // ├── .hidden.txt
        // ├── another.md
        // ├── noextension
        // ├── sub2.md
        // ├── subexists.md
        // └── test.md
        //
        // So, we have 6 items.
        let (version, fetched_files_map) = retry_loop_for_pattern!(safe.files_container_get(&xorurl), Ok((version, _)) if *version == version2)?;
        assert!(version != VersionHash::default());
        assert_eq!(fetched_files_map.len(), 6);

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_add() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = retry_loop!(safe.files_container_create(
            Some("../testdata/subfolder/"),
            None,
            false,
            true,
            false
        ));
        assert_eq!(processed_files.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        let (version1, new_processed_files, new_files_map) = retry_loop!(safe.files_container_add(
            "../testdata/test.md",
            &format!("{}/new_filename_test.md", xorurl),
            false,
            false,
            false,
            false,
        ));

        assert!(version1 != version0);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT + 1);

        let filename1 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            new_files_map["/subexists.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/subfolder/sub2.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            new_files_map["/sub2.md"][PREDICATE_LINK]
        );

        let filename3 = "../testdata/test.md";
        assert_eq!(new_processed_files[filename3].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[filename3].1,
            new_files_map["/new_filename_test.md"][PREDICATE_LINK]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_add_dry_run() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = retry_loop!(safe.files_container_create(
            Some("../testdata/subfolder/"),
            None,
            false,
            true,
            false
        ));
        assert_eq!(processed_files.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        let (version1, new_processed_files, new_files_map) = retry_loop!(safe.files_container_add(
            "../testdata/test.md",
            &format!("{}/new_filename_test.md", xorurl),
            false,
            false,
            false,
            true, // dry run
        ));

        // skip version hash check since hash didn't change on dry run
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT + 1);

        // a dry run again should give the exact same results
        let (version2, new_processed_files2, new_files_map2) = retry_loop!(safe
            .files_container_add(
                "../testdata/test.md",
                &format!("{}/new_filename_test.md", xorurl),
                false,
                false,
                false,
                true, // dry run
            ));

        assert_eq!(version1, version2);
        assert_eq!(new_processed_files.len(), new_processed_files2.len());
        assert_eq!(new_files_map.len(), new_files_map2.len());

        let filename = "../testdata/test.md";
        assert_eq!(new_processed_files[filename].0, CONTENT_ADDED_SIGN);
        assert_eq!(new_processed_files2[filename].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[filename].1,
            new_files_map["/new_filename_test.md"][PREDICATE_LINK]
        );
        assert_eq!(
            new_processed_files2[filename].1,
            new_files_map2["/new_filename_test.md"][PREDICATE_LINK]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_add_dir() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = retry_loop!(safe.files_container_create(
            Some("../testdata/subfolder/"),
            None,
            false,
            true,
            false
        ));
        assert_eq!(processed_files.len(), SUBFOLDER_PUT_FILEITEM_COUNT); // root "/" + 2 files
        assert_eq!(files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        match safe
            .files_container_add("../testdata", &xorurl, false, false, false, false)
            .await
        {
            Ok(_) => Err(anyhow!(
                "Unexpectedly added a folder to files container".to_string(),
            )),
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(
                    msg,
                    "'../testdata' is a directory, only individual files can be added. Use files sync operation for uploading folders".to_string(),
                );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_files_container_add_existing_name() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = retry_loop!(safe.files_container_create(
            Some("../testdata/subfolder/"),
            None,
            false,
            true,
            false
        ));
        assert_eq!(processed_files.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        // let's try to add a file with same target name and same content, it should fail
        let (version1, new_processed_files, new_files_map) = retry_loop!(safe.files_container_add(
            "../testdata/subfolder/sub2.md",
            &format!("{}/sub2.md", xorurl),
            false,
            false,
            false,
            false,
        ));

        assert_eq!(version1, version0);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        assert_eq!(
            new_processed_files["../testdata/subfolder/sub2.md"].1,
            "File named \"/sub2.md\" with same content already exists on target. Use the \'force\' flag to replace it"
        );
        assert_eq!(files_map, new_files_map);

        // let's try to add a file with same target name but with different content, it should still fail
        let (version2, new_processed_files, new_files_map) = retry_loop!(safe.files_container_add(
            "../testdata/test.md",
            &format!("{}/sub2.md", xorurl),
            false,
            false,
            false,
            false,
        ));

        assert_eq!(version2, version0);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        assert_eq!(
            new_processed_files["../testdata/test.md"].1,
            "File named \"/sub2.md\" with different content already exists on target. Use the \'force\' flag to replace it"
        );
        assert_eq!(files_map, new_files_map);

        // let's now force it
        let (version3, new_processed_files, new_files_map) = retry_loop!(safe.files_container_add(
            "../testdata/test.md",
            &format!("{}/sub2.md", xorurl),
            true, //force it
            false,
            false,
            false,
        ));

        assert!(version3 != version0);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        assert_eq!(
            new_processed_files["../testdata/test.md"].0,
            CONTENT_UPDATED_SIGN
        );
        assert_eq!(
            new_processed_files["../testdata/test.md"].1,
            new_files_map["/sub2.md"]["link"]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_fail_add_or_sync_invalid_path() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = retry_loop!(safe.files_container_create(
            Some("../testdata/test.md"),
            None,
            false,
            true,
            false
        ));
        assert_eq!(processed_files.len(), 1);
        assert_eq!(files_map.len(), 1);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        match safe
            .files_container_sync(
                "/non-existing-path",
                &xorurl,
                false,
                false,
                false,
                false,
                false,
            )
            .await
        {
            Ok(_) => {
                bail!("Unexpectedly added a folder to files container".to_string(),)
            }
            Err(Error::FileSystemError(msg)) => {
                assert!(msg
                    .starts_with("Couldn't read metadata from source path ('/non-existing-path')"))
            }
            other => {
                bail!("Error returned is not the expected one: {:?}", other)
            }
        }

        match safe
            .files_container_add(
                "/non-existing-path",
                &format!("{}/test.md", xorurl),
                true, // force it
                false,
                false,
                false,
            )
            .await
        {
            Ok(_) => Err(anyhow!(
                "Unexpectedly added a folder to files container".to_string(),
            )),
            Err(Error::FileSystemError(msg)) => {
                assert!(msg
                    .starts_with("Couldn't read metadata from source path ('/non-existing-path')"));
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_files_container_add_a_url() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = retry_loop!(safe.files_container_create(
            Some("../testdata/subfolder/"),
            None,
            false,
            true,
            false
        ));
        assert_eq!(processed_files.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        let data = b"0123456789";
        let file_xorurl = retry_loop!(safe.files_store_public_blob(data, None, false));
        let new_filename = "/new_filename_test.md";

        let (version1, new_processed_files, new_files_map) = retry_loop!(safe.files_container_add(
            &file_xorurl,
            &format!("{}{}", xorurl, new_filename),
            false,
            false,
            false,
            false,
        ));

        assert!(version1 != version0);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT + 1);

        let filename1 = "../testdata/subfolder/subexists.md";
        assert_eq!(processed_files[filename1].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename1].1,
            new_files_map["/subexists.md"][PREDICATE_LINK]
        );

        let filename2 = "../testdata/subfolder/sub2.md";
        assert_eq!(processed_files[filename2].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            processed_files[filename2].1,
            new_files_map["/sub2.md"][PREDICATE_LINK]
        );

        assert_eq!(new_processed_files[new_filename].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[new_filename].1,
            new_files_map[new_filename][PREDICATE_LINK]
        );
        assert_eq!(new_files_map[new_filename][PREDICATE_LINK], file_xorurl);

        // let's add another file but with the same name
        let data = b"9876543210";
        let other_file_xorurl = retry_loop!(safe.files_store_public_blob(data, None, false));
        let (version2, new_processed_files, new_files_map) = retry_loop!(safe.files_container_add(
            &other_file_xorurl,
            &format!("{}{}", xorurl, new_filename),
            true, // force to overwrite it with new link
            false,
            false,
            false,
        ));

        assert!(version2 != version0);
        assert!(version2 != version1);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT + 1);
        assert_eq!(new_processed_files[new_filename].0, CONTENT_UPDATED_SIGN);
        assert_eq!(
            new_processed_files[new_filename].1,
            new_files_map[new_filename][PREDICATE_LINK]
        );
        assert_eq!(
            new_files_map[new_filename][PREDICATE_LINK],
            other_file_xorurl
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_add_from_raw() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) = retry_loop!(safe.files_container_create(
            Some("../testdata/subfolder/"),
            None,
            false,
            true,
            false
        ));
        assert_eq!(processed_files.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT);
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        let data = b"0123456789";
        let new_filename = "/new_filename_test.md";

        let (version1, new_processed_files, new_files_map) = retry_loop!(safe
            .files_container_add_from_raw(
                data,
                &format!("{}{}", xorurl, new_filename),
                false,
                false,
                false,
            ));

        assert!(version1 != VersionHash::default());
        assert!(version1 != version0);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT + 1);

        assert_eq!(new_processed_files[new_filename].0, CONTENT_ADDED_SIGN);
        assert_eq!(
            new_processed_files[new_filename].1,
            new_files_map[new_filename][PREDICATE_LINK]
        );

        // let's add another file but with the same name
        let data = b"9876543210";
        let (version2, new_processed_files, new_files_map) = retry_loop!(safe
            .files_container_add_from_raw(
                data,
                &format!("{}{}", xorurl, new_filename),
                true, // force to overwrite it with new link
                false,
                false,
            ));

        assert!(version2 != VersionHash::default());
        assert!(version2 != version0);
        assert!(version2 != version1);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), SUBFOLDER_PUT_FILEITEM_COUNT + 1);
        assert_eq!(new_processed_files[new_filename].0, CONTENT_UPDATED_SIGN);
        assert_eq!(
            new_processed_files[new_filename].1,
            new_files_map[new_filename][PREDICATE_LINK]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_files_container_remove_path() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, processed_files, files_map) =
            retry_loop!(safe.files_container_create(Some("../testdata/"), None, true, true, false));
        assert_eq!(processed_files.len(), TESTDATA_PUT_FILEITEM_COUNT);
        assert_eq!(files_map.len(), TESTDATA_PUT_FILEITEM_COUNT);
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl));

        // let's remove a file first
        let (version1, new_processed_files, new_files_map) = retry_loop!(
            safe.files_container_remove_path(&format!("{}/test.md", xorurl), false, false, false)
        );

        assert!(version1 != version0);
        assert_eq!(new_processed_files.len(), 1);
        assert_eq!(new_files_map.len(), TESTDATA_PUT_FILEITEM_COUNT - 1);

        let filepath = "/test.md";
        assert_eq!(new_processed_files[filepath].0, CONTENT_DELETED_SIGN);
        assert_eq!(
            new_processed_files[filepath].1,
            files_map[filepath][PREDICATE_LINK]
        );

        // let's remove an entire folder now with recursive flag
        let (version2, new_processed_files, new_files_map) = retry_loop!(
            safe.files_container_remove_path(&format!("{}/subfolder", xorurl), true, false, false)
        );

        assert!(version2 != version0);
        assert!(version2 != version1);
        assert_eq!(new_processed_files.len(), 2);
        assert_eq!(
            new_files_map.len(),
            TESTDATA_PUT_FILEITEM_COUNT - SUBFOLDER_PUT_FILEITEM_COUNT - 1
        );

        let filename1 = "/subfolder/subexists.md";
        assert_eq!(new_processed_files[filename1].0, CONTENT_DELETED_SIGN);
        assert_eq!(
            new_processed_files[filename1].1,
            files_map[filename1][PREDICATE_LINK]
        );

        let filename2 = "/subfolder/sub2.md";
        assert_eq!(new_processed_files[filename2].0, CONTENT_DELETED_SIGN);
        assert_eq!(
            new_processed_files[filename2].1,
            files_map[filename2][PREDICATE_LINK]
        );

        Ok(())
    }
}
