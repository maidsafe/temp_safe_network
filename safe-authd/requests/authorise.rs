// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::shared::{IncomingAuthReq, SharedAuthReqsHandle, SharedSafeAuthenticatorHandle};
use safe_api::{AuthReq, SafeAuthReq};
use serde_json::{json, Value};
use std::time::SystemTime;
use tokio::sync::mpsc;

// Authorisation requests wil be automatically rejected if the number of pending auth reqs reaches this number
// This should never happen and it's just for the containment to keep authd healthy in such an unexpected scenario
const MAX_NUMBER_QUEUED_AUTH_REQS: usize = 64;

enum AuthorisationResponse {
    Ready(Value),
    NotReady((mpsc::Receiver<bool>, u32, String)),
}

pub async fn process_req(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<Value, String> {
    let resp =
        handle_authorisation(params, safe_auth_handle.clone(), auth_reqs_handle.clone()).await?;

    match resp {
        AuthorisationResponse::NotReady((rx, req_id, auth_req_str)) => {
            // Let's await for the decision response
            await_authorisation_decision(
                safe_auth_handle,
                auth_reqs_handle,
                rx,
                req_id,
                auth_req_str,
            )
            .await
        }
        AuthorisationResponse::Ready(resp) => Ok(resp),
    }
}

async fn handle_authorisation(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<AuthorisationResponse, String> {
    if let Value::String(auth_req_str) = params {
        println!("Authorising application...");
        let safe_authenticator = safe_auth_handle.lock().await;
        match safe_authenticator.decode_req(&auth_req_str) {
            Ok((req_id, request)) => {
                // We have a valid decoded authorisation request,
                // let's now treat it according to its type
                match request {
                    SafeAuthReq::Auth(app_auth_req) => {
                        println!(
                            "The following application authorisation request ({}) was received:",
                            req_id
                        );
                        println!("{:?}", app_auth_req);

                        let mut auth_reqs_list = auth_reqs_handle.lock().await;

                        // Reject if there are too many pending auth reqs
                        if auth_reqs_list.len() >= MAX_NUMBER_QUEUED_AUTH_REQS {
                            Err(format!("Authorisation request ({}) is rejected by authd since it reached its maximum number ({}) of pending auth requests", req_id, MAX_NUMBER_QUEUED_AUTH_REQS))
                        } else {
                            // We need a channel to communicate with the thread which will be
                            // sending the notification to a subcribed endpoint. Once it got a response
                            // it will send it back through this channel so it can in turn be
                            // sent to the application requesting this authorisation.
                            let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) =
                                mpsc::channel(32);

                            // Let's add it to the list of pending authorisation requests
                            let auth_req = IncomingAuthReq {
                                timestamp: SystemTime::now(),
                                auth_req: AuthReq {
                                    req_id,
                                    app_id: app_auth_req.app.id.clone(),
                                    app_name: app_auth_req.app.name.clone(),
                                    app_vendor: app_auth_req.app.vendor.clone(),
                                    app_permissions: app_auth_req.app_permissions,
                                    containers: app_auth_req.containers.clone(),
                                    own_container: app_auth_req.app_container,
                                },
                                tx,
                                notified: false,
                            };
                            auth_reqs_list.insert(req_id, auth_req);

                            Ok(AuthorisationResponse::NotReady((
                                rx,
                                req_id,
                                auth_req_str.to_string(),
                            )))
                        }
                    }
                    SafeAuthReq::Containers(cont_req) => {
                        println!(
                            "The following authorisation request for containers was received:"
                        );
                        println!("{:?}", cont_req);
                        Err(
                            "Authorisation request for Containers is not yet supported by authd"
                                .to_string(),
                        )
                    }
                    SafeAuthReq::ShareMData(share_mdata_req) => {
                        println!("The following authorisation request to share a MutableData was received:");
                        println!("{:?}", share_mdata_req);
                        Err(
                            "Authorisation request for Share MutableData is not yet supported by authd"
                                .to_string(),
                        )
                    }
                    SafeAuthReq::Unregistered(_) => {
                        // We simply allow unregistered authorisation requests
                        match safe_authenticator.authorise_app(&auth_req_str) {
                            Ok(resp) => {
                                println!("Authorisation request ({}) was allowed and response sent back to the application", req_id);
                                Ok(AuthorisationResponse::Ready(json!(resp)))
                            }
                            Err(err) => {
                                println!("Failed to authorise application: {}", err);
                                Err(err.to_string())
                            }
                        }
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
                Err(err.to_string())
            }
        }
    } else {
        Err(format!(
            "Incorrect params for 'authorise' method: {:?}",
            params
        ))
    }
}

async fn await_authorisation_decision(
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    mut rx: mpsc::Receiver<bool>,
    req_id: u32,
    auth_req_str: String,
) -> Result<Value, String> {
    match rx.recv().await {
        Some(is_allowed) => {
            if is_allowed {
                println!(
                    "Let's request the authenticator lib to authorise the request '{}'...",
                    req_id
                );
                let safe_authenticator = safe_auth_handle.lock().await;
                match safe_authenticator.authorise_app(&auth_req_str) {
                    Ok(resp) => {
                        println!("Authorisation request ({}) was allowed and response sent back to the application", req_id);
                        Ok(serde_json::value::Value::String(resp))
                    }
                    Err(err) => {
                        println!("Failed to authorise application: {}", err);
                        Err(err.to_string())
                    }
                }
            } else {
                let msg = format!("Authorisation request ({}) was denied", req_id);
                println!("{}", msg);
                Err(msg)
            }
        }
        None => {
            // We didn't get a response in a timely manner, we cannot allow the list
            // to grow infinitelly, so let's remove the request from it,
            // even that the notifs thread may have removed it already
            let mut auth_reqs_list = auth_reqs_handle.lock().await;
            auth_reqs_list.remove(&req_id);
            Err("Failed to get authorisation response".to_string())
        }
    }
}
