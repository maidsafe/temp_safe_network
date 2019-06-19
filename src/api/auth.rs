// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub use super::helpers::vec_to_hex;
use super::helpers::{decode_ipc_msg, encode_ipc_msg};
use super::safe::Safe;

use log::{debug, info};
use reqwest::get as httpget;

use safe_core::ipc::{AppExchangeInfo, AuthReq, IpcReq};
use safe_nd::AppPermissions;
use std::collections::HashMap;

// Default URL where to send a GET request to the authenticator webservice for authorising a SAFE app
const SAFE_AUTH_WEBSERVICE_BASE_URL: &str = "http://localhost:41805/authorise/";

impl Safe {
    // Generate an authorisation request string and send it to a SAFE Authenticator.
    // Ir returns the credentials necessary to connect to the network, encoded in a single string.
    pub fn auth_app(
        &mut self,
        app_id: &str,
        app_name: &str,
        app_vendor: &str,
    ) -> Result<String, String> {
        info!("Sending authorisation request to SAFE Authenticator...");

        let ipc_req = IpcReq::Auth(AuthReq {
            app: AppExchangeInfo {
                id: app_id.to_string(),
                scope: None,
                name: app_name.to_string(),
                vendor: app_vendor.to_string(),
            },
            app_container: false,
            app_permissions: AppPermissions {
                transfer_coins: true,
            },
            // TODO: allow list of required containers permissions to be passed in as param
            containers: HashMap::new(),
        });

        match encode_ipc_msg(ipc_req) {
            Ok(auth_req_str) => {
                debug!(
                    "Authorisation request generated successfully: {}",
                    auth_req_str
                );

                let authenticator_webservice_url =
                    SAFE_AUTH_WEBSERVICE_BASE_URL.to_string() + &auth_req_str;
                let mut res = httpget(&authenticator_webservice_url)
                    .map_err(|err| format!("Failed to send request to Authenticator: {}", err))?;
                let mut auth_res = String::new();
                // res.read_to_string(&mut auth_res).map_err(|err| {
                // 	format!(
                // 		"Failed read authorisation response received from Authenticator: {}",
                // 		err
                // 	)
                // })?;
                info!("SAFE authorisation response received!");

                // Check if the app has been authorised
                match decode_ipc_msg(&auth_res) {
                    Ok(_) => {
                        info!("Application was authorisaed");
                        Ok(auth_res)
                    }
                    Err(e) => {
                        info!("Application was not authorised");
                        Err(e)
                    }
                }
            }
            Err(e) => Err(format!(
                "Failed encoding the authorisation request: {:?}",
                e
            )),
        }
    }

    // Connect to the SAFE Network using the provided app id and auth credentials
    pub fn connect(&mut self, app_id: &str, auth_credentials: &str) -> Result<(), String> {
        self.safe_app.connect(app_id, auth_credentials)
    }
}
