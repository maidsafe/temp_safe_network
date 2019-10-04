// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::constants::{SAFE_AUTHD_ENDPOINT_HOST, SAFE_AUTHD_ENDPOINT_PORT};
use super::helpers::decode_ipc_msg;
use super::quic_client::quic_send;
use super::{Error, Result, Safe, SafeApp};
use log::{debug, info};
#[cfg(not(any(target_os = "android", target_os = "androideabi", target_os = "ios")))]
use reqwest::get as httpget;
use safe_core::ipc::{encode_msg, gen_req_id, AppExchangeInfo, AuthReq, IpcMsg, IpcReq};
use safe_nd::AppPermissions;
use std::collections::HashMap;

// Path of authenticator endpoint for authorising applications
const SAFE_AUTHD_ENDPOINT_AUTHORISE: &str = "authorise/";

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
    ) -> Result<String> {
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
                get_balance: true,
                perform_mutations: true,
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

        // Send he auth req to authd and obtain the response
        let auth_res = send_app_auth_req(&auth_req_str, port)?;

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
    pub fn connect(&mut self, app_id: &str, auth_credentials: Option<&str>) -> Result<()> {
        self.safe_app.connect(app_id, auth_credentials)
    }
}

// Sends an authorisation request string to the SAFE Authenticator daemon endpoint.
// It returns the credentials necessary to connect to the network, encoded in a single string.
fn send_app_auth_req(auth_req_str: &str, port: Option<u16>) -> ResultReturn<String> {
    let port_number = port.unwrap_or(SAFE_AUTHD_ENDPOINT_PORT);
    let authd_service_url = format!(
        "{}:{}/{}{}",
        SAFE_AUTHD_ENDPOINT_HOST, port_number, SAFE_AUTHD_ENDPOINT_AUTHORISE, auth_req_str
    );

    info!("Sending authorisation request to SAFE Authenticator...");
    let authd_response = quic_send(&authd_service_url, false, None, None, false)?;

    info!("SAFE authorisation response received!");
    Ok(authd_response)
}
