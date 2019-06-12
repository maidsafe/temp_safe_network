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
use safe_app::{run, App, AppError};
// use safe_core::client::{Client/*, CoreError*/, XorNameConverter};
// use routing::{XorName as OldXorName, /*MutableData*/};

use safe_core::ipc::{
    decode_msg, encode_msg, gen_req_id, resp::AuthGranted, IpcMsg, IpcReq, IpcResp,
};
use safe_nd::mutable_data::{Action, MutableData, PermissionSet, SeqMutableData};
use std::str;
use threshold_crypto::serde_impl::SerdeSecret;
use threshold_crypto::{PublicKey, SecretKey, PK_SIZE};

static SAFE_URL_PROTOCOL: &str = "safe://";

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

    pub fn from_hex_keys(pk_hex_str: &str, sk_hex_str: &str) -> Result<Self, String> {
        let pk = pk_from_hex(pk_hex_str)?;
        let sk = sk_from_hex(sk_hex_str)?;
        Ok(KeyPair { pk, sk })
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

pub fn pk_from_hex(hex_str: &str) -> Result<PublicKey, String> {
    let pk_bytes = parse_hex(&hex_str);
    let mut pk_bytes_array: [u8; PK_SIZE] = [0; PK_SIZE];
    pk_bytes_array.copy_from_slice(&pk_bytes[..PK_SIZE]);
    PublicKey::from_bytes(pk_bytes_array).map_err(|_| "Invalid public key string".to_string())
}

pub fn sk_from_hex(hex_str: &str) -> Result<SecretKey, String> {
    let sk_bytes = parse_hex(&hex_str);
    bincode::deserialize(&sk_bytes)
        .map_err(|_| "Failed to deserialize provided secret key".to_string())
}

pub fn parse_coins_amount(amount_str: &str) -> Result<f64, String> {
    // TODO: implement our Error struct which is used across the lib and its API
    let mut itr = amount_str.splitn(2, '.');
    let _ = itr
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| {
            format!(
                "Invalid safecoins amount '{}', expected a numeric value",
                amount_str
            )
        })?;

    let amount: f64 = amount_str.parse::<f64>().map_err(|_| {
        format!(
            "Invalid safecoins amount '{}', expected a numeric value",
            amount_str
        )
    })?;
    Ok(amount)
}

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

pub fn xorurl_to_xorname2(xorurl: &str) -> Result<Vec<u8>, String> {
    let min_len = SAFE_URL_PROTOCOL.len();
    if xorurl.len() < min_len {
        return Err("Invalid XOR-URL".to_string());
    }

    let cid_str = &xorurl[min_len..];
    let cid = Cid::from(cid_str).map_err(|err| format!("Failed to decode XOR-URL: {:?}", err))?;
    let hash = multihash::decode(&cid.hash)
        .map_err(|err| format!("Failed to decode XOR-URL: {:?}", err))?;
    //    let mut xorname = XorName::default();
    //    xorname.0.copy_from_slice(&hash.digest);
    Ok(hash.digest.to_vec())
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

pub fn encode_ipc_msg(req: IpcReq) -> Result<String, AppError> {
    let req_id: u32 = gen_req_id();
    let encoded = encode_msg(&IpcMsg::Req { req_id, req })?;
    Ok(encoded)
}

pub fn decode_ipc_msg(ipc_msg: &str) -> Result<AuthGranted, String> {
    let msg =
        decode_msg(&ipc_msg).map_err(|e| format!("Failed to decode the credentials: {:?}", e))?;
    match msg {
        IpcMsg::Resp {
            resp: IpcResp::Auth(res),
            ..
        } => match res {
            Ok(auth_granted) => Ok(auth_granted),
            Err(err) => Err(format!("{:?}", err)),
        },
        IpcMsg::Revoked { .. } => Err("Authorisation denied".to_string()),
        other => Err(format!("{:?}", other)),
    }
}

#[test]
fn test_xorurl_base32_encoding() {
    use unwrap::unwrap;
    let xorname: XorName = *b"12345678901234567890123456789012";
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32".to_string()));
    let base32_xorurl = "safe://bbkulcamjsgm2dknrxha4tamjsgm2dknrxha4tamjsgm2dknrxha4tamjs";
    assert_eq!(xorurl, base32_xorurl);

    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"".to_string()));
    assert_eq!(xorurl, base32_xorurl);
}

#[test]
fn test_xorurl_base32z_encoding() {
    use unwrap::unwrap;
    let xorname: XorName = *b"12345678901234567890123456789012";
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32z".to_string()));
    let base32_xorurl = "safe://hbkwmnycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1gc4dkptz8yhuycj1";
    assert_eq!(xorurl, base32_xorurl);
}

#[test]
fn test_xorurl_decoding() {
    use unwrap::unwrap;
    let xorname: XorName = *b"12345678901234567890123456789012";
    let xorurl = unwrap!(xorname_to_xorurl(&xorname, &"base32".to_string()));
    let decoded_xorname = unwrap!(xorurl_to_xorname(&xorurl));
    assert_eq!(xorname, decoded_xorname);
}
