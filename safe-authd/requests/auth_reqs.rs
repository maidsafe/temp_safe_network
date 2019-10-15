// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::authd::SharedAuthReqsHandle;
use std::collections::BTreeSet;

pub fn process_req(
    args: &[&str],
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<String, String> {
    if !args.is_empty() {
        Err("Incorrect number of arguments for 'auth-reqs' action".to_string())
    } else {
        println!("Obtaining list of pending authorisation requests...");
        let auth_reqs_list = &mut *(auth_reqs_handle.lock().unwrap());
        let resp: BTreeSet<String> = auth_reqs_list
            .iter()
            .map(|(req_id, auth_req)| format!("Req ID: {} - App ID: {}", req_id, auth_req.app_id))
            .collect();

        println!("List of pending authorisation requests sent");
        Ok(format!("{:?}", resp))
    }
}
