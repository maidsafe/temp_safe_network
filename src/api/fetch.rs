// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::files::FilesMap;
use super::helpers::get_subnames_host_path_and_version;
use super::nrs_map::NrsMap;
pub use super::wallet::WalletSpendableBalances;
pub use super::xorurl::{SafeContentType, SafeDataType, XorUrlEncoder};
use super::{Error, ResultReturn, Safe, XorName};
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct NrsMapContainerInfo {
    pub public_name: String,
    pub xorurl: String,
    pub xorname: XorName,
    pub type_tag: u64,
    pub version: u64,
    pub nrs_map: NrsMap,
    pub data_type: SafeDataType,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum SafeData {
    SafeKey {
        xorname: XorName,
        resolved_from: Option<NrsMapContainerInfo>,
    },
    Wallet {
        xorname: XorName,
        type_tag: u64,
        balances: WalletSpendableBalances,
        data_type: SafeDataType,
        resolved_from: Option<NrsMapContainerInfo>,
    },
    FilesContainer {
        xorname: XorName,
        type_tag: u64,
        version: u64,
        files_map: FilesMap,
        data_type: SafeDataType,
        resolved_from: Option<NrsMapContainerInfo>,
    },
    PublishedImmutableData {
        xorname: XorName,
        data: Vec<u8>,
        resolved_from: Option<NrsMapContainerInfo>,
    },
}

#[allow(dead_code)]
impl Safe {
    /// # Retrieve data from a safe:// URL
    ///
    /// ## Examples
    ///
    /// ### Fetch FilesContainer relative path file
    /// ```rust
    /// # use safe_cli::{Safe, SafeData};
    /// # use unwrap::unwrap;
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::new("base32z");
    /// # unwrap!(safe.connect("", Some("fake-credentials")));
    /// let (xorurl, _, _) = unwrap!(safe.files_container_create("tests/testfolder/", None, true, false));
    ///
    /// let safe_data = unwrap!( safe.fetch( &format!( "{}/test.md", &xorurl.replace("?v=0", "") ) ) );
    /// let data_string = match safe_data {
    /// 	SafeData::PublishedImmutableData { data, .. } => {
    /// 		match String::from_utf8(data) {
    /// 			Ok(string) => string,
    /// 			Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    /// 		}
    /// 	}
    /// 	other => panic!(
    /// 		"Content type '{:?}' should not have been found. This should be immutable data.",
    /// 		other
    /// 	)
    /// };
    ///
    /// assert!(data_string.starts_with("hello tests!"));
    /// ```
    pub fn fetch(&self, url: &str) -> ResultReturn<SafeData> {
        let mut the_xor = Safe::parse_url(url)?;
        let xorurl = the_xor.to_string()?;
        info!("URL parsed successfully, fetching: {}", xorurl);
        debug!("Fetching content of type: {:?}", the_xor.content_type());

        // TODO: pass option to get raw content AKA: Do not resolve beyond first thing.
        match the_xor.content_type() {
            SafeContentType::Raw => match the_xor.data_type() {
                SafeDataType::SafeKey => Ok(SafeData::SafeKey {
                    xorname: the_xor.xorname(),
                    resolved_from: None,
                    // TODO: perhaps also return the balance if sk provided?
                }),
                SafeDataType::PublishedImmutableData => {
                    let data = self.files_get_published_immutable(&url)?;
                    Ok(SafeData::PublishedImmutableData {
                        xorname: the_xor.xorname(),
                        resolved_from: None,
                        data,
                    })
                }
                other => Err(Error::ContentError(format!(
                    "Data type '{:?}' not supported yet by fetch",
                    other
                ))),
            },
            SafeContentType::Wallet => {
                let balances = self.wallet_get(&url)?;
                Ok(SafeData::Wallet {
                    xorname: the_xor.xorname(),
                    type_tag: the_xor.type_tag(),
                    balances,
                    data_type: the_xor.data_type(),
                    resolved_from: None,
                })
            }
            SafeContentType::FilesContainer => {
                let (version, files_map) = self.files_container_get(&xorurl)?;

                debug!(
                    "Files container found w/ v:{}, on data type: {}, containing: {:?}",
                    version,
                    the_xor.data_type(),
                    files_map
                );

                let path = the_xor.path();
                if path != "/" && !path.is_empty() {
                    // TODO: Count how many redirects we've done... prevent looping forever
                    // TODO: Move this logic (resolver) to the FilesMap struct
                    match files_map.get(path) {
                        Some(file_item) => {
                            let new_target_xorurl = match file_item.get("link") {
        						Some(path_data) => path_data,
        						None => return Err(Error::ContentError(format!("FileItem is corrupt. It is missing a \"link\" property at path, \"{}\" on the FilesContainer at: {} ", path, xorurl))),
        					};

                            self.fetch(new_target_xorurl)
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
                                Ok(SafeData::FilesContainer {
                                    xorname: the_xor.xorname(),
                                    type_tag: the_xor.type_tag(),
                                    version,
                                    files_map: filtered_filesmap,
                                    data_type: the_xor.data_type(),
                                    resolved_from: None,
                                })
                            }
                        }
                    }
                } else {
                    Ok(SafeData::FilesContainer {
                        xorname: the_xor.xorname(),
                        type_tag: the_xor.type_tag(),
                        version,
                        files_map,
                        data_type: the_xor.data_type(),
                        resolved_from: None,
                    })
                }
            }
            SafeContentType::NrsMapContainer => {
                let (version, nrs_map) = self
                    .nrs_map_container_get(&xorurl)
                    .map_err(|_| Error::ContentNotFound(format!("Content not found at {}", url)))?;

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
                    xorurl_encoder.set_path(&format!(
                        "{}{}",
                        xorurl_encoder.path(),
                        the_xor.path()
                    ));
                }
                let url_with_path = xorurl_encoder.to_string()?;
                debug!("Resolving target from resolvable map: {}", url_with_path);

                let (_, public_name, _, _) = get_subnames_host_path_and_version(url)?;
                let content = self.fetch(&url_with_path)?;
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

                // TODO: find a simpler way to change the 'resolved_from' filed of the enum
                embed_resolved_from(content, nrs_map_container)
            }
        }
    }
}

fn embed_resolved_from(
    content: SafeData,
    nrs_map_container: NrsMapContainerInfo,
) -> ResultReturn<SafeData> {
    let safe_data = match content {
        SafeData::SafeKey { xorname, .. } => SafeData::SafeKey {
            xorname,
            resolved_from: Some(nrs_map_container),
        },
        SafeData::Wallet {
            xorname,
            type_tag,
            balances,
            data_type,
            ..
        } => SafeData::Wallet {
            xorname,
            type_tag,
            balances,
            data_type,
            resolved_from: Some(nrs_map_container),
        },
        SafeData::FilesContainer {
            xorname,
            type_tag,
            version,
            files_map,
            data_type,
            ..
        } => SafeData::FilesContainer {
            xorname,
            type_tag,
            version,
            files_map,
            data_type,
            resolved_from: Some(nrs_map_container),
        },
        SafeData::PublishedImmutableData { xorname, data, .. } => {
            SafeData::PublishedImmutableData {
                xorname,
                data,
                resolved_from: Some(nrs_map_container),
            }
        }
    };
    Ok(safe_data)
}

// Unit Tests

#[test]
fn test_fetch_key() {
    use super::xorurl::XorUrlEncoder;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let preload_amount = "1324.12";
    let (xorurl, _key_pair) = unwrap!(safe.keys_create_preload_test_coins(preload_amount));

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::SafeKey {
                xorname: xorurl_encoder.xorname(),
                resolved_from: None,
            }
    );
}

#[test]
fn test_fetch_wallet() {
    use super::xorurl::XorUrlEncoder;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let xorurl = unwrap!(safe.wallet_create());

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::Wallet {
                xorname: xorurl_encoder.xorname(),
                type_tag: 1_000,
                balances: WalletSpendableBalances::default(),
                data_type: SafeDataType::SeqMutableData,
                resolved_from: None,
            }
    );
}

#[test]
fn test_fetch_files_container() {
    use super::xorurl::XorUrlEncoder;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    safe.connect("", Some("")).unwrap();

    let (xorurl, _, files_map) =
        unwrap!(safe.files_container_create("tests/testfolder", None, true, false));

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));

    assert!(
        content
            == SafeData::FilesContainer {
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
}

#[test]
fn test_fetch_resolvable_container() {
    use super::xorurl::XorUrlEncoder;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;

    let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

    let mut safe = Safe::new("base32z");
    safe.connect("", Some("")).unwrap();

    let (xorurl, _, the_files_map) =
        unwrap!(safe.files_container_create("tests/testfolder", None, true, false));

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_content_version(Some(0));
    let (_nrs_map_xorurl, _, _nrs_map) = unwrap!(safe.nrs_map_container_create(
        &site_name,
        &unwrap!(xorurl_encoder.to_string()),
        true,
        true,
        false
    ));

    let content = unwrap!(safe.fetch(&format!("safe://{}", site_name)));

    // this should resolve to a FilesContainer until we enable prevent resolution.
    match content {
        SafeData::FilesContainer {
            xorname,
            type_tag,
            version,
            files_map,
            data_type,
            ..
        } => {
            assert_eq!(xorname, xorurl_encoder.xorname());
            assert_eq!(type_tag, 1_100);
            assert_eq!(version, 0);
            assert_eq!(data_type, SafeDataType::PublishedSeqAppendOnlyData);
            assert_eq!(files_map, the_files_map);
        }
        _ => panic!("Nrs map container was not returned."),
    }
}

#[test]
fn test_fetch_resolvable_map_data() {
    use super::xorurl::XorUrlEncoder;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;

    let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

    let mut safe = Safe::new("base32z");
    safe.connect("", Some("")).unwrap();

    let (xorurl, _, _the_files_map) =
        unwrap!(safe.files_container_create("tests/testfolder", None, true, false));

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    xorurl_encoder.set_content_version(Some(0));
    let (nrs_map_xorurl, _, the_nrs_map) = unwrap!(safe.nrs_map_container_create(
        &site_name,
        &unwrap!(xorurl_encoder.to_string()),
        true,
        true,
        false
    ));

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&nrs_map_xorurl));
    let content = unwrap!(safe.fetch(&format!("safe://{}", site_name)));

    // this should resolve to a FilesContainer until we enable prevent resolution.
    match content {
        SafeData::FilesContainer {
            resolved_from: Some(nrs_map_container),
            ..
        } => {
            assert_eq!(nrs_map_container.xorname, xorurl_encoder.xorname());
            assert_eq!(nrs_map_container.type_tag, 1_500);
            assert_eq!(nrs_map_container.version, 0);
            assert_eq!(
                nrs_map_container.data_type,
                SafeDataType::PublishedSeqAppendOnlyData
            );
            assert_eq!(nrs_map_container.nrs_map, the_nrs_map);
        }
        _ => panic!("Nrs map container was not returned."),
    }
}

#[test]
fn test_fetch_published_immutable_data() {
    use super::xorurl::XorUrlEncoder;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let data = b"Something super immutable";
    let xorurl = safe.files_put_published_immutable(data).unwrap();

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::PublishedImmutableData {
                xorname: xorurl_encoder.xorname(),
                data: data.to_vec(),
                resolved_from: None,
            }
    );
}

#[test]
fn test_fetch_unsupported() {
    use super::helpers::create_random_xorname;
    use super::xorurl::XorUrlEncoder;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let xorname = create_random_xorname();
    let type_tag = 575_756_443;
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        type_tag,
        SafeDataType::UnpublishedImmutableData,
        SafeContentType::Raw,
        None,
        None,
        None,
        "base32z"
    ));
    match safe.fetch(&xorurl) {
        Ok(c) => panic!(format!("Unxpected fetched content: {:?}", c)),
        Err(msg) => assert_eq!(
            msg,
            Error::ContentError(
                "Data type 'UnpublishedImmutableData' not supported yet by fetch".to_string()
            )
        ),
    };
}
