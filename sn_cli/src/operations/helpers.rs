// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

const BASE_DOWNLOAD_URL: &str = "https://github.com/maidsafe/safe_network/releases/download";

pub fn download_and_install_github_release_asset(
    target_path: PathBuf,
    exec_file_name: &str,
    repo_name: &str,
    version: Option<String>,
) -> Result<String> {
    let target = get_target();
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("maidsafe")
        .repo_name(repo_name)
        .target(&target)
        .bin_name(exec_file_name)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()
        .wrap_err(format!(
            "Error fetching list of releases for maidsafe/{} repository",
            repo_name
        ))?;
    let release;
    if let Some(version) = version {
        release = updater
            .get_release_version(format!("safe_network-v{}", version).as_str())
            .wrap_err(format!(
                "The maidsafe/{} repository has no release at version {}",
                repo_name, version
            ))?;
    } else {
        release = updater
            .get_latest_release()
            .wrap_err("Failed to find a release available to install")?;
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
    let version = get_version_from_release_version(&release.version)?;
    println!("Found release: {} {}", release.name, version);
    let asset = release.asset_for(target).ok_or_else(|| {
        eyre!(
            "No asset found in latest release for the target platform {}",
            target
        )
    })?;
    let tmp_dir = std::env::temp_dir();
    let tmp_tarball_path = tmp_dir.join(&asset.name);
    let tmp_tarball = File::create(&tmp_tarball_path).wrap_err_with(|| {
        format!(
            "Error creating temp file ('{}') for downloading the release",
            tmp_tarball_path.display(),
        )
    })?;

    let download_url = get_download_url(&version);
    println!("Downloading {}...", download_url);
    self_update::Download::from_url(&download_url)
        .show_progress(true)
        .download_to(&tmp_tarball)
        .wrap_err_with(|| format!("Error downloading release asset '{}'", asset.download_url))?;

    if !target_path.exists() {
        println!("Creating '{}' folder", target_path.display());
        create_dir_all(target_path.clone())
            .wrap_err("Couldn't create target path to install binary")?;
    }

    println!(
        "Installing {} binary at {} ...",
        exec_file_name,
        target_path.display()
    );
    self_update::Extract::from_source(&tmp_tarball_path)
        .extract_file(target_path.as_path(), exec_file_name)
        .wrap_err_with(|| {
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
    let file = File::open(&file_path).wrap_err_with(|| {
        format!(
            "Error when preparing to set execution permissions to installed binary '{}'",
            file_path.display(),
        )
    })?;

    let mut perms = file
        .metadata()
        .wrap_err_with(|| {
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
    file.set_permissions(perms).wrap_err_with(|| {
        format!(
            "Failed to set execution permissions to installed binary '{}'",
            file_path.display(),
        )
    })?;

    Ok(())
}

/// Gets the version number from the full version number string.
///
/// The `release_version` input is in the form "safe_network-v0.49.1". This function will return
/// the "v0.49.1" part.
fn get_version_from_release_version(release_version: &str) -> Result<String> {
    let mut parts = release_version.split('-');
    let version = parts
        .next_back()
        .ok_or_else(|| {
            eyre!(format!(
                "Could not parse version number from {}",
                release_version
            ))
        })?
        .to_string();
    Ok(version)
}

fn get_download_url(version: &str) -> String {
    let version_sans_v = &version[1..];
    let target = get_target();
    let url = format!(
        "{}/safe_network-{}/sn_node-{}-{}.tar.gz",
        BASE_DOWNLOAD_URL, version, version_sans_v, target
    );
    url
}
