// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub(super) mod log_ctx;

use self::log_ctx::LogCtx;
use sysinfo::{System, SystemExt};
use tracing::trace;

pub(super) async fn log_system_details(ctx: LogCtx) {
    let mut system = System::new_all();
    initial_log(&mut system, &ctx).await;
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
