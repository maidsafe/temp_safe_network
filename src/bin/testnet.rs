// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! sn_node provides the interface to Safe routing.  The resulting executable is the node
//! for the Safe network.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
    html_favicon_url = "https://maidsafe.net/img/favicon.ico",
    test(attr(deny(warnings)))
)]
// For explanation of lint checks, run `rustc -W help`.
#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

use dirs_next::home_dir;
use sn_launch_tool::run_with;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use structopt::StructOpt;
use tokio::fs::{create_dir_all, remove_dir_all};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

#[cfg(not(target_os = "windows"))]
const SAFE_NODE_EXECUTABLE: &str = "sn_node";

#[cfg(target_os = "windows")]
const SAFE_NODE_EXECUTABLE: &str = "sn_node.exe";

const NODES_DIR: &str = "local-test-network";
const INTERVAL: &str = "2";
const RUST_LOG: &str = "RUST_LOG";
const NODE_COUNT: &str = "33";

#[derive(Debug, StructOpt)]
#[structopt(name = "testnet")]
struct Cmd {
    /// All nodes will be joining existing testnet, none will be started as a genesis node.
    #[structopt(long = "add")]
    add_nodes_to_existing_network: bool,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt::init();

    let path = std::path::Path::new("nodes");
    remove_dir_all(&path).await.unwrap_or(()); // Delete nodes directory if it exists;
    create_dir_all(&path)
        .await
        .expect("Cannot create nodes directory");

    let mut args = vec!["build", "--release"];

    // Keep features consistent to avoid recompiling when possible
    if cfg!(feature = "always-joinable") {
        args.push("--features");
        args.push("always-joinable");
    }
    if cfg!(feature = "test-utils") {
        args.push("--features");
        args.push("test-utils");
    }

    println!("Building current sn_node");
    let _child = Command::new("cargo")
        .args(args.clone())
        // .env("RUST_LOG", "debug")
        // .env("RUST_BACKTRACE", "1")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| {
            format!(
                "Failed to run build command with args '{:?}': {}",
                args, err
            )
        })?;

    println!("sn_node built successfully");

    run_network().await?;

    Ok(())
}

fn get_node_bin_path(node_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match node_path {
        Some(p) => Ok(p),
        None => {
            let mut home_dirs =
                home_dir().ok_or_else(|| "Failed to obtain user's home path".to_string())?;

            home_dirs.push(".safe");
            home_dirs.push("node");
            Ok(home_dirs)
        }
    }
}

/// Uses SNLT to create a local network of nodes
pub async fn run_network() -> Result<(), String> {
    let args = Cmd::from_args();
    let adding_nodes = args.add_nodes_to_existing_network;

    info!("Starting local network");
    let node_path = Some(PathBuf::from("./target/release"));
    let node_path = get_node_bin_path(node_path)?;

    let arg_node_path = node_path.join(SAFE_NODE_EXECUTABLE).display().to_string();
    debug!("Running node from {}", arg_node_path);

    let base_log_dir = get_node_bin_path(None)?;
    let node_log_dir = base_log_dir.join(NODES_DIR);
    if !node_log_dir.exists() {
        debug!("Creating '{}' folder", node_log_dir.display());
        create_dir_all(node_log_dir.clone()).await.map_err(|err| {
            format!(
                "Couldn't create target path to store nodes' generated data: {}",
                err
            )
        })?;
    }
    let arg_node_log_dir = node_log_dir.display().to_string();
    info!("Storing nodes' generated data at {}", arg_node_log_dir);

    // Let's create an args array to pass to the network launcher tool
    let mut sn_launch_tool_args = vec![
        "sn_launch_tool",
        "-v",
        "--node-path",
        &arg_node_path,
        "--nodes-dir",
        &arg_node_log_dir,
        "--interval",
        INTERVAL,
        "--local",
    ];

    if adding_nodes {
        sn_launch_tool_args.push("--add")
    }

    // If RUST_LOG was set we pass it down to the launch tool
    // so it's set for each of the nodes logs as well.
    let rust_log = std::env::var(RUST_LOG).unwrap_or_else(|_| "safe_network=info".to_string());
    if !rust_log.is_empty() {
        sn_launch_tool_args.push("--rust-log");
        sn_launch_tool_args.push(&rust_log);
    }

    let interval_as_int = &INTERVAL
        .parse::<u64>()
        .map_err(|_| String::from("Error parsing Interval argument"))?;

    let node_count = std::env::var("NODE_COUNT").unwrap_or_else(|_| NODE_COUNT.to_string());
    let node_count_as_int = node_count
        .parse::<u64>()
        .map_err(|_| String::from("Error parsing Node Count argument"))?;

    debug!(
        "Running network launch tool with args: {:?}",
        sn_launch_tool_args
    );

    // We can now call the tool with the args
    info!("Launching local Safe network...");
    run_with(Some(&sn_launch_tool_args))?;

    // leave a longer interval with more nodes to allow for splits if using split amounts
    let interval_duration = Duration::from_secs(interval_as_int * node_count_as_int);

    sleep(interval_duration).await;

    Ok(())
}
