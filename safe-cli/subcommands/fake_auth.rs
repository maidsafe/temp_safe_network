// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::debug;
use safe_api::Safe;
use structopt::StructOpt;
use crate::APP_ID;

#[derive(StructOpt, Debug)]
pub enum AuthSubCommands {
    #[structopt(name = "clear")]
    /// Clear authorisation credentials from local file
    Clear {},
}

pub fn auth_commander(
    cmd: Option<AuthSubCommands>,
    _endpoint: Option<String>,
    _safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(AuthSubCommands::Clear {}) => {
            debug!("Fake-auth is enabled so we don't try to clear the credentials file");
            Ok(())
        }
        _other => {
            debug!("Fake-auth is enabled so we don't try to read the credentials file or send auth request");
            Ok(())
        }
    }
}

pub fn auth_connect(safe: &mut Safe) -> Result<(), String> {
    debug!("Fake-auth is enabled so we don't try to read the credentials file");
    safe.connect(APP_ID, Some("fake-auth-credentials")).map_err(|err| {
        format!(
            "You need to authorise the safe CLI first with 'auth' command: {}",
            err
        )
    })
}
