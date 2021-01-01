// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[cfg(feature = "self-update")]
use log::debug;
use std::error::Error;

#[cfg(feature = "self-update")]
const REPO_NAME: &str = "sn_api";

#[cfg(not(feature = "self-update"))]
pub fn update_commander() -> Result<(), Box<dyn Error>> {
    println!("Self updates are disabled.");
    Ok(())
}

#[cfg(feature = "self-update")]
pub fn update_commander() -> Result<(), Box<dyn Error>> {
    let target = self_update::get_target();
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("maidsafe")
        .repo_name(REPO_NAME)
        .with_target(&target)
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
            .target(&target)
            .bin_name(&bin_name)
            .show_download_progress(true)
            .current_version(env!("CARGO_PKG_VERSION"))
            .build()?
            .update()?;
        println!("Update status: `{}`!", status.version());
    }

    Ok(())
}
