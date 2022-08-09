// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::download_and_install_node;
use crate::operations::config::NetworkLauncher;
use color_eyre::{eyre::bail, eyre::eyre, eyre::WrapErr, Result};
use std::{
    fs::create_dir_all,
    io::{self, Write},
    net::SocketAddr,
    path::PathBuf,
    process::{Command, Stdio},
};
use tracing::debug;

#[cfg(not(target_os = "windows"))]
pub(crate) const SN_NODE_EXECUTABLE: &str = "sn_node";

#[cfg(target_os = "windows")]
pub(crate) const SN_NODE_EXECUTABLE: &str = "sn_node.exe";

/// Tries to print the version of the node binary pointed to
pub fn node_version(node_path: Option<PathBuf>) -> Result<()> {
    let bin_path = get_node_bin_path(node_path)?.join(SN_NODE_EXECUTABLE);
    let path_str = bin_path.display().to_string();

    if !bin_path.as_path().is_file() {
        return Err(eyre!(format!(
            "node executable not found at '{}'.",
            path_str
        )));
    }

    let output = Command::new(&path_str)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| {
            eyre!(format!(
                "Failed to execute node from '{}': {}",
                path_str, err
            ))
        })?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| eyre!(format!("failed to write to stdout: {}", err)))
    } else {
        Err(eyre!(
            "Failed to get node version nodes when invoking executable from '{}': {}",
            path_str,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn node_install(target_dir_path: PathBuf, version: Option<String>) -> Result<()> {
    let _ =
        download_and_install_node(target_dir_path, SN_NODE_EXECUTABLE, "safe_network", version)?;
    Ok(())
}

pub fn node_run(
    network_launcher: &mut Box<impl NetworkLauncher>,
    node_directory_path: PathBuf,
    node_data_dir_name: &str,
    interval: u64,
    num_of_nodes: &str,
    ip: Option<String>,
) -> Result<()> {
    let mut sn_launch_tool_args =
        get_initial_sn_launch_args(node_directory_path, node_data_dir_name)?;
    sn_launch_tool_args.push("--interval".to_string());
    let interval_msecs = 1000 * interval;
    sn_launch_tool_args.push(interval_msecs.to_string());
    sn_launch_tool_args.push("--num-nodes".to_string());
    sn_launch_tool_args.push(num_of_nodes.to_string());
    if let Some(launch_ip) = ip {
        sn_launch_tool_args.push("--ip".to_string());
        sn_launch_tool_args.push(launch_ip);
    } else {
        sn_launch_tool_args.push("--local".to_string());
    }
    network_launcher.launch(sn_launch_tool_args, interval)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn node_join(
    network_launcher: &mut Box<impl NetworkLauncher>,
    node_directory_path: PathBuf,
    node_data_dir_name: &str,
    verbosity: u8,
    local_addr: Option<SocketAddr>,
    public_addr: Option<SocketAddr>,
    clear_data: bool,
    local: bool,
    disable_port_forwarding: bool,
    network_contacts_file: PathBuf,
) -> Result<()> {
    let mut sn_launch_tool_args =
        get_initial_sn_launch_args(node_directory_path, node_data_dir_name)?;
    if local {
        sn_launch_tool_args.push("--local".to_string());
    }
    if disable_port_forwarding {
        sn_launch_tool_args.push("--skip-auto-port-forwarding".to_string());
    }
    if let Some(local_ip) = local_addr.map(|addr| addr.to_string()) {
        sn_launch_tool_args.push("--local-addr".to_string());
        sn_launch_tool_args.push(local_ip);
    }
    if let Some(public_ip) = public_addr.map(|addr| addr.to_string()) {
        sn_launch_tool_args.push("--public-addr".to_string());
        sn_launch_tool_args.push(public_ip);
    }
    if clear_data {
        sn_launch_tool_args.push("--clear-data".to_string());
    }
    let mut verbosity_arg = String::from("-");
    if verbosity > 0 {
        let v = "y".repeat(verbosity as usize);
        println!("V: {}", v);
        verbosity_arg.push_str(&v);
        sn_launch_tool_args.push(verbosity_arg);
    }

    sn_launch_tool_args.push("--network-contacts-file".to_string());
    sn_launch_tool_args.push(network_contacts_file.display().to_string());

    network_launcher.join(sn_launch_tool_args)?;
    Ok(())
}

pub fn node_shutdown(node_path: Option<PathBuf>) -> Result<()> {
    let node_exec_name = match node_path {
        Some(ref path) => {
            let filepath = path.as_path();
            if filepath.is_file() {
                match filepath.file_name() {
                    Some(filename) => match filename.to_str() {
                        Some(name) => name,
                        None => bail!("Node path provided ({}) contains invalid unicode chars", filepath.display()),
                    }
                    None => bail!("Node path provided ({}) is invalid as it doens't include the executable filename", filepath.display()),
                }
            } else {
                bail!("Node path provided ({}) is invalid as it doens't include the executable filename", filepath.display())
            }
        }
        None => SN_NODE_EXECUTABLE,
    };

    debug!(
        "Killing all running nodes launched with {}...",
        node_exec_name
    );
    kill_nodes(node_exec_name)
}

fn get_initial_sn_launch_args(
    node_directory_path: PathBuf,
    node_data_dir_name: &str,
) -> Result<Vec<String>> {
    let arg_node_path = node_directory_path
        .join(SN_NODE_EXECUTABLE)
        .display()
        .to_string();
    debug!("Running node from {}", arg_node_path);

    let node_data_dir_path = node_directory_path.join(node_data_dir_name);
    if !node_data_dir_path.exists() {
        println!("Creating '{}' folder", node_data_dir_path.display());
        create_dir_all(node_data_dir_path.clone())
            .wrap_err("Couldn't create target path to store nodes' generated data")?;
    }
    let arg_nodes_dir = node_data_dir_path.display().to_string();
    println!("Storing nodes' generated data at {}", arg_nodes_dir);

    // This first positional "sn_launch_tool" argument is required to get the tool to run
    // correctly, even though it doesn't appear to actually do anything. It seems you can't just
    // pass an optional argument as the first one.
    let sn_launch_tool_args = vec![
        String::from("sn_launch_tool"),
        String::from("--node-path"),
        arg_node_path,
        String::from("--nodes-dir"),
        arg_nodes_dir,
    ];
    Ok(sn_launch_tool_args)
}

fn get_node_bin_path(node_path: Option<PathBuf>) -> Result<PathBuf> {
    match node_path {
        Some(p) => Ok(p),
        None => {
            let mut path =
                dirs_next::home_dir().ok_or_else(|| eyre!("Failed to obtain user's home path"))?;

            path.push(".safe");
            path.push("node");
            Ok(path)
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn kill_nodes(exec_name: &str) -> Result<()> {
    let output = Command::new("killall")
        .arg(exec_name)
        .output()
        .wrap_err_with(|| {
            format!(
                "Error when atempting to stop nodes ({}) processes",
                exec_name
            )
        })?;

    if output.status.success() {
        println!(
            "Success, all processes instances of {} were stopped!",
            exec_name
        );
        Ok(())
    } else {
        Err(eyre!(
            "Failed to stop nodes ({}) processes: {}",
            exec_name,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(target_os = "windows")]
fn kill_nodes(exec_name: &str) -> Result<()> {
    let output = Command::new("taskkill")
        .args(&["/F", "/IM", exec_name])
        .output()
        .wrap_err_with(|| {
            format!(
                "Error when atempting to stop nodes ({}) processes",
                exec_name
            )
        })?;

    if output.status.success() {
        println!(
            "Success, all processes instances of {} were stopped!",
            exec_name
        );
        Ok(())
    } else {
        Err(eyre!(
            "Failed to stop nodes ({}) processes: {}",
            exec_name,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn node_update(node_path: Option<PathBuf>) -> Result<()> {
    let node_path = get_node_bin_path(node_path)?;

    let arg_node_path = node_path.join(SN_NODE_EXECUTABLE).display().to_string();
    debug!("Updating node at {}", arg_node_path);

    let child = Command::new(&arg_node_path)
        .args(vec!["--update-only"])
        .spawn()
        .wrap_err_with(|| format!("Failed to update node at '{}'", arg_node_path))?;

    let output = child
        .wait_with_output()
        .wrap_err_with(|| format!("Failed to update node at '{}'", arg_node_path))?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .wrap_err("Failed to output stdout")?;
        Ok(())
    } else {
        Err(eyre!(
            "Failed when invoking node executable from '{}':\n{}",
            arg_node_path,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
