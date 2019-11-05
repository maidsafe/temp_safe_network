// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::quic_client::quic_send;
use super::shared::*;
use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;

// Frequency for checking pending auth requests
const AUTH_REQS_CHECK_FREQ: u64 = 1000;

// Time elapsed since an auth request was received to consider it timed out
// This is used to keep the list of auth requests always clean from unhandled requests
const AUTH_REQS_TIMEOUT: u64 = 3 * 60000;

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

            // TODO: send a "keep subscription?" notif/request to subscriptors periodically,
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
                for (url, cert_base_path) in notif_endpoints_list.iter() {
                    println!("Notifying subscriptor: {}", url);
                    match quic_send(
                        &format!(
                            "{}/{}/{}",
                            url,
                            incoming_auth_req.auth_req.app_id,
                            incoming_auth_req.auth_req.req_id
                        ),
                        false,
                        None,
                        cert_base_path.as_ref().map(String::as_str),
                        false,
                    ) {
                        Ok(notif_resp) => {
                            // TODO: implement JSON-RPC or some other format
                            response = if notif_resp.starts_with("true") {
                                Some(true)
                            } else if notif_resp.starts_with("false") {
                                Some(false)
                            } else {
                                None
                            };
                            println!("Subscriptor's response: {}", notif_resp);
                            let _ =
                                lock_auth_reqs_list(auth_reqs_handle.clone(), |auth_reqs_list| {
                                    let mut current_auth_req = incoming_auth_req.clone();
                                    current_auth_req.notified = true;
                                    auth_reqs_list.insert(*req_id, current_auth_req).unwrap();
                                    Ok(())
                                });

                            // We don't notify other subscriptors as it was allowed/denied already
                            if response.is_some() {
                                break;
                            }
                        }
                        Err(err) => {
                            // Let's unsubscribe it immediately, ... we could be more laxed
                            // in the future allowing some unresponsiveness
                            println!(
                                "Subscriptor '{}' is being automatically unsubscribed since it didn't respond to notification: {}",
                                url, err
                            );
                            remove_notif_endpoint_from_list(notif_endpoints_handle.clone(), url);
                        }
                    }
                }
                println!(
                    "Decision for auth req ID: {} - App ID: {}: {:?}",
                    incoming_auth_req.auth_req.req_id, incoming_auth_req.auth_req.app_id, response
                );
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
