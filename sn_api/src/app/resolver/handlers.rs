use super::{Range, SafeData};
use crate::app::{
    files::{self, FileInfo, FilesMap},
    multimap::MultimapKeyValues,
    DataType, Safe, SafeUrl,
};
use crate::{Error, Result};
use bytes::Bytes;
use log::{debug, warn};
use safe_network::types::BytesAddress;
use std::collections::BTreeSet;

impl Safe {
    pub(crate) async fn resolve_nrs_map_container(&self, input_url: SafeUrl) -> Result<SafeData> {
        // get NRS resolution
        let (mut target_url, nrs_map) = self
            .nrs_get(input_url.public_name(), input_url.content_version())
            .await
            .map_err(|e| {
                warn!("NRS failed to resolve {}: {}", input_url, e);
                Error::ContentNotFound(format!("Content not found at {}", input_url))
            })?;
        debug!("NRS Resolved {} => {}", input_url, target_url);

        // concatenate paths
        let url_path = input_url.path_decoded()?;
        let target_path = target_url.path_decoded()?;
        target_url.set_path(&format!("{}{}", target_path, url_path));

        // create safe_data ignoring the input path or subnames
        let version = target_url.content_version().ok_or_else(|| {
            Error::ContentError(format!(
                "Missing content version in Url: {} while resolving: {}",
                &target_url, &input_url
            ))
        })?;
        let mut nrs_url = input_url.clone();
        nrs_url.set_path("");
        nrs_url.set_sub_names("")?;
        let safe_data = SafeData::NrsMapContainer {
            public_name: if nrs_url.is_xorurl() {
                None
            } else {
                Some(nrs_url.top_name().to_string())
            },
            xorurl: nrs_url.to_xorurl_string(),
            xorname: nrs_url.xorname(),
            type_tag: nrs_url.type_tag(),
            version,
            nrs_map,
            data_type: nrs_url.data_type(),
            resolves_into: Some(target_url),
            resolved_from: nrs_url.to_string(),
        };

        Ok(safe_data)
    }

    pub(crate) async fn resolve_multimap(
        &self,
        input_url: SafeUrl,
        retrieve_data: bool,
    ) -> Result<SafeData> {
        let data: MultimapKeyValues = if retrieve_data {
            match input_url.content_version() {
                None => self.fetch_multimap_values(&input_url).await?,
                Some(v) => vec![(
                    v.entry_hash(),
                    self.fetch_multimap_value_by_hash(&input_url, v.entry_hash())
                        .await?,
                )]
                .into_iter()
                .collect(),
            }
        } else {
            MultimapKeyValues::new()
        };

        let safe_data = SafeData::Multimap {
            xorurl: input_url.to_xorurl_string(),
            xorname: input_url.xorname(),
            type_tag: input_url.type_tag(),
            data,
            resolved_from: input_url.to_string(),
        };

        Ok(safe_data)
    }

    pub(crate) async fn resolve_raw(
        &self,
        input_url: SafeUrl,
        metadata: Option<FileInfo>,
        retrieve_data: bool,
        range: Range,
    ) -> Result<SafeData> {
        ensure_no_subnames(&input_url, "raw data")?;

        match input_url.data_type() {
            DataType::SafeKey => {
                let safe_data = SafeData::SafeKey {
                    xorurl: input_url.to_xorurl_string(),
                    xorname: input_url.xorname(),
                    resolved_from: input_url.to_string(),
                };
                Ok(safe_data)
            }
            DataType::File => {
                self.retrieve_data(&input_url, retrieve_data, None, &metadata, range)
                    .await
            }
            DataType::Register => {
                let data = if retrieve_data {
                    match input_url.content_version() {
                        None => self.register_fetch_entries(&input_url).await?,
                        Some(v) => vec![(
                            v.entry_hash(),
                            self.register_fetch_entry(&input_url, v.entry_hash())
                                .await?,
                        )]
                        .into_iter()
                        .collect(),
                    }
                } else {
                    BTreeSet::new()
                };

                let safe_data = SafeData::PublicRegister {
                    xorurl: input_url.to_xorurl_string(),
                    xorname: input_url.xorname(),
                    type_tag: input_url.type_tag(),
                    data,
                    resolved_from: input_url.to_string(),
                };
                Ok(safe_data)
            }
        }
    }

    pub(crate) async fn resolve_mediatype(
        &self,
        input_url: SafeUrl,
        metadata: Option<FileInfo>,
        retrieve_data: bool,
        range: Range,
        media_type_str: String,
    ) -> Result<SafeData> {
        ensure_no_subnames(&input_url, "media type")?;

        match input_url.data_type() {
            DataType::File => {
                self.retrieve_data(
                    &input_url,
                    retrieve_data,
                    Some(media_type_str),
                    &metadata,
                    range,
                )
                .await
            }
            other => Err(Error::ContentError(format!(
                "Data type '{:?}' not supported yet",
                other
            ))),
        }
    }

    pub(crate) async fn resolve_file_container(
        &self,
        mut input_url: SafeUrl,
        resolve_path: bool,
    ) -> Result<SafeData> {
        ensure_no_subnames(&input_url, "file container")?;

        // Fetch files container
        let (version, files_map) = match self.fetch_files_container(&input_url).await? {
            Some((version, files_map)) => (Some(version), files_map),
            None => (None, FilesMap::default()),
        };

        debug!(
            "Files container at {}, with version: {:?}, of data type: {}, containing: {:?}",
            input_url,
            version,
            input_url.data_type(),
            files_map
        );

        // cd there if it is a dir
        let path = input_url.path_decoded()?;
        let (files_map, resolves_into, metadata) = if !resolve_path
            || path == "/"
            || path.is_empty()
        {
            debug!(
                "Skipping path resolution for FilesContainer resolved with {}",
                input_url
            );
            (files_map, None, None)
        } else {
            let files_map_for_path = files::file_map_for_path(files_map, &path).map_err(|e| Error::ContentError(
                format!("Failed to obtain file map for path: {}, on FileContainer at: {}, because: {:?}",
                &path,
                input_url,
                e.to_string()),
            ))?;

            // try to gather file link and metadata for a file
            let (link, metadata) = files::get_file_link_and_metadata(&files_map_for_path, &path)
                .map_err(|e| {
                    Error::ContentError(format!(
                        "Failed to obtain file link or info on FileContainer at: {}: {}",
                        input_url,
                        e.to_string(),
                    ))
                })?;

            let resolves_into = match link {
                Some(l) => Some(SafeUrl::from_url(&l)?),
                None => None,
            };

            (files_map_for_path, resolves_into, metadata)
        };

        // We don't want the path just the FilesContainer XOR-URL and version
        input_url.set_path("");
        let safe_data = SafeData::FilesContainer {
            xorurl: input_url.to_xorurl_string(),
            xorname: input_url.xorname(),
            type_tag: input_url.type_tag(),
            version,
            files_map,
            data_type: input_url.data_type(),
            metadata,
            resolves_into,
            resolved_from: input_url.to_string(),
        };

        Ok(safe_data)
    }

    async fn retrieve_data(
        &self,
        input_url: &SafeUrl,
        retrieve_data: bool,
        media_type: Option<String>,
        metadata: &Option<FileInfo>,
        range: Range,
    ) -> Result<SafeData> {
        if !input_url.path().is_empty() {
            return Err(Error::ContentError(format!(
                "Cannot get relative path of Immutable Data {:?}",
                input_url.path_decoded()?
            )));
        };

        let data = if retrieve_data {
            self.safe_client
                .get_bytes(BytesAddress::Public(input_url.xorname()), range)
                .await?
        } else {
            Bytes::new()
        };

        let safe_data = SafeData::PublicFile {
            xorurl: input_url.to_xorurl_string(),
            xorname: input_url.xorname(),
            data,
            media_type,
            metadata: metadata.clone(),
            resolved_from: input_url.to_string(),
        };

        Ok(safe_data)
    }
}

// private helper to ensure the SafeUrl contains no subnames
fn ensure_no_subnames(url: &SafeUrl, data_type: &str) -> Result<()> {
    if !url.sub_names_vec().is_empty() {
        let msg = format!(
            "Cannot resolve URL targetting {} as it contains subnames: {}",
            data_type, url
        );
        debug!("{}", msg);
        return Err(Error::InvalidXorUrl(msg));
    }
    Ok(())
}
