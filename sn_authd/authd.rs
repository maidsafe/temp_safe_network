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
use log::{debug, error, info};
use qjsonrpc::{Endpoint, IncomingJsonRpcRequest, JsonRpcRequest, JsonRpcResponseStream};
use sn_api::SafeAuthenticator;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    net::SocketAddr,
    path::Path,
    str,
    sync::Arc,
};
use url::Url;

// Number of milliseconds to allow an idle connection before closing it
const CONNECTION_IDLE_TIMEOUT: u64 = 120_000;

pub async fn run(
    listen: &str,
    cert_base_path: Option<&Path>,
    config_dir_path: Option<&Path>,
) -> Result<()> {
    let bootstrap_contacts = get_current_network_conn_info()?;
    info!("Bootstrapping with contacts: {:?}", bootstrap_contacts);
    let safe_auth_handle: SharedSafeAuthenticatorHandle = Arc::new(Mutex::new(
        SafeAuthenticator::new(config_dir_path, Some(bootstrap_contacts)),
    ));

    // We keep a queue for all the authorisation requests
    let auth_reqs_handle = Arc::new(Mutex::new(AuthReqsList::new()));

    // We keep a list of the notifications subscribers' endpoints
    let notif_endpoints_handle = Arc::new(Mutex::new(BTreeMap::new()));

    let base_path = cert_base_path.map_or_else(
        || match dirs_next::home_dir() {
            Some(mut path) => {
                path.push(".safe");
                path.push("authd");
                Ok(path)
            }
            None => Err(Error::GeneralError(
                "Failed to obtain local project directory where to write certificate from"
                    .to_string(),
            )),
        },
        |path| Ok(path.to_path_buf()),
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
}

// Private helpers

fn get_current_network_conn_info() -> Result<HashSet<SocketAddr>> {
    let mut conn_info_path = dirs_next::home_dir()
        .ok_or_else(|| Error::GeneralError("Failed to obtain user's home path".to_string()))?;

    conn_info_path.push(".safe");
    conn_info_path.push("node");
    conn_info_path.push("node_connection_info.config");

    // Fetch it from a local file then
    debug!(
        "Reading network connection information from {} ...",
        conn_info_path.display()
    );
    let bytes = fs::read(&conn_info_path).map_err(|err| {
        Error::GeneralError(format!(
            "Unable to read connection information from '{}': {}",
            conn_info_path.display(),
            err
        ))
    })?;

    serde_json::from_slice(&bytes).map_err(|err| {
        Error::GeneralError(format!(
            "Format of the contacts addresses is not valid and couldn't be parsed: {}",
            err
        ))
    })
}

async fn start_listening(
    listen: &str,
    cert_base_path: &Path,
    idle_timeout: Option<u64>,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<()> {
    let listen_socket_addr = Url::parse(listen)
        .map_err(|_| Error::GeneralError("Invalid endpoint address".to_string()))?
        .socket_addrs(|| None)
        .map_err(|_| Error::GeneralError("Invalid endpoint address".to_string()))?[0];

    let qjsonrpc_endpoint = Endpoint::new(cert_base_path, idle_timeout)
        .map_err(|err| Error::GeneralError(format!("Failed to create endpoint: {}", err)))?;

    let mut incoming_conn = qjsonrpc_endpoint
        .bind(&listen_socket_addr)
        .map_err(|err| Error::GeneralError(format!("Failed to bind endpoint: {}", err)))?;
    info!("Listening on {}", listen_socket_addr);

    // Let's spawn a task which will monitor pending auth reqs
    // and get them allowed/denied by the user using any of the subcribed endpoints
    let auth_reqs_handle2 = auth_reqs_handle.clone();
    let notif_endpoints_handle2 = notif_endpoints_handle.clone();
    tokio::spawn(monitor_pending_auth_reqs(
        auth_reqs_handle2,
        notif_endpoints_handle2,
    ));

    while let Some(conn) = incoming_conn.get_next().await {
        tokio::spawn(
            handle_connection(
                conn,
                safe_auth_handle.clone(),
                auth_reqs_handle.clone(),
                notif_endpoints_handle.clone(),
            )
            .unwrap_or_else(move |e| error!("{reason}", reason = e.to_string())),
        );
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
    let req_id = jsonrpc_req.id;

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

    info!("Response sent, request {} complete", req_id);
    Ok(())
}
