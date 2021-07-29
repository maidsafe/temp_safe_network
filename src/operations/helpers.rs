// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#![cfg(feature = "self-update")]

use anyhow::{anyhow, Context, Result};
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

pub fn download_and_install_github_release_asset(
    target_path: PathBuf,
    exec_file_name: &str,
    repo_name: &str,
    version: Option<String>,
) -> Result<String> {
    let target = get_target();
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("maidsafe")
        .repo_name(&repo_name)
        .target(&target)
        .bin_name(&exec_file_name)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()
        .context(format!(
            "Error fetching list of releases for maidsafe/{} repository",
            repo_name
        ))?;
    let release;
    if let Some(version) = version {
        release = updater
            .get_release_version(format!("v{}", version).as_str())
            .context(format!(
                "The maidsafe/{} repository has no release at version {}",
                repo_name, version
            ))?;
    } else {
        release = updater
            .get_latest_release()
            .context("Failed to find a release available to install")?;
    }
    download_and_install_bin(target_path, &target, release, exec_file_name)
}

// Private helpers

fn get_target() -> String {
    let target = self_update::get_target();
    if target.contains("linux") {
        // For now, all of our Linux builds are using musl, so we can make this
        // assumption. We would need to update this code if we changed that.
        target.replace("gnu", "musl")
    } else {
        target.to_string()
    }
}

fn download_and_install_bin(
    target_path: PathBuf,
    target: &str,
    release: self_update::update::Release,
    exec_file_name: &str,
) -> Result<String> {
    println!(
        "Latest release found: {} v{}",
        release.name, release.version
    );
    // get the corresponding asset from the release
    let asset = release.asset_for(&target).ok_or_else(|| {
        anyhow!(
            "No asset found in latest release for the target platform {}",
            target
        )
    })?;
    let tmp_dir = std::env::temp_dir();
    let tmp_tarball_path = tmp_dir.join(&asset.name);
    let tmp_tarball = File::create(&tmp_tarball_path).with_context(|| {
        format!(
            "Error creating temp file ('{}') for downloading the release",
            tmp_tarball_path.display(),
        )
    })?;

    println!("Downloading {}...", asset.download_url);
    self_update::Download::from_url(&asset.download_url)
        .show_progress(true)
        .download_to(&tmp_tarball)
        .with_context(|| format!("Error downloading release asset '{}'", asset.download_url))?;

    if !target_path.exists() {
        println!("Creating '{}' folder", target_path.display());
        create_dir_all(target_path.clone())
            .context("Couldn't create target path to install binary")?;
    }

    println!(
        "Installing {} binary at {} ...",
        exec_file_name,
        target_path.display()
    );
    self_update::Extract::from_source(&tmp_tarball_path)
        .extract_file(&target_path.as_path(), exec_file_name)
        .with_context(|| {
            format!(
                "Error extracting binary from downloaded asset '{}'",
                tmp_tarball_path.display(),
            )
        })?;

    set_exec_perms(target_path.join(exec_file_name))?;

    println!("Done!");
    Ok(target_path.display().to_string())
}

#[cfg(target_os = "windows")]
#[allow(clippy::unnecessary_wraps)]
#[inline]
fn set_exec_perms(_file_path: PathBuf) -> Result<()> {
    // no need to set execution permissions on Windows
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[inline]
fn set_exec_perms(file_path: PathBuf) -> Result<()> {
    println!(
        "Setting execution permissions to installed binary '{}'...",
        file_path.display()
    );
    let file = File::open(&file_path).with_context(|| {
        format!(
            "Error when preparing to set execution permissions to installed binary '{}'",
            file_path.display(),
        )
    })?;

    let mut perms = file
        .metadata()
        .with_context(|| {
            format!(
                "Error when reading metadata from installed binary '{}'",
                file_path.display(),
            )
        })?
        .permissions();

    // set execution permissions bits for owner, group and others
    // Allow unusual bit grouping to clearly separate 3 bits parts
    #[allow(clippy::unusual_byte_groupings)]
    perms.set_mode(perms.mode() | 0b0_001_001_001);
    file.set_permissions(perms).with_context(|| {
        format!(
            "Failed to set execution permissions to installed binary '{}'",
            file_path.display(),
        )
    })?;

    Ok(())
}
