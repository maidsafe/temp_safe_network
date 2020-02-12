// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::helpers::download_from_github_and_install_bin;
use directories::BaseDirs;
use std::path::PathBuf;

const ENV_VAR_SAFE_VAULT_PATH: &str = "SAFE_VAULT_PATH";

#[cfg(not(target_os = "windows"))]
const SAFE_VAULT_EXECUTABLE: &str = "safe_vault";

#[cfg(target_os = "windows")]
const SAFE_VAULT_EXECUTABLE: &str = "safe_vault.exe";

pub fn vault_install(vault_path: Option<String>) -> Result<(), String> {
    let target_path = get_vault_bin_path(vault_path)?;
    let _ = download_from_github_and_install_bin(target_path, "safe_vault", SAFE_VAULT_EXECUTABLE)?;
    Ok(())
}

#[inline]
fn get_vault_bin_path(vault_path: Option<String>) -> Result<PathBuf, String> {
    match vault_path {
        Some(p) => Ok(PathBuf::from(p)),
        None => {
            // if SAFE_VAULT_PATH is set it then overrides default
            if let Ok(vault_path) = std::env::var(ENV_VAR_SAFE_VAULT_PATH) {
                Ok(PathBuf::from(vault_path))
            } else {
                let base_dirs = BaseDirs::new()
                    .ok_or_else(|| "Failed to obtain user's home path".to_string())?;

                let mut path = PathBuf::from(base_dirs.home_dir());
                path.push(".safe");
                path.push("vault");
                Ok(path)
            }
        }
    }
}
