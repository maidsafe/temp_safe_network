// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod allow;
mod auth_reqs;
mod authed_apps;
mod authorise;
mod create_acc;
mod deny;
mod log_in;
mod log_out;
mod revoke;
mod status;
mod subscribe;
mod unsubscribe;

use crate::{
    errors::Error,
    shared::{SharedAuthReqsHandle, SharedNotifEndpointsHandle, SharedSafeAuthenticatorHandle},
};
use jsonrpc_quic::{JsonRpcRequest, JsonRpcResponse};

const JSONRPC_AUTH_ERROR: isize = -1;

// Process the JSON-RPC request based on the method
pub async fn process_jsonrpc_request(
    jsonrpc_req: JsonRpcRequest,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<JsonRpcResponse, Error> {
    println!(
        "Processing new incoming request ({}): '{}'",
        jsonrpc_req.id, jsonrpc_req.method
    );

    let params = jsonrpc_req.params;
    let outcome = match jsonrpc_req.method.as_str() {
        "status" => {
            status::process_req(
                params,
                safe_auth_handle,
                auth_reqs_handle,
                notif_endpoints_handle,
            )
            .await
        }
        "login" => log_in::process_req(params, safe_auth_handle).await,
        "logout" => log_out::process_req(params, safe_auth_handle, auth_reqs_handle).await,
        "create-acc" => create_acc::process_req(params, safe_auth_handle).await,
        "authed-apps" => authed_apps::process_req(params, safe_auth_handle).await,
        "revoke" => revoke::process_req(params, safe_auth_handle).await,
        "auth-reqs" => auth_reqs::process_req(params, auth_reqs_handle).await,
        "allow" => allow::process_req(params, auth_reqs_handle).await,
        "deny" => deny::process_req(params, auth_reqs_handle).await,
        "subscribe" => subscribe::process_req(params, notif_endpoints_handle).await,
        "unsubscribe" => unsubscribe::process_req(params, notif_endpoints_handle).await,
        "authorise" => {
            authorise::process_req(params, safe_auth_handle.clone(), auth_reqs_handle.clone()).await
        }
        other => {
            let msg = format!(
                "Action '{}' not supported or unknown by the Authenticator daemon",
                other
            );
            println!("{}", msg);
            Err(msg)
        }
    };

    // Return a JsonRpcResponse containing either a result or an error according to the outcome
    match outcome {
        Ok(result) => Ok(JsonRpcResponse::result(result, jsonrpc_req.id)),
        Err(err_msg) => {
            println!(
                "Failed processing incoming request {}: {}",
                jsonrpc_req.id, err_msg
            );
            Ok(JsonRpcResponse::error(
                err_msg,
                JSONRPC_AUTH_ERROR,
                Some(jsonrpc_req.id),
            ))
        }
    }
}
