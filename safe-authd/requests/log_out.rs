// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::shared::{
    lock_auth_reqs_list, lock_safe_authenticator, SharedAuthReqsHandle,
    SharedSafeAuthenticatorHandle,
};
use serde_json::{json, Value};

pub fn process_req(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<Value, String> {
    if Value::Null != params {
        Err(format!(
            "Unexpected param for 'logout' method: {:?}",
            params
        ))
    } else {
        println!("Logging out...");
        lock_safe_authenticator(
            safe_auth_handle,
            |safe_authenticator| match safe_authenticator.log_out() {
                Ok(()) => {
                    let msg = "Logged out successfully";
                    println!("{}", msg);
                    Ok(json!(msg))
                }
                Err(err) => {
                    let msg = format!("Failed to log out: {}", err);
                    println!("{}", msg);
                    Err(msg)
                }
            },
        )
        .and_then(|msg| {
            let _ = lock_auth_reqs_list(auth_reqs_handle, |auth_reqs_list| {
                auth_reqs_list.clear();
                Ok(())
            });
            Ok(msg)
        })
    }
}
