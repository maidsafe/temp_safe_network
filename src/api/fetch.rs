// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::files::FilesMap;
use super::helpers::get_subnames_host_and_path;
use super::nrs::{xorname_from_nrs_string, NRS_MAP_TYPE_TAG};
use super::xorurl::SafeContentType;
pub use super::xorurl::SafeDataType;

use super::{Error, ResultReturn, Safe, XorName, XorUrlEncoder};
use log::{debug, info};

#[derive(Debug, PartialEq)]
pub enum SafeData {
    Key {
        xorname: XorName,
    },
    Wallet {
        xorname: XorName,
        type_tag: u64,
        data_type: SafeDataType,
    },
    FilesContainer {
        xorname: XorName,
        type_tag: u64,
        version: u64,
        files_map: FilesMap,
        data_type: SafeDataType,
    },
    // TODO: Enable preventing resolution
    // NrsMapContainer {
    //     xorname: XorName,
    //     type_tag: u64,
    //     version: u64,
    //     nrs_map: NrsMap,
    //     data_type: SafeDataType,
    // },
    PublishedImmutableData {
        xorname: XorName,
        data: Vec<u8>,
    },
}

#[allow(dead_code)]
impl Safe {
    /// # Retrieve data from a xorurl
    ///
    /// ## Examples
    ///
    /// ### Fetch FilesContainer relative path file
    /// ```rust
    /// # use safe_cli::{Safe, SafeData};
    /// # use unwrap::unwrap;
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::new("base32z".to_string());
    /// # unwrap!(safe.connect("", Some("fake-credentials")));
    /// let (xorurl, _, _) = unwrap!(safe.files_container_create("tests/testfolder/", None, true, false));
    ///
    /// let safe_data = unwrap!( safe.fetch( &format!( "{}/test.md", &xorurl ) ) );
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
    pub fn fetch(&self, xorurl: &str) -> ResultReturn<SafeData> {
        debug!("Attempting to fetch url: {}", xorurl);
        let the_xor =
            XorUrlEncoder::from_url(xorurl).or_else(|err| -> ResultReturn<XorUrlEncoder> {
                info!(
                    "Falling back to NRS. XorUrl decoding failed with: {:?}",
                    err
                );

                let (sub_names, host_str, path) = get_subnames_host_and_path(xorurl)?;
                let hashed_host = xorname_from_nrs_string(&host_str)?;

                let encoded_xor = XorUrlEncoder::new(
                    hashed_host,
                    NRS_MAP_TYPE_TAG,
                    SafeDataType::PublishedSeqAppendOnlyData,
                    SafeContentType::NrsMapContainer,
                    Some(&path),
                    Some(sub_names),
                );

                debug!(
                    "Checking NRS system for URL: {}",
                    encoded_xor.to_string("base32z")?
                );
                Ok(encoded_xor)
            })?;

        let the_xorurl = the_xor.to_string("base32z")?;
        info!("URL parsed successfully, fetching: {}", the_xorurl);
        let path = the_xor.path();
        let sub_names = the_xor.sub_names();

        debug!("Fetching content of type: {:?}", the_xor.content_type());

        // TODO: pass option to get raw content AKA: Do not resolve beyond first thing.
        match the_xor.content_type() {
            SafeContentType::Raw => match the_xor.data_type() {
                SafeDataType::CoinBalance => Ok(SafeData::Key {
                    xorname: the_xor.xorname(),
                    // TODO: perhaps also return the balance if sk provided?
                }),
                SafeDataType::PublishedImmutableData => {
                    let data = self.files_get_published_immutable(&xorurl)?;
                    Ok(SafeData::PublishedImmutableData {
                        xorname: the_xor.xorname(),
                        data,
                    })
                }
                other => Err(Error::ContentError(format!(
                    "Data type '{:?}' not supported yet by fetch",
                    other
                ))),
            },
            SafeContentType::Wallet => Ok(SafeData::Wallet {
                xorname: the_xor.xorname(),
                type_tag: the_xor.type_tag(),
                data_type: the_xor.data_type(),
            }),
            SafeContentType::FilesContainer => {
                let (version, files_map) = self.files_container_get_latest(&the_xorurl)?;

                debug!(
                    "Files container found w/ v:{}, on data type: {}, containing: {:?}",
                    version,
                    the_xor.data_type(),
                    files_map
                );

                if path != "/" && !path.is_empty() {
                    // TODO: Count how many redirects we've done... prevent looping forever
                    let file_item = match files_map.get(path) {
                        Some(item_data) => item_data,
                        None => {
                            return Err(Error::ContentError(format!(
                                "No data found for path \"{}\" on the FilesContainer at \"{}\"",
                                path, the_xorurl
                            )))
                        }
                    };

                    let new_target_xorurl = match file_item.get("link") {
						Some(path_data) => path_data,
						None => return Err(Error::ContentError(format!("FileItem is corrupt. It is missing a \"link\" property at path, \"{}\" on the FilesContainer at: {} ", path, the_xorurl))),
					};

                    let path_data = self.fetch(new_target_xorurl);
                    return path_data;
                }

                Ok(SafeData::FilesContainer {
                    xorname: the_xor.xorname(),
                    type_tag: the_xor.type_tag(),
                    version,
                    files_map,
                    data_type: the_xor.data_type(),
                })
            }
            SafeContentType::NrsMapContainer => {
                let (version, nrs_map) = self.nrs_map_container_get_latest(&the_xorurl)?;
                debug!(
                    "Nrs map container found w/ v:{}, of type: {}, containing: {:?}",
                    version,
                    the_xor.data_type(),
                    nrs_map
                );

                let mut new_target_xorurl = nrs_map.get_default_link()?;

                debug!(
                    "Fetch found a default link for domain: \"{}\"",
                    new_target_xorurl
                );

                if !sub_names.is_empty() {
                    new_target_xorurl = nrs_map.resolve_for_subnames(sub_names)?;

                    debug!("Resolved target from subnames: {}", new_target_xorurl);
                }

                let url_with_path = format!("{}{}", &new_target_xorurl, path);

                info!("Resolving target from resolvable map: {}", url_with_path);

                // TODO: Properly prevent resolution
                // if prevent_resolution {
                // 	return Ok(SafeData::NrsMapContainer {
                // 		xorname: the_xor.xorname(),
                // 		type_tag: the_xor.type_tag(),
                // 		version,
                // 		nrs_map,
                // 		data_type: the_xor.data_type(),
                // 	})
                // }

                self.fetch(&url_with_path)
            }
        }
    }
}

// Unit Tests

#[test]
fn test_fetch_key() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let preload_amount = "1324.12";
    let (xorurl, _key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::Key {
                xorname: xorurl_encoder.xorname()
            }
    );
}

#[test]
fn test_fetch_wallet() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let xorurl = unwrap!(safe.wallet_create());

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::Wallet {
                xorname: xorurl_encoder.xorname(),
                type_tag: 1_000,
                data_type: SafeDataType::SeqMutableData,
            }
    );
}

#[test]
fn test_fetch_files_container() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
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
                version: 1,
                files_map,
                data_type: SafeDataType::PublishedSeqAppendOnlyData,
            }
    );

    let xorurl_with_path = format!("{}/subfolder/subexists.md", xorurl);
    let xorurl_encoder_with_path = unwrap!(XorUrlEncoder::from_url(&xorurl_with_path));
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
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;

    let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("")).unwrap();

    let (xorurl, _, the_files_map) =
        unwrap!(safe.files_container_create("tests/testfolder", None, true, false));

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));

    let (_nrs_map_xorurl, _, _nrs_map) =
        unwrap!(safe.nrs_map_container_create(&site_name, Some(&xorurl), true, false));

    println!("NRS CREATED???? {:?}", _nrs_map);
    println!("fetching CREATED site nammeee???? safe://{}", site_name);

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
            assert_eq!(version, 1);
            assert_eq!(data_type, SafeDataType::PublishedSeqAppendOnlyData);
            assert_eq!(files_map, the_files_map);
        }
        _ => panic!("Nrs map container was not returned."),
    }
}

#[test]
fn test_fetch_published_immutable_data() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let data = b"Something super immutable";
    let xorurl = safe.files_put_published_immutable(data).unwrap();

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::PublishedImmutableData {
                xorname: xorurl_encoder.xorname(),
                data: data.to_vec()
            }
    );
}

#[test]
fn test_fetch_unsupported() {
    use super::xorurl::create_random_xorname;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
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
