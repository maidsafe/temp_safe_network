// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use log::debug;

#[cfg(feature = "mock-network")]
pub fn update_commander() -> Result<(), Box<dyn (::std::error::Error)>> {
    debug!("The update command is not supported for the development build.");
    println!("The update command is not supported for the development build.");
    Ok(())
}

#[cfg(not(feature = "mock-network"))]
pub fn update_commander() -> Result<(), Box<dyn (::std::error::Error)>> {
    let target = self_update::get_target()?;
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("maidsafe")
        .repo_name("safe-cli")
        .with_target(&target)
        .build()?
        .fetch()?;
    if !releases.is_empty() {
        debug!("Found releases: {:#?}\n", releases);
        let bin_name = if target.contains("pc-windows") {
            "safe.exe"
        } else {
            "safe"
        };
        let status = self_update::backends::github::Update::configure()?
            .repo_owner("maidsafe")
            .repo_name("safe-cli")
            .target(&target)
            .bin_name(&bin_name)
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .build()?
            .update()?;
        println!("Update status: `{}`!", status.version());
    } else {
        println!("Current version is {}", cargo_crate_version!());
        println!("No releases are available on GitHub to perform an update");
    }
    Ok(())
}
