// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

pub fn download_from_s3_and_install_bin(
    target_path: PathBuf,
    bucket: &str,
    asset_prefix: &str,
    exec_file_name: &str,
) -> Result<String, String> {
    let target = self_update::get_target();
    let available_releases = self_update::backends::s3::Update::configure()
        .bucket_name(bucket)
        .target(&target)
        .asset_prefix(asset_prefix)
        .region("eu-west-2")
        .bin_name("")
        .current_version("")
        .build()
        .map_err(|err| {
            format!(
                "Error when preparing to fetch the list of releases: {}",
                err
            )
        })?;

    download_and_install_bin(target_path, target, available_releases, exec_file_name)
}

pub fn download_from_github_and_install_bin(
    target_path: PathBuf,
    repo_name: &str,
    exec_file_name: &str,
) -> Result<String, String> {
    //let target = self_update::get_target();
    let target = "x86_64-unknown-linux-musl";
    let available_releases = self_update::backends::github::Update::configure()
        .repo_owner("maidsafe")
        .repo_name(repo_name)
        .target(&target)
        .bin_name("")
        .current_version("")
        .build()
        .map_err(|err| {
            format!(
                "Error when preparing to fetch the list of releases: {}",
                err
            )
        })?;

    download_and_install_bin(target_path, target, available_releases, exec_file_name)
}

fn download_and_install_bin(
    target_path: PathBuf,
    target: &str,
    available_releases: Box<dyn self_update::update::ReleaseUpdate>,
    exec_file_name: &str,
) -> Result<String, String> {
    let latest_release = available_releases
        .get_latest_release()
        .map_err(|err| format!("Failed to find a release available to install: {}", err))?;

    println!(
        "Latest release found: {} v{}",
        latest_release.name, latest_release.version
    );
    // get the corresponding asset from the release
    let asset = latest_release.asset_for(&target).ok_or_else(|| {
        format!(
            "No asset found in latest release for the target platform {}",
            target
        )
    })?;
    let tmp_dir = std::env::temp_dir();
    let tmp_tarball_path = tmp_dir.join(&asset.name);
    let tmp_tarball = File::create(&tmp_tarball_path).map_err(|err| {
        format!(
            "Error creating temp file ('{}') for downloading the release: {}",
            tmp_tarball_path.display(),
            err
        )
    })?;

    println!("Downloading {}...", asset.download_url);
    self_update::Download::from_url(&asset.download_url)
        .show_progress(true)
        .download_to(&tmp_tarball)
        .map_err(|err| {
            format!(
                "Error downloading release asset '{}': {}",
                asset.download_url, err
            )
        })?;

    if !target_path.exists() {
        println!("Creating '{}' folder", target_path.display());
        create_dir_all(target_path.clone())
            .map_err(|err| format!("Couldn't create target path to install binary: {}", err))?;
    }

    println!(
        "Installing safe-authd binary at {} ...",
        target_path.display()
    );
    self_update::Extract::from_source(&tmp_tarball_path)
        .extract_file(&target_path.as_path(), exec_file_name)
        .map_err(|err| {
            format!(
                "Error extracting binary from downloaded asset '{}': {}",
                tmp_tarball_path.display(),
                err
            )
        })?;

    set_exec_perms(target_path.join(exec_file_name))?;

    println!("Done!");
    Ok(target_path.display().to_string())
}

#[cfg(target_os = "windows")]
#[inline]
fn set_exec_perms(_file_path: PathBuf) -> Result<(), String> {
    // no need to set execution permissions on Windows
    Ok(())
}

#[cfg(not(target_os = "windows"))]
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
