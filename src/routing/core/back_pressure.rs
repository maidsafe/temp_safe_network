// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use qp2p::config::RetryConfig;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};
use sysinfo::{RefreshKind, System, SystemExt};
use tokio::{sync::RwLock, time::Instant};

const MIN_REPORT_INTERVAL: Duration = Duration::from_secs(10);

/// Average cpu load to be sent over the wire.
/// The values represent percentages, e.g. 12.234.., 21.721.., etc.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct LoadAvg {
    /// Average cpu load within one minute.
    pub(crate) one: f64,
    /// Average cpu load within five minutes.
    pub(crate) five: f64,
    /// Average cpu load within fifteen minutes.
    pub(crate) fifteen: f64,
}

#[derive(Clone)]
pub(crate) struct BackPressure {
    system: Arc<RwLock<System>>,
    our_last_report: Arc<RwLock<(Instant, LoadAvg)>>,
    reports: Arc<RwLock<BTreeMap<SocketAddr, RetryConfig>>>,
}

impl BackPressure {
    pub(crate) fn new() -> Self {
        let mut system = System::new_with_specifics(RefreshKind::new());
        system.refresh_cpu();
        let load = map_reading(system.load_average());

        Self {
            system: Arc::new(RwLock::new(system)),
            our_last_report: Arc::new(RwLock::new((Instant::now(), load))),
            reports: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Gets retry policy if exists, or default.
    pub(crate) async fn get(&self, addr: &SocketAddr) -> RetryConfig {
        self.reports
            .read()
            .await
            .get(addr)
            .copied()
            .unwrap_or_default()
    }

    /// If we get a response back from a node, we remove them.
    pub(crate) async fn remove(&self, addr: &SocketAddr) {
        let _ = self.reports.write().await.remove(addr);
    }

    /// Sets reported load for a node, when it errored back saying it was strained.
    pub(crate) async fn set(&self, addr: &SocketAddr, load: LoadAvg) {
        // evaluate the passed in load
        let (short_term, mid_term, long_term) = evaluate(&load);

        let default_cfg = RetryConfig::default();

        let (initial_retry_interval, retry_delay_multiplier, retrying_max_elapsed_time) =
            if long_term.triggered() {
                (Duration::from_millis(4000), 4.0, Duration::from_secs(180))
            } else if mid_term.triggered() {
                (Duration::from_millis(2000), 3.0, Duration::from_secs(90))
            } else if short_term.triggered() {
                (Duration::from_millis(1000), 2.0, Duration::from_secs(45))
            } else {
                (
                    default_cfg.initial_retry_interval,
                    default_cfg.retry_delay_multiplier,
                    default_cfg.retrying_max_elapsed_time,
                )
            };

        let cfg = RetryConfig {
            initial_retry_interval,
            max_retry_interval: default_cfg.max_retry_interval,
            retry_delay_multiplier,
            retry_delay_rand_factor: default_cfg.retry_delay_rand_factor,
            retrying_max_elapsed_time,
        };

        let _ = self.reports.write().await.insert(*addr, cfg);
    }

    /// Sent to nodes calling us, if we are strained
    pub(crate) async fn load_report(&self) -> Option<LoadAvg> {
        let now = Instant::now();
        let (time, our_last_report) = { *self.our_last_report.read().await };
        // do not refresh too often
        let load = if now - time > MIN_REPORT_INTERVAL {
            {
                self.system.write().await.refresh_cpu();
            }
            let load = { map_reading(self.system.read().await.load_average()) };
            *self.our_last_report.write().await = (now, load);
            load
        } else {
            our_last_report // use previous report if to short time has elapsed
        };

        let (short_term, mid_term, long_term) = evaluate(&load);

        if long_term.triggered() || mid_term.triggered() || short_term.triggered() {
            Some(load)
        } else {
            None
        }
    }
}

fn map_reading(reading: sysinfo::LoadAvg) -> LoadAvg {
    LoadAvg {
        one: reading.one,
        five: reading.five,
        fifteen: reading.fifteen,
    }
}

fn evaluate(load: &LoadAvg) -> (ShortTerm, MidTerm, LongTerm) {
    let short_term = ShortTerm {
        moderate: load.one > 60.0 && load.five > 30.0 && load.fifteen > 15.0,
        high: load.one > 70.0 && load.five > 20.0 && load.fifteen > 10.0,
        very_high: load.one > 80.0 && load.five > 10.0 && load.fifteen > 5.0,
        critical: load.one > 90.0 && load.five >= 0.0 && load.fifteen >= 0.0,
    };

    let mid_term = MidTerm {
        moderate: load.one > 50.0 && load.five > 60.0 && load.fifteen > 30.0,
        high: load.one > 60.0 && load.five > 70.0 && load.fifteen > 20.0,
        very_high: load.one > 70.0 && load.five > 80.0 && load.fifteen > 10.0,
        critical: load.one > 80.0 && load.five > 90.0 && load.fifteen >= 0.0,
    };

    let long_term = LongTerm {
        moderate: load.one > 40.0 && load.five > 50.0 && load.fifteen > 60.0,
        high: load.one > 50.0 && load.five > 60.0 && load.fifteen > 70.0,
        very_high: load.one > 60.0 && load.five > 70.0 && load.fifteen > 80.0,
        critical: load.one > 70.0 && load.five >= 80.0 && load.fifteen >= 90.0,
    };

    (short_term, mid_term, long_term)
}

struct ShortTerm {
    moderate: bool,
    high: bool,
    very_high: bool,
    critical: bool,
}

struct MidTerm {
    moderate: bool,
    high: bool,
    very_high: bool,
    critical: bool,
}

struct LongTerm {
    moderate: bool,
    high: bool,
    very_high: bool,
    critical: bool,
}

trait Alarm {
    fn triggered(&self) -> bool;
}

impl Alarm for ShortTerm {
    fn triggered(&self) -> bool {
        self.moderate || self.high || self.very_high || self.critical
    }
}

impl Alarm for MidTerm {
    fn triggered(&self) -> bool {
        self.moderate || self.high || self.very_high || self.critical
    }
}

impl Alarm for LongTerm {
    fn triggered(&self) -> bool {
        self.moderate || self.high || self.very_high || self.critical
    }
}
