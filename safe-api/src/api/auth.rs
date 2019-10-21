// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::decode_ipc_msg;
use super::{Error, ResultReturn, Safe, SafeApp};
use log::{debug, info};
#[cfg(not(any(target_os = "android", target_os = "androideabi", target_os = "ios")))]
use reqwest::get as httpget;
use safe_core::ipc::{encode_msg, gen_req_id, AppExchangeInfo, AuthReq, IpcMsg, IpcReq};
use safe_nd::AppPermissions;
use std::collections::HashMap;
use std::io::Read;

// Default host where to send a GET request to the authenticator webservice for authorising a SAFE app
const SAFE_AUTH_ENDPOINT_HOST: &str = "http://localhost";
// Default port number where to send a GET request for authorising the CLI app
const SAFE_AUTH_ENDPOINT_PORT: u16 = 41805;
// Path where the authenticator webservice endpoint
const SAFE_AUTH_ENDPOINT_PATH: &str = "authorise/";

#[allow(dead_code)]
impl Safe {
    // Generate an authorisation request string and send it to a SAFE Authenticator.
    // Ir returns the credentials necessary to connect to the network, encoded in a single string.
    #[cfg(not(any(target_os = "android", target_os = "androideabi", target_os = "ios")))]
    pub fn auth_app(
        &mut self,
        app_id: &str,
        app_name: &str,
        app_vendor: &str,
        port: Option<u16>,
    ) -> ResultReturn<String> {
        info!("Sending authorisation request to SAFE Authenticator...");

        let req = IpcReq::Auth(AuthReq {
            app: AppExchangeInfo {
                id: app_id.to_string(),
                scope: None,
                name: app_name.to_string(),
                vendor: app_vendor.to_string(),
            },
            app_container: false,
            app_permissions: AppPermissions {
                /*get_balance: true,
                perform_mutations: true,*/
                transfer_coins: true,
            },
            // TODO: allow list of required containers permissions to be passed in as param
            containers: HashMap::new(),
        });

        let req_id: u32 = gen_req_id();
        let auth_req_str = encode_msg(&IpcMsg::Req { req_id, req }).map_err(|err| {
            Error::AuthError(format!(
                "Failed encoding the authorisation request: {:?}",
                err
            ))
        })?;

        debug!(
            "Authorisation request generated successfully: {}",
            auth_req_str
        );

        let port_number = port.unwrap_or(SAFE_AUTH_ENDPOINT_PORT);
        let authenticator_webservice_url = format!(
            "{}:{}/{}{}",
            SAFE_AUTH_ENDPOINT_HOST, port_number, SAFE_AUTH_ENDPOINT_PATH, auth_req_str
        );
        let mut res = httpget(&authenticator_webservice_url).map_err(|err| {
            Error::AuthError(format!("Failed to send request to Authenticator: {}", err))
        })?;
        let mut auth_res = String::new();
        res.read_to_string(&mut auth_res).map_err(|err| {
            Error::AuthError(format!(
                "Failed read authorisation response received from Authenticator: {}",
                err
            ))
        })?;
        info!("SAFE authorisation response received!");

        // Check if the app has been authorised
        match decode_ipc_msg(&auth_res) {
            Ok(_) => {
                info!("Application was authorised");
                Ok(auth_res)
            }
            Err(e) => {
                info!("Application was not authorised");
                Err(Error::AuthError(format!(
                    "Application was not authorised: {:?}",
                    e
                )))
            }
        }
    }

    // Connect to the SAFE Network using the provided app id and auth credentials
    pub fn connect(&mut self, app_id: &str, auth_credentials: Option<&str>) -> ResultReturn<()> {
        self.safe_app.connect(app_id, auth_credentials)
    }
}
