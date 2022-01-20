// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::error::Error;
#[cfg(feature = "self-update")]
use tracing::debug;

#[cfg(feature = "self-update")]
const REPO_NAME: &str = "sn_cli";

#[cfg(not(feature = "self-update"))]
pub fn update_commander(_no_confirm: bool) -> Result<(), Box<dyn Error>> {
    println!("Self updates are disabled.");
    Ok(())
}

#[cfg(feature = "self-update")]
pub fn update_commander(no_confirm: bool) -> Result<(), Box<dyn Error>> {
    let target = self_update::get_target();
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("maidsafe")
        .repo_name(REPO_NAME)
        .with_target(target)
        .build()?
        .fetch()?;

    if releases.is_empty() {
        println!("Current version is {}", env!("CARGO_PKG_VERSION"));
        println!("No releases are available on GitHub to perform an update");
    } else {
        debug!("Found releases: {:#?}\n", releases);
        let bin_name = if target.contains("pc-windows") {
            "safe.exe"
        } else {
            "safe"
        };
        let status = self_update::backends::github::Update::configure()
            .repo_owner("maidsafe")
            .repo_name(REPO_NAME)
            .target(target)
            .bin_name(bin_name)
            .no_confirm(no_confirm)
            .show_download_progress(true)
            .current_version(env!("CARGO_PKG_VERSION"))
            .build()?
            .update()?;
        println!("Update status: `{}`!", status.version());
    }

    Ok(())
}
