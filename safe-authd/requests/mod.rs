// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

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

use crate::{
    errors::Error,
    shared::{
        lock_safe_authenticator, remove_auth_req_from_list, SharedAuthReqsHandle,
        SharedNotifEndpointsHandle, SharedSafeAuthenticatorHandle,
    },
};
use futures::{Async, Future, Poll, Stream};
use jsonrpc_quic::{
    jsonrpc_serialised_error, jsonrpc_serialised_result, parse_jsonrpc_request, JsonRpcReq,
};
use std::str;
use tokio::sync::mpsc;

type FutureItemType = Box<[u8]>;
type FutureErrorType = Error;

const JSONRPC_AUTH_ERROR: isize = -1;

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
        authd_req_id: u32,
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
                    let jsonrpc_req = match parse_jsonrpc_request(req.to_vec()) {
                        Ok(jsonrpc) => jsonrpc,
                        Err(err_response) => {
                            return Ok(Async::Ready(err_response.into_bytes().into()))
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
                    authd_req_id,
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
                                        return Ok(Async::Ready(gen_successful_response(
                                            serde_json::value::Value::String(resp),
                                            *authd_req_id,
                                        )?))
                                    }
                                    Err(err) => {
                                        return Ok(Async::Ready(gen_err_response(
                                            &err,
                                            "",
                                            JSONRPC_AUTH_ERROR,
                                            *authd_req_id,
                                        )?))
                                    }
                                }
                            } else {
                                let msg = format!("Authorisation request ({}) was denied", req_id);
                                println!("{}", msg);
                                return Ok(Async::Ready(gen_err_response(
                                    &msg,
                                    "",
                                    JSONRPC_AUTH_ERROR,
                                    *authd_req_id,
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
                            return Ok(Async::Ready(gen_err_response(
                                msg,
                                "",
                                JSONRPC_AUTH_ERROR,
                                *authd_req_id,
                            )?));
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
    let outcome = match jsonrpc_req.method.as_str() {
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
                        safe_auth_handle,
                        auth_reqs_handle,
                        rx,
                        req_id,
                        auth_req_str,
                        authd_req_id: jsonrpc_req.id,
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
        Ok(result) => Ok(AuthdResponse::Ready(gen_successful_response(
            result,
            jsonrpc_req.id,
        )?)),
        Err(err_msg) => Ok(AuthdResponse::Ready(gen_err_response(
            &err_msg,
            "",
            JSONRPC_AUTH_ERROR,
            jsonrpc_req.id,
        )?)),
    }
}

fn gen_err_response(
    message: &str,
    data: &str,
    code: isize,
    id: u32,
) -> Result<FutureItemType, FutureErrorType> {
    let serialised_err_res =
        jsonrpc_serialised_error(message, data, code, Some(id)).map_err(Error::Unexpected)?;
    Ok(serialised_err_res.into_bytes().into())
}

fn gen_successful_response(
    result: serde_json::Value,
    id: u32,
) -> Result<FutureItemType, FutureErrorType> {
    let serialised_res = jsonrpc_serialised_result(result, id).map_err(Error::Unexpected)?;
    Ok(serialised_res.into_bytes().into())
}
