// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::common::{parse_hex, sk_from_hex};
use crate::{Error, Result};
use chrono::{SecondsFormat, Utc};
use log::debug;
use safe_core::ipc::{decode_msg, resp::AuthGranted, BootstrapConfig, IpcMsg, IpcResp};
use safe_nd::{Coins, Error as SafeNdError, PublicKey as SafeNdPublicKey, XorName};
use std::{
    iter::FromIterator,
    str::{self, FromStr},
};
use threshold_crypto::{serde_impl::SerdeSecret, PublicKey, SecretKey, PK_SIZE};
use url::Url;

const URL_VERSION_QUERY_NAME: &str = "v=";

/// The conversion from coin to raw value
const COIN_TO_RAW_CONVERSION: u64 = 1_000_000_000;
// The maximum amount of safecoin that can be represented by a single `Coins`
const MAX_COINS_VALUE: u64 = (u32::max_value() as u64 + 1) * COIN_TO_RAW_CONVERSION - 1;

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

pub fn xorname_to_hex(xorname: &XorName) -> String {
    xorname.0.iter().map(|b| format!("{:02x}", b)).collect()
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

pub fn parse_coins_amount(amount_str: &str) -> Result<Coins> {
    Coins::from_str(amount_str).map_err(|err| {
        match err {
            SafeNdError::ExcessiveValue => Error::InvalidAmount(format!(
                "Invalid safecoins amount '{}', it exceeds the maximum possible value '{}'",
                amount_str, Coins::from_nano(MAX_COINS_VALUE)
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

#[allow(clippy::large_enum_variant)]
pub enum AuthResponseType {
    Registered(AuthGranted),
    Unregistered(BootstrapConfig),
}

pub fn decode_ipc_msg(ipc_msg: &str) -> Result<AuthResponseType> {
    let msg = decode_msg(&ipc_msg)
        .map_err(|e| Error::InvalidInput(format!("Failed to decode the credentials: {:?}", e)))?;
    match msg {
        IpcMsg::Resp { response, .. } => match response {
            IpcResp::Auth(res) => match res {
                Ok(authgranted) => Ok(AuthResponseType::Registered(authgranted)),
                Err(e) => Err(Error::AuthError(format!("{:?}", e))),
            },
            IpcResp::Unregistered(res) => match res {
                Ok(config) => Ok(AuthResponseType::Unregistered(config)),
                Err(e) => Err(Error::AuthError(format!("{:?}", e))),
            },
            _ => Err(Error::AuthError(
                "Doesn't support other request.".to_string(),
            )),
        },
        IpcMsg::Revoked { .. } => Err(Error::AuthError("Authorisation denied".to_string())),
        other => Err(Error::AuthError(format!("{:?}", other))),
    }
}

#[allow(clippy::type_complexity)]
pub fn extract_all_url_parts(
    xorurl: &str,
) -> Result<(Vec<String>, String, String, Option<u64>, Vec<String>)> {
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

    let mut version = None;
    let mut query_params = vec![];
    if let Some(query) = parsing_url.query() {
        for p in query.split('&').collect::<Vec<&str>>().iter() {
            if p.starts_with(URL_VERSION_QUERY_NAME) {
                let version_str = p.replace(URL_VERSION_QUERY_NAME, "");
                version = str::parse::<u64>(&version_str).ok()
            } else {
                // Let's accumulate the query params which are not the version param
                query_params.push((*p).to_string());
            }
        }
    };

    debug!(
        "Data from url: sub names: {:?}, host: {}, path: {}, version: {:?}, query_params: {:?}",
        sub_names.to_vec(),
        top_level_name.to_string(),
        path,
        version,
        query_params,
    );
    Ok((
        sub_names.to_vec(),
        top_level_name.to_string(),
        path.to_string(),
        version,
        query_params,
    ))
}

pub fn gen_timestamp_secs() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn gen_timestamp_nanos() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true)
}
