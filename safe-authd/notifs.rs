// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::quic_client::quic_send;
use super::shared::*;
use std::collections::BTreeSet;
use std::thread;
use std::time::Duration;

// Frequency for checking pending auth requests
const AUTH_REQS_CHECK_FREQ: u64 = 1000;

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
                        Ok((AuthReqsList::default(), BTreeSet::default()))
                    } else {
                        lock_notif_endpoints_list(
                            notif_endpoints_handle.clone(),
                            |notif_endpoints_list| {
                                if notif_endpoints_list.is_empty() {
                                    // We don't have subscriptors so we won't need a copy of the auths reqs list
                                    Ok((AuthReqsList::default(), BTreeSet::default()))
                                } else {
                                    Ok((auth_reqs_list.clone(), notif_endpoints_list.clone()))
                                }
                            },
                        )
                    }
                })
                .unwrap_or_else(|_| (AuthReqsList::default(), BTreeSet::default()));

            // TODO: send a "keep subscription?" notif/request to subscriptors periodically,
            // and remove them if they don't respond or if they reply with a negative response.
            for (req_id, incoming_auth_req) in reqs_to_process.iter_mut() {
                if incoming_auth_req.notified {
                    // TODO: if we checked the same auth req X number of times we should then remove it
                    continue;
                }

                let mut response = None;
                for endpoint in notif_endpoints_list.iter() {
                    println!("Notifying subscriptor: {}", endpoint);
                    match quic_send(
                        &format!(
                            "{}/{}/{}",
                            endpoint,
                            incoming_auth_req.auth_req.app_id,
                            incoming_auth_req.auth_req.req_id
                        ),
                        false,
                        None,
                        None,
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
                            break;
                        }
                        Err(err) => {
                            // TODO: we may need to remove it immediately
                            println!(
                                "Subscriptor '{}' didn't respond to notification: {}",
                                endpoint, err
                            );
                        }
                    }
                }
                println!(
                    "Decision for Req ID: {} - App ID: {} ??: {:?}",
                    incoming_auth_req.auth_req.req_id, incoming_auth_req.auth_req.app_id, response
                );
                if let Some(is_allowed) = response {
                    match incoming_auth_req.tx.try_send(is_allowed) {
                        Ok(_) => println!("Auth req decision ready to be sent to application"),
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
