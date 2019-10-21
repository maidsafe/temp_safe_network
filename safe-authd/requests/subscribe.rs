// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::shared::{lock_notif_endpoints_list, SharedNotifEndpointsHandle};

// Maximum number of allowed auth reqs notifs subscriptors
const MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS: usize = 3;

pub fn process_req(
    args: &[&str],
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<String, String> {
    if args.len() != 1 {
        Err("Incorrect number of arguments for 'subscribe' action".to_string())
    } else {
        println!("Subscribing to authorisation requests notifications...");
        let mut notif_endpoint = match urlencoding::decode(args[0]) {
            Ok(url) => url,
            Err(err) => {
                let msg = format!(
                    "Subscription rejected, the endpoint URL ('{}') is invalid: {:?}",
                    args[0], err
                );
                println!("{}", msg);
                return Err(msg);
            }
        };

        lock_notif_endpoints_list(notif_endpoints_handle, |notif_endpoints_list| {
            if notif_endpoints_list.len() >= MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS {
                let msg = format!("Subscription rejected. Maximum number of subscriptions ({}) has been already reached", MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS);
                println!("{}", msg);
                Err(msg)
            } else {
                if notif_endpoint.ends_with('/') {
                    notif_endpoint.pop();
                }
                notif_endpoints_list.insert(notif_endpoint.clone());

                let msg = format!(
                        "Subscription successful. Endpoint '{}' will receive authorisation requests notifications",
                        notif_endpoint
                    );
                println!("{}", msg);
                Ok(msg)
            }
        })
    }
}
