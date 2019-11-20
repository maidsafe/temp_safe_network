// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::quic_client::quic_send;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;

// Version of the JSON-RPC used in the requests
const SAFE_AUTHD_JSONRPC_VERSION: &str = "2.0";

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcReq<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    params: serde_json::Value,
    id: usize,
}

#[derive(Deserialize, Debug)]
struct JsonRpcRes<'a> {
    jsonrpc: &'a str,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError<'a>>,
    id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError<'a> {
    code: isize,
    message: &'a str,
    data: &'a str,
}

pub fn send_request<T>(
    endpoint: &str,
    method: &str,
    params: serde_json::Value,
    cert_base_path: Option<&str>,
    timeout: Option<u64>,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let jsonrpc_req = JsonRpcReq {
        jsonrpc: SAFE_AUTHD_JSONRPC_VERSION,
        method,
        params,
        id: 1,
    };
    let serialised_req = serde_json::to_string(&jsonrpc_req)
        .map_err(|err| format!("Failed to serialise authd request: {}", err))?;

    // Send request over QUIC, and await for JSON-RPC response
    let received_bytes = quic_send(
        &endpoint,
        &serialised_req,
        false,
        None,
        cert_base_path,
        false,
        timeout,
    )?;

    let res_payload = std::str::from_utf8(received_bytes.as_slice())
        .map_err(|err| format!("Failed to decode response data: {}", err))?;

    match serde_json::from_str(&res_payload) {
        Ok(JsonRpcRes {
            jsonrpc,
            result: Some(r),
            ..
        }) => {
            if jsonrpc != SAFE_AUTHD_JSONRPC_VERSION {
                Err(format!(
                    "JSON-RPC version {} not supported, only version {} is supported",
                    jsonrpc, SAFE_AUTHD_JSONRPC_VERSION
                ))
            } else {
                let result = serde_json::from_value(r)
                    .map_err(|err| format!("Failed to decode response result: {}", err))?;

                Ok(result)
            }
        }
        Ok(JsonRpcRes {
            error: Some(err), ..
        }) => Err(err.message.to_string()),
        Ok(JsonRpcRes {
            result: None,
            error: None,
            ..
        }) => Err("Received an invalid JSON-RPC response from authd".to_string()),
        Err(err) => Err(format!("Failed to parse authd response: {}", err)),
    }
}
