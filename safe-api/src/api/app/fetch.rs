// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    files::{FileItem, FileMeta, FilesMap},
    nrs_map::NrsMap,
    Safe, XorName,
};
pub use super::{
    wallet::WalletSpendableBalances,
    xorurl::{SafeContentType, SafeDataType, XorUrlBase, XorUrlEncoder},
};
use crate::{Error, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub type Range = Option<(Option<u64>, Option<u64>)>;

// Maximum number of indirections allowed when resolving a safe:// URL following links
const INDIRECTION_LIMIT: u8 = 10;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum SafeData {
    SafeKey {
        xorurl: String,
        xorname: XorName,
        resolved_from: String,
    },
    Wallet {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        balances: WalletSpendableBalances,
        data_type: SafeDataType,
        resolved_from: String,
    },
    FilesContainer {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        version: u64,
        files_map: FilesMap,
        data_type: SafeDataType,
        resolved_from: String,
    },
    PublishedImmutableData {
        xorurl: String,
        xorname: XorName,
        data: Vec<u8>,
        media_type: Option<String>,
        metadata: Option<FileItem>,
        resolved_from: String,
    },
    NrsMapContainer {
        public_name: String,
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        version: u64,
        nrs_map: NrsMap,
        data_type: SafeDataType,
        resolved_from: String,
    },
}

impl SafeData {
    pub fn xorurl(&self) -> String {
        use SafeData::*;
        match self {
            SafeKey { xorurl, .. }
            | Wallet { xorurl, .. }
            | FilesContainer { xorurl, .. }
            | PublishedImmutableData { xorurl, .. }
            | NrsMapContainer { xorurl, .. } => xorurl.clone(),
        }
    }

    pub fn resolved_from(&self) -> String {
        use SafeData::*;
        match self {
            SafeKey { resolved_from, .. }
            | Wallet { resolved_from, .. }
            | FilesContainer { resolved_from, .. }
            | PublishedImmutableData { resolved_from, .. }
            | NrsMapContainer { resolved_from, .. } => resolved_from.clone(),
        }
    }
}

impl Safe {
    /// # Retrieve data from a safe:// URL
    ///
    /// ## Examples
    ///
    /// ### Fetch FilesContainer relative path file
    /// ```rust
    /// # use safe_api::{Safe, fetch::SafeData};
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::default();
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// # async_std::task::block_on(async {
    ///     let (xorurl, _, _) = safe.files_container_create(Some("../testdata/"), None, true, false).await.unwrap();
    ///
    ///     let safe_data = safe.fetch( &format!( "{}/test.md", &xorurl.replace("?v=0", "") ), None ).await.unwrap();
    ///     let data_string = match safe_data {
    ///         SafeData::PublishedImmutableData { data, .. } => {
    ///             match String::from_utf8(data) {
    ///                 Ok(string) => string,
    ///                 Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    ///             }
    ///         }
    ///         other => panic!(
    ///             "Content type '{:?}' should not have been found. This should be immutable data.",
    ///             other
    ///         )
    ///     };
    ///
    ///     assert!(data_string.starts_with("hello tests!"));
    /// # });
    /// ```
    pub async fn fetch(&self, url: &str, range: Range) -> Result<SafeData> {
        let mut resolution_chain = self.retrieve_from_url(url, true, range, true).await?;
        // Construct return data using the last and first items from the resolution chain
        match resolution_chain.pop() {
            Some(other_safe_data) => Ok(other_safe_data),
            None => {
                // weird...it didn't fail but it returned an empty list...
                Err(Error::Unexpected(format!("Failed to resolve {}", url)))
            }
        }
    }

    /// # Inspect a safe:// URL and retrieve metadata information but the actual target content
    /// # As opposed to 'fetch' function, the actual target content won't be fetched, and only
    /// # the URL will be inspected resolving it as necessary to find the target location.
    /// # This is helpful if you are interested in knowing about the target content rather than
    /// # trying to revieve the actual content.
    ///
    /// ## Examples
    ///
    /// ### Inspect FilesContainer relative path file
    /// ```rust
    /// # use safe_api::{Safe, fetch::SafeData};
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::default();
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// # async_std::task::block_on(async {
    ///     let (container_xorurl, _, _) = safe.files_container_create(Some("../testdata/"), None, true, false).await.unwrap();
    ///
    ///     let inspected_content = safe.inspect( &format!( "{}/test.md", &container_xorurl.replace("?v=0", "") ) ).await.unwrap();
    ///     match &inspected_content[0] {
    ///         SafeData::FilesContainer { xorurl, .. } => {
    ///             assert_eq!(*xorurl, container_xorurl);
    ///         }
    ///         other => panic!(
    ///             "Content type '{:?}' should not have been found. This should be a Files Container.",
    ///             other
    ///         )
    ///     };
    ///     match &inspected_content[1] {
    ///         SafeData::PublishedImmutableData { data, media_type, .. } => {
    ///             assert_eq!(*media_type, Some("text/markdown".to_string()));
    ///             assert!(data.is_empty());
    ///         }
    ///         other => panic!(
    ///             "Content type '{:?}' should not have been found. This should be an Immutable Data.",
    ///             other
    ///         )
    ///     };
    ///
    /// # });
    /// ```
    pub async fn inspect(&self, url: &str) -> Result<Vec<SafeData>> {
        self.retrieve_from_url(url, false, None, true).await
    }

    // Retrieves all pieces of data that resulted from resolving the given URL.
    // An optional 'while_is' argment can be set as a filter to stop reslution process
    // upon the first non-matching content type.
    pub(crate) async fn retrieve_from_url(
        &self,
        url: &str,
        retrieve_data: bool,
        range: Range,
        resolve_path: bool,
    ) -> Result<Vec<SafeData>> {
        let current_xorurl_encoder = Safe::parse_url(url)?;
        info!(
            "URL parsed successfully, fetching: {}",
            current_xorurl_encoder
        );
        debug!(
            "Fetching content of type: {:?}",
            current_xorurl_encoder.content_type()
        );

        // Let's create a list keeping track each of the resolution hops we go through
        // TODO: pass option to get raw content AKA: Do not resolve beyond first thing.
        let mut resolution_chain = Vec::<SafeData>::default();
        let mut next_to_resolve = Some((current_xorurl_encoder, None));
        let mut indirections_count = 0;
        while let Some((next_xorurl_encoder, metadata)) = next_to_resolve {
            if indirections_count == INDIRECTION_LIMIT {
                return Err(Error::ContentError(format!("The maximum number of indirections ({}) was reached when trying to resolve the URL provided", INDIRECTION_LIMIT)));
            }

            let (step, next) = resolve_one_indirection(
                &self,
                next_xorurl_encoder,
                metadata,
                retrieve_data,
                range,
                resolve_path,
            )
            .await?;

            resolution_chain.push(step);
            next_to_resolve = next;
            indirections_count += 1;
        }

        Ok(resolution_chain)
    }
}

// This contains information for the next step to be made
// in each iteration of the resolution process
type NextStepInfo = (XorUrlEncoder, Option<FileItem>);

async fn resolve_one_indirection(
    safe: &Safe,
    mut the_xor: XorUrlEncoder,
    metadata: Option<FileItem>,
    retrieve_data: bool,
    range: Range,
    resolve_path: bool,
) -> Result<(SafeData, Option<NextStepInfo>)> {
    let url = the_xor.to_string();
    let xorurl = the_xor.to_xorurl_string();
    debug!("Going into a new step in the URL resolution for {}", xorurl);
    match the_xor.content_type() {
        SafeContentType::FilesContainer => {
            let (version, files_map) = safe.fetch_files_container(&the_xor).await?;
            debug!(
                "Files container found with v:{}, on data type: {}, containing: {:?}",
                version,
                the_xor.data_type(),
                files_map
            );

            let path = the_xor.path_decoded()?;
            let (files_map, next) = if resolve_path && path != "/" && !path.is_empty() {
                // TODO: Move this logic (path resolver) to the FilesMap struct
                match &files_map.get(&path) {
                    Some(file_item) => match file_item.get("type") {
                        Some(file_type) => {
                            if FileMeta::filetype_is_file(&file_type) {
                                match file_item.get("link") {
                                    Some(link) => {
                                        let new_target_xorurl = XorUrlEncoder::from_url(link)?;
                                        let mut metadata = (*file_item).clone();
                                        Path::new(&path).file_name().map(|name| {
                                            name.to_str().map(|str| {
                                                metadata.insert("name".to_string(), str.to_string())
                                            })
                                        });
                                        (files_map, Some((new_target_xorurl, Some(metadata))))
                                    }
                                    None => {
                                        let msg = format!("FileItem is corrupt. It is missing a \"link\" property at path, \"{}\" on the FilesContainer at: {} ", path, xorurl);
                                        return Err(Error::ContentError(msg));
                                    }
                                }
                            } else {
                                (gen_filtered_filesmap(&path, &files_map, &xorurl)?, None)
                            }
                        }
                        None => {
                            let msg = format!("FileItem is corrupt. It is missing a \"type\" property at path, \"{}\" on the FilesContainer at: {} ", path, xorurl);
                            return Err(Error::ContentError(msg));
                        }
                    },
                    None => (gen_filtered_filesmap(&path, &files_map, &xorurl)?, None),
                }
            } else {
                (files_map, None)
            };

            // We don't want the path in the SafeData field,
            // just the FilesContainer XOR-URL and version
            the_xor.set_path("");
            let safe_data = SafeData::FilesContainer {
                xorurl: the_xor.to_xorurl_string(),
                xorname: the_xor.xorname(),
                type_tag: the_xor.type_tag(),
                version,
                files_map,
                data_type: the_xor.data_type(),
                resolved_from: url,
            };

            Ok((safe_data, next))
        }
        SafeContentType::NrsMapContainer => {
            let (version, nrs_map) = safe
                .nrs_map_container_get(&xorurl)
                .await
                .map_err(|_| Error::ContentNotFound(format!("Content not found at {}", url)))?;

            debug!(
                "Nrs map container found w/ v:{}, of type: {}, containing: {:?}",
                version,
                the_xor.data_type(),
                nrs_map
            );

            let target_url = nrs_map.resolve_for_subnames(the_xor.sub_names_vec())?;
            debug!("Resolved target: {}", target_url);

            let mut target_xorurl_encoder = Safe::parse_url(&target_url)?;
            // Let's concatenate the path corresponding to the URL we are processing
            // to the URL we resolved from NRS Map
            let url_path = the_xor.path_decoded()?;
            if target_xorurl_encoder.path().is_empty() {
                target_xorurl_encoder.set_path(&url_path);
            } else if !the_xor.path().is_empty() {
                target_xorurl_encoder.set_path(&format!(
                    "{}{}",
                    target_xorurl_encoder.path_decoded()?,
                    url_path
                ));
            }

            debug!(
                "Resolving target from resolvable map: {}",
                target_xorurl_encoder
            );

            the_xor.set_path(""); // we don't want the path, just the NRS Map xorurl and version
            let nrs_map_container = SafeData::NrsMapContainer {
                public_name: the_xor.top_name().to_string(),
                xorurl: the_xor.to_xorurl_string(),
                xorname: the_xor.xorname(),
                type_tag: the_xor.type_tag(),
                version,
                nrs_map,
                data_type: the_xor.data_type(),
                resolved_from: url,
            };

            Ok((nrs_map_container, Some((target_xorurl_encoder, None))))
        }
        SafeContentType::Raw => match the_xor.data_type() {
            SafeDataType::SafeKey => {
                let safe_data = SafeData::SafeKey {
                    xorurl,
                    xorname: the_xor.xorname(),
                    resolved_from: url,
                };
                Ok((safe_data, None))
            }
            SafeDataType::PublishedImmutableData => {
                retrieve_immd(safe, &the_xor, retrieve_data, None, &metadata, range).await
            }
            other => Err(Error::ContentError(format!(
                "Data type '{:?}' not supported yet",
                other
            ))),
        },
        SafeContentType::MediaType(media_type_str) => match the_xor.data_type() {
            SafeDataType::PublishedImmutableData => {
                retrieve_immd(
                    safe,
                    &the_xor,
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
        },
        SafeContentType::Wallet => {
            let balances = if retrieve_data {
                safe.fetch_wallet(&the_xor).await?
            } else {
                WalletSpendableBalances::new()
            };

            let safe_data = SafeData::Wallet {
                xorurl,
                xorname: the_xor.xorname(),
                type_tag: the_xor.type_tag(),
                balances,
                data_type: the_xor.data_type(),
                resolved_from: url,
            };

            Ok((safe_data, None))
        }
    }
}

async fn retrieve_immd(
    safe: &Safe,
    the_xor: &XorUrlEncoder,
    retrieve_data: bool,
    media_type: Option<String>,
    metadata: &Option<FileItem>,
    range: Range,
) -> Result<(SafeData, Option<NextStepInfo>)> {
    if !the_xor.path().is_empty() {
        return Err(Error::ContentError(format!(
            "Cannot get relative path of Immutable Data {:?}",
            the_xor.path_decoded()?
        )));
    };

    let data = if retrieve_data {
        safe.fetch_published_immutable_data(the_xor, range).await?
    } else {
        vec![]
    };

    let safe_data = SafeData::PublishedImmutableData {
        xorurl: the_xor.to_xorurl_string(),
        xorname: the_xor.xorname(),
        data,
        media_type,
        metadata: metadata.clone(),
        resolved_from: the_xor.to_string(),
    };

    Ok((safe_data, None))
}

fn gen_filtered_filesmap(urlpath: &str, files_map: &FilesMap, xorurl: &str) -> Result<FilesMap> {
    let mut filtered_filesmap = FilesMap::default();
    let folder_path = if !urlpath.ends_with('/') {
        format!("{}/", urlpath)
    } else {
        urlpath.to_string()
    };
    files_map.iter().for_each(|(filepath, fileitem)| {
        if filepath.starts_with(&folder_path) {
            let mut new_path = filepath.clone();
            new_path.replace_range(..folder_path.len(), "");
            filtered_filesmap.insert(new_path, fileitem.clone());
        }
    });

    if filtered_filesmap.is_empty() {
        Err(Error::ContentError(format!(
            "No data found for path \"{}\" on the FilesContainer at \"{}\"",
            folder_path, xorurl
        )))
    } else {
        Ok(filtered_filesmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::app::test_helpers::new_safe_instance;
    use crate::api::xorurl::XorUrlEncoder;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::io::Read;

    #[tokio::test]
    async fn test_fetch_key() -> Result<()> {
        let mut safe = new_safe_instance()?;
        let preload_amount = "1324.12";
        let (xorurl, _key_pair) = safe.keys_create_preload_test_coins(preload_amount).await?;

        let xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        let content = safe.fetch(&xorurl, None).await?;
        assert!(
            content
                == SafeData::SafeKey {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    resolved_from: xorurl.clone(),
                }
        );

        // let's also compare it with the result from inspecting the URL
        let inspected_content = safe.inspect(&xorurl).await?;
        assert_eq!(inspected_content.len(), 1);
        assert_eq!(content, inspected_content[0]);
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_wallet() -> Result<()> {
        let mut safe = new_safe_instance()?;
        let xorurl = safe.wallet_create().await?;

        let xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        let content = safe.fetch(&xorurl, None).await?;
        assert!(
            content
                == SafeData::Wallet {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    type_tag: 1_000,
                    balances: WalletSpendableBalances::default(),
                    data_type: SafeDataType::SeqMutableData,
                    resolved_from: xorurl.clone(),
                }
        );

        // let's also compare it with the result from inspecting the URL
        let inspected_content = safe.inspect(&xorurl).await?;
        assert_eq!(inspected_content.len(), 1);
        assert_eq!(content, inspected_content[0]);
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_files_container() -> Result<()> {
        let mut safe = new_safe_instance()?;
        let (xorurl, _, files_map) = safe
            .files_container_create(Some("../testdata/"), None, true, false)
            .await?;

        let xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        let content = safe.fetch(&xorurl, None).await?;

        assert!(
            content
                == SafeData::FilesContainer {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    type_tag: 1_100,
                    version: 0,
                    files_map,
                    data_type: SafeDataType::PublishedSeqAppendOnlyData,
                    resolved_from: xorurl.clone(),
                }
        );

        let mut xorurl_encoder_with_path = xorurl_encoder.clone();
        xorurl_encoder_with_path.set_path("/subfolder/subexists.md");
        assert_eq!(xorurl_encoder_with_path.path(), "/subfolder/subexists.md");
        assert_eq!(xorurl_encoder_with_path.xorname(), xorurl_encoder.xorname());
        assert_eq!(
            xorurl_encoder_with_path.type_tag(),
            xorurl_encoder.type_tag()
        );
        assert_eq!(
            xorurl_encoder_with_path.content_type(),
            xorurl_encoder.content_type()
        );

        // let's also compare it with the result from inspecting the URL
        let inspected_content = safe.inspect(&xorurl).await?;
        assert_eq!(inspected_content.len(), 1);
        assert_eq!(content, inspected_content[0]);
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_resolvable_container() -> Result<()> {
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

        let mut safe = new_safe_instance()?;

        let (xorurl, _, the_files_map) = safe
            .files_container_create(Some("../testdata/"), None, true, false)
            .await?;

        let mut xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        xorurl_encoder.set_content_version(Some(0));
        let (_nrs_map_xorurl, _, _nrs_map) = safe
            .nrs_map_container_create(&site_name, &xorurl_encoder.to_string(), true, true, false)
            .await?;

        let nrs_url = format!("safe://{}", site_name);
        let content = safe.fetch(&nrs_url, None).await?;

        // this should resolve to a FilesContainer until we enable prevent resolution.
        match &content {
            SafeData::FilesContainer {
                xorurl,
                xorname,
                type_tag,
                version,
                files_map,
                data_type,
                ..
            } => {
                assert_eq!(*xorurl, xorurl_encoder.to_string());
                assert_eq!(*xorname, xorurl_encoder.xorname());
                assert_eq!(*type_tag, 1_100);
                assert_eq!(*version, 0);
                assert_eq!(*data_type, SafeDataType::PublishedSeqAppendOnlyData);
                assert_eq!(*files_map, the_files_map);

                // let's also compare it with the result from inspecting the URL
                let inspected_content = safe.inspect(&nrs_url).await?;
                assert_eq!(inspected_content.len(), 2);
                assert_eq!(content, inspected_content[1]);
                Ok(())
            }
            _ => Err(Error::Unexpected(
                "NRS map container was not returned".to_string(),
            )),
        }
    }

    #[tokio::test]
    async fn test_fetch_resolvable_map_data() -> Result<()> {
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

        let mut safe = new_safe_instance()?;
        let (xorurl, _, _the_files_map) = safe
            .files_container_create(Some("../testdata/"), None, true, false)
            .await?;

        let mut xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        xorurl_encoder.set_content_version(Some(0));
        let files_container_url = xorurl_encoder.to_string();
        let _ = safe
            .nrs_map_container_create(&site_name, &files_container_url, true, true, false)
            .await?;

        let nrs_url = format!("safe://{}", site_name);
        let content = safe.fetch(&nrs_url, None).await?;

        // this should resolve to a FilesContainer
        match &content {
            SafeData::FilesContainer {
                xorurl,
                resolved_from,
                ..
            } => {
                assert_eq!(*resolved_from, files_container_url);
                assert_eq!(*xorurl, files_container_url);

                // let's also compare it with the result from inspecting the URL
                let inspected_content = safe.inspect(&nrs_url).await?;
                assert_eq!(inspected_content.len(), 2);
                assert_eq!(content, inspected_content[1]);
                Ok(())
            }
            _ => Err(Error::Unexpected(
                "Nrs map container was not returned".to_string(),
            )),
        }
    }

    #[tokio::test]
    async fn test_fetch_published_immutable_data() -> Result<()> {
        let mut safe = new_safe_instance()?;
        let data = b"Something super immutable";
        let xorurl = safe
            .files_put_published_immutable(data, Some("text/plain"), false)
            .await?;

        let xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        let content = safe.fetch(&xorurl, None).await?;
        assert!(
            content
                == SafeData::PublishedImmutableData {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    data: data.to_vec(),
                    resolved_from: xorurl.clone(),
                    media_type: Some("text/plain".to_string()),
                    metadata: None,
                }
        );

        // let's also compare it with the result from inspecting the URL
        let inspected_content = safe.inspect(&xorurl).await?;
        assert!(
            inspected_content[0]
                == SafeData::PublishedImmutableData {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    data: vec![],
                    resolved_from: xorurl,
                    media_type: Some("text/plain".to_string()),
                    metadata: None,
                }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_range_published_immutable_data() -> Result<()> {
        let mut safe = new_safe_instance()?;
        let saved_data = b"Something super immutable";
        let size = saved_data.len();
        let xorurl = safe
            .files_put_published_immutable(saved_data, Some("text/plain"), false)
            .await?;

        // Fetch first half and match
        let fetch_first_half = Some((None, Some(size as u64 / 2)));
        let content = safe.fetch(&xorurl, fetch_first_half).await?;

        if let SafeData::PublishedImmutableData { data, .. } = &content {
            assert_eq!(data.clone(), saved_data[0..size / 2].to_vec());
        } else {
            return Err(Error::Unexpected(format!(
                "Content fetched is not a PublishedImmutableData: {:?}",
                content
            )));
        }

        // Fetch second half and match
        let fetch_second_half = Some((Some(size as u64 / 2), Some(size as u64)));
        let content = safe.fetch(&xorurl, fetch_second_half).await?;

        if let SafeData::PublishedImmutableData { data, .. } = &content {
            assert_eq!(data.clone(), saved_data[size / 2..size].to_vec());
            Ok(())
        } else {
            Err(Error::Unexpected(format!(
                "Content fetched is not a PublishedImmutableData: {:?}",
                content
            )))
        }
    }

    #[tokio::test]
    async fn test_fetch_range_from_files_container() -> Result<()> {
        use std::fs::File;
        let mut safe = new_safe_instance()?;
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

        let (xorurl, _, _files_map) = safe
            .files_container_create(Some("../testdata/"), None, true, false)
            .await?;

        let mut xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        xorurl_encoder.set_content_version(Some(0));
        let (_nrs_map_xorurl, _, _nrs_map) = safe
            .nrs_map_container_create(&site_name, &xorurl_encoder.to_string(), true, true, false)
            .await?;

        let nrs_url = format!("safe://{}/test.md", site_name);

        let mut file = File::open("../testdata/test.md")
            .map_err(|err| Error::Unexpected(format!("Failed to open local file: {}", err)))?;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)
            .map_err(|err| Error::Unexpected(format!("Failed to read local file: {}", err)))?;
        let file_size = file_data.len();

        // Fetch full file and match
        let content = safe.fetch(&nrs_url, None).await?;
        if let SafeData::PublishedImmutableData { data, .. } = &content {
            assert_eq!(data.clone(), file_data.clone());
        } else {
            return Err(Error::Unexpected(format!(
                "Content fetched is not a PublishedImmutableData: {:?}",
                content
            )));
        }

        // Fetch first half and match
        let fetch_first_half = Some((None, Some(file_size as u64 / 2)));
        let content = safe.fetch(&nrs_url, fetch_first_half).await?;

        if let SafeData::PublishedImmutableData { data, .. } = &content {
            assert_eq!(data.clone(), file_data[0..file_size / 2].to_vec());
        } else {
            return Err(Error::Unexpected(format!(
                "Content fetched is not a PublishedImmutableData: {:?}",
                content
            )));
        }

        // Fetch second half and match
        let fetch_second_half = Some((Some(file_size as u64 / 2), Some(file_size as u64)));
        let content = safe.fetch(&nrs_url, fetch_second_half).await?;

        if let SafeData::PublishedImmutableData { data, .. } = &content {
            assert_eq!(data.clone(), file_data[file_size / 2..file_size].to_vec());
            Ok(())
        } else {
            Err(Error::Unexpected(format!(
                "Content fetched is not a PublishedImmutableData: {:?}",
                content
            )))
        }
    }

    #[tokio::test]
    async fn test_fetch_unsupported() -> Result<()> {
        let safe = new_safe_instance()?;
        let xorname = rand::random();
        let type_tag = 575_756_443;
        let xorurl = XorUrlEncoder::encode(
            xorname,
            None,
            type_tag,
            SafeDataType::UnpublishedImmutableData,
            SafeContentType::Raw,
            None,
            None,
            None,
            None,
            None,
            XorUrlBase::Base32z,
        )?;

        match safe.fetch(&xorurl, None).await {
            Ok(c) => {
                return Err(Error::Unexpected(format!(
                    "Unxpected fetched content: {:?}",
                    c
                )))
            }
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(
                    "Data type 'UnpublishedImmutableData' not supported yet".to_string()
                )
            ),
        };

        match safe.inspect(&xorurl).await {
            Ok(c) => {
                return Err(Error::Unexpected(format!(
                    "Unxpected fetched content: {:?}",
                    c
                )))
            }
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(
                    "Data type 'UnpublishedImmutableData' not supported yet".to_string()
                )
            ),
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_unsupported_with_media_type() -> Result<()> {
        let safe = new_safe_instance()?;
        let xorname = rand::random();
        let type_tag = 575_756_443;
        let xorurl = XorUrlEncoder::encode(
            xorname,
            None,
            type_tag,
            SafeDataType::UnpublishedImmutableData,
            SafeContentType::MediaType("text/html".to_string()),
            None,
            None,
            None,
            None,
            None,
            XorUrlBase::Base32z,
        )?;

        match safe.fetch(&xorurl, None).await {
            Ok(c) => {
                return Err(Error::Unexpected(format!(
                    "Unxpected fetched content: {:?}",
                    c
                )))
            }
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(
                    "Data type 'UnpublishedImmutableData' not supported yet".to_string()
                )
            ),
        };

        match safe.inspect(&xorurl).await {
            Ok(c) => {
                return Err(Error::Unexpected(format!(
                    "Unxpected fetched content: {:?}",
                    c
                )))
            }
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(
                    "Data type 'UnpublishedImmutableData' not supported yet".to_string()
                )
            ),
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_published_immutable_data_with_path() -> Result<()> {
        let mut safe = new_safe_instance()?;
        let data = b"Something super immutable";
        let xorurl = safe
            .files_put_published_immutable(data, None, false)
            .await?;

        let mut xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        let path = "/some_relative_filepath";
        xorurl_encoder.set_path(path);
        match safe.fetch(&xorurl_encoder.to_string(), None).await {
            Ok(c) => {
                return Err(Error::Unexpected(format!(
                    "Unxpected fetched content: {:?}",
                    c
                )))
            }
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(format!(
                    "Cannot get relative path of Immutable Data \"{}\"",
                    path
                ))
            ),
        };

        // test the same but a file with some media type
        let xorurl = safe
            .files_put_published_immutable(data, Some("text/plain"), false)
            .await?;

        let mut xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        xorurl_encoder.set_path("/some_relative_filepath");
        let url_with_path = xorurl_encoder.to_string();
        match safe.fetch(&url_with_path, None).await {
            Ok(c) => {
                return Err(Error::Unexpected(format!(
                    "Unxpected fetched content: {:?}",
                    c
                )))
            }
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(format!(
                    "Cannot get relative path of Immutable Data \"{}\"",
                    path
                ))
            ),
        };
        Ok(())
    }
}
