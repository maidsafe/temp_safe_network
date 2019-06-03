// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::scl_mock::XorHash;
use cid::{Cid, Codec, Version};
use multibase::{encode, Base};
use multihash;
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
        let sk: String = hash_to_hex(sk_serialised);

        (pk, sk)
    }
}

pub fn hash_to_hex(hash: Vec<u8>) -> String {
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

#[allow(dead_code)]
pub fn name_and_tag_xorurl(xorhash: &XorHash, type_tag: u64) -> String {
    let h = multihash::encode(multihash::Hash::SHA3256, xorhash).unwrap();
    let cid = Cid::new(Codec::Raw, Version::V1, &h);
    let cid_str = encode(Base::Base32z, cid.to_bytes().as_slice());
    format!("safe://{}:{}", cid_str, type_tag)
}

pub fn pk_to_hex(pk: &PublicKey) -> String {
    let pk_as_bytes: [u8; PK_SIZE] = pk.to_bytes();
    hash_to_hex(pk_as_bytes.to_vec())
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
