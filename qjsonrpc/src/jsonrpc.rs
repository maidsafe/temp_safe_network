// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::Error;
use rand::{self, Rng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

type Result<T> = std::result::Result<T, Error>;

// Version of the JSON-RPC used in the requests
const SN_AUTHD_JSONRPC_VERSION: &str = "2.0";

// JSON-RPC error codes as defined at https://www.jsonrpc.org/specification#response_object
const JSONRPC_PARSE_ERROR: isize = -32700;
const JSONRPC_INVALID_REQUEST: isize = -32600;

/// Spec-defined code for method not found
pub const JSONRPC_METHOD_NOT_FOUND: isize = -32601;

/// Spec-defined code for invalid method params
pub const JSONRPC_INVALID_PARAMS: isize = -32602;

/// Spec-defined catch-all error to use as a fallback
pub const JSONRPC_INTERNAL_ERROR: isize = -32603;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u32,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: SN_AUTHD_JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            params,
            id: rand::thread_rng().gen_range(0, std::u32::MAX) + 1,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    code: isize,
    message: String,
    data: String,
}

impl JsonRpcResponse {
    // Construct a JsonRpcResponse containing a successfull response
    pub fn result(result: serde_json::Value, id: u32) -> Self {
        Self {
            jsonrpc: SN_AUTHD_JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    // Construct a JsonRpcResponse containing an error response
    pub fn error(message: String, code: isize, id: Option<u32>) -> Self {
        Self {
            jsonrpc: SN_AUTHD_JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: "".to_string(),
            }),
            id,
        }
    }
}

// It parses the request bytes an returns a JsonRpcRequest, or a
// serialised JSON-RPC error response ready to send back to the origin
pub(crate) fn parse_jsonrpc_request(req: Vec<u8>) -> std::result::Result<JsonRpcRequest, String> {
    let req_payload = match String::from_utf8(req) {
        Ok(payload) => payload,
        Err(err) => {
            let err_str = serialised_jsonrpc_error(
                "Request payload is a malformed UTF-8 string".to_string(),
                err.to_string(),
                JSONRPC_PARSE_ERROR,
                None,
            )?;
            return Err(err_str);
        }
    };

    let jsonrpc_req: JsonRpcRequest = match serde_json::from_str(&req_payload) {
        Ok(jsonrpc) => jsonrpc,
        Err(err) => {
            let err_str = serialised_jsonrpc_error(
                "Failed to deserialise request payload as a JSON-RPC message".to_string(),
                err.to_string(),
                JSONRPC_INVALID_REQUEST,
                None,
            )?;
            return Err(err_str);
        }
    };

    Ok(jsonrpc_req)
}

// Parse bytes to construct a JsonRpcResponse expected to contain a result of type T
pub(crate) fn parse_jsonrpc_response<T>(response_bytes: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let res_payload = std::str::from_utf8(response_bytes)
        .map_err(|err| Error::ClientError(format!("Failed to decode response data: {}", err)))?;

    match serde_json::from_str(&res_payload) {
        Ok(JsonRpcResponse {
            jsonrpc,
            result: Some(r),
            ..
        }) => {
            if jsonrpc != SN_AUTHD_JSONRPC_VERSION {
                Err(Error::ClientError(format!(
                    "JSON-RPC version {} not supported, only version {} is supported",
                    jsonrpc, SN_AUTHD_JSONRPC_VERSION
                )))
            } else {
                let result = serde_json::from_value(r).map_err(|err| {
                    Error::ClientError(format!("Failed to decode response result: {}", err))
                })?;

                Ok(result)
            }
        }
        Ok(JsonRpcResponse {
            error: Some(err), ..
        }) => Err(Error::RemoteEndpointError(err.message)),
        Ok(JsonRpcResponse {
            result: None,
            error: None,
            ..
        }) => Err(Error::ClientError(
            "Received an invalid JSON-RPC response from authd".to_string(),
        )),
        Err(err) => Err(Error::ClientError(format!(
            "Failed to parse authd response: {}",
            err
        ))),
    }
}

// Generates a serialised JSON-RPC error response
fn serialised_jsonrpc_error(
    message: String,
    data: String,
    code: isize,
    id: Option<u32>,
) -> std::result::Result<String, String> {
    let jsonrpc_err = JsonRpcResponse {
        jsonrpc: SN_AUTHD_JSONRPC_VERSION.to_string(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data,
        }),
        id,
    };
    let serialised_err_res = serde_json::to_string(&jsonrpc_err)
        .map_err(|err| format!("Failed to serialise authd error response: {:?}", err))?;

    Ok(serialised_err_res)
}
