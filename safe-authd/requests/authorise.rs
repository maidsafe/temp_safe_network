// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::authd::{AuthReq, SharedAuthReqsHandle, SharedSafeAuthenticatorHandle};
use safe_api::SafeAuthReq;
use tokio::sync::mpsc;

pub fn process_req(
    args: &[&str],
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<(mpsc::Receiver<bool>, u32, String), String> {
    if args.len() != 1 {
        Err("Incorrect number of arguments for 'authorise' action".to_string())
    } else {
        // TODO: automatically reject if there are too many pending auth reqs
        println!("Authorising application...");
        let auth_req_str = args[0];

        let safe_authenticator = &mut *(safe_auth_handle.lock().unwrap());
        match safe_authenticator.decode_req(auth_req_str) {
            Ok((req_id, safe_auth_req)) => {
                println!("Sending request to user to allow/deny request...");

                let rx = enqueue_auth_req(req_id, safe_auth_req, &auth_reqs_handle);
                Ok((rx, req_id, auth_req_str.to_string()))
            }
            Err(err) => {
                println!("{}", err);
                Err(err.to_string())
            }
        }
    }
}

fn enqueue_auth_req(
    req_id: u32,
    req: SafeAuthReq,
    auth_reqs_handle: &SharedAuthReqsHandle,
) -> mpsc::Receiver<bool> {
    let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel(32);
    match req {
        SafeAuthReq::Auth(app_auth_req) => {
            println!("The following application authorisation request was received:");
            println!("{:?}", app_auth_req);

            // Let's add it to the list of pending authorisation requests
            let auth_req = AuthReq {
                app_id: app_auth_req.app.id,
                tx,
            };
            let auth_reqs_list = &mut *(auth_reqs_handle.lock().unwrap());
            auth_reqs_list.insert(req_id, auth_req);
        }
        SafeAuthReq::Containers(cont_req) => {
            println!("The following authorisation request for containers was received:");
            println!("{:?}", cont_req);
        }
        SafeAuthReq::ShareMData(share_mdata_req) => {
            println!("The following authorisation request to share a MutableData was received:");
            println!("{:?}", share_mdata_req);
        }
        SafeAuthReq::Unregistered(_) => {
            // we simply allow unregistered authorisation requests
        }
    }
    rx
}
