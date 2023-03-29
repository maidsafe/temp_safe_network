// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! sn_node provides the interface to Safe routing.  The resulting executable is the node
//! for the Safe network.
// boop
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
    unused_results,
    clippy::unwrap_used
)]

mod log;
#[cfg(feature = "rpc-service")]
mod rpc;

use sn_node::node::{new_node, Config, Error as NodeError, NodeEvent, NodeRef, RejoinReason};
use sn_updater::{update_binary, UpdateType};

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use color_eyre::{
    eyre::{eyre, Error},
    Result, Section, SectionExt,
};
use std::{io::Write, process::exit, sync::Arc, thread::sleep};
use tokio::{
    runtime::Runtime,
    sync::{mpsc, RwLock},
    time::Duration,
};
use tracing::{self, error, info, warn};

// Time we allow a node to keep attempting to join
const JOIN_ATTEMPT_TIMEOUT_SEC: u64 = 30;
// Time between retry attempts after fail to join
const JOIN_TIMEOUT_WAIT_BEFORE_RETRY_TIME_SEC: u64 = 10;
// Time to wait before trying to join again when joins are not allowed
const JOIN_DISALLOWED_RETRY_TIME_SEC: u64 = 60;

#[derive(Debug)]
#[allow(dead_code)]
// To be sent to the main thread in order to stop/restart the execution of the safenode app.
enum NodeCtrl {
    // Request to stop the exeution of the safenode app, providing an error as a reason for it.
    Stop { delay: Duration, cause: Error },
    // Request to restart the exeution of the safenode app,
    // retrying to join the network, after the requested delay.
    Restart(Duration),
    // Request to update the safenode app, and restart it, after the requested delay.
    Update(Duration),
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut config = Config::new()?;

    #[cfg(not(feature = "otlp"))]
    let _log_guard = log::init_node_logging(&config)?;
    #[cfg(feature = "otlp")]
    let (_rt, _guard) = {
        // init logging in a separate runtime if we are sending traces to an opentelemetry server
        let rt = Runtime::new()?;
        let guard = rt.block_on(async { log::init_node_logging(&config) })?;
        (rt, guard)
    };

    setup_parking_lot();

    loop {
        println!("Node started");
        create_runtime_and_node(&config)?;

        // if we've had an issue, lets put the brakes on any crazy looping here
        sleep(Duration::from_secs(1));

        // pull config again in case it has been updated meanwhile
        config = Config::new()?;
    }
}

/// Create a tokio runtime per `start_node` attempt.
/// This ensures any spawned tasks are closed before this would be run again.
fn create_runtime_and_node(config: &Config) -> Result<()> {
    if let Some(c) = &config.completions() {
        let shell = c.parse().map_err(|err: String| eyre!(err))?;
        let buf = gen_completions_for_shell(shell, Config::command()).map_err(|err| eyre!(err))?;
        std::io::stdout().write_all(&buf)?;

        return Ok(());
    }

    if config.update() {
        match update(config.no_confirm()) {
            Ok(_) => exit(0),
            Err(e) => {
                println!("{e:?}");
                exit(1);
            }
        }
    }

    config.validate()?;

    let our_pid = std::process::id();
    let log_dir = if let Some(path) = config.log_dir() {
        format!("{}", path.display())
    } else {
        "stdout".to_string()
    };

    loop {
        // make a fresh runtime
        let rt = Runtime::new()?;

        let message = format!(
            "Running {} v{}",
            Config::clap().get_name(),
            env!("CARGO_PKG_VERSION")
        );

        info!("\n{}\n{}", message, "=".repeat(message.len()));

        rt.block_on(async {
            info!("Initial node config: {config:?}");
            let join_timeout = Duration::from_secs(JOIN_ATTEMPT_TIMEOUT_SEC);
            let node_ref = match new_node(config, join_timeout).await {
                Ok(node_ref) => node_ref,
                Err(NodeError::JoinTimeout) => {
                    let join_retry_sec = Duration::from_secs(JOIN_TIMEOUT_WAIT_BEFORE_RETRY_TIME_SEC);
                    let message = format!("(PID: {our_pid}): Encountered a timeout while trying to join the network. Retrying after {join_retry_sec:?}.");
                    println!("{message} Node log path: {log_dir}");
                    error!("{message}");
                    sleep(join_retry_sec);
                    return Ok(());
                }
                Err(error) => {
                    let err = Err(error).suggestion(format!(
                        "Cannot start node. Node log path: {log_dir}").header(
                        "If this is the first node on the network pass the local address to be used using --first",
                    ));
                    error!("{err:?}");
                    return err;
                }
            };

            #[cfg(feature = "chaos")]
            {
                // Simulate failed node starts
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let x: f64 = rng.gen_range(0.0..1.0);

                if config.first().is_none() && x > 0.6 {
                    println!(
                       "\n =========== [Chaos] (PID: {our_pid}): Startup chaos crash w/ x of: {x}. ============== \n",
                   );

                    // tiny sleep so testnet doesn't detect a faulty node and exit
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    warn!("[Chaos] (PID: {our_pid}): ChaoticStartupCrash");
                    return Ok(());
                }
            }

            // Node is running
            let (ctrl_tx, mut ctrl_rx) = mpsc::channel::<NodeCtrl>(5);

            let node_ref = Arc::new(RwLock::new(node_ref));

            // Monitor NodeEvents just in case we need to rejoin
            monitor_node_events(node_ref.clone(), ctrl_tx.clone(), log_dir.clone());

            // Start up gRPC interface
            #[cfg(feature = "rpc-service")]
            {
                // Defining address for our RPC service: we just use the same as sn_node's IP
                let mut addr = node_ref.read().await.context.socket_addr();
                // ... and let's use subsequent port number to the one used by sn_node.
                addr.set_port(addr.port() + 1);

                rpc::start_rpc_service(addr, log_dir.clone(), node_ref, ctrl_tx);
            }

            // We'll block here until there is a reason reported to restart/stop,
            // otherwise both the node and the gRPC service (if enabled) will be running.
            // If there is an event that requires the node to restart we
            // do so by returning Ok(()).
            while let Some(node_ctrl) = ctrl_rx.recv().await {
                match node_ctrl {
                NodeCtrl::Restart(delay) => {
                    let msg = format!("Node is restartig in {delay:?}...");
                    info!("{msg}");
                    println!("{msg} Node log path: {log_dir}");
                    sleep(delay);
                    break;
                },
                NodeCtrl::Stop {delay, cause} => {
                    let msg = format!("Node is stopping in {delay:?}...");
                    info!("{msg}");
                    println!("{msg} Node log path: {log_dir}");
                    sleep(delay);
                    return Err(cause);
                }
                NodeCtrl::Update(delay) => {
                    let msg = format!("Node is updating in {delay:?}...");
                    info!("{msg}");
                    println!("{msg} Node log path: {log_dir}");
                    sleep(delay);
                    match update(true) {
                        Ok(true) => break, // binary updated, let's restart
                        Ok(false) => continue, // binary was not updated, we don't need to restart
                        Err(err) => {
                            println!("Failed to update node: {err:?}");
                            continue;
                        }
                    }
                }}
            }
            Ok(())
        })?;

        // actively shut down the runtime
        rt.shutdown_timeout(Duration::from_secs(2));
    }
}

fn setup_parking_lot() {
    use parking_lot::deadlock;
    use std::thread;

    // Create a background thread which checks for deadlocks every 3s
    let _handle = thread::spawn(move || loop {
        sleep(Duration::from_secs(3));
        let deadlocks = deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        println!("{} deadlocks detected", deadlocks.len());
        warn!("{} deadlocks detected", deadlocks.len());
        for (i, threads) in deadlocks.iter().enumerate() {
            println!("Deadlock #{i}");
            warn!("Deadlock #{i}");
            for t in threads {
                println!("Thread Id {:#?}", t.thread_id());
                warn!("Thread Id {:#?}", t.thread_id());
                println!("{:#?}", t.backtrace());
                warn!("{:#?}", t.backtrace());
            }
        }
    });
}

fn update(no_confirm: bool) -> Result<bool> {
    let current_version = env!("CARGO_PKG_VERSION");
    update_binary(UpdateType::Node, current_version, !no_confirm)
        .map_err(|e| eyre!(format!("Failed to update sn_node: {e}")))
}

fn gen_completions_for_shell(shell: Shell, mut cmd: clap::Command) -> Result<Vec<u8>, String> {
    // Get exe path
    let exe_path =
        std::env::current_exe().map_err(|err| format!("Can't get the exec path: {err}"))?;

    // get filename without preceding path as std::ffi::OsStr (C string)
    let exec_name_ffi = match exe_path.file_name() {
        Some(v) => v,
        None => {
            return Err(format!(
                "Can't extract file_name of executable from path {}",
                exe_path.display()
            ))
        }
    };

    // Convert OsStr to string.  Can fail if OsStr contains any invalid unicode.
    let exec_name = match exec_name_ffi.to_str() {
        Some(v) => v.to_string(),
        None => {
            return Err(format!(
                "Can't decode unicode in executable name '{exec_name_ffi:?}'",
            ))
        }
    };

    // Generates shell completions for <shell> and prints to stdout
    let mut buf: Vec<u8> = vec![];
    generate(shell, &mut cmd, exec_name, &mut buf);

    Ok(buf)
}

fn monitor_node_events(
    node_ref: Arc<RwLock<NodeRef>>,
    ctrl_tx: mpsc::Sender<NodeCtrl>,
    log_dir: String,
) {
    let _handle = tokio::spawn(async move {
        let mut events_rx = node_ref.read().await.events_channel.subscribe();
        while let Ok(event) = events_rx.recv().await {
            match event {
                NodeEvent::ContextUpdated(new_context) => {
                    // update our copy of the node context
                    // so the RPC services can read up to date info
                    let mut node_ref = node_ref.write().await;
                    node_ref.context = new_context;
                }
                NodeEvent::RejoinRequired(reason) => {
                    error!("Rejoin reason: {reason:?}");
                    let ctrl_msg = match reason {
                        RejoinReason::RemovedFromSection => {
                            let msg = "Restarting node since it has been removed from section...";
                            println!("{msg} Node log path: {log_dir}");
                            info!("{msg}");
                            NodeCtrl::Restart(Duration::from_secs(2))
                        }
                        RejoinReason::JoinsDisallowed => {
                            let delay = JOIN_DISALLOWED_RETRY_TIME_SEC;
                            let message = format!(
                                "The network is not accepting nodes right now. \
                                Retrying after {delay} seconds."
                            );
                            println!("{message} Node log path: {log_dir}");
                            info!("{message}");
                            NodeCtrl::Restart(Duration::from_secs(delay))
                        }
                    };

                    if let Err(err) = ctrl_tx.send(ctrl_msg).await {
                        error!(
                            "Failed to send node control msg to safenode bin main thread: {err}"
                        );
                    }
                }
                _ => {
                    /* we are not interested in any other type of node events here */
                    info!("Dropping an event we currently don't expose through gRPC: {event:?}");
                }
            }
        }
    });
}
