// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub mod log_ctx;
mod system;

use self::log_ctx::LogCtx;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt};
use system::Process;
use tracing::trace;

const LOG_INTERVAL: Duration = std::time::Duration::from_secs(30);

pub async fn run_system_logger(ctx: LogCtx) {
    let mut system = System::new_all();
    let mut last_log = Instant::now();
    initial_log(&mut system, &ctx).await;
    log(&mut last_log, &mut system, &ctx).await;
    let _ = tokio::task::spawn(async move {
        loop {
            let elapsed = last_log.elapsed();
            if elapsed >= LOG_INTERVAL {
                log(&mut last_log, &mut system, &ctx).await;
            } else {
                tokio::time::sleep(LOG_INTERVAL - elapsed).await;
            }
        }
    });
}

async fn initial_log(system: &mut System, ctx: &LogCtx) {
    let prefix: &str = &format!("{}", ctx.prefix().await.name());
    let os_name: &str = &fmt(system.name());
    let kernel_version: &str = &fmt(system.kernel_version());
    let os_version: &str = &fmt(system.os_version());
    let host_name: &str = &fmt(system.host_name());
    trace!(prefix, os_name, kernel_version, os_version, host_name,);
}

fn fmt(string: Option<String>) -> String {
    string.unwrap_or_else(|| "Unknown".to_string())
}

#[tracing::instrument]
async fn log(last_log: &mut Instant, system: &mut System, ctx: &LogCtx) {
    *last_log = Instant::now();

    system.refresh_all();

    let prefix: &str = &format!("{}", ctx.prefix().await.name());

    let total_memory = system.total_memory();
    let system_memory = system.used_memory();
    let total_swap = system.total_swap();
    let used_swap = system.used_swap();

    trace!(prefix, total_memory, system_memory, total_swap, used_swap,);

    let load_avg = system.load_average();
    let processors = system.processors();
    let processor_count = processors.len();

    let one_min_load_avg = (100_f64 * load_avg.one / processor_count as f64) as u8;
    let five_min_load_avg = (100_f64 * load_avg.five / processor_count as f64) as u8;
    let fifteen_min_load_avg = (100_f64 * load_avg.fifteen / processor_count as f64) as u8;

    trace!(
        prefix,
        one_min_load_avg,
        five_min_load_avg,
        fifteen_min_load_avg,
        "System load avg",
    );

    let our_pid = &(std::process::id() as usize);

    for (pid, proc_) in system.processes() {
        if pid != our_pid {
            continue;
        }
        trace!(
            prefix,
            "Node resource usage: {:?}",
            Process::map(proc_, processor_count)
        );
    }
}
