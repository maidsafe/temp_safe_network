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
    if let Value::Array(args) = &params {
        if args.is_empty() || args.len() > 2 {
            Err(format!(
                "Incorrect number of params for 'login' method: {:?}",
                params
            ))
        } else {
            println!("Logging in to SAFE account...");
            let passphrase = args[0].as_str().ok_or_else(|| {
                format!(
                    "Invalid type for passphrase param for 'login' method: {:?}",
                    args[0]
                )
            })?;
            let password = args[1].as_str().ok_or_else(|| {
                format!(
                    "Invalid type for password param for 'login' method: {:?}",
                    args[1]
                )
            })?;

            let mut safe_authenticator = safe_auth_handle.lock().await;
            match safe_authenticator.log_in(passphrase, password) {
                Ok(_) => {
                    let msg = "Logged in successfully!";
                    println!("{}", msg);
                    Ok(json!(msg))
                }
                Err(err) => {
                    println!("Error occurred when trying to log in: {}", err);
                    Err(err.to_string())
                }
            }
        }
    } else {
        Err(format!("Incorrect params for 'login' method: {:?}", params))
    }
}
