// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod allow;
mod auth_reqs;
mod authed_apps;
mod authorise;
mod create_acc;
mod deny;
mod log_in;
mod log_out;
mod revoke;
mod status;
mod subscribe;
mod unsubscribe;

use crate::errors::Error;
use crate::shared::{
    lock_safe_authenticator, remove_auth_req_from_list, SharedAuthReqsHandle,
    SharedNotifEndpointsHandle, SharedSafeAuthenticatorHandle,
};
use futures::{Async, Future, Poll, Stream};
use serde::{Deserialize, Serialize};
use std::str;
use tokio::sync::mpsc;

type FutureItemType = Box<[u8]>;
type FutureErrorType = Error;

// Version of the JSON-RPC used
const SAFE_AUTHD_JSONRPC_VERSION: &str = "2.0";

// JSON-RPC error codes as defined at https://www.jsonrpc.org/specification#response_object
const JSONRPC_PARSE_ERROR: isize = -32700;
const JSONRPC_INVALID_REQUEST: isize = -32600;
const JSONRPC_AUTH_ERROR: isize = -1;

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcReq<'a> {
    pub jsonrpc: &'a str,
    method: &'a str,
    params: serde_json::Value,
    id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse<'a> {
    jsonrpc: &'a str,
    result: serde_json::Value,
    id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponseError<'a> {
    jsonrpc: &'a str,
    error: JsonRpcError<'a>,
    id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError<'a> {
    code: isize,
    message: &'a str,
    data: &'a str,
}

fn err_response(message: &str, data: &str, code: isize) -> Result<FutureItemType, FutureErrorType> {
    let jsonrpc_err = JsonRpcResponseError {
        jsonrpc: SAFE_AUTHD_JSONRPC_VERSION,
        error: JsonRpcError {
            code,
            message,
            data,
        },
        id: 1,
    };
    let serialised_err_res = serde_json::to_string(&jsonrpc_err).map_err(|err| {
        Error::Unexpected(format!(
            "Failed to serialise authd error response: {:?}",
            err
        ))
    })?;

    Ok(serialised_err_res.into_bytes().into())
}

fn successful_response(result: serde_json::Value) -> Result<FutureItemType, FutureErrorType> {
    let jsonrpc_res = JsonRpcResponse {
        jsonrpc: SAFE_AUTHD_JSONRPC_VERSION,
        result,
        id: 1,
    };
    let serialised_res = serde_json::to_string(&jsonrpc_res).map_err(|err| {
        Error::Unexpected(format!("Failed to serialise authd response: {:?}", err))
    })?;

    Ok(serialised_res.into_bytes().into())
}

pub fn process_request(
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
    req: Vec<u8>,
) -> ProcessRequest {
    ProcessRequest::HandleRequest {
        safe_auth_handle,
        auth_reqs_handle,
        notif_endpoints_handle,
        req,
    }
}

pub enum ProcessRequest {
    HandleRequest {
        safe_auth_handle: SharedSafeAuthenticatorHandle,
        auth_reqs_handle: SharedAuthReqsHandle,
        notif_endpoints_handle: SharedNotifEndpointsHandle,
        req: Vec<u8>,
    },
    ProcessingResponse {
        safe_auth_handle: SharedSafeAuthenticatorHandle,
        auth_reqs_handle: SharedAuthReqsHandle,
        rx: mpsc::Receiver<bool>,
        req_id: u32,
        auth_req_str: String,
    },
}

impl Future for ProcessRequest {
    type Item = FutureItemType;
    type Error = FutureErrorType;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::ProcessRequest::*;

        loop {
            match self {
                HandleRequest {
                    safe_auth_handle,
                    auth_reqs_handle,
                    notif_endpoints_handle,
                    req,
                } => {
                    let req_payload = match String::from_utf8(req.to_vec()) {
                        Ok(payload) => payload,
                        Err(err) => {
                            return Ok(Async::Ready(err_response(
                                "Request payload is a malformed UTF-8 string",
                                &err.to_string(),
                                JSONRPC_PARSE_ERROR,
                            )?))
                        }
                    };

                    let jsonrpc_req: JsonRpcReq = match serde_json::from_str(&req_payload) {
                        Ok(req) => req,
                        Err(err) => {
                            return Ok(Async::Ready(err_response(
                                "Failed to deserialise request payload as a JSON-RPC message",
                                &err.to_string(),
                                JSONRPC_INVALID_REQUEST,
                            )?))
                        }
                    };

                    match process_authenticator_req(
                        jsonrpc_req,
                        safe_auth_handle.clone(),
                        auth_reqs_handle.clone(),
                        notif_endpoints_handle.clone(),
                    ) {
                        Ok(AuthdResponse::Ready(response)) => return Ok(Async::Ready(response)),
                        Err(err) => return Err(err),
                        Ok(AuthdResponse::NotReady(processing_resp)) => *self = processing_resp,
                    };
                }
                ProcessingResponse {
                    safe_auth_handle,
                    auth_reqs_handle,
                    rx,
                    req_id,
                    auth_req_str,
                } => {
                    match rx.poll() {
                        Ok(Async::Ready(Some(is_allowed))) => {
                            if is_allowed {
                                println!("Let's request the authenticator lib to authorise the request '{}'...", req_id);
                                match lock_safe_authenticator(
                                    safe_auth_handle.clone(),
                                    |safe_authenticator| match safe_authenticator
                                        .authorise_app(auth_req_str)
                                    {
                                        Ok(resp) => {
                                            println!("Authorisation request ({}) was allowed and response sent back to the application", req_id);
                                            Ok(resp)
                                        }
                                        Err(err) => {
                                            println!("Failed to authorise application: {}", err);
                                            Ok(err.to_string())
                                        }
                                    },
                                ) {
                                    Ok(resp) => {
                                        return Ok(Async::Ready(successful_response(
                                            serde_json::value::Value::String(resp),
                                        )?))
                                    }
                                    Err(err) => {
                                        return Ok(Async::Ready(err_response(
                                            &err,
                                            "",
                                            JSONRPC_AUTH_ERROR,
                                        )?))
                                    }
                                }
                            } else {
                                let msg = format!("Authorisation request ({}) was denied", req_id);
                                println!("{}", msg);
                                return Ok(Async::Ready(err_response(
                                    &msg,
                                    "",
                                    JSONRPC_AUTH_ERROR,
                                )?));
                            }
                        }
                        Ok(Async::NotReady) => {
                            return Ok(Async::NotReady);
                        }
                        Ok(Async::Ready(None)) | Err(_) => {
                            // We didn't get a response in a timely manner, we cannot allow the list
                            // to grow infinitelly, so let's remove the request from it,
                            // even that the notifs thread may have removed it already
                            remove_auth_req_from_list(auth_reqs_handle.clone(), *req_id);
                            let msg = "Failed to get authorisation response";
                            println!("{}", msg);
                            return Ok(Async::Ready(err_response(msg, "", JSONRPC_AUTH_ERROR)?));
                        }
                    }
                }
            }
        }
    }
}

enum AuthdResponse {
    NotReady(ProcessRequest),
    Ready(FutureItemType),
}

fn process_authenticator_req(
    jsonrpc_req: JsonRpcReq,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<AuthdResponse, FutureErrorType> {
    println!("Processing new incoming request: '{}'", jsonrpc_req.method);

    let params = jsonrpc_req.params;
    let outcome = match jsonrpc_req.method {
        "status" => status::process_req(
            params,
            safe_auth_handle,
            auth_reqs_handle,
            notif_endpoints_handle,
        ),
        "login" => log_in::process_req(params, safe_auth_handle),
        "logout" => log_out::process_req(params, safe_auth_handle, auth_reqs_handle),
        "create-acc" => create_acc::process_req(params, safe_auth_handle),
        "authed-apps" => authed_apps::process_req(params, safe_auth_handle),
        "revoke" => revoke::process_req(params, safe_auth_handle),
        "auth-reqs" => auth_reqs::process_req(params, auth_reqs_handle),
        "allow" => allow::process_req(params, auth_reqs_handle),
        "deny" => deny::process_req(params, auth_reqs_handle),
        "subscribe" => subscribe::process_req(params, notif_endpoints_handle),
        "unsubscribe" => unsubscribe::process_req(params, notif_endpoints_handle),
        "authorise" => {
            match authorise::process_req(params, safe_auth_handle.clone(), auth_reqs_handle.clone())
            {
                Ok(authorise::AuthorisationResponse::NotReady((rx, req_id, auth_req_str))) => {
                    let processing_resp = ProcessRequest::ProcessingResponse {
                        safe_auth_handle: safe_auth_handle.clone(),
                        auth_reqs_handle: auth_reqs_handle.clone(),
                        rx,
                        req_id,
                        auth_req_str,
                    };
                    return Ok(AuthdResponse::NotReady(processing_resp));
                }
                Ok(authorise::AuthorisationResponse::Ready(resp)) => Ok(resp),
                Err(err) => Err(err),
            }
        }
        other => {
            let msg = format!(
                "Action '{}' not supported or unknown by the Authenticator daemon",
                other
            );
            println!("{}", msg);
            Err(msg)
        }
    };

    match outcome {
        Ok(result) => Ok(AuthdResponse::Ready(successful_response(result)?)),
        Err(err_msg) => Ok(AuthdResponse::Ready(err_response(
            &err_msg,
            "",
            JSONRPC_AUTH_ERROR,
        )?)),
    }
}
