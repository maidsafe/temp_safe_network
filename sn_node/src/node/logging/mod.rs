// Copyright 2021 MaidSafe.net limited.
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
use sysinfo::PidExt;
use sysinfo::{System, SystemExt};
use system::Process;
use tokio::time::MissedTickBehavior;
use tracing::trace;

const LOG_INTERVAL: Duration = std::time::Duration::from_secs(10);

pub(super) async fn run_system_logger(ctx: LogCtx, print_resources_usage: bool) {
    let mut system = System::new_all();
    initial_log(&mut system, &ctx).await;

    let _handle = tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(LOG_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip); // default is `Burst`, probably not what we want
        loop {
            let _instant = interval.tick().await;
            system.refresh_all();
            const LOW_MEM_THRESHOLD: u64 = 100_000;
            let available_memory = system.available_memory();
            if available_memory < LOW_MEM_THRESHOLD {
                warn!("========================>>> LOW MEMORY: {available_memory}KB AVAILABLE");
            }
            log(&mut system, &ctx, print_resources_usage).await;
        }
    });
}

async fn initial_log(system: &mut System, ctx: &LogCtx) {
    let prefix: &str = &format!("{}", ctx.prefix().await.name());
    let os_name: &str = &fmt(system.name());
    let kernel_version: &str = &fmt(system.kernel_version());
    let os_version: &str = &fmt(system.os_version());
    let host_name: &str = &fmt(system.host_name());
    trace!(prefix, os_name, kernel_version, os_version, host_name);
}

fn fmt(string: Option<String>) -> String {
    string.unwrap_or_else(|| "Unknown".to_string())
}

#[tracing::instrument(skip(ctx))]
async fn log(system: &mut System, ctx: &LogCtx, print_resources_usage: bool) {
    let prefix: &str = &format!("({:?})", ctx.prefix().await);

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
