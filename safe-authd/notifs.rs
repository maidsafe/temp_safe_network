// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::shared::*;
use jsonrpc_quic::ClientEndpoint;
use log::info;
use serde_json::json;
use std::{collections::BTreeMap, time::Duration};
use tokio::time::delay_for;

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

pub async fn monitor_pending_auth_reqs(
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) {
    loop {
        // Let's clone the notif and auth reqs lists so we don't lock
        // the mutex while sending notifications
        let (mut reqs_to_process, notif_endpoints_list) = {
            let auth_reqs_list = auth_reqs_handle.lock().await;
            if auth_reqs_list.is_empty() {
                // We don't have auth reqs so we won't need a copy of notif endpoints list
                (AuthReqsList::default(), BTreeMap::default())
            } else {
                let notif_endpoints_list = notif_endpoints_handle.lock().await;
                (auth_reqs_list.clone(), notif_endpoints_list.clone())
            }
        };

        // TODO: send a "keep subscription?" notif/request to subscribers periodically,
        // and remove them if they don't respond or if they reply with a negative response.
        for (req_id, incoming_auth_req) in reqs_to_process.iter_mut() {
            // Let's remove this auth req from the list if it's been standing for too long,
            // we assume the requestor already timed out by now
            let is_timeout = match incoming_auth_req.timestamp.elapsed() {
                Ok(elapsed) => {
                    if elapsed >= Duration::from_millis(AUTH_REQS_TIMEOUT) {
                        info!(
                                "Removing auth req '{}' from the queue since it timed out (it was received more than {} milliseconds ago)",
                                req_id, AUTH_REQS_TIMEOUT
                            );
                        true
                    } else {
                        false
                    }
                }
                Err(err) => {
                    info!("Unexpected error when checking auth req ('{}') elapsed time so it's being removed from the list: {:?}", req_id, err);
                    true
                }
            };

            if is_timeout {
                let mut auth_reqs_list = auth_reqs_handle.lock().await;
                auth_reqs_list.remove(req_id);
                continue;
            }

            // If it has been already notified we skip it
            if incoming_auth_req.notified {
                continue;
            }

            let mut response = None;
            let mut current_req_notified = false;
            for (url, cert_base_path) in notif_endpoints_list.iter() {
                match send_notification(url, incoming_auth_req, cert_base_path).await {
                    None => {
                        let mut notif_endpoints_list = notif_endpoints_handle.lock().await;
                        notif_endpoints_list.remove(url);
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

            info!(
                "Decision obtained for auth req sent by app id '{}' with id '{}': {:?}",
                incoming_auth_req.auth_req.app_id, incoming_auth_req.auth_req.req_id, response
            );

            if current_req_notified {
                // Then update its state in the list
                // ...but only if the auth req is still in the list, as it could
                // have been removed already if a user allowed/denied it with a request
                // while we were sending the notifications.
                let mut auth_reqs_list = auth_reqs_handle.lock().await;
                if response.is_some() {
                    // We can even remove it, since a decision to allow/deny was received
                    auth_reqs_list.remove(req_id);
                } else if auth_reqs_list.contains_key(req_id) {
                    let mut current_auth_req = incoming_auth_req.clone();
                    current_auth_req.notified = true;
                    let _ = auth_reqs_list.insert(*req_id, current_auth_req);
                }
            }

            if let Some(is_allowed) = response {
                match incoming_auth_req.tx.try_send(is_allowed) {
                    Ok(_) => info!("Auth req decision ready to be sent back to the application"),
                    Err(_) => {
                        info!("Auth req decision couldn't be sent, and therefore already denied")
                    }
                };
            }
        }

        delay_for(Duration::from_millis(AUTH_REQS_CHECK_FREQ)).await;
    }
}

async fn send_notification(
    url: &str,
    auth_req: &IncomingAuthReq,
    cert_base_path: &str,
) -> Option<NotifResponse> {
    info!("Notifying subscriber: {}", url);
    match jsonrpc_send(
        url,
        JSONRPC_METHOD_AUTH_REQ_NOTIF,
        json!(auth_req.auth_req),
        cert_base_path,
        None,
    )
    .await
    {
        Ok(notif_result) => {
            let response = if notif_result == "true" {
                Some(true)
            } else if notif_result == "false" {
                Some(false)
            } else {
                None
            };
            info!("Subscriber's response: {}", notif_result);
            Some(response)
        }
        Err(err) => {
            // Let's unsubscribe it immediately, ... we could be more laxed
            // in the future allowing some unresponsiveness
            info!(
                "Subscriber '{}' is being automatically unsubscribed since response to notification couldn't be obtained: {:?}",
                url, err
            );
            None
        }
    }
}

async fn jsonrpc_send(
    url: &str,
    method: &str,
    params: serde_json::Value,
    cert_base_path: &str,
    idle_timeout: Option<u64>,
) -> Result<String, String> {
    let jsonrpc_quic_client = ClientEndpoint::new(cert_base_path, idle_timeout, false)
        .map_err(|err| format!("Failed to create client endpoint: {}", err))?;

    let mut outgoing_conn = {
        jsonrpc_quic_client
            .bind()
            .map_err(|err| format!("Failed to bind endpoint: {}", err))?
    };

    let url2 = url.to_string();
    let method2 = method.to_string();
    let mut new_conn = outgoing_conn.connect(&url2, None).await?;

    // TODO: here the response result type should be changed to Option<bool>
    // We didn't do it yet as it breaks compatibility with other authd client apps
    let response = new_conn.send::<String>(&method2, params).await;

    // Allow the endpoint driver to automatically shut down
    drop(outgoing_conn);

    match response {
        Ok(r) => Ok(r),
        Err(err) => match err {
            jsonrpc_quic::Error::RemoteEndpointError(msg) => {
                // Subscriber responded but with an error, we won't unsubscribe it, but will
                // consider this response as a "no decision" for the auth req
                info!(
                    "Subscriber '{}' responded to the notification with an error: {:?}",
                    url, msg
                );
                Ok(msg)
            }
            other => Err(format!("{}", other)),
        },
    }
}
