// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::shared::{lock_safe_authenticator, SharedSafeAuthenticatorHandle};
use serde_json::{json, Value};

pub fn process_req(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
) -> Result<Value, String> {
    if let Value::String(app_id) = params {
        println!("Revoking application...");
        lock_safe_authenticator(
            safe_auth_handle,
            |safe_authenticator| match safe_authenticator.revoke_app(&app_id) {
                Ok(()) => {
                    let msg = "Application revoked successfully";
                    println!("{}", msg);
                    Ok(json!(msg))
                }
                Err(err) => {
                    println!("Failed to revoke application '{}': {}", app_id, err);
                    Err(err.to_string())
                }
            },
        )
    } else {
        Err(format!(
            "Incorrect params for 'revoke' method: {:?}",
            params
        ))
    }
}
