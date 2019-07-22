// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, ResultReturn};
use chrono::{SecondsFormat, Utc};
use safe_core::ipc::{decode_msg, resp::AuthGranted, IpcMsg, IpcResp};
use safe_nd::{XorName, XOR_NAME_LEN};
use std::str;
use threshold_crypto::serde_impl::SerdeSecret;
use threshold_crypto::{PublicKey, SecretKey, PK_SIZE};
use url::Url;

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

    /*
    pub fn from_hex_keys(pk_hex_str: &str, sk_hex_str: &str) -> ResultReturn<Self> {
        let pk = pk_from_hex(pk_hex_str)?;
        let sk = sk_from_hex(sk_hex_str)?;
        Ok(KeyPair { pk, sk })
    }
    */
    pub fn to_hex_key_pair(&self) -> ResultReturn<(String, String)> {
        let pk: String = pk_to_hex(&self.pk);

        let sk_serialised = bincode::serialize(&SerdeSecret(&self.sk))
            .expect("Failed to serialise the generated secret key");
        let sk: String = vec_to_hex(sk_serialised);

        Ok((pk, sk))
    }
}

pub fn xorname_from_pk(pk: &PublicKey) -> XorName {
    let pk_as_bytes: [u8; 48] = pk.to_bytes();
    let mut xorname = XorName::default();
    xorname.0.copy_from_slice(&pk_as_bytes[..XOR_NAME_LEN]);
    xorname
}

pub fn vec_to_hex(hash: Vec<u8>) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

#[allow(dead_code)]
pub fn xorname_to_hex(xorname: &XorName) -> String {
    xorname.0.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn parse_hex(hex_str: &str) -> Vec<u8> {
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

pub fn pk_from_hex(hex_str: &str) -> ResultReturn<PublicKey> {
    let pk_bytes = parse_hex(&hex_str);
    let mut pk_bytes_array: [u8; PK_SIZE] = [0; PK_SIZE];
    pk_bytes_array.copy_from_slice(&pk_bytes[..PK_SIZE]);
    PublicKey::from_bytes(pk_bytes_array)
        .map_err(|_| Error::InvalidInput("Invalid public key bytes".to_string()))
}

pub fn sk_from_hex(hex_str: &str) -> ResultReturn<SecretKey> {
    let sk_bytes = parse_hex(&hex_str);
    bincode::deserialize(&sk_bytes)
        .map_err(|_| Error::InvalidInput("Failed to deserialize provided secret key".to_string()))
}

pub fn parse_coins_amount(amount_str: &str) -> ResultReturn<f64> {
    // TODO: implement our Error struct which is used across the lib and its API
    let mut itr = amount_str.splitn(2, '.');
    let _ = itr
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| {
            Error::InvalidAmount(format!(
                "Invalid safecoins amount '{}', expected a numeric value",
                amount_str
            ))
        })?;

    let amount: f64 = amount_str.parse::<f64>().map_err(|_| {
        Error::InvalidAmount(format!(
            "Invalid safecoins amount '{}', expected a numeric value",
            amount_str
        ))
    })?;
    Ok(amount)
}

pub fn decode_ipc_msg(ipc_msg: &str) -> ResultReturn<AuthGranted> {
    let msg = decode_msg(&ipc_msg)
        .map_err(|e| Error::InvalidInput(format!("Failed to decode the credentials: {:?}", e)))?;
    match msg {
        IpcMsg::Resp {
            resp: IpcResp::Auth(res),
            ..
        } => res.map_err(|err| Error::AuthError(format!("{:?}", err))),
        IpcMsg::Revoked { .. } => Err(Error::AuthError("Authorisation denied".to_string())),
        other => Err(Error::AuthError(format!("{:?}", other))),
    }
}

pub fn get_host_and_path(xorurl: &str) -> ResultReturn<(String, String)> {
    let parsing_url = Url::parse(&xorurl).map_err(|parse_err| {
        Error::InvalidXorUrl(format!("Problem parsing the SAFE:// URL {:?}", parse_err))
    })?;

    let host_str = parsing_url
        .host_str()
        .unwrap_or_else(|| "Failed parsing the URL");

    let mut path = str::replace(parsing_url.path(), "\\", "/");
    if path == "/" {
        path = "".to_string();
    }

    Ok((host_str.to_string(), path))
}

pub fn gen_timestamp_secs() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn gen_timestamp_nanos() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true)
}
