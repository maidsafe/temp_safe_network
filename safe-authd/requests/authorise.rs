// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::authd::{IncomingAuthReq, SharedAuthReqsHandle, SharedSafeAuthenticatorHandle};
use safe_api::{AuthReq, SafeAuthReq};
use tokio::sync::mpsc;

// Authorisation requests wil be rejected if the numbre of pending auth reqs reaches this number
const MAX_NUMBER_QUEUED_AUTH_REQS: usize = 64;

pub enum AuthorisationResponse {
    Ready(String),
    NotReady((mpsc::Receiver<bool>, u32, String)),
}

pub fn process_req(
    args: &[&str],
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<AuthorisationResponse, String> {
    if args.len() != 1 {
        Err("Incorrect number of arguments for 'authorise' action".to_string())
    } else {
        println!("Authorising application...");
        let auth_req_str = args[0];

        let safe_authenticator = &mut *(safe_auth_handle.lock().unwrap());
        match safe_authenticator.decode_req(auth_req_str) {
            Ok((req_id, request)) => {
                // We have a valid decoded authorisation request,
                // let's now treat it according to its type
                match request {
                    SafeAuthReq::Auth(app_auth_req) => {
                        println!("The following application authorisation request was received:");
                        println!("{:?}", app_auth_req);

                        let auth_reqs_list = &mut *(auth_reqs_handle.lock().unwrap());

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
                                auth_req: AuthReq {
                                    req_id,
                                    app_id: app_auth_req.app.id,
                                    app_name: app_auth_req.app.name,
                                    app_vendor: app_auth_req.app.vendor,
                                    app_permissions: app_auth_req.app_permissions,
                                    containers: app_auth_req.containers,
                                    own_container: app_auth_req.app_container,
                                },
                                tx,
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
                        match safe_authenticator.authorise_app(auth_req_str) {
                            Ok(resp) => {
                                println!("Authorisation request ({}) was allowed and response sent back to the application", req_id);
                                Ok(AuthorisationResponse::Ready(resp))
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
    }
}
