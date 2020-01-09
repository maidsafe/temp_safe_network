// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::shared::{lock_notif_endpoints_list, SharedNotifEndpointsHandle};
use serde_json::{json, Value};

pub fn process_req(
    params: Value,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<Value, String> {
    if let Value::String(notif_endpoint) = params {
        println!("Unsubscribing from authorisation requests notifications...");
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
    } else {
        Err(format!(
            "Incorrect params for 'unsubscribe' method: {:?}",
            params
        ))
    }
}
