// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use futures::future::Future;

use crate::lib_helpers::{
    decode_ipc_msg, encode_ipc_msg, parse_coins_amount, pk_from_hex, pk_to_hex, sk_from_hex,
    xorname_to_xorurl, xorurl_to_xorname, xorurl_to_xorname2, KeyPair, vec_to_hex
};
use log::{debug, info, warn};
use rand::{OsRng, Rng};
use rand_core::RngCore;


use routing::XorName;
#[cfg(feature = "fake-auth")]

use safe_app::{run, App, AppError};
use safe_core::client::Client;
use safe_core::ipc::{AppExchangeInfo, AuthReq, IpcReq};
use safe_nd::mutable_data::{
    Action, MutableData, PermissionSet, SeqEntryAction, SeqMutableData, Value,
};
use safe_nd::{Error, /*XorName,*/ PublicKey};


use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Read;
use threshold_crypto::SecretKey;
use unwrap::unwrap;
use uuid::Uuid;


fn from_slice(bytes: &[u8]) -> [u8; 32] {
    let mut array = [0; 32];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes);
    array
}

pub fn get_seq_mutable_data(safe_app: &App, xorurl: &str, type_tag: u64) -> SeqMutableData {
    let xorurl_string: String = xorurl.to_string();
    let md = unwrap!(run(safe_app, move |client, _app_context| {
        let xorname = unwrap!(xorurl_to_xorname2(&xorurl_string));
        client
            .get_seq_mdata(XorName(from_slice(&xorname)), type_tag)
            .map_err(|e| panic!("Failed to get MD: {:?}", e))
    }));
    md
}

pub fn get_seq_mutable_data_entries(
    safe_app: &App,
    xorurl: &str,
    type_tag: u64,
) -> BTreeMap<Vec<u8>, Value> {
    let xorurl_string: String = xorurl.to_string();
    let entries = unwrap!(run(safe_app, move |client, _app_context| {
        let xorname = unwrap!(xorurl_to_xorname2(&xorurl_string));
        client
            .list_seq_mdata_entries(XorName(from_slice(&xorname)), type_tag)
            .map_err(|e| panic!("Failed to get MD: {:?}", e))
    }));
    entries
}

pub fn seq_mutable_data_insert(
    safe_app: &App,
    xorurl: &str,
    type_tag: u64,
    key: &[u8],
    value: &[u8]
) -> Result<(), String> {
    let mut entry_actions: BTreeMap<Vec<u8>, SeqEntryAction> = Default::default();
    let _ = entry_actions.insert(
        key.to_vec(),
        SeqEntryAction::Ins(Value::new(value.to_vec(), 0)),
    );

    let xorurl_string: String = xorurl.to_string();
    unwrap!(run(safe_app, move |client, _app_context| {
        let xorname = unwrap!(xorurl_to_xorname2(&xorurl_string));

        client
            .mutate_seq_mdata_entries(
                XorName(from_slice(&xorname)),
                type_tag,
                entry_actions.clone(),
            )
            .map_err(|e| panic!("Failed to insert to MD: {:?}", e))
    }));

    Ok(())
}

pub fn seq_mutable_data_update(
    safe_app: &App,
    xorurl: &str,
    type_tag: u64,
    key: &[u8],
    value: &[u8],
    version: u64,
) -> Result<(), String> {
    let mut entry_actions: BTreeMap<Vec<u8>, SeqEntryAction> = Default::default();
    let _ = entry_actions.insert(
        key.to_vec(),
        SeqEntryAction::Update(Value::new(value.to_vec(), version)),
    );

    let xorurl_string: String = xorurl.to_string();
    unwrap!(run(safe_app, move |client, _app_context| {
        let xorname = unwrap!(xorurl_to_xorname2(&xorurl_string));

        client
            .mutate_seq_mdata_entries(
                XorName(from_slice(&xorname)),
                type_tag,
                entry_actions.clone(),
            )
            .map_err(|e| panic!("Failed to update MD: {:?}", e))
    }));

    Ok(())
}
