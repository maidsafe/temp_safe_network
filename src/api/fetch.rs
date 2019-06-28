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

impl Safe {
    pub fn fetch(&self, xorurl: &str) -> Result<SafeData, String> {
        let xorurl_encoder = XorUrlEncoder::from_url(xorurl)?;
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
    let (xorurl, key_pair) =
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
fn test_fetch_files_container() {
    use std::collections::BTreeMap;
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    let mut content_map = BTreeMap::new();
    content_map.insert(
        "./tests/testfolder/test.md".to_string(),
        "safe://oneurl".to_string(),
    );
    content_map.insert(
        "./tests/testfolder/subfolder/subexists.md".to_string(),
        "safe://otherurl".to_string(),
    );
    let files_map = unwrap!(safe.files_map_create(&content_map));
    let xorurl = unwrap!(safe.files_container_create(files_map.clone().into_bytes().to_vec()));

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
    let mut safe = Safe::new("base32z".to_string());
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
