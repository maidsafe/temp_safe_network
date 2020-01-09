// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::shared::{
    lock_auth_reqs_list, lock_notif_endpoints_list, lock_safe_authenticator, SharedAuthReqsHandle,
    SharedNotifEndpointsHandle, SharedSafeAuthenticatorHandle,
};
use safe_api::AuthdStatus;
use serde_json::{json, Value};

pub fn process_req(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<Value, String> {
    if Value::Null != params {
        Err(format!(
            "Unexpected param for 'status' method: {:?}",
            params
        ))
    } else {
        println!("Preparing authd status report...");

        let logged_in = lock_safe_authenticator(safe_auth_handle, |safe_authenticator| {
            Ok(safe_authenticator.is_logged_in())
        })?;

        let num_auth_reqs = lock_auth_reqs_list(auth_reqs_handle, |auth_reqs_list| {
            Ok(auth_reqs_list.len() as u32)
        })?;

        let num_notif_subs =
            lock_notif_endpoints_list(notif_endpoints_handle, |notif_endpoints_list| {
                Ok(notif_endpoints_list.len() as u32)
            })?;

        let status_report = AuthdStatus {
            logged_in,
            num_auth_reqs,
            num_notif_subs,
        };

        println!("Status report sent: {:?}", status_report);
        Ok(json!(status_report))
    }
}
