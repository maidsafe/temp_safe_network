// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.
pub mod errors;

use crate::errors::{Error, Result};
use self_update::version::bump_is_greater;
use std::path::PathBuf;

const ORG_NAME: &str = "maidsafe";
const REPO_NAME: &str = "safe_network";

pub enum UpdateType {
    Safe,
    Node,
}

/// Updates the binary to the latest available version.
///
/// This function queries the target platform then uses the self_update library to check for a
/// newer release on GitHub. If a newer release is found, the existing binary is replaced with the
/// new one.
///
/// # Arguments
///
/// * `update_type` - The type of binary to update (Safe or Node).
/// * `current_version` - The current version of the binary.
/// * `confirm_update` - A flag indicating if the update should be confirmed by the user.
///
/// # Errors
///
/// An error is returned if:
///
/// * Getting the target platform or binary name fails.
/// * Configuring the self_update library fails.
/// * Getting the latest release from GitHub fails.
/// * Determining the latest version from the latest release fails.
/// * Downloading or installing the new binary fails.
///
/// # Examples
///
/// ```
/// use sn_updater::{UpdateType, update_binary};
///
/// let current_version = env!("CARGO_PKG_VERSION");
/// update_binary(UpdateType::Safe, current_version, false);
/// ```
pub fn update_binary(
    update_type: UpdateType,
    current_version: &str,
    confirm_update: bool,
) -> Result<()> {
    let target_platform = get_target_platform();
    let bin_name = get_bin_name(&update_type, &target_platform);
    println!("Current version is: {current_version}");
    let latest_release = self_update::backends::github::Update::configure()
        .repo_owner(ORG_NAME)
        .repo_name(REPO_NAME)
        .target(&target_platform)
        .bin_name(&bin_name)
        .current_version(current_version)
        .build()?
        .get_latest_release()?;
    let latest_version = get_version_from_release_version(&update_type, &latest_release.version)?;
    if bump_is_greater(current_version, &latest_version)? {
        println!("Newer version is available: {latest_version}");
        println!("The existing binary will be replaced.");
        if confirm_update {
            println!("Proceed with the update? [y/n]");
            if !proceed()? {
                return Ok(());
            }
        }
        let url = get_download_url(&update_type, &latest_version, &target_platform)?;
        download_and_install_bin(&url, std::env::current_exe()?)?;
    } else {
        println!("Newer version is not available at this time.");
        println!("Everything is up to date.");
    }
    Ok(())
}

fn get_download_url(
    update_type: &UpdateType,
    version: &str,
    target_platform: &str,
) -> Result<String> {
    match update_type {
        UpdateType::Safe => Ok(format!(
            "https://sn-cli.s3.eu-west-2.amazonaws.com/sn_cli-{version}-{target_platform}.tar.gz"
        )),
        UpdateType::Node => Ok(format!(
            "https://sn-node.s3.eu-west-2.amazonaws.com/sn_node-{version}-{target_platform}.tar.gz"
        )),
    }
}

fn get_bin_name(update_type: &UpdateType, target_platform: &str) -> String {
    match update_type {
        UpdateType::Safe => {
            if target_platform.contains("pc-windows") {
                "safe.exe".to_string()
            } else {
                "safe".to_string()
            }
        }
        UpdateType::Node => {
            if target_platform.contains("pc-windows") {
                "sn_node.exe".to_string()
            } else {
                "sn_node".to_string()
            }
        }
    }
}

fn get_target_platform() -> String {
    let target = self_update::get_target();
    if target.contains("linux") {
        // For now, all of our Linux builds are using musl, so we can make this
        // assumption. We would need to update this code if we changed that.
        target.replace("gnu", "musl")
    } else {
        target.to_string()
    }
}

/// Gets the version number from the full version number string.
///
/// The `release_version` input is in the form "0.17.1-0.15.3-0.2.1-0.78.2-0.73.3-0.76.1-0.69.0",
/// which is the `sn_interface`, `sn_fault_detection`, `sn_comms`, `sn_client`, `sn_node`,
/// `sn_api`, `sn_cli` respectively.
///
/// Returns either the `sn_node` or `sn_cli` part.
fn get_version_from_release_version(source: &UpdateType, release_version: &str) -> Result<String> {
    let mut parts = release_version.split('-');
    let version = match source {
        UpdateType::Safe => parts
            .last()
            .ok_or_else(|| Error::InvalidReleaseVersionFormat(release_version.to_string()))?
            .to_string(),
        UpdateType::Node => {
            parts.next();
            parts.next();
            parts.next();
            parts.next();
            parts
                .next()
                .ok_or_else(|| Error::InvalidReleaseVersionFormat(release_version.to_string()))?
                .to_string()
        }
    };
    Ok(version)
}

fn proceed() -> Result<bool> {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    let s = s.trim().to_lowercase();
    if !s.is_empty() && s != "y" {
        return Ok(false);
    }
    Ok(true)
}

fn download_and_install_bin(url: &str, target_file_path: PathBuf) -> Result<()> {
    println!("Downloading release from: {url}");
    let url = url::Url::parse(url)?;
    let archive_file_name = match url.path_segments() {
        Some(mut segments) => match segments.next_back() {
            Some(file_name) => file_name,
            None => {
                return Err(Error::InvalidDownloadUrl("No filename in URL".to_string()));
            }
        },
        None => {
            return Err(Error::InvalidDownloadUrl("No path in URL".to_string()));
        }
    };

    let tmp_dir = std::env::temp_dir();
    let tmp_tarball_path = tmp_dir.join(archive_file_name);
    let tmp_tarball = std::fs::File::create(&tmp_tarball_path)?;

    self_update::Download::from_url(url.as_ref())
        .show_progress(true)
        .download_to(&tmp_tarball)?;

    let target_dir_path = target_file_path.parent().ok_or_else(|| {
        Error::InvalidTargetPath("Could not obtain parent from target path".to_string())
    })?;
    if !target_dir_path.exists() {
        println!("Creating parent directory at {}", target_dir_path.display());
        std::fs::create_dir_all(target_dir_path)?;
    }

    let bin_name = target_file_path
        .file_name()
        .ok_or_else(|| {
            Error::InvalidTargetPath("Could not obtain file name from target file path".to_string())
        })?
        .to_str()
        .ok_or_else(|| Error::InvalidTargetPath("Could not obtain str from OsStr".to_string()))?;
    println!(
        "Extracting {} binary to {}...",
        bin_name,
        target_file_path.display()
    );
    self_update::Extract::from_source(&tmp_tarball_path).extract_file(target_dir_path, bin_name)?;
    set_exec_perms(target_file_path)?;
    println!("Done!");
    Ok(())
}

#[cfg(target_os = "windows")]
#[allow(clippy::unnecessary_wraps)]
#[inline]
fn set_exec_perms(_file_path: PathBuf) -> Result<()> {
    // no need to set execution permissions on Windows
    Ok(())
}

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
#[cfg(not(target_os = "windows"))]
#[inline]
fn set_exec_perms(bin_path: PathBuf) -> Result<()> {
    let file = std::fs::File::open(bin_path)?;
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms)?;
    Ok(())
}
