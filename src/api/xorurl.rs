// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use cid::{Cid, Codec, Version};
use multibase::{encode, Base};
use multihash;
use safe_nd::XorName;

static SAFE_URL_PROTOCOL: &str = "safe://";

// The XOR-URL type
// TODO: make it a struct with all the helpers below to be methods
pub type XorUrl = String;

pub fn xorname_to_xorurl(xorname: &XorName, base: &str) -> Result<String, String> {
    let h = temp_multihash_encode(multihash::Hash::SHA3256, &xorname.0).unwrap();
    let cid = Cid::new(Codec::Raw, Version::V1, &h);
    let base_encoding = match base {
        "base32z" => Base::Base32z,
        "base32" => Base::Base32,
        base => {
            if !base.is_empty() {
                println!(
                    "Base encoding '{}' not supported for XOR-URL. Using default 'base32'.",
                    base
                );
            }
            Base::Base32
        }
    };
    let cid_str = encode(base_encoding, cid.to_bytes().as_slice());
    Ok(format!("{}{}", SAFE_URL_PROTOCOL, cid_str))
}

pub fn xorurl_to_xorname(xorurl: &str) -> Result<XorName, String> {
    let min_len = SAFE_URL_PROTOCOL.len();
    if xorurl.len() < min_len {
        return Err("Invalid XOR-URL".to_string());
    }

    let cid_str = &xorurl[min_len..];
    let cid = Cid::from(cid_str).map_err(|err| format!("Failed to decode XOR-URL: {:?}", err))?;
    let hash = multihash::decode(&cid.hash)
        .map_err(|err| format!("Failed to decode XOR-URL: {:?}", err))?;
    let mut xorname = XorName::default();
    xorname.0.copy_from_slice(&hash.digest);
    Ok(xorname)
}

// FIXME: temp_multihash_encode is a temporary solution until a PR in multihash project is
// merged and solves the problem of the 'encode' which not only encodes but also hashes the string.
// Issue: https://github.com/multiformats/rust-multihash/issues/32
// PR: https://github.com/multiformats/rust-multihash/pull/26
fn temp_multihash_encode(hash: multihash::Hash, digest: &[u8]) -> Result<Vec<u8>, String> {
    let size = hash.size();
    if digest.len() != size as usize {
        return Err("Bad input size".to_string());
    }
    let mut output = Vec::with_capacity(2 + size as usize);
    output.push(hash.code());
    output.push(size);
    output.extend_from_slice(digest);
    Ok(output)
}

#[test]
fn test_xorurl_base32_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32".to_string()));
    let base32_xorurl = "safe://bbkulcamjsgm2dknrxha4tamjsgm2dknrxha4tamjsgm2dknrxha4tamjs";
    assert_eq!(xorurl, base32_xorurl);

    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"".to_string()));
    assert_eq!(xorurl, base32_xorurl);
}

#[test]
fn test_xorurl_base32z_encoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32z".to_string()));
    let base32_xorurl = "safe://hbkwmnycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1";
    assert_eq!(xorurl, base32_xorurl);
}

#[test]
fn test_xorurl_decoding() {
    use unwrap::unwrap;
    let xorname = XorName(*b"12345678901234567890123456789012");
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32".to_string()));
    let decoded_xorname = unwrap!(xorurl_to_xorname(&xorurl));
    assert_eq!(xorname, decoded_xorname);
}
