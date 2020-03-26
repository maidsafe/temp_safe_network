// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    notifs::monitor_pending_auth_reqs, requests::process_jsonrpc_request, shared::*, Error, Result,
};
use futures::{lock::Mutex, TryFutureExt};
use jsonrpc_quic::{Endpoint, IncomingJsonRpcRequest, JsonRpcRequest, JsonRpcResponseStream};
use log::{error, info};
use safe_api::SafeAuthenticator;
use std::{collections::BTreeMap, str, sync::Arc};
use url::Url;

// Number of milliseconds to allow an idle connection before closing it
const CONNECTION_IDLE_TIMEOUT: u64 = 60_000;

pub async fn run(
    listen: &str,
    cert_base_path: Option<&str>,
    config_dir_path: Option<&str>,
) -> Result<()> {
    let safe_auth_handle: SharedSafeAuthenticatorHandle =
        Arc::new(Mutex::new(SafeAuthenticator::new(config_dir_path)));

    // We keep a queue for all the authorisation requests
    let auth_reqs_handle = Arc::new(Mutex::new(AuthReqsList::new()));

    // We keep a list of the notifications subscribers' endpoints
    let notif_endpoints_handle = Arc::new(Mutex::new(BTreeMap::new()));

    let base_path = cert_base_path.map_or_else(
        || match directories::ProjectDirs::from("net", "maidsafe", "safe-authd") {
            Some(dirs) => Ok(dirs.config_dir().display().to_string()),
            None => Err(Error::GeneralError(
                "Failed to obtain local project directory where to write certificate from"
                    .to_string(),
            )),
        },
        |path| Ok(path.to_string()),
    )?;

    start_listening(
        listen,
        &base_path,
        Some(CONNECTION_IDLE_TIMEOUT),
        safe_auth_handle,
        auth_reqs_handle,
        notif_endpoints_handle,
    )
    .await
    .map_err(|err| Error::GeneralError(err.to_string()))
}

// Private helpers

async fn start_listening(
    listen: &str,
    cert_base_path: &str,
    idle_timeout: Option<u64>,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<()> {
    let listen_socket_addr = Url::parse(listen)
        .map_err(|_| Error::GeneralError("Invalid endpoint address".to_string()))?
        .socket_addrs(|| None)
        .map_err(|_| Error::GeneralError("Invalid endpoint address".to_string()))?[0];

    let jsonrpc_quic_endpoint = Endpoint::new(cert_base_path, idle_timeout)
        .map_err(|err| Error::GeneralError(format!("Failed to create endpoint: {}", err)))?;

    let mut incoming_conn = jsonrpc_quic_endpoint
        .bind(&listen_socket_addr)
        .map_err(|err| Error::GeneralError(format!("Failed to bind endpoint: {}", err)))?;
    println!("Listening on {}", listen_socket_addr);

    // Let's spawn a task which will monitor pending auth reqs
    // and get them allowed/denied by the user using any of the subcribed endpoints
    let auth_reqs_handle2 = auth_reqs_handle.clone();
    let notif_endpoints_handle2 = notif_endpoints_handle.clone();
    tokio::spawn(async {
        monitor_pending_auth_reqs(auth_reqs_handle2, notif_endpoints_handle2).await
    });

    while let Some(conn) = incoming_conn.get_next().await {
        tokio::spawn({
            handle_connection(
                conn,
                safe_auth_handle.clone(),
                auth_reqs_handle.clone(),
                notif_endpoints_handle.clone(),
            )
            .unwrap_or_else(move |e| error!("{reason}", reason = e.to_string()))
        });
    }

    Ok(())
}

async fn handle_connection(
    mut conn: IncomingJsonRpcRequest,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<()> {
    // Each stream initiated by the client constitutes a new request.
    while let Some((jsonrpc_req, send)) = conn.get_next().await {
        tokio::spawn(
            handle_request(
                jsonrpc_req,
                send,
                safe_auth_handle.clone(),
                auth_reqs_handle.clone(),
                notif_endpoints_handle.clone(),
            )
            .unwrap_or_else(move |e| error!("{reason}", reason = e.to_string())),
        );
    }

    Ok(())
}

async fn handle_request(
    jsonrpc_req: JsonRpcRequest,
    mut send: JsonRpcResponseStream,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<()> {
    // Execute the request
    let resp = process_jsonrpc_request(
        jsonrpc_req,
        safe_auth_handle,
        auth_reqs_handle,
        notif_endpoints_handle,
    )
    .await?;

    // Write the response
    send.respond(&resp)
        .await
        .map_err(|e| Error::GeneralError(format!("Failed to send response: {}", e)))?;

    // Gracefully terminate the stream
    send.finish()
        .await
        .map_err(|e| Error::GeneralError(format!("Failed to shutdown stream: {}", e)))?;

    info!("Request complete");
    Ok(())
}
