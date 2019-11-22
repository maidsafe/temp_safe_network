// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::shared::*;
use jsonrpc_quic::jsonrpc_send;
use serde_json::json;
use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;

// Frequency for checking pending auth requests
const AUTH_REQS_CHECK_FREQ: u64 = 1000;

// Time elapsed since an auth request was received to consider it timed out
// This is used to keep the list of auth requests always clean from unhandled requests
const AUTH_REQS_TIMEOUT: u64 = 3 * 60000;

// JSON-RPC Method name for authorisation request notification
const JSONRPC_METHOD_AUTH_REQ_NOTIF: &str = "auth-req-notif";

// Am auth request notification can be responded with a positive (Some(true))
// or negative (Some(false)) decision, or simply with an acknowledgment (None)
type NotifResponse = Option<bool>;

pub fn monitor_pending_auth_reqs(
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) {
    thread::spawn(move || loop {
        {
            // Let's clone the notif and auth reqs lists so we don't lock
            // the mutex while sending notifications
            let (mut reqs_to_process, notif_endpoints_list) =
                lock_auth_reqs_list(auth_reqs_handle.clone(), |auth_reqs_list| {
                    if auth_reqs_list.is_empty() {
                        // We don't have auth reqs so we won't need a copy of notif endpoints list
                        Ok((AuthReqsList::default(), BTreeMap::default()))
                    } else {
                        lock_notif_endpoints_list(
                            notif_endpoints_handle.clone(),
                            |notif_endpoints_list| {
                                Ok((auth_reqs_list.clone(), notif_endpoints_list.clone()))
                            },
                        )
                    }
                })
                .unwrap_or_else(|_| (AuthReqsList::default(), BTreeMap::default()));

            // TODO: send a "keep subscription?" notif/request to subscribers periodically,
            // and remove them if they don't respond or if they reply with a negative response.
            for (req_id, incoming_auth_req) in reqs_to_process.iter_mut() {
                // Let's remove this auth req from the list if it's been standing for too long,
                // we assume the requestor already timed out out by now
                let is_timeout = match incoming_auth_req.timestamp.elapsed() {
                    Ok(elapsed) => {
                        if elapsed >= Duration::from_millis(AUTH_REQS_TIMEOUT) {
                            println!(
                                "Removing auth req '{}' from the queue since it timed out (it was received more than {} milliseconds ago)",
                                req_id, AUTH_REQS_TIMEOUT
                            );
                            true
                        } else {
                            false
                        }
                    }
                    Err(err) => {
                        println!("Unexpected error when checking auth req ('{}') elapsed time so it's being removed from the list: {:?}", req_id, err);
                        true
                    }
                };

                if is_timeout {
                    remove_auth_req_from_list(auth_reqs_handle.clone(), *req_id);
                    continue;
                }

                // If it has been already notified we skip it
                if incoming_auth_req.notified {
                    continue;
                }

                let mut response = None;
                let mut current_req_notified = false;
                for (url, cert_base_path) in notif_endpoints_list.iter() {
                    match send_notification(
                        url,
                        incoming_auth_req,
                        cert_base_path.as_ref().map(String::as_str),
                    ) {
                        None => {
                            remove_notif_endpoint_from_list(notif_endpoints_handle.clone(), url)
                        }
                        Some(resp) => {
                            // We know at least one subscriber has been notified since it replied
                            current_req_notified = true;

                            // We don't notify other subscribers as it was allowed/denied already
                            if resp.is_some() {
                                response = resp;
                                break;
                            }
                        }
                    }
                }

                println!(
                    "Decision obtained for auth req id: {} - from app id: {}: {:?}",
                    incoming_auth_req.auth_req.req_id, incoming_auth_req.auth_req.app_id, response
                );

                if current_req_notified {
                    // then update its state in the list
                    let _ = lock_auth_reqs_list(auth_reqs_handle.clone(), |auth_reqs_list| {
                        // ...but only if the auth req is still in the list, as it could
                        // have been removed already if a user allowed/denied it with a request
                        // while we were sending the notifications
                        if auth_reqs_list.contains_key(req_id) {
                            let mut current_auth_req = incoming_auth_req.clone();
                            current_auth_req.notified = true;
                            let _ = auth_reqs_list.insert(*req_id, current_auth_req);
                        }
                        Ok(())
                    });
                }

                if let Some(is_allowed) = response {
                    match incoming_auth_req.tx.try_send(is_allowed) {
                        Ok(_) => {
                            println!("Auth req decision ready to be sent back to the application")
                        }
                        Err(_) => println!(
                            "Auth req decision couldn't be sent, and therefore already denied"
                        ),
                    };
                }
            }
        }

        thread::sleep(Duration::from_millis(AUTH_REQS_CHECK_FREQ));
    });
}

fn send_notification(
    url: &str,
    auth_req: &IncomingAuthReq,
    cert_base_path: Option<&str>,
) -> Option<NotifResponse> {
    println!("Notifying subscriber: {}", url);
    match jsonrpc_send::<String>(
        url,
        JSONRPC_METHOD_AUTH_REQ_NOTIF,
        json!(auth_req.auth_req),
        cert_base_path,
        None,
    ) {
        Ok(notif_result) => {
            let response = if notif_result == "true" {
                Some(true)
            } else if notif_result == "false" {
                Some(false)
            } else {
                None
            };
            println!("Subscriber's response: {}", notif_result);
            Some(response)
        }
        Err(err) => {
            match err {
                jsonrpc_quic::Error::ClientError(msg) => {
                    // Let's unsubscribe it immediately, ... we could be more laxed
                    // in the future allowing some unresponsiveness
                    println!(
                        "Subscriber '{}' is being automatically unsubscribed since it didn't respond to notification: {:?}",
                        url, msg
                    );
                    None
                }
                jsonrpc_quic::Error::ServerError(msg) => {
                    // Subscriber responded but with an error, we won't unsubscribe it, but will
                    // consider this response as a "no decision" for the auth req
                    println!(
                        "Subscriber '{}' responded to the notification with an error: {:?}",
                        url, msg
                    );
                    Some(None)
                }
            }
        }
    }
}
