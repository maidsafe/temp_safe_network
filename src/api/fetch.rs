// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::files::FilesMap;
use super::xorurl::SafeContentType;
use super::{Safe, XorName, XorUrlEncoder};
use log::debug;

#[derive(Debug, PartialEq)]
pub enum SafeData {
    CoinBalance {
        xorname: XorName,
    },
    Wallet {
        xorname: XorName,
        type_tag: u64,
    },
    FilesContainer {
        xorname: XorName,
        type_tag: u64,
        files_map: FilesMap,
    },
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
    /// let (xorurl, _) = unwrap!(safe.files_container_create("tests/testfolder", true, None));
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
    ///
    /// assert!(data_string.starts_with("hello tests!"));
    /// ```
    pub fn fetch(&self, xorurl: &str) -> Result<SafeData, String> {
        debug!("Fetching url: {:?}", xorurl);

        let xorurl_encoder = XorUrlEncoder::from_url(&xorurl)?;
        let path = xorurl_encoder.path();

        match xorurl_encoder.content_type() {
            SafeContentType::CoinBalance => Ok(SafeData::CoinBalance {
                xorname: xorurl_encoder.xorname(),
            }),
            SafeContentType::Wallet => Ok(SafeData::Wallet {
                xorname: xorurl_encoder.xorname(),
                type_tag: xorurl_encoder.type_tag(),
            }),
            SafeContentType::FilesContainer => {
                let files_map = self.files_container_get_latest(&xorurl)?;

                debug!("FilesMap found: {:?}", files_map);

                if path != "/" && !path.is_empty() {
                    // TODO: Count how many redirects we've done... prevent looping forever
                    let file_item = match files_map.get(path) {
                        Some(item_data) => item_data,
                        None => {
                            return Err(format!(
                                "No data found for, \"{}\" on the FilesContainer at: {}",
                                &path, &xorurl
                            ))
                        }
                    };

                    let new_target_xorurl = match file_item.get("link") {
						Some( path_data ) => path_data,
						None => return Err(format!("FileItem is corrupt. It is missing a \"link\" property at path, \"{}\" on the FilesContainer at: {} ", &path, &xorurl) ),
					};

                    let path_data = self.fetch(new_target_xorurl);
                    return path_data;
                }

                Ok(SafeData::FilesContainer {
                    xorname: xorurl_encoder.xorname(),
                    type_tag: xorurl_encoder.type_tag(),
                    files_map,
                })
            }
            SafeContentType::ImmutableData => {
                let data = self.files_get_published_immutable(&xorurl)?;
                Ok(SafeData::ImmutableData {
                    xorname: xorurl_encoder.xorname(),
                    data,
                })
            }
            SafeContentType::Unknown => Ok(SafeData::Unknown {
                xorname: xorurl_encoder.xorname(),
                type_tag: xorurl_encoder.type_tag(),
            }),
            other => Err(format!(
                "Content type '{:?}' not supported yet by fetch",
                other
            )),
        }
    }
}

// Unit Tests

#[test]
fn test_fetch_coin_balance() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    let preload_amount = "1324.12";
    let (xorurl, _key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::CoinBalance {
                xorname: xorurl_encoder.xorname()
            }
    );
}

#[test]
fn test_fetch_wallet() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    let xorurl = unwrap!(safe.wallet_create());

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));
    assert!(
        content
            == SafeData::Wallet {
                xorname: xorurl_encoder.xorname(),
                type_tag: 10_000
            }
    );
}

#[test]
#[ignore]
fn test_fetch_files_container() {
    /*    use std::collections::BTreeMap;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());

    let (xorurl, _) = unwrap!(safe.files_container_create("tests/testfolder", true, None));

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    let content = unwrap!(safe.fetch(&xorurl));

    assert!(
        content
            == SafeData::FilesContainer {
                xorname: xorurl_encoder.xorname(),
                type_tag: 10_100,
                files_map: unwrap!(serde_json::from_str(&files_map))
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
    );*/
}

#[test]
fn test_fetch_immutable_data() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
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
    let safe = Safe::new("base32z".to_string());
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
    let safe = Safe::new("base32z".to_string());
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
            "Content type 'UnpublishedImmutableData' not supported yet by fetch"
        ),
    };
}
