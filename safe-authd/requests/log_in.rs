// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

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
