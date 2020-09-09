// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::shared::{
    SharedAuthReqsHandle, SharedNotifEndpointsHandle, SharedSafeAuthenticatorHandle,
};
use log::info;
use serde_json::{json, Value};
use sn_api::AuthdStatus;

pub async fn process_req(
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
        info!("Preparing authd status report...");

        let logged_in = {
            let safe_authenticator = safe_auth_handle.lock().await;
            safe_authenticator.is_logged_in()
        };

        let num_auth_reqs = {
            let auth_reqs_list = auth_reqs_handle.lock().await;
            auth_reqs_list.len() as u32
        };

        let num_notif_subs = {
            let notif_endpoints_list = notif_endpoints_handle.lock().await;
            notif_endpoints_list.len() as u32
        };

        let authd_version = cargo_crate_version!().to_string();

        let status_report = AuthdStatus {
            logged_in,
            num_auth_reqs,
            num_notif_subs,
            authd_version,
        };

        info!("Status report sent: {:?}", status_report);
        Ok(json!(status_report))
    }
}
