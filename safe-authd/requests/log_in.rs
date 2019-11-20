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
    if let Value::Array(args) = &params {
        if args.len() > 2 || !args[0].is_string() || !args[1].is_string() {
            Err(format!("Incorrect params for 'login' method: {:?}", params))
        } else {
            println!("Logging in to SAFE account...");
            let passphrase = args[0].to_string();
            let password = args[1].to_string();

            lock_safe_authenticator(
                safe_auth_handle,
                |safe_authenticator| match safe_authenticator.log_in(&passphrase, &password) {
                    Ok(_) => {
                        let msg = "Logged in successfully!";
                        println!("{}", msg);
                        Ok(json!(msg))
                    }
                    Err(err) => {
                        println!("Error occurred when trying to log in: {}", err);
                        Err(err.to_string())
                    }
                },
            )
        }
    } else {
        Err(format!("Incorrect params for 'login' method: {:?}", params))
    }
}
