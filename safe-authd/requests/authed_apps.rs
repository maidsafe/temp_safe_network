// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::authd::SharedSafeAuthenticatorHandle;

pub fn process_req(
    args: &[&str],
    safe_auth_handle: SharedSafeAuthenticatorHandle,
) -> Result<String, String> {
    if !args.is_empty() {
        Err("Incorrect number of arguments for 'authed-apps' action".to_string())
    } else {
        println!("Obtaining list of authorised applications...");
        let safe_authenticator = &mut *(safe_auth_handle.lock().unwrap());
        match safe_authenticator.authed_apps() {
            Ok(resp) => {
                println!("List of authorised apps sent");
                Ok(format!("{:?}", resp))
            }
            Err(err) => {
                println!("Failed to get list of authorised apps: {}", err);
                Err(err.to_string())
            }
        }
    }
}
