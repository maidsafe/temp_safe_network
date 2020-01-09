// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

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
