// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::scl_mock::XorName;
use cid::{Cid, Codec, Version};
use multibase::{encode, Base};
use multihash;
use std::str;
use threshold_crypto::serde_impl::SerdeSecret;
use threshold_crypto::{PublicKey, SecretKey, PK_SIZE};
use unwrap::unwrap;

// Out internal key pair structure to manage BLS keys
pub struct KeyPair {
    pub pk: PublicKey,
    pub sk: SecretKey,
}

impl KeyPair {
    pub fn random() -> Self {
        let sk = SecretKey::random();
        let pk = sk.public_key();
        KeyPair { pk, sk }
    }

    pub fn from_hex_keys(pk_hex_str: &str, sk_hex_str: &str) -> Self {
        let pk = pk_from_hex(pk_hex_str);
        let sk = sk_from_hex(sk_hex_str);
        KeyPair { pk, sk }
    }

    pub fn to_hex_key_pair(&self) -> (String, String) {
        let pk: String = pk_to_hex(&self.pk);

        let sk_serialised = bincode::serialize(&SerdeSecret(&self.sk))
            .expect("Failed to serialise the generated secret key");
        let sk: String = vec_to_hex(sk_serialised);

        (pk, sk)
    }
}

pub fn vec_to_hex(hash: Vec<u8>) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

fn parse_hex(hex_str: &str) -> Vec<u8> {
    let mut hex_bytes = hex_str
        .as_bytes()
        .iter()
        .filter_map(|b| match b {
            b'0'...b'9' => Some(b - b'0'),
            b'a'...b'f' => Some(b - b'a' + 10),
            b'A'...b'F' => Some(b - b'A' + 10),
            _ => None,
        })
        .fuse();

    let mut bytes = Vec::new();
    while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
        bytes.push(h << 4 | l)
    }
    bytes
}

pub fn pk_to_hex(pk: &PublicKey) -> String {
    let pk_as_bytes: [u8; PK_SIZE] = pk.to_bytes();
    vec_to_hex(pk_as_bytes.to_vec())
}

pub fn pk_from_hex(hex_str: &str) -> PublicKey {
    let pk_bytes = parse_hex(&hex_str);
    let mut pk_bytes_array: [u8; PK_SIZE] = [0; PK_SIZE];
    pk_bytes_array.copy_from_slice(&pk_bytes[..PK_SIZE]);
    unwrap!(PublicKey::from_bytes(pk_bytes_array))
}

pub fn sk_from_hex(hex_str: &str) -> SecretKey {
    let sk_bytes = parse_hex(&hex_str);
    bincode::deserialize(&sk_bytes).expect("Failed to deserialize provided secret key")
}

pub fn xorname_to_xorurl(xorname: &XorName, base: &String) -> String {
    // FIXME: temp_multihash_encode is a temporary solution until a PR in multihash project is
    // merged and solves the problem of the encode to not only encoding but also hashing the string.
    // Issue: https://github.com/multiformats/rust-multihash/issues/32
    // PR: https://github.com/multiformats/rust-multihash/pull/26
    let h = temp_multihash_encode(multihash::Hash::SHA3256, xorname).unwrap();
    let cid = Cid::new(Codec::Raw, Version::V1, &h);
    let base_encoding = match base.as_str() {
        "base32z" => Base::Base32z,
        "base32" => Base::Base32,
        a => {
            if a.len() > 0 {
                println!(
                    "Base encoding '{}' not supported for XOR-URL. Using default 'base32'.",
                    a
                );
            }
            Base::Base32
        }
    };
    let cid_str = encode(base_encoding, cid.to_bytes().as_slice());
    format!("safe://{}", cid_str)
}

pub fn xorurl_to_xorname(xorurl: &String) -> XorName {
    let cid_str = &xorurl[("safe://".len())..];
    let cid = unwrap!(Cid::from(cid_str));
    let hash = multihash::decode(&cid.hash).unwrap();
    let mut xorname = XorName::default();
    xorname.copy_from_slice(&hash.digest);
    xorname
}

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
fn test_xor_url_base32_encoding() {
    let xorname: XorName = *b"12345678901234567890123456789012";
    let xor_url = xorname_to_xorurl(&xorname, &"base32".to_string());
    let base32_xorurl = "safe://bbkulcamjsgm2dknrxha4tamjsgm2dknrxha4tamjsgm2dknrxha4tamjs";
    assert_eq!(xor_url, base32_xorurl);

    let xor_url = xorname_to_xorurl(&xorname, &"".to_string());
    assert_eq!(xor_url, base32_xorurl);
}

#[test]
fn test_xor_url_base32z_encoding() {
    let xorname: XorName = *b"12345678901234567890123456789012";
    let xor_url = xorname_to_xorurl(&xorname, &"base32z".to_string());
    let base32_xorurl = "safe://hbkwmnycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1";
    assert_eq!(xor_url, base32_xorurl);
}

#[test]
fn test_xor_url_decoding() {
    let xorname: XorName = *b"12345678901234567890123456789012";
    let xor_url = xorname_to_xorurl(&xorname, &"base32".to_string());
    let decoded_xorname = xorurl_to_xorname(&xor_url);
    assert_eq!(xorname, decoded_xorname);
}
