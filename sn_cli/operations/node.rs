// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::helpers::download_from_s3_and_install_bin;
use log::debug;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sn_launch_tool::{join_with, run_with};
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
const SN_NODE_EXECUTABLE: &str = "sn_node";

#[cfg(target_os = "windows")]
const SN_NODE_EXECUTABLE: &str = "sn_node.exe";

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

pub fn node_install(node_path: Option<PathBuf>) -> Result<(), String> {
    let target_path = get_node_bin_path(node_path)?;
    let _ = download_from_s3_and_install_bin(
        target_path,
        "sn-node",
        "sn_node",
        SN_NODE_EXECUTABLE,
        if cfg!(target_os = "linux") {
            Some("x86_64-unknown-linux-musl")
        } else {
            None
        },
    )?;
    Ok(())
}

pub fn node_run(
    node_path: Option<PathBuf>,
    nodes_dir: &str,
    verbosity: u8,
    interval: &str,
    ip: Option<String>,
    test: bool,
) -> Result<(), String> {
    let node_path = get_node_bin_path(node_path)?;

    let arg_node_path = node_path.join(SN_NODE_EXECUTABLE).display().to_string();
    debug!("Running node from {}", arg_node_path);

    let nodes_dir = node_path.join(nodes_dir);
    if !nodes_dir.exists() {
        println!("Creating '{}' folder", nodes_dir.display());
        create_dir_all(nodes_dir.clone()).map_err(|err| {
            format!(
                "Couldn't create target path to store nodes' generated data: {}",
                err
            )
        })?;
    }
    let arg_nodes_dir = nodes_dir.display().to_string();
    println!("Storing nodes' generated data at {}", arg_nodes_dir);

    // Let's create an args array to pass to the network launcher tool
    let mut sn_launch_tool_args = vec![
        "sn_launch_tool",
        "-v",
        "--node-path",
        &arg_node_path,
        "--nodes-dir",
        &arg_nodes_dir,
        "--interval",
        &interval,
        "--local",
    ];

    let interval_as_int = &interval.parse::<u64>().unwrap();

    let mut verbosity_arg = String::from("-");
    if verbosity > 0 {
        let v = "y".repeat(verbosity as usize);
        println!("V: {}", v);
        verbosity_arg.push_str(&v);
        sn_launch_tool_args.push(&verbosity_arg);
    }

    if let Some(ref launch_ip) = ip {
        sn_launch_tool_args.push("--ip");
        sn_launch_tool_args.push(launch_ip);
    };

    debug!(
        "Running network launch tool with args: {:?}",
        sn_launch_tool_args
    );

    // We can now call the tool with the args
    println!("Launching local Safe network...");
    run_with(Some(&sn_launch_tool_args))?;

    let interval_duration = Duration::from_secs(interval_as_int * 15);
    thread::sleep(interval_duration);

    let ignore_errors = true;
    let report_errors = false;

    if test {
        println!("Setting up authenticator against local Safe network...");

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

pub fn node_join(
    node_path: Option<PathBuf>,
    node_data_dir: &str,
    verbosity: u8,
    contacts: &str,
) -> Result<(), String> {
    let node_path = get_node_bin_path(node_path)?;

    let arg_node_path = node_path.join(SN_NODE_EXECUTABLE).display().to_string();
    debug!("Running node from {}", arg_node_path);

    let node_data_dir = node_path.join(node_data_dir);
    if !node_data_dir.exists() {
        println!("Creating '{}' folder", node_data_dir.display());
        create_dir_all(node_data_dir.clone()).map_err(|err| {
            format!(
                "Couldn't create target path to store nodes' generated data: {}",
                err
            )
        })?;
    }
    let arg_nodes_dir = node_data_dir.display().to_string();
    println!("Storing nodes' generated data at {}", arg_nodes_dir);

    // Let's create an args array to pass to the network launcher tool
    let mut sn_launch_tool_args = vec![
        "sn_launch_tool-join",
        "-v",
        "--node-path",
        &arg_node_path,
        "--nodes-dir",
        &arg_nodes_dir,
    ];

    let mut verbosity_arg = String::from("-");
    if verbosity > 0 {
        let v = "y".repeat(verbosity as usize);
        println!("V: {}", v);
        verbosity_arg.push_str(&v);
        sn_launch_tool_args.push(&verbosity_arg);
    }

    sn_launch_tool_args.push("--hard-coded-contacts");
    sn_launch_tool_args.push(contacts);

    debug!(
        "Running network launch tool with args: {:?}",
        sn_launch_tool_args
    );

    // We can now call the tool with the args
    println!("Starting a node to join a Safe network...");
    join_with(Some(&sn_launch_tool_args))?;
    Ok(())
}

pub fn node_shutdown(node_path: Option<PathBuf>) -> Result<(), String> {
    let node_exec_name = match node_path {
        Some(ref path) => {
            let filepath = path.as_path();
            if filepath.is_file() {
                match filepath.file_name() {
                    Some(filename) => match filename.to_str() {
                        Some(name) => Ok(name),
                        None => Err(format!("Node path provided ({}) contains invalid unicode chars", filepath.display())),
                    }
                    None => Err(format!("Node path provided ({}) is invalid as it doens't include the executable filename", filepath.display())),
                }
            } else {
                Err(format!("Node path provided ({}) is invalid as it doens't include the executable filename", filepath.display()))
            }
        }
        None => Ok(SN_NODE_EXECUTABLE),
    }?;

    debug!(
        "Killing all running nodes launched with {}...",
        node_exec_name
    );
    kill_nodes(node_exec_name)
}

fn get_node_bin_path(node_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match node_path {
        Some(p) => Ok(p),
        None => {
            let mut path = dirs_next::home_dir()
                .ok_or_else(|| "Failed to obtain user's home path".to_string())?;

            path.push(".safe");
            path.push("node");
            Ok(path)
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn kill_nodes(exec_name: &str) -> Result<(), String> {
    let output = Command::new("killall")
        .arg(exec_name)
        .output()
        .map_err(|err| {
            format!(
                "Error when atempting to stop nodes ({}) processes: {}",
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
            "Failed to stop nodes ({}) processes: {}",
            exec_name,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(target_os = "windows")]
fn kill_nodes(exec_name: &str) -> Result<(), String> {
    let output = Command::new("taskkill")
        .args(&["/F", "/IM", exec_name])
        .output()
        .map_err(|err| {
            format!(
                "Error when atempting to stop nodes ({}) processes: {}",
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
            "Failed to stop nodes ({}) processes: {}",
            exec_name,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn node_update(node_path: Option<PathBuf>) -> Result<(), String> {
    let node_path = get_node_bin_path(node_path)?;

    let arg_node_path = node_path.join(SN_NODE_EXECUTABLE).display().to_string();
    debug!("Updating node at {}", arg_node_path);

    let child = Command::new(&arg_node_path)
        .args(vec!["--update-only"])
        .spawn()
        .map_err(|err| format!("Failed to update node at '{}': {}", arg_node_path, err))?;

    let output = child
        .wait_with_output()
        .map_err(|err| format!("Failed to update node at '{}': {}", arg_node_path, err))?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| format!("Failed to output stdout: {}", err))?;
        Ok(())
    } else {
        Err(format!(
            "Failed when invoking node executable from '{}':\n{}",
            arg_node_path,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
