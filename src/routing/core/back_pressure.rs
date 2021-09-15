// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{CpuLoad, LoadReport};

use itertools::Itertools;
use qp2p::config::RetryConfig;
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};
use sysinfo::{RefreshKind, System, SystemExt};
use tokio::{sync::RwLock, time::Instant};

const MIN_REPORT_INTERVAL: Duration = Duration::from_secs(10);
const REPORT_TTL: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Clone)]
pub(crate) struct BackPressure {
    system: Arc<RwLock<System>>,
    our_last_report: Arc<RwLock<(Instant, LoadReport)>>,
    reports: Arc<RwLock<BTreeMap<SocketAddr, (Instant, RetryConfig)>>>,
    last_eviction: Arc<RwLock<Instant>>,
}

impl BackPressure {
    pub(crate) fn new() -> Self {
        let mut system = System::new_with_specifics(RefreshKind::new());
        system.refresh_cpu();
        let load = evaluate(system.load_average());

        Self {
            system: Arc::new(RwLock::new(system)),
            our_last_report: Arc::new(RwLock::new((Instant::now(), load))),
            reports: Arc::new(RwLock::new(BTreeMap::new())),
            last_eviction: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Gets retry policy if exists, or default.
    pub(crate) async fn get(&self, addr: &SocketAddr) -> RetryConfig {
        self.reports
            .read()
            .await
            .get(addr)
            .copied()
            .map(|(_, cfg)| cfg)
            .unwrap_or_default()
    }

    /// Remove regulation for specific nodes when we don't need them anymore (e.g. they left).
    pub(crate) async fn remove(&self, addr: SocketAddr) {
        let _ = self.reports.write().await.remove(&addr);
    }

    /// Sets reported load for a node, when it errored back saying it was strained.
    pub(crate) async fn set(&self, addr: SocketAddr, load: LoadReport) {
        let (initial_retry_interval, retry_delay_multiplier, retrying_max_elapsed_time) =
            if load.long_term.critical {
                (Duration::from_millis(12000), 7.0, Duration::from_secs(480))
            } else if load.long_term.very_high {
                (Duration::from_millis(6000), 5.5, Duration::from_secs(240))
            } else if load.mid_term.critical || load.long_term.high {
                (Duration::from_millis(3000), 4.0, Duration::from_secs(120))
            } else if load.mid_term.very_high {
                (Duration::from_millis(1500), 2.5, Duration::from_secs(60))
            } else if load.is_ok() {
                // (this is currently not a reachable case as reporting is only done when load is bad.)
                (Duration::from_millis(1000), 2.0, Duration::from_secs(45))
            } else if load.is_good() {
                // (this is currently not a reachable case as reporting is only done when load is bad.)
                // remove regulation, i.e. effectively change back to defaults
                return self.remove(addr).await;
            } else {
                // somewhere between ok and good, so slightly higher values than default, but lower than when ok.
                (Duration::from_millis(750), 1.7, Duration::from_secs(40))
            };

        let default_cfg = RetryConfig::default();
        let cfg = RetryConfig {
            initial_retry_interval,
            max_retry_interval: default_cfg.max_retry_interval,
            retry_delay_multiplier,
            retry_delay_rand_factor: default_cfg.retry_delay_rand_factor,
            retrying_max_elapsed_time,
        };

        let _ = self
            .reports
            .write()
            .await
            .insert(addr, (Instant::now(), cfg));
    }

    /// Sent to nodes calling us, if we are strained
    pub(crate) async fn load_report(&self) -> Option<LoadReport> {
        let now = Instant::now();
        let (then, our_last_report) = { *self.our_last_report.read().await };
        // do not refresh too often
        let load = if now - then > MIN_REPORT_INTERVAL {
            {
                self.system.write().await.refresh_cpu();
            }
            let current_load = { evaluate(self.system.read().await.load_average()) };
            *self.our_last_report.write().await = (now, current_load);

            // reduce the checks for this somewhat
            let last_eviction = { *self.last_eviction.read().await };
            // then only try evict when there's any likelihood of being any expired
            if now - last_eviction > REPORT_TTL {
                self.evict_expired(now).await;
            }

            current_load
        } else {
            our_last_report // use previous report if too short time has elapsed
        };

        if load.is_bad() {
            Some(load)
        } else {
            None
        }
    }

    async fn evict_expired(&self, now: Instant) {
        let expired = {
            self.reports
                .read()
                .await
                .iter()
                .filter_map(|(key, (last_seen, _))| {
                    if now - *last_seen > REPORT_TTL {
                        Some(*key)
                    } else {
                        None
                    }
                })
                .collect_vec()
        };

        for addr in expired {
            self.remove(addr).await
        }

        *self.last_eviction.write().await = now;
    }
}

fn evaluate(load: sysinfo::LoadAvg) -> LoadReport {
    // Normalize the reading (e.g. `load=4` when `cores=4` => `normalized_load=1`)
    let cores = num_cpus::get_physical() as f64;
    let load = sysinfo::LoadAvg {
        one: load.one / cores,
        five: load.five / cores,
        fifteen: load.fifteen / cores,
    };

    // #TODO: Improve on this messy evaluation..
    let short_term = CpuLoad {
        low: load.one < 0.6 && load.five < 0.6 && load.fifteen < 0.6,
        moderate: load.one > 0.7 && load.five > 0.4 && load.fifteen > 0.2,
        high: load.one > 0.8 && load.five > 0.3 && load.fifteen > 0.1,
        very_high: load.one > 0.9 && load.five > 0.2 && load.fifteen > 0.05,
        critical: load.one > 3.0 && load.five > 0.1 && load.fifteen >= 0.0,
    };

    let mid_term = CpuLoad {
        low: load.one < 0.5 && load.five < 0.6 && load.fifteen < 0.6,
        moderate: load.one > 0.6 && load.five > 0.7 && load.fifteen > 0.2,
        high: load.one > 0.7 && load.five > 0.8 && load.fifteen > 0.1,
        very_high: load.one > 0.8 && load.five > 0.9 && load.fifteen >= 0.0,
        critical: load.one > 1.0 && load.five > 2.0 && load.fifteen >= 0.0,
    };

    let long_term = CpuLoad {
        low: load.one < 0.4 && load.five < 0.6 && load.fifteen < 0.6,
        moderate: load.one > 0.5 && load.five > 0.6 && load.fifteen > 0.7,
        high: load.one > 0.6 && load.five > 0.7 && load.fifteen > 0.8,
        very_high: load.one > 0.7 && load.five > 0.8 && load.fifteen > 0.9,
        critical: load.one > 1.0 && load.five > 2.0 && load.fifteen > 1.0,
    };

    LoadReport {
        short_term,
        mid_term,
        long_term,
    }
}

impl LoadReport {
    fn is_good(&self) -> bool {
        self.mid_term.low && self.long_term.low && !self.short_term.critical
    }

    fn is_ok(&self) -> bool {
        !self.is_good() && !self.mid_term.high && !self.long_term.moderate
    }

    fn is_bad(&self) -> bool {
        self.short_term.critical
            || self.mid_term.very_high
            || self.mid_term.critical
            || self.long_term.high
            || self.long_term.very_high
            || self.long_term.critical
    }
}
