// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::shared::{
    lock_auth_reqs_list, lock_safe_authenticator, IncomingAuthReq, SharedAuthReqsHandle,
    SharedSafeAuthenticatorHandle,
};
use safe_api::{AuthReq, SafeAuthReq};
use serde_json::{json, Value};
use std::time::SystemTime;
use tokio::sync::mpsc;

// Authorisation requests wil be automatically rejected if the number of pending auth reqs reaches this number
// This should never happen and it's just for the containment to keep authd healthy in such an unexpected scenario
const MAX_NUMBER_QUEUED_AUTH_REQS: usize = 64;

pub enum AuthorisationResponse {
    Ready(Value),
    NotReady((mpsc::Receiver<bool>, u32, String)),
}

pub fn process_req(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<AuthorisationResponse, String> {
    if let Value::String(auth_req_str) = params {
        println!("Authorising application...");
        lock_safe_authenticator(safe_auth_handle, |safe_authenticator| {
            match safe_authenticator.decode_req(&auth_req_str) {
                Ok((req_id, request)) => {
                    // We have a valid decoded authorisation request,
                    // let's now treat it according to its type
                    match request {
                        SafeAuthReq::Auth(app_auth_req) => {
                            println!(
                                "The following application authorisation request was received:"
                            );
                            println!("{:?}", app_auth_req);

                            lock_auth_reqs_list(auth_reqs_handle.clone(), |auth_reqs_list| {
                                // Reject if there are too many pending auth reqs
                                if auth_reqs_list.len() >= MAX_NUMBER_QUEUED_AUTH_REQS {
                                    Err(format!("Authorisation request is rejected by authd since it reached its maximum number ({}) of pending auth requests", MAX_NUMBER_QUEUED_AUTH_REQS))
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
                            })
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
        })
    } else {
        Err(format!(
            "Incorrect params for 'authorise' method: {:?}",
            params
        ))
    }
}
