// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use color_eyre::{eyre::eyre, Result};
use sn_updater::{update_binary, UpdateType};

pub fn update_commander(no_confirm: bool) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let _ = update_binary(UpdateType::Safe, current_version, !no_confirm)
        .map_err(|e| eyre!(format!("Failed to update safe: {e}")))?;
    Ok(())
}
