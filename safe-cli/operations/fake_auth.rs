// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::APP_ID;
use log::debug;
use safe_api::Safe;

pub fn authorise_cli(_safe: &mut Safe, _port: Option<u16>) -> Result<(), String> {
    debug!("Fake-auth is enabled so we don't try to read the credentials file or send authorisation request");
    Ok(())
}

pub fn clear_credentials() -> Result<(), String> {
    debug!("Fake-auth is enabled so we don't try to clear the credentials file");
    Ok(())
}

pub fn connect(safe: &mut Safe) -> Result<(), String> {
    debug!("Fake-auth is enabled so we don't try to read the credentials file");

    safe.connect(APP_ID, Some("fake-auth-credentials"))
        .map_err(|err| {
            format!(
                "Unexpected error when trying to connect with fake auth/network: {}",
                err
            )
        })
}
