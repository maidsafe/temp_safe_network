// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::debug;

pub fn update_commander() -> Result<(), Box<::std::error::Error>> {
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
