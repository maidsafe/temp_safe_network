// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::helpers::download_from_s3_and_install_bin;
use directories::BaseDirs;
use log::debug;
use safe_nlt::run_with;
use std::{fs::create_dir_all, path::PathBuf, process::Command};

#[cfg(not(target_os = "windows"))]
const SAFE_VAULT_EXECUTABLE: &str = "safe_vault";

#[cfg(target_os = "windows")]
const SAFE_VAULT_EXECUTABLE: &str = "safe_vault.exe";

pub fn vault_install(vault_path: Option<PathBuf>) -> Result<(), String> {
    let target_path = get_vault_bin_path(vault_path)?;
    let _ = download_from_s3_and_install_bin(
        target_path,
        "safe-vault",
        "safe_vault",
        SAFE_VAULT_EXECUTABLE,
        if cfg!(target_os = "linux") {
            Some("x86_64-unknown-linux-musl")
        } else {
            None
        },
    )?;
    Ok(())
}

pub fn vault_run(
    vault_path: Option<PathBuf>,
    vaults_dir: &str,
    verbosity: u8,
) -> Result<(), String> {
    let vault_path = get_vault_bin_path(vault_path)?;

    let arg_vault_path = vault_path.join(SAFE_VAULT_EXECUTABLE).display().to_string();
    debug!("Running vault from {}", arg_vault_path);

    let vaults_dir = vault_path.join(vaults_dir);
    if !vaults_dir.exists() {
        println!("Creating '{}' folder", vaults_dir.display());
        create_dir_all(vaults_dir.clone()).map_err(|err| {
            format!(
                "Couldn't create target path to store vaults' generated data: {}",
                err
            )
        })?;
    }
    let arg_vaults_dir = vaults_dir.display().to_string();
    println!("Storing vaults' generated data at {}", arg_vaults_dir);

    // Let's create an args array to pass to the network launcher tool
    let mut nlt_args = vec![
        "safe-nlt",
        "-v",
        "--vault-path",
        &arg_vault_path,
        "--vaults-dir",
        &arg_vaults_dir,
    ];

    let mut verbosity_arg = String::from("-");
    if verbosity > 0 {
        let v = "y".repeat(verbosity as usize);
        println!("V: {}", v);
        verbosity_arg.push_str(&v);
        nlt_args.push(&verbosity_arg);
    }

    debug!("Running network launch tool with args: {:?}", nlt_args);

    // We can now call the tool with the args
    println!("Launching local SAFE network...");
    run_with(Some(&nlt_args))
}

pub fn vault_shutdown(vault_path: Option<PathBuf>) -> Result<(), String> {
    let vault_exec_name = match vault_path {
        Some(ref path) => {
            let filepath = path.as_path();
            if filepath.is_file() {
                match filepath.file_name() {
                    Some(filename) => match filename.to_str() {
                        Some(name) => Ok(name),
                        None => Err(format!("Vault path provided ({}) contains invalid unicode chars", filepath.display())),
                    }
                    None => Err(format!("Vault path provided ({}) is invalid as it doens't include the executable filename", filepath.display())),
                }
            } else {
                Err(format!("Vault path provided ({}) is invalid as it doens't include the executable filename", filepath.display()))
            }
        }
        None => Ok(SAFE_VAULT_EXECUTABLE),
    }?;

    debug!(
        "Killing all running vaults launched with {}...",
        vault_exec_name
    );
    kill_vaults(vault_exec_name)
}

fn get_vault_bin_path(vault_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match vault_path {
        Some(p) => Ok(p),
        None => {
            let base_dirs =
                BaseDirs::new().ok_or_else(|| "Failed to obtain user's home path".to_string())?;

            let mut path = PathBuf::from(base_dirs.home_dir());
            path.push(".safe");
            path.push("vault");
            Ok(path)
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn kill_vaults(exec_name: &str) -> Result<(), String> {
    let output = Command::new("killall")
        .arg(exec_name)
        .output()
        .map_err(|err| {
            format!(
                "Error when atempting to stop vaults ({}) processes: {}",
                exec_name, err
            )
        })?;

    if output.status.success() {
        println!(
            "Success, all processes instances of {} were stopped!",
            exec_name
        );
        Ok(())
    } else {
        Err(format!(
            "Failed to stop vaults ({}) processes: {}",
            exec_name,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(target_os = "windows")]
fn kill_vaults(exec_name: &str) -> Result<(), String> {
    let output = Command::new("taskkill")
        .args(&["/F", "/IM", exec_name])
        .output()
        .map_err(|err| {
            format!(
                "Error when atempting to stop vaults ({}) processes: {}",
                exec_name, err
            )
        })?;

    if output.status.success() {
        println!(
            "Success, all processes instances of {} were stopped!",
            exec_name
        );
        Ok(())
    } else {
        Err(format!(
            "Failed to stop vaults ({}) processes: {}",
            exec_name,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
