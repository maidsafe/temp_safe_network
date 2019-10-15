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
    if args.len() != 2 {
        Err("Incorrect number of arguments for 'login' action".to_string())
    } else {
        println!("Logging in to SAFE account...");
        let secret = args[0];
        let password = args[1];

        let safe_authenticator = &mut *(safe_auth_handle.lock().unwrap());
        match safe_authenticator.log_in(secret, password) {
            Ok(_) => {
                let msg = "Logged in successfully!";
                println!("{}", msg);
                Ok(msg.to_string())
            }
            Err(err) => {
                let msg = format!("Error occurred when trying to log in: {}", err);
                println!("{}", msg);
                Err(err.to_string())
            }
        }
    }
}
