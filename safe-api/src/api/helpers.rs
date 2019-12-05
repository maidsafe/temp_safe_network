// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result};
use chrono::{SecondsFormat, Utc};

use super::constants::SAFE_AUTHD_CONNECTION_IDLE_TIMEOUT;
use jsonrpc_quic::jsonrpc_send;
use log::debug;
use rand::rngs::OsRng;
use rand_core::RngCore;
use safe_core::ipc::{decode_msg, resp::AuthGranted, BootstrapConfig, IpcMsg, IpcResp};
use safe_nd::{
    Coins, Error as SafeNdError, PublicKey as SafeNdPublicKey, XorName, MAX_COINS_VALUE,
};
use serde::de::DeserializeOwned;
use std::iter::FromIterator;
use std::str;
use std::str::FromStr;
use threshold_crypto::{serde_impl::SerdeSecret, PublicKey, SecretKey, PK_SIZE};
use url::Url;

const URL_VERSION_QUERY_NAME: &str = "v=";

// Out internal key pair structure to manage BLS keys
#[derive(Debug)]
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

    #[allow(dead_code)]
    pub fn from_hex_keys(pk_hex_str: &str, sk_hex_str: &str) -> Result<Self> {
        let pk = pk_from_hex(pk_hex_str)?;
        let sk = sk_from_hex(sk_hex_str)?;
        if pk != sk.public_key() {
            Err(Error::InvalidInput(
                "Secret key doesn't correspond to public key provided".to_string(),
            ))
        } else {
            Ok(KeyPair { pk, sk })
        }
    }

    pub fn from_hex_sk(sk_hex_str: &str) -> Result<Self> {
        let sk = sk_from_hex(sk_hex_str)?;
        let pk = sk.public_key();
        Ok(KeyPair { pk, sk })
    }

    pub fn to_hex_key_pair(&self) -> Result<(String, String)> {
        let pk: String = pk_to_hex(&self.pk);

        let sk_serialised = bincode::serialize(&SerdeSecret(&self.sk))
            .expect("Failed to serialise the generated secret key");
        let sk: String = vec_to_hex(sk_serialised);

        Ok((pk, sk))
    }
}

pub fn xorname_from_pk(pk: PublicKey) -> XorName {
    XorName::from(SafeNdPublicKey::from(pk))
}

pub fn vec_to_hex(hash: Vec<u8>) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn create_random_xorname() -> Result<XorName> {
    let mut os_rng = OsRng::new().map_err(|err| {
        Error::Unexpected(format!("Failed to generate a random xorname: {}", err))
    })?;
    let mut xorname = XorName::default();
    os_rng.fill_bytes(&mut xorname.0);
    Ok(xorname)
}

pub fn unwrap_or_gen_random(name: Option<XorName>) -> Result<XorName> {
    match name {
        Some(xorname) => Ok(xorname),
        None => create_random_xorname(),
    }
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
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
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

pub fn pk_from_hex(hex_str: &str) -> Result<PublicKey> {
    let pk_bytes = parse_hex(&hex_str);
    let mut pk_bytes_array: [u8; PK_SIZE] = [0; PK_SIZE];
    pk_bytes_array.copy_from_slice(&pk_bytes[..PK_SIZE]);
    PublicKey::from_bytes(pk_bytes_array)
        .map_err(|_| Error::InvalidInput("Invalid public key bytes".to_string()))
}

pub fn sk_from_hex(hex_str: &str) -> Result<SecretKey> {
    let sk_bytes = parse_hex(&hex_str);
    bincode::deserialize(&sk_bytes)
        .map_err(|_| Error::InvalidInput("Failed to deserialize provided secret key".to_string()))
}

pub fn parse_coins_amount(amount_str: &str) -> Result<Coins> {
    Coins::from_str(amount_str).map_err(|err| {
        match err {
            SafeNdError::ExcessiveValue => Error::InvalidAmount(format!(
                "Invalid safecoins amount '{}', it exceeds the maximum possible value '{}'",
                amount_str, MAX_COINS_VALUE
            )),
            SafeNdError::LossOfPrecision => {
                Error::InvalidAmount(format!("Invalid safecoins amount '{}', the minimum possible amount is one nano coin (0.000000001)", amount_str))
            }
            SafeNdError::FailedToParse(msg) => {
                Error::InvalidAmount(format!("Invalid safecoins amount '{}' ({})", amount_str, msg))
            },
            _ => Error::InvalidAmount(format!("Invalid safecoins amount '{}'", amount_str)),
        }
    })
}

pub enum AuthResponseType {
    Registered(AuthGranted),
    Unregistered(BootstrapConfig),
}

pub fn decode_ipc_msg(ipc_msg: &str) -> Result<AuthResponseType> {
    let msg = decode_msg(&ipc_msg)
        .map_err(|e| Error::InvalidInput(format!("Failed to decode the credentials: {:?}", e)))?;
    match msg {
        IpcMsg::Resp { resp, .. } => {
            match resp {
                IpcResp::Auth(res) => res
                    .map(|authgranted| AuthResponseType::Registered(authgranted))
                    .map_err(|err| Error::AuthError(format!("{:?}", err))),
                IpcResp::Unregistered(res) => res
                    .map(|config| AuthResponseType::Unregistered(config))
                    .map_err(|err| Error::AuthError(format!("{:?}", err))),
                _ => Err(Error::AuthError(
                    "Doesn't support other request.".to_string(),
                )),
                // IpcResp::Auth(res) => match res {
                //     Ok(authgranted) => Ok(AuthResponseType::Registered(authgranted)),
                //     Err(e) => Err(Error::AuthError(format!("{:?}", e)))
                // },
                // IpcResp::Unregistered(res) => match res {
                //     Ok(config) => Ok(AuthResponseType::Unregistered(config)),
                //     Err(e) => Err(Error::AuthError(format!("{:?}", e)))
                // },
                // _ => Err(Error::AuthError("Doesn't support other request.".to_string()))
            }
        }
        IpcMsg::Revoked { .. } => Err(Error::AuthError("Authorisation denied".to_string())),
        other => Err(Error::AuthError(format!("{:?}", other))),
    }
}

pub fn get_subnames_host_path_and_version(
    xorurl: &str,
) -> Result<(Vec<String>, String, String, Option<u64>)> {
    let parsing_url = Url::parse(&xorurl).map_err(|parse_err| {
        Error::InvalidXorUrl(format!(
            "Problem parsing the safe:// URL \"{}\": {}",
            xorurl, parse_err
        ))
    })?;

    let host_str = parsing_url
        .host_str()
        .unwrap_or_else(|| "Failed parsing the URL");
    let names_vec = Vec::from_iter(host_str.split('.').map(String::from));
    let top_level_name = &names_vec[names_vec.len() - 1];
    let sub_names = &names_vec[0..names_vec.len() - 1];

    let mut path = parsing_url.path();
    if path == "/" {
        path = "";
    }

    let version = match parsing_url.query() {
        Some(query) => {
            let items: Vec<&str> = query.split('&').collect();
            match items.iter().find(|q| q.starts_with(URL_VERSION_QUERY_NAME)) {
                Some(version_item) => {
                    let version_str = version_item.replace(URL_VERSION_QUERY_NAME, "");
                    str::parse::<u64>(&version_str).ok()
                }
                None => None,
            }
        }
        None => None,
    };

    debug!(
        "Data from url: sub names: {:?}, host: {}, path: {}, version: {:?}",
        sub_names.to_vec(),
        top_level_name.to_string(),
        path,
        version
    );
    Ok((
        sub_names.to_vec(),
        top_level_name.to_string(),
        path.to_string(),
        version,
    ))
}

pub fn gen_timestamp_secs() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn gen_timestamp_nanos() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true)
}

pub fn send_authd_request<T>(endpoint: &str, method: &str, params: serde_json::Value) -> Result<T>
where
    T: DeserializeOwned,
{
    match directories::ProjectDirs::from("net", "maidsafe", "safe-authd") {
        None => Err(Error::AuthdClientError(
            "Failed to obtain local project directory where to read certificate from".to_string(),
        )),
        Some(dirs) => {
            let cert_base_path = dirs.config_dir().display().to_string();
            jsonrpc_send::<T>(
                endpoint,
                method,
                params,
                Some(&cert_base_path),
                Some(SAFE_AUTHD_CONNECTION_IDLE_TIMEOUT),
            )
            .map_err(|err| match err {
                jsonrpc_quic::Error::ClientError(msg) => Error::AuthdClientError(msg),
                jsonrpc_quic::Error::ServerError(msg) => Error::AuthdError(msg),
            })
        }
    }
}
