// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::authd::SharedAuthReqsHandle;
use safe_api::PendingAuthReqs;

pub fn process_req(
    args: &[&str],
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<String, String> {
    if !args.is_empty() {
        Err("Incorrect number of arguments for 'auth-reqs' action".to_string())
    } else {
        println!("Obtaining list of pending authorisation requests...");
        let auth_reqs_list = &mut *(auth_reqs_handle.lock().unwrap());
        let pending_auth_reqs: PendingAuthReqs = auth_reqs_list
            .iter()
            .map(|(_req_id, pending_req)| pending_req.auth_req.clone())
            .collect();

        println!(
            "List of pending authorisation requests sent: {:?}",
            pending_auth_reqs
        );
        let auth_reqs_serialised = serde_json::to_string(&pending_auth_reqs)
            .unwrap_or_else(|_| "Failed to serialise output to json".to_string());

        Ok(auth_reqs_serialised)
    }
}
