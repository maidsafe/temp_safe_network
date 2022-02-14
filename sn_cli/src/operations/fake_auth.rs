// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::APP_ID;
// use tracing::debug;
use sn_api::Safe;
use color_eyre::Result;

pub async fn authorise_cli(
    _safe: Safe,
    _endpoint: Option<String>,
    _is_self_authing: bool,
) -> Result<()> {
   // debug!("Fake-auth is enabled so we don't try to read the credentials file or send authorisation request");
    Ok(())
}

pub fn clear_credentials() -> Result<()> {
   // debug!("Fake-auth is enabled so we don't try to clear the credentials file");
    Ok(())
}

pub async fn connect(mut safe: Safe) -> Result<()> {
   // debug!("Fake-auth is enabled so we don't try to read the credentials file");

    safe.connect(APP_ID, Some("fake-auth-credentials"))
        .await
        .context("Unexpected error when trying to connect with fake auth/network")
}
