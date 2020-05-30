// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::AuthReq;
use jsonrpc_quic::{
    Endpoint, IncomingJsonRpcRequest, JsonRpcRequest, JsonRpcResponse, JsonRpcResponseStream,
};
use log::{debug, info};
use serde_json::json;
use tokio::{
    runtime,
    sync::{mpsc, oneshot},
};
use url::Url;

const JSONRPC_NOTIF_ERROR: isize = -1;

// Start listening for incoming notifications from authd,
// by setting up a JSON-RPC over QUIC server endpoint
pub async fn jsonrpc_listen(
    listen: &str,
    cert_base_path: &str,
    notif_channel: mpsc::UnboundedSender<(AuthReq, oneshot::Sender<Option<bool>>)>,
) -> Result<(), String> {
    debug!("Launching new QUIC endpoint on '{}'", listen);

    let listen_socket_addr = Url::parse(listen)
        .map_err(|_| "Invalid endpoint address".to_string())?
        .socket_addrs(|| None)
        .map_err(|_| "Invalid endpoint address".to_string())?[0];

    let jsonrpc_quic_endpoint = Endpoint::new(cert_base_path, None)
        .map_err(|err| format!("Failed to create endpoint: {}", err))?;

    // We try to obtain current runtime or create a new one if there is none
    let runtime = match runtime::Handle::try_current() {
        Ok(r) => r,
        Err(_) => runtime::Runtime::new()
            .map_err(|err| format!("Failed to create runtime: {}", err))?
            .handle()
            .clone(),
    };

    let mut incoming_conn = runtime
        .enter(|| jsonrpc_quic_endpoint.bind(&listen_socket_addr))
        .map_err(|err| format!("Failed to bind endpoint: {}", err))?;

    while let Some(conn) = incoming_conn.get_next().await {
        tokio::spawn(handle_connection(conn, notif_channel.clone()));
    }

    Ok(())
}

async fn handle_connection(
    mut conn: IncomingJsonRpcRequest,
    notif_channel: mpsc::UnboundedSender<(AuthReq, oneshot::Sender<Option<bool>>)>,
) -> Result<(), String> {
    // Each stream initiated by the client constitutes a new request.
    tokio::spawn(async move {
        // Each stream initiated by the client constitutes a new request.
        while let Some((jsonrpc_req, send)) = conn.get_next().await {
            tokio::spawn(handle_request(jsonrpc_req, send, notif_channel.clone()));
        }
    });

    Ok(())
}

async fn handle_request(
    jsonrpc_req: JsonRpcRequest,
    mut send: JsonRpcResponseStream,
    notif_channel: mpsc::UnboundedSender<(AuthReq, oneshot::Sender<Option<bool>>)>,
) -> Result<(), String> {
    // Execute the request
    let resp = process_jsonrpc_request(jsonrpc_req, notif_channel).await;

    // Write the response
    send.respond(&resp)
        .await
        .map_err(|e| format!("Failed to send response: {}", e))?;

    // Gracefully terminate the stream
    send.finish()
        .await
        .map_err(|e| format!("Failed to shutdown stream: {}", e))?;

    info!("Request complete");
    Ok(())
}

async fn process_jsonrpc_request(
    jsonrpc_req: JsonRpcRequest,
    notif_channel: mpsc::UnboundedSender<(AuthReq, oneshot::Sender<Option<bool>>)>,
) -> JsonRpcResponse {
    let auth_req: AuthReq = match serde_json::from_value(jsonrpc_req.params) {
        Ok(auth_req) => auth_req,
        Err(err) => {
            return JsonRpcResponse::error(
                err.to_string(),
                JSONRPC_NOTIF_ERROR,
                Some(jsonrpc_req.id),
            )
        }
    };

    // New notification for auth req to be sent to user
    let app_id = auth_req.app_id.clone();
    let (decision_tx, decision_rx) = oneshot::channel::<Option<bool>>();
    match notif_channel.send((auth_req, decision_tx)) {
        Ok(_) => debug!(
            "Auth req notification from app id '{}' sent to user",
            app_id
        ),
        Err(err) => debug!(
            "Auth req notification for app id '{}' couldn't be sent to user: {}",
            app_id, err
        ),
    }

    // Send the decision made by the user back to authd
    // TODO: send the decision as Option<bool> instead of as String
    match decision_rx.await {
        Ok(Some(true)) => JsonRpcResponse::result(json!("true"), jsonrpc_req.id),
        Ok(Some(false)) => JsonRpcResponse::result(json!("false"), jsonrpc_req.id),
        Ok(None) => JsonRpcResponse::result(json!("No decision made"), jsonrpc_req.id),
        Err(err) => JsonRpcResponse::error(
            format!("Failed to obtain decision made for auth req: {}", err),
            JSONRPC_NOTIF_ERROR,
            Some(jsonrpc_req.id),
        ),
    }
}
