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
use log::{error, info};

// Error code in JSON-RPC response when failed to process a request
// TODO: have different error codes for each error case
const JSONRPC_AUTH_ERROR: isize = -1;

// Method for requesting application's authorisation
const METHOD_AUTHORISE: &str = "authorise";

// Method for getting a status report of the safe-authd
const METHOD_STATUS: &str = "status";

// Method for logging into a SAFE account
const METHOD_LOGIN: &str = "login";

// Method for logging out from a SAFE account
const METHOD_LOGOUT: &str = "logout";

// Method for creating a new SAFE account
const METHOD_CREATE: &str = "create-acc";

// Method for fetching list of authorised apps
const METHOD_AUTHED_APPS: &str = "authed-apps";

// Method for revoking applications and/or permissions
const METHOD_REVOKE: &str = "revoke";

// Method for retrieving the list of pending authorisation requests
const METHOD_AUTH_REQS: &str = "auth-reqs";

// Method for allowing an authorisation request
const METHOD_ALLOW: &str = "allow";

// Method for denying an authorisation request
const METHOD_DENY: &str = "deny";

// Method for subscribing to authorisation requests notifications
const METHOD_SUBSCRIBE: &str = "subscribe";

// Method for unsubscribing from authorisation requests notifications
const METHOD_UNSUBSCRIBE: &str = "unsubscribe";

// Process the JSON-RPC request based on the method
pub async fn process_jsonrpc_request(
    jsonrpc_req: JsonRpcRequest,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<JsonRpcResponse, Error> {
    info!(
        "Processing new incoming request ({}): '{}'",
        jsonrpc_req.id, jsonrpc_req.method
    );

    let params = jsonrpc_req.params;
    let outcome = match jsonrpc_req.method.as_str() {
        METHOD_STATUS => {
            status::process_req(
                params,
                safe_auth_handle,
                auth_reqs_handle,
                notif_endpoints_handle,
            )
            .await
        }
        METHOD_LOGIN => log_in::process_req(params, safe_auth_handle).await,
        METHOD_LOGOUT => log_out::process_req(params, safe_auth_handle, auth_reqs_handle).await,
        METHOD_CREATE => create_acc::process_req(params, safe_auth_handle).await,
        METHOD_AUTHED_APPS => authed_apps::process_req(params, safe_auth_handle).await,
        METHOD_REVOKE => revoke::process_req(params, safe_auth_handle).await,
        METHOD_AUTH_REQS => auth_reqs::process_req(params, auth_reqs_handle).await,
        METHOD_ALLOW => allow::process_req(params, auth_reqs_handle).await,
        METHOD_DENY => deny::process_req(params, auth_reqs_handle).await,
        METHOD_SUBSCRIBE => subscribe::process_req(params, notif_endpoints_handle).await,
        METHOD_UNSUBSCRIBE => unsubscribe::process_req(params, notif_endpoints_handle).await,
        METHOD_AUTHORISE => {
            authorise::process_req(params, safe_auth_handle.clone(), auth_reqs_handle.clone()).await
        }
        other => {
            let msg = format!(
                "Method '{}' not supported or unknown by the Authenticator daemon",
                other
            );
            error!("{}", msg);
            Err(msg)
        }
    };

    // Return a JsonRpcResponse containing either a result or an error according to the outcome
    match outcome {
        Ok(result) => Ok(JsonRpcResponse::result(result, jsonrpc_req.id)),
        Err(err_msg) => {
            error!(
                "Error when processing incoming '{}' request with id {}: {}",
                jsonrpc_req.method, jsonrpc_req.id, err_msg
            );
            Ok(JsonRpcResponse::error(
                err_msg,
                JSONRPC_AUTH_ERROR,
                Some(jsonrpc_req.id),
            ))
        }
    }
}
