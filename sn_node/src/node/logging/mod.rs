// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub(super) mod log_ctx;
mod system;

use self::log_ctx::LogCtx;

use std::time::Duration;
use sysinfo::{PidExt, System, SystemExt};
use system::Process;
use tokio::time::MissedTickBehavior;
use tracing::trace;
use xor_name::Prefix;

const LOG_INTERVAL: Duration = std::time::Duration::from_secs(60);

pub(super) async fn run_system_logger(ctx: LogCtx, print_resources_usage: bool) {
    let mut system = System::new_all();
    let prefix = ctx.prefix().await;
    initial_log(&mut system, prefix).await;

    let _handle = tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(LOG_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip); // default is `Burst`, probably not what we want
        loop {
            let _instant = interval.tick().await;
            system.refresh_all();
            let prefix = ctx.prefix().await;
            log(&mut system, prefix, print_resources_usage).await;
        }
    });
}

async fn initial_log(system: &mut System, prefix: Prefix) {
    let prefix: &str = &format!("{}", prefix.name());
    let os_name: &str = &fmt(system.name());
    let kernel_version: &str = &fmt(system.kernel_version());
    let os_version: &str = &fmt(system.os_version());
    let host_name: &str = &fmt(system.host_name());
    trace!(prefix, os_name, kernel_version, os_version, host_name);
}

fn fmt(string: Option<String>) -> String {
    string.unwrap_or_else(|| "Unknown".to_string())
}

async fn log(system: &mut System, prefix: Prefix, print_resources_usage: bool) {
    let prefix: &str = &format!("({:?})", prefix);

    let processors = system.processors();
    let processor_count = processors.len();

    let our_pid = &std::process::id();

    for (pid, proc_) in system.processes() {
        if pid.as_u32() != *our_pid {
            continue;
        }

        if print_resources_usage {
            println!(
                "{}: Node resource usage: {:?}",
                prefix,
                Process::map(proc_, processor_count)
            )
        } else {
            trace!(
                "{}: Node resource usage: {:?}",
                prefix,
                Process::map(proc_, processor_count)
            )
        }
    }
}
