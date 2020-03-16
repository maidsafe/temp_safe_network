// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    files::FilesMap, helpers::get_subnames_host_path_and_version, nrs_map::NrsMap, Safe, XorName,
};
pub use super::{
    wallet::WalletSpendableBalances,
    xorurl::{SafeContentType, SafeDataType, XorUrlBase, XorUrlEncoder},
};
use crate::{Error, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};

pub type Range = Option<(Option<u64>, Option<u64>)>;

// Maximum number of indirections allowed when resolving a safe:// URL following links
const INDIRECTION_LIMIT: u8 = 5;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct NrsMapContainerInfo {
    pub public_name: String,
    pub xorurl: String,
    pub xorname: XorName,
    pub type_tag: u64,
    pub version: u64,
    pub nrs_map: NrsMap,
    pub data_type: SafeDataType,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum SafeData {
    SafeKey {
        xorurl: String,
        xorname: XorName,
        resolved_from: Option<NrsMapContainerInfo>,
    },
    Wallet {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        balances: WalletSpendableBalances,
        data_type: SafeDataType,
        resolved_from: Option<NrsMapContainerInfo>,
    },
    FilesContainer {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        version: u64,
        files_map: FilesMap,
        data_type: SafeDataType,
        resolved_from: Option<NrsMapContainerInfo>,
    },
    PublishedImmutableData {
        xorurl: String,
        xorname: XorName,
        data: Vec<u8>,
        resolved_from: Option<NrsMapContainerInfo>,
        media_type: Option<String>,
    },
}

enum ResolutionStep {
    NrsMap(NrsMapContainerInfo),
    Container(SafeData),
    Data(SafeData),
}

impl Safe {
    /// # Retrieve data from a safe:// URL
    ///
    /// ## Examples
    ///
    /// ### Fetch FilesContainer relative path file
    /// ```rust
    /// # use safe_api::{Safe, fetch::SafeData};
    /// # use unwrap::unwrap;
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::default();
    /// # unwrap!(safe.connect("", Some("fake-credentials")));
    /// async_std::task::block_on(async {
    ///     let (xorurl, _, _) = unwrap!(safe.files_container_create(Some("../testdata/"), None, true, false).await);
    ///
    ///     let safe_data = unwrap!( safe.fetch( &format!( "{}/test.md", &xorurl.replace("?v=0", "") ), None ).await );
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
    /// });
    /// ```
    pub async fn fetch(&self, url: &str, range: Range) -> Result<SafeData> {
        retrieve_from_url(self, url, true, range).await
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
    /// # use unwrap::unwrap;
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::default();
    /// # unwrap!(safe.connect("", Some("fake-credentials")));
    /// async_std::task::block_on(async {
    ///     let (xorurl, _, _) = unwrap!(safe.files_container_create(Some("../testdata/"), None, true, false).await);
    ///
    ///     let safe_data = unwrap!( safe.inspect( &format!( "{}/test.md", &xorurl.replace("?v=0", "") ) ).await );
    ///     let data_string = match safe_data {
    ///         SafeData::PublishedImmutableData { data, media_type, .. } => {
    ///             assert_eq!(media_type, Some("text/markdown".to_string()));
    ///             assert!(data.is_empty());
    ///         }
    ///         other => panic!(
    ///             "Content type '{:?}' should not have been found. This should be immutable data.",
    ///             other
    ///         )
    ///     };
    ///
    /// });
    /// ```
    pub async fn inspect(&self, url: &str) -> Result<SafeData> {
        retrieve_from_url(self, url, false, None).await
    }
}

async fn retrieve_from_url(
    safe: &Safe,
    url: &str,
    retrieve_data: bool,
    range: Range,
) -> Result<SafeData> {
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
    let mut resolved_content = Vec::<ResolutionStep>::default();
    let mut next_to_resolve = Some(current_xorurl_encoder);
    let mut indirections_count = 0;
    while let Some(next_xorurl_encoder) = next_to_resolve {
        if indirections_count == INDIRECTION_LIMIT {
            break;
        }

        let (step, next) =
            resolve_one_indirection(safe, url, next_xorurl_encoder, retrieve_data, range).await?;

        resolved_content.push(step);
        next_to_resolve = next;
        indirections_count += 1;
    }

    // Contruct return data using the last and first items in the steps list
    // TODO: return the complete resolution list from the 'inspect' API
    if let Some(ResolutionStep::Data(safe_data)) = resolved_content.pop() {
        if resolved_content.is_empty() {
            Ok(safe_data)
        } else {
            match &resolved_content[0] {
                ResolutionStep::NrsMap(nrs_map_container) => {
                    // original URL was an NRS-URL
                    // TODO: find a simpler way to change the 'resolved_from' field of the enum
                    embed_resolved_from(safe_data, nrs_map_container.clone())
                }
                ResolutionStep::Container(_) => Ok(safe_data),
                ResolutionStep::Data(_) => {
                    Err(Error::Unexpected(format!("Failed to resolve {}", url)))
                }
            }
        }
    } else {
        // weird...last element should always be a ResolutionStep::Data type
        Err(Error::Unexpected(format!("Failed to resolve {}", url)))
    }
}

async fn resolve_one_indirection(
    safe: &Safe,
    original_url: &str,
    mut the_xor: XorUrlEncoder,
    retrieve_data: bool,
    range: Range,
) -> Result<(ResolutionStep, Option<XorUrlEncoder>)> {
    let xorurl = the_xor.to_string()?;
    match the_xor.content_type() {
        SafeContentType::FilesContainer => {
            let (version, files_map) = safe.files_container_get(&xorurl).await?;
            debug!(
                "Files container found with v:{}, on data type: {}, containing: {:?}",
                version,
                the_xor.data_type(),
                files_map
            );

            let path = the_xor.path();
            if path != "/" && !path.is_empty() {
                // TODO: Move this logic (resolver) to the FilesMap struct
                match &files_map.get(path) {
                    Some(file_item) => {
                        let new_target_xorurl = match file_item.get("link") {
                            Some(path_data) => XorUrlEncoder::from_url(path_data)?,
                            None => return Err(Error::ContentError(format!("FileItem is corrupt. It is missing a \"link\" property at path, \"{}\" on the FilesContainer at: {} ", path, xorurl))),
                        };

                        let safe_data = SafeData::FilesContainer {
                            xorurl,
                            xorname: the_xor.xorname(),
                            type_tag: the_xor.type_tag(),
                            version,
                            files_map,
                            data_type: the_xor.data_type(),
                            resolved_from: None,
                        };

                        Ok((
                            ResolutionStep::Container(safe_data),
                            Some(new_target_xorurl),
                        ))
                    }
                    None => {
                        let mut filtered_filesmap = FilesMap::default();
                        let folder_path = if !path.ends_with('/') {
                            format!("{}/", path)
                        } else {
                            path.to_string()
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
                            let safe_data = SafeData::FilesContainer {
                                xorurl,
                                xorname: the_xor.xorname(),
                                type_tag: the_xor.type_tag(),
                                version,
                                files_map: filtered_filesmap,
                                data_type: the_xor.data_type(),
                                resolved_from: None,
                            };

                            Ok((ResolutionStep::Data(safe_data), None))
                        }
                    }
                }
            } else {
                let safe_data = SafeData::FilesContainer {
                    xorurl,
                    xorname: the_xor.xorname(),
                    type_tag: the_xor.type_tag(),
                    version,
                    files_map,
                    data_type: the_xor.data_type(),
                    resolved_from: None,
                };

                Ok((ResolutionStep::Data(safe_data), None))
            }
        }
        SafeContentType::NrsMapContainer => {
            let (version, nrs_map) = safe.nrs_map_container_get(&xorurl).await.map_err(|_| {
                Error::ContentNotFound(format!("Content not found at {}", original_url))
            })?;

            debug!(
                "Nrs map container found w/ v:{}, of type: {}, containing: {:?}",
                version,
                the_xor.data_type(),
                nrs_map
            );

            let new_target_xorurl = nrs_map.resolve_for_subnames(the_xor.sub_names())?;
            debug!("Resolved target: {}", new_target_xorurl);

            let mut xorurl_encoder = XorUrlEncoder::from_url(&new_target_xorurl)?;
            if xorurl_encoder.path().is_empty() {
                xorurl_encoder.set_path(the_xor.path());
            } else if !the_xor.path().is_empty() {
                xorurl_encoder.set_path(&format!("{}{}", xorurl_encoder.path(), the_xor.path()));
            }
            let url_with_path = xorurl_encoder.to_string()?;
            debug!("Resolving target from resolvable map: {}", url_with_path);

            let (_, public_name, _, _) = get_subnames_host_path_and_version(original_url)?;
            the_xor.set_path(""); // we don't want the path, just the NRS Map xorurl and version
            let nrs_map_container = NrsMapContainerInfo {
                public_name,
                xorurl: the_xor.to_string()?,
                xorname: the_xor.xorname(),
                type_tag: the_xor.type_tag(),
                version,
                nrs_map,
                data_type: the_xor.data_type(),
            };

            Ok((
                ResolutionStep::NrsMap(nrs_map_container),
                Some(xorurl_encoder),
            ))
        }
        SafeContentType::Raw => match the_xor.data_type() {
            SafeDataType::SafeKey => {
                let safe_data = SafeData::SafeKey {
                    xorurl,
                    xorname: the_xor.xorname(),
                    resolved_from: None,
                };
                Ok((ResolutionStep::Data(safe_data), None))
            }
            SafeDataType::PublishedImmutableData => {
                retrieve_immd(safe, the_xor, xorurl, retrieve_data, None, range).await
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
                    the_xor,
                    xorurl,
                    retrieve_data,
                    Some(media_type_str),
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
                safe.wallet_get(&original_url).await?
            } else {
                WalletSpendableBalances::new()
            };

            let safe_data = SafeData::Wallet {
                xorurl,
                xorname: the_xor.xorname(),
                type_tag: the_xor.type_tag(),
                balances,
                data_type: the_xor.data_type(),
                resolved_from: None,
            };

            Ok((ResolutionStep::Data(safe_data), None))
        }
    }
}

async fn retrieve_immd(
    safe: &Safe,
    the_xor: XorUrlEncoder,
    xorurl: String,
    retrieve_data: bool,
    media_type: Option<String>,
    range: Range,
) -> Result<(ResolutionStep, Option<XorUrlEncoder>)> {
    if !the_xor.path().is_empty() {
        return Err(Error::ContentError(format!(
            "Cannot get relative path of Immutable Data {:?}",
            the_xor.path()
        )));
    };

    let data = if retrieve_data {
        safe.files_get_published_immutable(&xorurl, range).await?
    } else {
        vec![]
    };

    let safe_data = SafeData::PublishedImmutableData {
        xorurl,
        xorname: the_xor.xorname(),
        resolved_from: None,
        data,
        media_type,
    };

    Ok((ResolutionStep::Data(safe_data), None))
}
fn embed_resolved_from(
    content: SafeData,
    nrs_map_container: NrsMapContainerInfo,
) -> Result<SafeData> {
    let safe_data = match content {
        SafeData::SafeKey {
            xorurl, xorname, ..
        } => SafeData::SafeKey {
            xorurl,
            xorname,
            resolved_from: Some(nrs_map_container),
        },
        SafeData::Wallet {
            xorurl,
            xorname,
            type_tag,
            balances,
            data_type,
            ..
        } => SafeData::Wallet {
            xorurl,
            xorname,
            type_tag,
            balances,
            data_type,
            resolved_from: Some(nrs_map_container),
        },
        SafeData::FilesContainer {
            xorurl,
            xorname,
            type_tag,
            version,
            files_map,
            data_type,
            ..
        } => SafeData::FilesContainer {
            xorurl,
            xorname,
            type_tag,
            version,
            files_map,
            data_type,
            resolved_from: Some(nrs_map_container),
        },
        SafeData::PublishedImmutableData {
            xorurl,
            xorname,
            data,
            media_type,
            ..
        } => SafeData::PublishedImmutableData {
            xorurl,
            xorname,
            data,
            media_type,
            resolved_from: Some(nrs_map_container),
        },
    };
    Ok(safe_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::xorurl::XorUrlEncoder;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::io::Read;
    use unwrap::unwrap;

    #[tokio::test]
    async fn test_fetch_key() {
        let mut safe = Safe::default();
        unwrap!(safe.connect("", Some("fake-credentials")));
        let preload_amount = "1324.12";
        let (xorurl, _key_pair) =
            unwrap!(safe.keys_create_preload_test_coins(preload_amount).await);

        let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        let content = unwrap!(safe.fetch(&xorurl, None).await);
        assert!(
            content
                == SafeData::SafeKey {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    resolved_from: None,
                }
        );

        // let's also compare it with the result from inspecting the URL
        let inspected_url = unwrap!(safe.inspect(&xorurl).await);
        assert_eq!(content, inspected_url);
    }

    #[tokio::test]
    async fn test_fetch_wallet() {
        let mut safe = Safe::default();
        unwrap!(safe.connect("", Some("fake-credentials")));
        let xorurl = unwrap!(safe.wallet_create().await);

        let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        let content = unwrap!(safe.fetch(&xorurl, None).await);
        assert!(
            content
                == SafeData::Wallet {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    type_tag: 1_000,
                    balances: WalletSpendableBalances::default(),
                    data_type: SafeDataType::SeqMutableData,
                    resolved_from: None,
                }
        );

        // let's also compare it with the result from inspecting the URL
        let inspected_url = unwrap!(safe.inspect(&xorurl).await);
        assert_eq!(content, inspected_url);
    }

    #[tokio::test]
    async fn test_fetch_files_container() {
        let mut safe = Safe::default();
        unwrap!(safe.connect("", Some("fake-credentials")));
        safe.connect("", Some("")).unwrap();
        let (xorurl, _, files_map) = unwrap!(
            safe.files_container_create(Some("../testdata/"), None, true, false)
                .await
        );

        let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        let content = unwrap!(safe.fetch(&xorurl, None).await);

        assert!(
            content
                == SafeData::FilesContainer {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    type_tag: 1_100,
                    version: 0,
                    files_map,
                    data_type: SafeDataType::PublishedSeqAppendOnlyData,
                    resolved_from: None,
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
        let inspected_url = unwrap!(safe.inspect(&xorurl).await);
        assert_eq!(content, inspected_url);
    }

    #[tokio::test]
    async fn test_fetch_resolvable_container() {
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

        let mut safe = Safe::default();
        safe.connect("", Some("")).unwrap();

        let (xorurl, _, the_files_map) = unwrap!(
            safe.files_container_create(Some("../testdata/"), None, true, false)
                .await
        );

        let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        xorurl_encoder.set_content_version(Some(0));
        let (_nrs_map_xorurl, _, _nrs_map) = unwrap!(
            safe.nrs_map_container_create(
                &site_name,
                &unwrap!(xorurl_encoder.to_string()),
                true,
                true,
                false
            )
            .await
        );

        let nrs_url = format!("safe://{}", site_name);
        let content = unwrap!(safe.fetch(&nrs_url, None).await);

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
                assert_eq!(*xorurl, unwrap!(xorurl_encoder.to_string()));
                assert_eq!(*xorname, xorurl_encoder.xorname());
                assert_eq!(*type_tag, 1_100);
                assert_eq!(*version, 0);
                assert_eq!(*data_type, SafeDataType::PublishedSeqAppendOnlyData);
                assert_eq!(*files_map, the_files_map);
            }
            _ => panic!("Nrs map container was not returned."),
        }

        // let's also compare it with the result from inspecting the URL
        let inspected_url = unwrap!(safe.inspect(&nrs_url).await);
        assert_eq!(content, inspected_url);
    }

    #[tokio::test]
    async fn test_fetch_resolvable_map_data() {
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

        let mut safe = Safe::default();
        safe.connect("", Some("")).unwrap();
        let (xorurl, _, _the_files_map) = unwrap!(
            safe.files_container_create(Some("../testdata/"), None, true, false)
                .await
        );

        let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        xorurl_encoder.set_content_version(Some(0));
        let (nrs_map_xorurl, _, the_nrs_map) = unwrap!(
            safe.nrs_map_container_create(
                &site_name,
                &unwrap!(xorurl_encoder.to_string()),
                true,
                true,
                false
            )
            .await
        );

        let nrs_xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&nrs_map_xorurl));
        let nrs_url = format!("safe://{}", site_name);
        let content = unwrap!(safe.fetch(&nrs_url, None).await);

        // this should resolve to a FilesContainer until we enable prevent resolution.
        match &content {
            SafeData::FilesContainer {
                xorurl,
                resolved_from: Some(nrs_map_container),
                ..
            } => {
                assert_eq!(*xorurl, unwrap!(xorurl_encoder.to_string()));
                assert_eq!(nrs_map_container.xorname, nrs_xorurl_encoder.xorname());
                assert_eq!(nrs_map_container.version, 0);
                assert_eq!(nrs_map_container.type_tag, 1_500);
                assert_eq!(
                    nrs_map_container.data_type,
                    SafeDataType::PublishedSeqAppendOnlyData
                );
                assert_eq!(nrs_map_container.nrs_map, the_nrs_map);
            }
            _ => panic!("Nrs map container was not returned."),
        }

        // let's also compare it with the result from inspecting the URL
        let inspected_url = unwrap!(safe.inspect(&nrs_url).await);
        assert_eq!(content, inspected_url);
    }

    #[tokio::test]
    async fn test_fetch_published_immutable_data() {
        let mut safe = Safe::default();
        unwrap!(safe.connect("", Some("fake-credentials")));
        let data = b"Something super immutable";
        let xorurl = safe
            .files_put_published_immutable(data, Some("text/plain"), false)
            .await
            .unwrap();

        let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        let content = unwrap!(safe.fetch(&xorurl, None).await);
        assert!(
            content
                == SafeData::PublishedImmutableData {
                    xorurl: xorurl.clone(),
                    xorname: xorurl_encoder.xorname(),
                    data: data.to_vec(),
                    resolved_from: None,
                    media_type: Some("text/plain".to_string())
                }
        );

        // let's also compare it with the result from inspecting the URL
        let inspected_url = unwrap!(safe.inspect(&xorurl).await);
        assert!(
            inspected_url
                == SafeData::PublishedImmutableData {
                    xorurl,
                    xorname: xorurl_encoder.xorname(),
                    data: vec![],
                    resolved_from: None,
                    media_type: Some("text/plain".to_string())
                }
        );
    }

    #[tokio::test]
    async fn test_fetch_range_published_immutable_data() {
        let mut safe = Safe::default();
        unwrap!(safe.connect("", Some("fake-credentials")));
        let saved_data = b"Something super immutable";
        let size = saved_data.len();
        let xorurl = safe
            .files_put_published_immutable(saved_data, Some("text/plain"), false)
            .await
            .unwrap();

        // Fetch first half and match
        let fetch_first_half = Some((None, Some(size as u64 / 2)));
        let content = unwrap!(safe.fetch(&xorurl, fetch_first_half).await);

        match &content {
            SafeData::PublishedImmutableData { data, .. } => {
                assert_eq!(data.clone(), saved_data[0..size / 2].to_vec());
            }
            _ => panic!("unable to fetch published immutable data was not returned."),
        }

        // Fetch second half and match
        let fetch_second_half = Some((Some(size as u64 / 2), Some(size as u64)));
        let content = unwrap!(safe.fetch(&xorurl, fetch_second_half).await);

        match &content {
            SafeData::PublishedImmutableData { data, .. } => {
                assert_eq!(data.clone(), saved_data[size / 2..size].to_vec());
            }
            _ => panic!("unable to fetch published immutable data was not returned."),
        }
    }

    #[tokio::test]
    async fn test_fetch_range_from_files_container() {
        use std::fs::File;
        let mut safe = Safe::default();
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        unwrap!(safe.connect("", Some("fake-credentials")));

        let (xorurl, _, _files_map) = unwrap!(
            safe.files_container_create(Some("../testdata/"), None, true, false)
                .await
        );

        let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        xorurl_encoder.set_content_version(Some(0));
        let (_nrs_map_xorurl, _, _nrs_map) = unwrap!(
            safe.nrs_map_container_create(
                &site_name,
                &unwrap!(xorurl_encoder.to_string()),
                true,
                true,
                false
            )
            .await
        );

        let nrs_url = format!("safe://{}/test.md", site_name);

        let mut file = File::open("../testdata/test.md").unwrap();
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data).unwrap();
        let file_size = file_data.len();

        // Fetch full file and match
        let content = unwrap!(safe.fetch(&nrs_url, None).await);
        match &content {
            SafeData::PublishedImmutableData { data, .. } => {
                assert_eq!(data.clone(), file_data.clone());
            }
            _ => panic!("unable to fetch published immutable data was not returned."),
        }

        // Fetch first half and match
        let fetch_first_half = Some((None, Some(file_size as u64 / 2)));
        let content = unwrap!(safe.fetch(&nrs_url, fetch_first_half).await);

        match &content {
            SafeData::PublishedImmutableData { data, .. } => {
                assert_eq!(data.clone(), file_data[0..file_size / 2].to_vec());
            }
            _ => panic!("unable to fetch published immutable data was not returned."),
        }

        // Fetch second half and match
        let fetch_second_half = Some((Some(file_size as u64 / 2), Some(file_size as u64)));
        let content = unwrap!(safe.fetch(&nrs_url, fetch_second_half).await);

        match &content {
            SafeData::PublishedImmutableData { data, .. } => {
                assert_eq!(data.clone(), file_data[file_size / 2..file_size].to_vec());
            }
            _ => panic!("unable to fetch published immutable data was not returned."),
        }
    }

    #[tokio::test]
    async fn test_fetch_unsupported() {
        let mut safe = Safe::default();
        unwrap!(safe.connect("", Some("fake-credentials")));
        let xorname = rand::random();
        let type_tag = 575_756_443;
        let xorurl = unwrap!(XorUrlEncoder::encode(
            xorname,
            type_tag,
            SafeDataType::UnpublishedImmutableData,
            SafeContentType::Raw,
            None,
            None,
            None,
            XorUrlBase::Base32z
        ));

        match safe.fetch(&xorurl, None).await {
            Ok(c) => panic!(format!("Unxpected fetched content: {:?}", c)),
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(
                    "Data type 'UnpublishedImmutableData' not supported yet".to_string()
                )
            ),
        };

        match safe.inspect(&xorurl).await {
            Ok(c) => panic!(format!("Unxpected fetched content: {:?}", c)),
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(
                    "Data type 'UnpublishedImmutableData' not supported yet".to_string()
                )
            ),
        };
    }

    #[tokio::test]
    async fn test_fetch_unsupported_with_media_type() {
        let mut safe = Safe::default();
        unwrap!(safe.connect("", Some("fake-credentials")));
        let xorname = rand::random();
        let type_tag = 575_756_443;
        let xorurl = unwrap!(XorUrlEncoder::encode(
            xorname,
            type_tag,
            SafeDataType::UnpublishedImmutableData,
            SafeContentType::MediaType("text/html".to_string()),
            None,
            None,
            None,
            XorUrlBase::Base32z
        ));

        match safe.fetch(&xorurl, None).await {
            Ok(c) => panic!(format!("Unxpected fetched content: {:?}", c)),
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(
                    "Data type 'UnpublishedImmutableData' not supported yet".to_string()
                )
            ),
        };

        match safe.inspect(&xorurl).await {
            Ok(c) => panic!(format!("Unxpected fetched content: {:?}", c)),
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(
                    "Data type 'UnpublishedImmutableData' not supported yet".to_string()
                )
            ),
        };
    }

    #[tokio::test]
    async fn test_fetch_published_immutable_data_with_path() {
        let mut safe = Safe::default();
        unwrap!(safe.connect("", Some("fake-credentials")));
        let data = b"Something super immutable";
        let xorurl = safe
            .files_put_published_immutable(data, None, false)
            .await
            .unwrap();

        let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        let path = "/some_relative_filepath";
        xorurl_encoder.set_path(path);
        match safe.fetch(&unwrap!(xorurl_encoder.to_string()), None).await {
            Ok(c) => panic!(format!("Unxpected fetched content: {:?}", c)),
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
            .await
            .unwrap();

        let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
        xorurl_encoder.set_path("/some_relative_filepath");
        match safe.fetch(&unwrap!(xorurl_encoder.to_string()), None).await {
            Ok(c) => panic!(format!("Unxpected fetched content: {:?}", c)),
            Err(msg) => assert_eq!(
                msg,
                Error::ContentError(format!(
                    "Cannot get relative path of Immutable Data \"{}\"",
                    path
                ))
            ),
        };
    }
}
