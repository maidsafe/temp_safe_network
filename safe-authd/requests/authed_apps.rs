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
    if Value::Null != params {
        Err(format!(
            "Unexpected param for 'authed-apps' method: {:?}",
            params
        ))
    } else {
        println!("Obtaining list of authorised applications...");
        lock_safe_authenticator(
            safe_auth_handle,
            |safe_authenticator| match safe_authenticator.authed_apps() {
                Ok(authed_apps_list) => {
                    println!("List of authorised apps sent: {:?}", authed_apps_list);
                    Ok(json!(authed_apps_list))
                }
                Err(err) => {
                    println!("Failed to get list of authorised apps: {}", err);
                    Err(err.to_string())
                }
            },
        )
    }
}
