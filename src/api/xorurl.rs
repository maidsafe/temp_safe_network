// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::xorname_to_hex;
use multibase::{decode, encode, Base};
use rand::rngs::OsRng;
use rand_core::RngCore;
use safe_nd::{XorName, XOR_NAME_LEN};

const SAFE_URL_PROTOCOL: &str = "safe://";

// The XOR-URL type
pub type XorUrl = String;

#[derive(Debug, Clone)]
pub enum SafeContentType {
    Unknown,
    CoinBalance,
    Wallet,
    SeqMutableData,
    UnseqMutableData,
    FilesContainer,
    // UnpublishedFilesContainer,
    ImmutableData,
    // UnpublishedImmutableData,
}

pub fn create_random_xorname() -> XorName {
    let mut os_rng = OsRng::new().unwrap();
    let mut xorname = XorName::default();
    os_rng.fill_bytes(&mut xorname.0);
    xorname
}

pub struct XorUrlEncoder {
    version: u32, // currently only v1 supported
    xorname: XorName,
    type_tag: u64,
    content_type: SafeContentType,
}

impl XorUrlEncoder {
    pub fn new(xorname: XorName, type_tag: u64, content_type: Option<String>) -> Self {
        Self {
            version: 1,
            xorname,
            type_tag,
            content_type: SafeContentType::ImmutableData,
        }
    }

    pub fn from_url(xorurl: &str) -> Result<Self, String> {
        let min_len = SAFE_URL_PROTOCOL.len();
        if xorurl.len() < min_len {
            return Err("Invalid XOR-URL".to_string());
        }

        let cid_str = &xorurl[min_len..];
        let decoded_xorurl =
            decode(&cid_str).map_err(|err| format!("Failed to decode XOR-URL: {:?}", err))?;

        let mut content_type_bytes = [0; 2];
        content_type_bytes.copy_from_slice(&decoded_xorurl.1[1..3]);
        let content_type = match content_type_bytes {
            [0, 1] => SafeContentType::ImmutableData,
            [0, 2] => SafeContentType::FilesContainer,
            other => SafeContentType::Unknown,
        };

        let mut xorname = XorName::default();
        xorname
            .0
            .copy_from_slice(&decoded_xorurl.1[3..XOR_NAME_LEN + 3]);

        let mut tag_type_bytes = [0; 8];
        tag_type_bytes.copy_from_slice(&decoded_xorurl.1[XOR_NAME_LEN + 3..]);
        let type_tag: u64 = u64::from_be_bytes(tag_type_bytes);

        Ok(Self {
            version: 1,
            xorname: xorname,
            type_tag,
            content_type,
        })
    }

    pub fn to_string(&self, base: &str) -> Result<String, String> {
        xorname_to_xorurl(
            &self.xorname,
            self.type_tag,
            self.content_type.clone(),
            base,
        )
    }

    pub fn content_type(&self) -> SafeContentType {
        self.content_type.clone()
    }
}

pub fn xorname_to_xorurl(
    xorname: &XorName,
    type_tag: u64,
    content_type: SafeContentType,
    base: &str,
) -> Result<String, String> {
    // 1 /*version*/ + 2 /*content_type*/ + 32 /*xorname*/ + 8 /*type_tag*/ == 43;
    let mut cid: [u8; 43] = [0; 43];
    cid[0] = 0x1; // version = 1
    match content_type {
        SafeContentType::ImmutableData => cid[1..3].copy_from_slice(&[0, 1]),
        SafeContentType::FilesContainer => cid[1..3].copy_from_slice(&[0, 2]),
        other => cid[1..3].copy_from_slice(&[0, 0]),
    };

    cid[3..XOR_NAME_LEN + 3].copy_from_slice(&xorname.0);
    let base_encoding = match base {
        "base32z" => Base::Base32z,
        "base32" => Base::Base32,
        base => {
            if !base.is_empty() {
                println!(
                    "Base encoding '{}' not supported for XOR-URL. Using default 'base32z'.",
                    base
                );
            }
            Base::Base32z
        }
    };
    cid[XOR_NAME_LEN + 3..].copy_from_slice(&type_tag.to_be_bytes());
    let cid_str = encode(base_encoding, cid.to_vec());
    Ok(format!("{}{}", SAFE_URL_PROTOCOL, cid_str))
}

pub fn xorurl_to_xorname(xorurl: &str) -> Result<XorName, String> {
    let min_len = SAFE_URL_PROTOCOL.len();
    if xorurl.len() < min_len {
        return Err("Invalid XOR-URL".to_string());
    }

    let cid_str = &xorurl[min_len..];
    let decoded_xorurl =
        decode(&cid_str).map_err(|err| format!("Failed to decode XOR-URL: {:?}", err))?;
    let mut xorname = XorName::default();
    xorname
        .0
        .copy_from_slice(&decoded_xorurl.1[3..XOR_NAME_LEN + 3]);

    let mut tag_type_bytes = [0; 8];
    tag_type_bytes.copy_from_slice(&decoded_xorurl.1[XOR_NAME_LEN + 3..]);
    let type_tag: u64 = u64::from_be_bytes(tag_type_bytes);

    Ok(xorname)
}

#[test]
fn test_xorurl_base32_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(
        &xorname,
        0,
        SafeContentType::ImmutableData,
        &"base32".to_string()
    ));
    let base32_xorurl =
        "safe://bcaabgezdgnbvgy3tqojqgezdgnbvgy3tqojqgezdgnbvgy3tqojqgezaaaaaaaaaaaaa";
    assert_eq!(xorurl, base32_xorurl);

    let base32z_xorurl =
        "safe://hnyybgr3dgpbiga5uoqjogr3dgpbiga5uoqjogr3dgpbiga5uoqjogr3yyyyyyyyyyyyy";
    let xorurl = unwrap!(xorname_to_xorurl(
        &xorname,
        0,
        SafeContentType::ImmutableData,
        &"".to_string()
    ));
    assert_eq!(xorurl, base32z_xorurl);
}

#[test]
fn test_xorurl_base32z_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(
        &xorname,
        0,
        SafeContentType::ImmutableData,
        &"base32z".to_string()
    ));
    let base32z_xorurl =
        "safe://hnyybgr3dgpbiga5uoqjogr3dgpbiga5uoqjogr3dgpbiga5uoqjogr3yyyyyyyyyyyyy";
    assert_eq!(xorurl, base32z_xorurl);
}

#[test]
fn test_xorurl_decoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(
        &xorname,
        0,
        SafeContentType::ImmutableData,
        &"base32z".to_string()
    ));
    let decoded_xorname = unwrap!(xorurl_to_xorname(&xorurl));
    assert_eq!(xorname, decoded_xorname);
}
