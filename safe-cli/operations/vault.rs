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
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use safe_nlt::run_with;
use std::{
    collections::HashMap,
    fs::create_dir_all,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

#[cfg(not(target_os = "windows"))]
const SAFE_VAULT_EXECUTABLE: &str = "safe_vault";

#[cfg(target_os = "windows")]
const SAFE_VAULT_EXECUTABLE: &str = "safe_vault.exe";

fn run_safe_cmd(
    args: &[&str],
    envs: Option<HashMap<String, String>>,
    ignore_errors: bool,
    verbosity: u8,
) -> Result<(), String> {
    let env: HashMap<String, String> = envs.unwrap_or_else(HashMap::default);

    let msg = format!("Running 'safe' with args {:?} ...", args);
    if verbosity > 1 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let _child = Command::new("safe")
        .args(args)
        .envs(&env)
        .stdout(Stdio::inherit())
        .stderr(if ignore_errors {
            Stdio::null()
        } else {
            Stdio::inherit()
        })
        .spawn()
        .map_err(|err| format!("Failed to run 'safe' with args '{:?}': {}", args, err))?;

    Ok(())
}

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
    interval: &str,
    ip: Option<String>,
    test: bool,
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
        "--interval",
        &interval,
    ];

    let interval_as_int = &interval.parse::<u64>().unwrap();

    let mut verbosity_arg = String::from("-");
    if verbosity > 0 {
        let v = "y".repeat(verbosity as usize);
        println!("V: {}", v);
        verbosity_arg.push_str(&v);
        nlt_args.push(&verbosity_arg);
    }

    if let Some(ref launch_ip) = ip {
        nlt_args.push("--ip");
        nlt_args.push(launch_ip);
    };

    debug!("Running network launch tool with args: {:?}", nlt_args);

    // We can now call the tool with the args
    println!("Launching local SAFE network...");
    run_with(Some(&nlt_args))?;

    let interval_duration = Duration::from_secs(interval_as_int * 15);
    thread::sleep(interval_duration);

    let ignore_errors = true;
    let report_errors = false;

    if test {
        println!("Setting up authenticator against local SAFE network...");

        if cfg!(windows) {
            // On Windows authd must be installed as a service
            let auth_install_win_args = vec!["auth", "install"];
            run_safe_cmd(&auth_install_win_args, None, report_errors, verbosity)?;
        }

        //stop
        let stop_auth_args = vec!["auth", "stop"];
        run_safe_cmd(&stop_auth_args, None, ignore_errors, verbosity)?;

        let between_command_interval = Duration::from_secs(interval_as_int * 5);
        thread::sleep(between_command_interval);
        //stop
        let start_auth_args = vec!["auth", "start"];
        run_safe_cmd(&start_auth_args, None, report_errors, verbosity)?;

        thread::sleep(between_command_interval);

        // // Q: can we assume network is correct here? Or do we need to do networks switch?
        let pass: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        let phrase: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

        // setup env for create acc / login
        let mut env = HashMap::new();
        env.insert("SAFE_AUTH_PASSPHRASE".to_string(), pass);
        env.insert("SAFE_AUTH_PASSWORD".to_string(), phrase);

        // create-acc
        let create_account = vec!["auth", "create-acc", "--test-coins"];

        run_safe_cmd(&create_account, Some(env.clone()), report_errors, verbosity)?;
        thread::sleep(between_command_interval);

        // login
        let login = vec!["auth", "login", "--self-auth"];
        run_safe_cmd(&login, Some(env), report_errors, verbosity)?;
    }

    Ok(())
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

pub fn vault_update(vault_path: Option<PathBuf>) -> Result<(), String> {
    let vault_path = get_vault_bin_path(vault_path)?;

    let arg_vault_path = vault_path.join(SAFE_VAULT_EXECUTABLE).display().to_string();
    debug!("Updating vault at {}", arg_vault_path);

    let child = Command::new(&arg_vault_path)
        .args(vec!["--update-only"])
        .spawn()
        .map_err(|err| format!("Failed to update vault at '{}': {}", arg_vault_path, err))?;

    let output = child
        .wait_with_output()
        .map_err(|err| format!("Failed to update vault at '{}': {}", arg_vault_path, err))?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| format!("Failed to output stdout: {}", err))?;
        Ok(())
    } else {
        Err(format!(
            "Failed when invoking vault executable from '{}':\n{}",
            arg_vault_path,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
