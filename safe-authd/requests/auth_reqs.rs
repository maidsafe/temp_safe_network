// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::shared::SharedAuthReqsHandle;
use log::info;
use safe_api::PendingAuthReqs;
use serde_json::{json, Value};

pub async fn process_req(
    params: Value,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<Value, String> {
    if Value::Null != params {
        Err(format!(
            "Unexpected param for 'auth-reqs' method: {:?}",
            params
        ))
    } else {
        info!("Obtaining list of pending authorisation requests...");
        let auth_reqs_list = auth_reqs_handle.lock().await;
        let pending_auth_reqs: PendingAuthReqs = auth_reqs_list
            .iter()
            .map(|(_req_id, pending_req)| pending_req.auth_req.clone())
            .collect();

        info!(
            "List of pending authorisation requests sent: {:?}",
            pending_auth_reqs
        );
        Ok(json!(pending_auth_reqs))
    }
}
