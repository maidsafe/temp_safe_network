// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::shared::{
    lock_auth_reqs_list, lock_notif_endpoints_list, lock_safe_authenticator, SharedAuthReqsHandle,
    SharedNotifEndpointsHandle, SharedSafeAuthenticatorHandle,
};
use safe_api::AuthdStatus;

pub fn process_req(
    args: &[&str],
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<String, String> {
    if !args.is_empty() {
        Err("Incorrect number of arguments for 'status' request".to_string())
    } else {
        println!("Preparing authd status report...");

        let logged_in = lock_safe_authenticator(safe_auth_handle, |safe_authenticator| {
            Ok(safe_authenticator.is_logged_in())
        })?;

        let num_auth_reqs = lock_auth_reqs_list(auth_reqs_handle.clone(), |auth_reqs_list| {
            Ok(auth_reqs_list.len() as u32)
        })?;

        let num_notif_subs =
            lock_notif_endpoints_list(notif_endpoints_handle.clone(), |notif_endpoints_list| {
                Ok(notif_endpoints_list.len() as u32)
            })?;

        let status_report = AuthdStatus {
            logged_in,
            num_auth_reqs,
            num_notif_subs,
        };

        let status_report_serialised = serde_json::to_string(&status_report)
            .unwrap_or_else(|_| "Failed to serialise output to json".to_string());

        println!("Status report sent: {:?}", status_report);

        Ok(status_report_serialised)
    }
}
