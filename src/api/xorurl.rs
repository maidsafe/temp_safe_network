// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::get_subnames_host_path_and_version;
use super::xorurl_media_types::{MEDIA_TYPE_CODES, MEDIA_TYPE_STR};
use super::{Error, ResultReturn};
use log::debug;
use multibase::{decode, encode, Base};
use safe_nd::{XorName, XOR_NAME_LEN};
use serde::{Deserialize, Serialize};
use std::fmt;

const SAFE_URL_PROTOCOL: &str = "safe://";
const XOR_URL_VERSION_1: u64 = 0x1; // TODO: consider using 16 bits
const XOR_URL_STR_MAX_LENGTH: usize = 44;
const XOR_NAME_BYTES_OFFSET: usize = 4; // offset where to find the XoR name bytes

// The XOR-URL type
pub type XorUrl = String;

// We encode the content type that a XOR-URL is targetting, this allows the consumer/user to
// treat the content in particular ways when the content requires it.
// TODO: support MIME types
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum SafeContentType {
    Raw,
    Wallet,
    FilesContainer,
    NrsMapContainer,
    MediaType(String),
}

impl std::fmt::Display for SafeContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// We also encode the native SAFE data type where the content is being stored on the SAFE Network,
// this allows us to fetch the targetted data using the corresponding API, regardless of the
// data that is being held which is identified by the SafeContentType instead.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum SafeDataType {
    SafeKey = 0x00,
    PublishedImmutableData = 0x01,
    UnpublishedImmutableData = 0x02,
    SeqMutableData = 0x03,
    UnseqMutableData = 0x04,
    PublishedSeqAppendOnlyData = 0x05,
    PublishedUnseqAppendOnlyData = 0x06,
    UnpublishedSeqAppendOnlyData = 0x07,
    UnpublishedUnseqAppendOnlyData = 0x08,
}

impl std::fmt::Display for SafeDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct XorUrlEncoder {
    encoding_version: u64, // currently only v1 supported
    xorname: XorName,
    type_tag: u64,
    data_type: SafeDataType,
    content_type: SafeContentType,
    path: String,
    sub_names: Vec<String>,
    content_version: Option<u64>,
}

impl XorUrlEncoder {
    pub fn new(
        xorname: XorName,
        type_tag: u64,
        data_type: SafeDataType,
        content_type: SafeContentType,
        path: Option<&str>,
        sub_names: Option<Vec<String>>,
        content_version: Option<u64>,
    ) -> ResultReturn<Self> {
        if let SafeContentType::MediaType(ref media_type) = content_type {
            if !XorUrlEncoder::is_media_type_supported(media_type) {
                return Err(Error::InvalidMediaType(format!(
                        "Media-type '{}' not supported. You can use 'SafeContentType::Raw' as the 'content_type' for this type of content",
                        media_type
                    )));
            }
        }

        Ok(Self {
            encoding_version: XOR_URL_VERSION_1,
            xorname,
            type_tag,
            data_type,
            content_type,
            path: path.unwrap_or_else(|| "").to_string(),
            sub_names: sub_names.unwrap_or_else(|| vec![]),
            content_version,
        })
    }

    // A non-member utility function to check if a media-type is currently supported by XOR-URL encoding
    pub fn is_media_type_supported(media_type: &str) -> bool {
        MEDIA_TYPE_CODES.get(media_type).is_some()
    }

    // A non-member encoder function for convinience in some cases
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        xorname: XorName,
        type_tag: u64,
        data_type: SafeDataType,
        content_type: SafeContentType,
        path: Option<&str>,
        sub_names: Option<Vec<String>>,
        content_version: Option<u64>,
        base: &str,
    ) -> ResultReturn<String> {
        let xorurl_encoder = XorUrlEncoder::new(
            xorname,
            type_tag,
            data_type,
            content_type,
            path,
            sub_names,
            content_version,
        )?;
        xorurl_encoder.to_base(base)
    }

    pub fn from_url(xorurl: &str) -> ResultReturn<Self> {
        let (sub_names, cid_str, path, content_version) =
            get_subnames_host_path_and_version(&xorurl)?;

        let (_base, xorurl_bytes): (Base, Vec<u8>) = decode(&cid_str)
            .map_err(|err| Error::InvalidXorUrl(format!("Failed to decode XOR-URL: {:?}", err)))?;

        // let's do a sanity check before analysing byte by byte
        if xorurl_bytes.len() > XOR_URL_STR_MAX_LENGTH {
            return Err(Error::InvalidXorUrl(format!(
                "Invalid XOR-URL, encoded string too long: {} bytes",
                xorurl_bytes.len()
            )));
        }

        // let's make sure we support the XOR_URL version
        let u8_version: u8 = xorurl_bytes[0];
        let encoding_version: u64 = u64::from(u8_version);
        if encoding_version != XOR_URL_VERSION_1 {
            return Err(Error::InvalidXorUrl(format!(
                "Invalid or unsupported XOR-URL encoding version: {}",
                encoding_version
            )));
        }

        let mut content_type_bytes = [0; 2];
        content_type_bytes[0..].copy_from_slice(&xorurl_bytes[1..3]);
        let content_type = match u16::from_be_bytes(content_type_bytes) {
            0 => SafeContentType::Raw,
            1 => SafeContentType::Wallet,
            2 => SafeContentType::FilesContainer,
            3 => SafeContentType::NrsMapContainer,
            other => match MEDIA_TYPE_STR.get(&other) {
                Some(media_type_str) => SafeContentType::MediaType(media_type_str.to_string()),
                None => {
                    return Err(Error::InvalidXorUrl(format!(
                        "Invalid content type encoded in the XOR-URL string: {}",
                        other
                    )))
                }
            },
        };

        debug!(
            "Attempting to match content type of URL: {}, {:?}",
            &xorurl, content_type
        );

        let data_type = match xorurl_bytes[3] {
            0 => SafeDataType::SafeKey,
            1 => SafeDataType::PublishedImmutableData,
            2 => SafeDataType::UnpublishedImmutableData,
            3 => SafeDataType::SeqMutableData,
            4 => SafeDataType::UnseqMutableData,
            5 => SafeDataType::PublishedSeqAppendOnlyData,
            6 => SafeDataType::PublishedUnseqAppendOnlyData,
            7 => SafeDataType::UnpublishedSeqAppendOnlyData,
            8 => SafeDataType::UnpublishedUnseqAppendOnlyData,
            other => {
                return Err(Error::InvalidXorUrl(format!(
                    "Invalid SAFE data type encoded in the XOR-URL string: {}",
                    other
                )))
            }
        };

        let type_tag_offset = XOR_NAME_BYTES_OFFSET + XOR_NAME_LEN; // offset where to find the type tag bytes

        let mut xorname = XorName::default();
        xorname
            .0
            .copy_from_slice(&xorurl_bytes[XOR_NAME_BYTES_OFFSET..type_tag_offset]);

        let type_tag_bytes_len = xorurl_bytes.len() - type_tag_offset;

        let mut type_tag_bytes = [0; 8];
        type_tag_bytes[8 - type_tag_bytes_len..].copy_from_slice(&xorurl_bytes[type_tag_offset..]);
        let type_tag: u64 = u64::from_be_bytes(type_tag_bytes);

        Ok(Self {
            encoding_version,
            xorname,
            type_tag,
            data_type,
            content_type,
            path: path.to_string(),
            sub_names,
            content_version,
        })
    }

    #[allow(dead_code)]
    pub fn encoding_version(&self) -> u64 {
        self.encoding_version
    }

    pub fn data_type(&self) -> SafeDataType {
        self.data_type.clone()
    }

    pub fn content_type(&self) -> SafeContentType {
        self.content_type.clone()
    }

    pub fn xorname(&self) -> XorName {
        self.xorname
    }

    pub fn type_tag(&self) -> u64 {
        self.type_tag
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    #[allow(dead_code)]
    pub fn set_path(&mut self, path: &str) {
        if path.is_empty() || path.starts_with('/') {
            self.path = path.to_string();
        } else {
            self.path = format!("/{}", path);
        }
    }

    pub fn sub_names(&self) -> Vec<String> {
        self.sub_names.to_vec()
    }

    pub fn content_version(&self) -> Option<u64> {
        self.content_version
    }

    pub fn set_content_version(&mut self, version: Option<u64>) {
        self.content_version = version;
    }

    // XOR-URL encoding format (var length from 36 to 44 bytes):
    // 1 byte for encoding version
    // 2 bytes for content type (enough to start including some MIME types also)
    // 1 byte for SAFE native data type
    // 32 bytes for XoR Name
    // and up to 8 bytes for type_tag
    // query param "v=" is treated as the content version
    pub fn to_string(&self) -> ResultReturn<String> {
        self.to_base("")
    }

    pub fn to_base(&self, base: &str) -> ResultReturn<String> {
        // let's set the first byte with the XOR-URL format version
        let mut cid_vec: Vec<u8> = vec![XOR_URL_VERSION_1 as u8];

        // add the content type bytes
        let content_type: u16 = match &self.content_type {
            SafeContentType::Raw => 0x0000,
            SafeContentType::Wallet => 0x0001,
            SafeContentType::FilesContainer => 0x0002,
            SafeContentType::NrsMapContainer => 0x0003,
            SafeContentType::MediaType(media_type) => match MEDIA_TYPE_CODES.get(media_type) {
                Some(media_type_code) => *media_type_code,
                None => {
                    return Err(Error::Unexpected(format!(
                        "Failed to encode Media-type '{}'",
                        media_type
                    )))
                }
            },
        };
        cid_vec.extend_from_slice(&content_type.to_be_bytes());

        // push the SAFE data type byte
        cid_vec.push(self.data_type.clone() as u8);

        let sub_names = if !self.sub_names.is_empty() {
            format!("{}.", self.sub_names.join("."))
        } else {
            "".to_string()
        };

        // add the xorname 32 bytes
        cid_vec.extend_from_slice(&self.xorname.0);

        // let's get non-zero bytes only from th type_tag
        let start_byte: usize = (self.type_tag.leading_zeros() / 8) as usize;
        // add the non-zero bytes of type_tag
        cid_vec.extend_from_slice(&self.type_tag.to_be_bytes()[start_byte..]);

        let base_encoding = match base {
            "base32z" => Base::Base32z,
            "base32" => Base::Base32,
            "base64" => Base::Base64,
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
        let cid_str = encode(base_encoding, cid_vec);
        let xorurl = format!("{}{}{}{}", SAFE_URL_PROTOCOL, sub_names, cid_str, self.path);

        match self.content_version {
            Some(v) => Ok(format!("{}?v={}", xorurl, v)),
            None => Ok(xorurl),
        }
    }
}

impl fmt::Display for XorUrlEncoder {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = self.to_string().map_err(|_| fmt::Error)?;
        write!(fmt, "{}", str)
    }
}

#[test]
fn test_xorurl_base32_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        0xa632_3c4d_4a32,
        SafeDataType::PublishedImmutableData,
        SafeContentType::Raw,
        None,
        None,
        None,
        "base32"
    ));
    let base32_xorurl =
        "safe://biaaaatcmrtgq2tmnzyheydcmrtgq2tmnzyheydcmrtgq2tmnzyheydcmvggi6e2srs";
    assert_eq!(xorurl, base32_xorurl);
}

#[test]
fn test_xorurl_base32z_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        0,
        SafeDataType::PublishedImmutableData,
        SafeContentType::Raw,
        None,
        None,
        None,
        "base32z"
    ));
    let base32z_xorurl = "safe://hbyyyyncj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1";
    assert_eq!(xorurl, base32z_xorurl);
}

#[test]
fn test_xorurl_base64_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        4_584_545,
        SafeDataType::PublishedSeqAppendOnlyData,
        SafeContentType::FilesContainer,
        None,
        None,
        None,
        "base64"
    ));
    let base64_xorurl = "safe://mQACBTEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyRfRh";
    assert_eq!(xorurl, base64_xorurl);
    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&base64_xorurl));
    assert_eq!(base64_xorurl, unwrap!(xorurl_encoder.to_base("base64")));
    assert_eq!("", xorurl_encoder.path());
    assert_eq!(XOR_URL_VERSION_1, xorurl_encoder.encoding_version());
    assert_eq!(xorname, xorurl_encoder.xorname());
    assert_eq!(4_584_545, xorurl_encoder.type_tag());
    assert_eq!(
        SafeDataType::PublishedSeqAppendOnlyData,
        xorurl_encoder.data_type()
    );
    assert_eq!(
        SafeContentType::FilesContainer,
        xorurl_encoder.content_type()
    );
}

#[test]
fn test_xorurl_default_base_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let base32z_xorurl = "safe://hbyyyyncj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1";
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        0,
        SafeDataType::PublishedImmutableData,
        SafeContentType::Raw,
        None,
        None,
        None,
        "" // forces it to use the default
    ));
    assert_eq!(xorurl, base32z_xorurl);
}

#[test]
fn test_xorurl_decoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let type_tag: u64 = 0x0eef;
    let xorurl_encoder = unwrap!(XorUrlEncoder::new(
        xorname,
        type_tag,
        SafeDataType::PublishedImmutableData,
        SafeContentType::Raw,
        None,
        None,
        None,
    ));
    assert_eq!("", xorurl_encoder.path());
    assert_eq!(XOR_URL_VERSION_1, xorurl_encoder.encoding_version());
    assert_eq!(xorname, xorurl_encoder.xorname());
    assert_eq!(type_tag, xorurl_encoder.type_tag());
    assert_eq!(
        SafeDataType::PublishedImmutableData,
        xorurl_encoder.data_type()
    );
    assert_eq!(SafeContentType::Raw, xorurl_encoder.content_type());
}

#[test]
fn test_xorurl_decoding_with_path() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let type_tag: u64 = 0x0eef;
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        type_tag,
        SafeDataType::PublishedSeqAppendOnlyData,
        SafeContentType::Wallet,
        None,
        None,
        None,
        "base32z"
    ));

    let xorurl_with_path = format!("{}/subfolder/file", xorurl);
    let xorurl_encoder_with_path = unwrap!(XorUrlEncoder::from_url(&xorurl_with_path));
    assert_eq!(
        xorurl_with_path,
        unwrap!(xorurl_encoder_with_path.to_base("base32z"))
    );
    assert_eq!("/subfolder/file", xorurl_encoder_with_path.path());
    assert_eq!(
        XOR_URL_VERSION_1,
        xorurl_encoder_with_path.encoding_version()
    );
    assert_eq!(xorname, xorurl_encoder_with_path.xorname());
    assert_eq!(type_tag, xorurl_encoder_with_path.type_tag());
    assert_eq!(
        SafeDataType::PublishedSeqAppendOnlyData,
        xorurl_encoder_with_path.data_type()
    );
    assert_eq!(
        SafeContentType::Wallet,
        xorurl_encoder_with_path.content_type()
    );
}

#[test]
fn test_xorurl_decoding_with_subname() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let type_tag: u64 = 0x0eef;
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        type_tag,
        SafeDataType::PublishedImmutableData,
        SafeContentType::NrsMapContainer,
        None,
        Some(vec!("sub".to_string())),
        None,
        "base32z"
    ));

    let xorurl_with_subname = xorurl.to_string();
    assert!(xorurl_with_subname.contains("safe://sub."));
    let xorurl_encoder_with_subname = unwrap!(XorUrlEncoder::from_url(&xorurl_with_subname));
    assert_eq!(
        xorurl_with_subname,
        unwrap!(xorurl_encoder_with_subname.to_base("base32z"))
    );
    assert_eq!("", xorurl_encoder_with_subname.path());
    assert_eq!(1, xorurl_encoder_with_subname.encoding_version());
    assert_eq!(xorname, xorurl_encoder_with_subname.xorname());
    assert_eq!(type_tag, xorurl_encoder_with_subname.type_tag());
    assert_eq!(vec!("sub"), xorurl_encoder_with_subname.sub_names());
    assert_eq!(
        SafeContentType::NrsMapContainer,
        xorurl_encoder_with_subname.content_type()
    );
}

#[test]
fn test_xorurl_encoding_decoding_with_media_type() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let type_tag: u64 = 0x4c2f;
    let xorurl = unwrap!(XorUrlEncoder::encode(
        xorname,
        type_tag,
        SafeDataType::PublishedImmutableData,
        SafeContentType::MediaType("text/html".to_string()),
        None,
        None,
        None,
        "base32z"
    ));

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&xorurl));
    assert_eq!(
        SafeContentType::MediaType("text/html".to_string()),
        xorurl_encoder.content_type()
    );
}
