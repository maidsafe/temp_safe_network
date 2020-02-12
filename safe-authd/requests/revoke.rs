// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::shared::SharedSafeAuthenticatorHandle;
use serde_json::{json, Value};

pub async fn process_req(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
) -> Result<Value, String> {
    if let Value::String(app_id) = params {
        println!("Revoking application...");
        let safe_authenticator = safe_auth_handle.lock().await;
        match safe_authenticator.revoke_app(&app_id) {
            Ok(()) => {
                let msg = "Application revoked successfully";
                println!("{}", msg);
                Ok(json!(msg))
            }
            Err(err) => {
                println!("Failed to revoke application '{}': {}", app_id, err);
                Err(err.to_string())
            }
        }
    } else {
        Err(format!(
            "Incorrect params for 'revoke' method: {:?}",
            params
        ))
    }
}
