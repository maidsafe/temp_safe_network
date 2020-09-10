// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use log::debug;
use std::path::PathBuf;
#[cfg(not(windows))]
use std::{fs::File, os::unix::fs::PermissionsExt};

pub fn update_commander() -> Result<(), Box<dyn (::std::error::Error)>> {
    let target = self_update::get_target();
    let releases = self_update::backends::s3::ReleaseList::configure()
        .bucket_name("sn-api")
        .with_target(&target)
        .asset_prefix("sn_authd")
        .region("eu-west-2")
        .build()?
        .fetch()?;

    if releases.is_empty() {
        println!("Current version is {}", cargo_crate_version!());
        println!("No new releases are available on S3 to perform an update");
    } else {
        debug!("Found releases: {:#?}\n", releases);
        let bin_name = if target.contains("pc-windows") {
            "sn_authd.exe"
        } else {
            "sn_authd"
        };

        let release_updater = self_update::backends::s3::Update::configure()
            .bucket_name("sn-api")
            .target(&target)
            .asset_prefix("sn_authd")
            .region("eu-west-2")
            .bin_name(&bin_name)
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .build()?;

        let status = release_updater.update()?;

        set_exec_perms(release_updater.bin_install_path())?;

        println!("Update status: `{}`!", status.version());
    }

    Ok(())
}

#[cfg(windows)]
#[inline]
fn set_exec_perms(_file_path: PathBuf) -> Result<(), String> {
    // no need to set execution permissions on Windows
    Ok(())
}

#[cfg(not(windows))]
#[inline]
fn set_exec_perms(file_path: PathBuf) -> Result<(), String> {
    println!(
        "Setting execution permissions to installed binary '{}'...",
        file_path.display()
    );
    let file = File::open(&file_path).map_err(|err| {
        format!(
            "Error when preparing to set execution permissions to installed binary '{}': {}",
            file_path.display(),
            err
        )
    })?;

    let mut perms = file
        .metadata()
        .map_err(|err| {
            format!(
                "Error when reading metadata from installed binary '{}': {}",
                file_path.display(),
                err
            )
        })?
        .permissions();

    // set execution permissions bits for owner, group and others
    perms.set_mode(perms.mode() | 0b0_001_001_001);
    file.set_permissions(perms).map_err(|err| {
        format!(
            "Failed to set execution permissions to installed binary '{}': {}",
            file_path.display(),
            err
        )
    })?;

    Ok(())
}
