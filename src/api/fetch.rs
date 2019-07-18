// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::files::FilesMap;
use super::nrs::xorname_from_nrs_string;
use super::xorurl::SafeContentType;

use url::Url;

use super::{Error, ResultReturn, Safe, XorName, XorUrl, XorUrlEncoder};
use log::{debug, info};

#[derive(Debug, PartialEq)]
pub enum SafeData {
    Key {
        xorname: XorName,
    },
    Wallet {
        xorname: XorName,
        type_tag: u64,
        native_type: String,
    },
    FilesContainer {
        xorname: XorName,
        type_tag: u64,
        version: u64,
        files_map: FilesMap,
        native_type: String,
    },
    // TODO: Enable preventing resolution
    // ResolvableMapContainer {
    //     xorname: XorName,
    //     type_tag: u64,
    //     version: u64,
    //     resolvable_map: ResolvableMap,
    //     native_type: String,
    // },
    ImmutableData {
        xorname: XorName,
        data: Vec<u8>,
    },
    Unknown {
        xorname: XorName,
        type_tag: u64,
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
    /// 	SafeData::ImmutableData { data, .. } => {
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
        debug!("Attempting to fetch url: {:?}", xorurl);

        let xorurl_encoder = XorUrlEncoder::from_url(&xorurl);

        let the_xorurl: &XorUrl;

        let the_xor = match xorurl_encoder {
            Ok(encoder) => Ok(encoder),
            Err(err) => {
                let parsing_url = Url::parse(&xorurl).map_err(|parse_err| {
                    Error::InvalidXorUrl(format!(
                        "Problem parsing the SAFE:// URL {:?} : {:?}",
                        err, parse_err
                    ))
                })?;

                let host_str = parsing_url
                    .host_str()
                    .unwrap_or_else(|| "Failed parsing the URL");

                // TODO: DRY this out with XorUrlEncoder path finder
                let mut path = str::replace(parsing_url.path(), "\\", "/");
                if path == "/" {
                    path = "".to_string();
                }

                info!(
                    "Falling back to NRS. XorUrl decoding failed with: {:?}",
                    err
                );

                const RESOLVABLE_MAP_TYPE_TAG: u64 = 1500;
                let hashed_host = xorname_from_nrs_string(&host_str)?;

                let encoder = XorUrlEncoder::new(
                    hashed_host,
                    RESOLVABLE_MAP_TYPE_TAG,
                    SafeContentType::ResolvableMapContainer,
                );

                let base_xor_url = encoder.to_string("base32z")?;

                let full_new_url = format!("{}{}", base_xor_url, path);
                debug!("Checking NRS system for url: {:?}", &full_new_url);
                Ok(XorUrlEncoder::from_url(&full_new_url)?)
            }
        }?;

        let xorurl_string = the_xor.to_string("base32z")?;
        the_xorurl = &xorurl_string;
        debug!("URL parsed successfully, fetching: {:?}", the_xorurl);
        let path = the_xor.path();

        debug!("Fetching content of type: {:?}", the_xor.content_type());

        // TODO: pass option to get raw content AKA: Do not resolve beyond first thing.
        match the_xor.content_type() {
            SafeContentType::CoinBalance => Ok(SafeData::Key {
                xorname: the_xor.xorname(),
            }),
            SafeContentType::Wallet => Ok(SafeData::Wallet {
                xorname: the_xor.xorname(),
                type_tag: the_xor.type_tag(),
                native_type: "MutableData".to_string(), // TODO: to be retrieved from wallet API
            }),
            SafeContentType::FilesContainer => {
                let (version, files_map, native_type) =
                    self.files_container_get_latest(&the_xorurl)?;

                debug!(
                    "Files container found w/ v:{:?}, of type: {:?}, containing: {:?}",
                    &version, &native_type, &files_map
                );

                if path != "/" && !path.is_empty() {
                    // TODO: Count how many redirects we've done... prevent looping forever
                    let file_item = match files_map.get(path) {
                        Some(item_data) => item_data,
                        None => {
                            return Err(Error::ContentError(format!(
                                "No data found for path \"{}\" on the FilesContainer at \"{}\"",
                                &path, &the_xorurl
                            )))
                        }
                    };

                    let new_target_xorurl = match file_item.get("link") {
						Some( path_data ) => path_data,
						None => return Err(Error::ContentError(format!("FileItem is corrupt. It is missing a \"link\" property at path, \"{}\" on the FilesContainer at: {} ", &path, &the_xorurl))),
					};

                    let path_data = self.fetch(new_target_xorurl);
                    return path_data;
                }

                Ok(SafeData::FilesContainer {
                    xorname: the_xor.xorname(),
                    type_tag: the_xor.type_tag(),
                    version,
                    files_map,
                    native_type,
                })
            }
            SafeContentType::ResolvableMapContainer => {
                let (version, resolvable_map, native_type) =
                    self.resolvable_map_container_get_latest(&the_xorurl)?;
                debug!(
                    "Resolvable map container found w/ v:{:?}, of type: {:?}, containing: {:?}",
                    &version, &native_type, &resolvable_map
                );

                let new_target_xorurl = &resolvable_map.get_default_link()?;

                let url_with_path = format!("{}{}", new_target_xorurl, path);

                debug!("Resolving target from resolvable map: {:?}", url_with_path);

                // TODO: Properly prevent resolution
                // if prevent_resolution {
                // 	return Ok(SafeData::ResolvableMapContainer {
                // 		xorname: the_xor.xorname(),
                // 		type_tag: the_xor.type_tag(),
                // 		version,
                // 		resolvable_map,
                // 		native_type,
                // 	})
                // }

                self.fetch(&url_with_path)
            }
            SafeContentType::ImmutableData => {
                let data = self.files_get_published_immutable(&xorurl)?;
                Ok(SafeData::ImmutableData {
                    xorname: the_xor.xorname(),
                    data,
                })
            }
            SafeContentType::Unknown => Ok(SafeData::Unknown {
                xorname: the_xor.xorname(),
                type_tag: the_xor.type_tag(),
            }),
            other => Err(Error::ContentError(format!(
                "Content type '{:?}' not supported yet by fetch",
                other
            ))),
        }
    }
}

// Unit Tests

#[test]
fn test_fetch_coin_balance() {
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
                type_tag: 10_000,
                native_type: "MutableData".to_string(),
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
                type_tag: 10_100,
                version: 1,
                files_map,
                native_type: "AppendOnlyData".to_string(),
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
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("")).unwrap();

    let prevent_resolution = true;

    let (xorurl, _, files_map) =
        unwrap!(safe.files_container_create("tests/testfolder", None, true, false));

    let (reslovable_map_xorurl, _, resolvable_map) =
        unwrap!(safe.resolvable_map_container_create("somesite", &xorurl, true, false));

    let content = unwrap!(safe.fetch("safe://somesite"));

    // this should actually resolve to a FilesContainer until we enable the option to prevent resolution beyond the mao itself.
    match content {
        SafeData::FilesContainer {
        	..
		    // xorname,
            // type_tag,
            // version,
            // resolvable_map,
            // native_type,
        } => {
            // assert_eq!(xorname, reslovable_map_xorurl);
            // assert_eq!(xorname, xorurl_encoder.xorname());
            // assert_eq!(type_tag, 1500);
            // assert_eq!(version, 1);
            // assert_eq!(native_type, "AppendOnlyData".to_string());
			assert!(true);
        }
        _ => panic!("Resolvable map container was not returned."),
    }
}

#[test]
fn test_fetch_immutable_data() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let data = b"Something super immutable";
    let xorurl = safe.files_put_published_immutable(data).unwrap();

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::ImmutableData {
                xorname: xorurl_encoder.xorname(),
                data: data.to_vec()
            }
    );
}

#[test]
fn test_fetch_unknown() {
    use super::xorurl::create_random_xorname;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let xorname = create_random_xorname();
    let type_tag = 575756443;
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        type_tag,
        SafeContentType::Unknown,
        "base32z"
    ));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(content == SafeData::Unknown { xorname, type_tag });
}

#[test]
fn test_fetch_unsupported() {
    use super::xorurl::create_random_xorname;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let xorname = create_random_xorname();
    let type_tag = 575756443;
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        type_tag,
        SafeContentType::UnpublishedImmutableData,
        "base32z"
    ));
    match safe.fetch(&xorurl) {
        Ok(c) => panic!(format!("Unxpected fetched content: {:?}", c)),
        Err(msg) => assert_eq!(
            msg,
            Error::ContentError(
                "Content type 'UnpublishedImmutableData' not supported yet by fetch".to_string()
            )
        ),
    };
}
