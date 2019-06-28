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
use safe_nd::XorName;

const SAFE_URL_PROTOCOL: &str = "safe://";

// The XOR-URL type
pub type XorUrl = String;

#[derive(Debug, Clone)]
pub enum SafeContentType {
    Unknown,
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
    type_tag: Option<u64>,
    content_type: SafeContentType,
}

impl XorUrlEncoder {
    pub fn new(xorname: XorName, type_tag: Option<u64>, content_type: Option<String>) -> Self {
        Self {
            version: 1,
            xorname,
            type_tag,
            content_type: SafeContentType::ImmutableData,
        }
    }

    pub fn from_url(xorurl: &str) -> Result<Self, String> {
        let xorname = xorurl_to_xorname(xorurl)?;
        Ok(Self {
            version: 1,
            xorname: xorname,
            type_tag: None,
            content_type: SafeContentType::ImmutableData,
        })
    }

    pub fn to_string(&self, base: &str) -> Result<String, String> {
        /*let cid = format!(
            "{}.{:?}.{}:{}",
            self.version,
            self.content_type,
            self.xorname,
            self.type_tag.unwrap_or(0)
        );*/
        xorname_to_xorurl(&self.xorname, base)
    }

    pub fn content_type(&self) -> SafeContentType {
        self.content_type.clone()
    }
}

pub fn xorname_to_xorurl(xorname: &XorName, base: &str) -> Result<String, String> {
    let cid = xorname.0;
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
    let cid_str = encode(base_encoding, cid);
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
    xorname.0.copy_from_slice(&decoded_xorurl.1);
    Ok(xorname)
}

#[test]
fn test_xorurl_base32_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32".to_string()));
    let base32_xorurl = "safe://bmjsgm2dknrxha4tamjsgm2dknrxha4tamjsgm2dknrxha4tamjs";
    assert_eq!(xorurl, base32_xorurl);

    let base32z_xorurl = "safe://hcj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1";
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"".to_string()));
    assert_eq!(xorurl, base32z_xorurl);
}

#[test]
fn test_xorurl_base32z_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32z".to_string()));
    let base32z_xorurl = "safe://hcj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1";
    assert_eq!(xorurl, base32z_xorurl);
}

#[test]
fn test_xorurl_decoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32z".to_string()));
    let decoded_xorname = unwrap!(xorurl_to_xorname(&xorurl));
    assert_eq!(xorname, decoded_xorname);
}
