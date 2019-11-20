// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::shared::{lock_notif_endpoints_list, SharedNotifEndpointsHandle};
use serde_json::{json, Value};

pub fn process_req(
    args: Vec<&str>,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<Value, String> {
    if args.len() != 1 {
        Err("Incorrect number of arguments for 'unsubscribe' action".to_string())
    } else {
        println!("Unsubscribing from authorisation requests notifications...");
        let notif_endpoint = match urlencoding::decode(&args[0]) {
            Ok(url) => url,
            Err(err) => {
                let msg = format!(
                    "Unsubscription request rejected, the endpoint URL ('{}') is invalid: {:?}",
                    args[0], err
                );
                println!("{}", msg);
                return Err(msg);
            }
        };

        lock_notif_endpoints_list(notif_endpoints_handle, |notif_endpoints_list| {
            match notif_endpoints_list.remove(&notif_endpoint) {
                Some(_) => {
                    let msg = format!(
                    "Unsubscription successful. Endpoint '{}' will no longer receive authorisation requests notifications",
                    notif_endpoint
                    );
                    println!("{}", msg);
                    Ok(json!(msg))
                }
                None => {
                    let msg = format!(
                "Unsubscription request ignored, no such the endpoint URL ('{}') was found to be subscribed",
                notif_endpoint
                );
                    println!("{}", msg);
                    Err(msg)
                }
            }
        })
    }
}
