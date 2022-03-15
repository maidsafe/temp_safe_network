// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use sysinfo::{LoadAvg, RefreshKind, System, SystemExt};
use tokio::{sync::RwLock, time::MissedTickBehavior};

pub(crate) const INITIAL_MSGS_PER_S: usize = 100;

const ONE_MINUTE_AS_SECONDS: u64 = 60;

const SAMPLING_INTERVAL_ONE: Duration = Duration::from_secs(ONE_MINUTE_AS_SECONDS);
const SAMPLING_INTERVAL_FIVE: Duration = Duration::from_secs(5 * ONE_MINUTE_AS_SECONDS);
const SAMPLING_INTERVAL_FIFTEEN: Duration = Duration::from_secs(15 * ONE_MINUTE_AS_SECONDS);

const INITIAL_MSGS_PER_MINUTE: usize = ONE_MINUTE_AS_SECONDS as usize * INITIAL_MSGS_PER_S;

const MAX_CPU_LOAD: f64 = 0.8;
const ORDER: Ordering = Ordering::SeqCst;

/// Measure and return the rate of msgs per second that we can handle

#[derive(Clone)]
pub(crate) struct LoadMonitoring {
    system: Arc<RwLock<System>>,
    load_sample: Arc<RwLock<LoadAvg>>,
    msg_samples: BTreeMap<Duration, MsgCount>,
    msgs_per_s: BTreeMap<Duration, Arc<AtomicUsize>>,
}

impl LoadMonitoring {
    pub(crate) fn new() -> Self {
        let mut system = System::new_with_specifics(RefreshKind::new());
        system.refresh_cpu();

        let mut msg_samples = BTreeMap::new();

        let _ = msg_samples.insert(SAMPLING_INTERVAL_ONE, MsgCount::new());
        let _ = msg_samples.insert(SAMPLING_INTERVAL_FIVE, MsgCount::new());
        let _ = msg_samples.insert(SAMPLING_INTERVAL_FIFTEEN, MsgCount::new());

        let mut msgs_per_s = BTreeMap::new();

        let _ = msgs_per_s.insert(
            SAMPLING_INTERVAL_ONE,
            Arc::new(AtomicUsize::new(INITIAL_MSGS_PER_MINUTE)),
        );
        let _ = msgs_per_s.insert(
            SAMPLING_INTERVAL_FIVE,
            Arc::new(AtomicUsize::new(5 * INITIAL_MSGS_PER_MINUTE)),
        );
        let _ = msgs_per_s.insert(
            SAMPLING_INTERVAL_FIFTEEN,
            Arc::new(AtomicUsize::new(15 * INITIAL_MSGS_PER_MINUTE)),
        );

        let load_sample = Arc::new(RwLock::new(normalize(system.load_average())));

        let instance = Self {
            system: Arc::new(RwLock::new(system)),
            msg_samples,
            msgs_per_s,
            load_sample,
        };

        for (period, _) in instance.msg_samples.iter() {
            let period = *period;
            let clone = instance.clone();
            // kick off runner
            let _ = tokio::task::spawn(async move {
                clone.run_sampler(period).await;
            });
        }

        instance
    }

    pub(crate) fn count_msg(&self) {
        self.msg_samples
            .iter()
            .for_each(|(_, count)| count.increment());
    }

    pub(crate) async fn msgs_per_s(&self) -> usize {
        let data_points: Vec<_> = self
            .msgs_per_s
            .iter()
            .map(|(_, count)| count.load(ORDER))
            .collect();
        let number_of_points = data_points.len();
        if number_of_points > 0 {
            data_points.into_iter().sum::<usize>() / number_of_points
        } else {
            // should be unreachable, since self.msgs_per_s is always > 0 len
            INITIAL_MSGS_PER_S
        }
    }

    async fn run_sampler(&self, period: Duration) {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            let _instant = interval.tick().await;
            if period == SAMPLING_INTERVAL_ONE {
                {
                    self.system.write().await.refresh_cpu();
                }
                *self.load_sample.write().await =
                    normalize(self.system.read().await.load_average());
            } else {
                // allow the one min interval to come first and wait for it to finish
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            if let Some(sample) = self.msg_samples.get(&period) {
                let load = self.load_sample.read().await;

                sample.snapshot();
                let msg_count = sample.read();

                let load_per_msg = if period == SAMPLING_INTERVAL_ONE {
                    load.one / msg_count as f64
                } else if period == SAMPLING_INTERVAL_FIVE {
                    load.five / msg_count as f64
                } else if period == SAMPLING_INTERVAL_FIFTEEN {
                    load.fifteen / msg_count as f64
                } else {
                    MAX_CPU_LOAD / INITIAL_MSGS_PER_S as f64
                };

                let max_msgs_to_handle = MAX_CPU_LOAD / load_per_msg;
                if let Some(counter) = self.msgs_per_s.get(&period) {
                    counter.store(
                        usize::max(INITIAL_MSGS_PER_S, max_msgs_to_handle as usize),
                        ORDER,
                    )
                }
            }
        }
    }
}

fn normalize(load: LoadAvg) -> LoadAvg {
    // Normalize the reading (e.g. `load=4` when `cores=4` => `normalized_load=1`)
    let cores = num_cpus::get_physical() as f64;
    LoadAvg {
        one: load.one / cores,
        five: load.five / cores,
        fifteen: load.fifteen / cores,
    }
}

#[derive(Clone)]
struct MsgCount {
    running: Arc<AtomicUsize>,
    snapshot: Arc<AtomicUsize>,
}

impl MsgCount {
    pub(crate) fn new() -> Self {
        Self {
            running: Arc::new(AtomicUsize::new(0)),
            snapshot: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) fn read(&self) -> usize {
        self.snapshot.load(ORDER)
    }

    pub(crate) fn increment(&self) {
        let _ = self.running.fetch_add(1, ORDER);
    }

    pub(crate) fn snapshot(&self) {
        if let Ok(previous_val) = self.running.fetch_update(ORDER, ORDER, |_| Some(0)) {
            self.snapshot.store(previous_val, ORDER);
        }
    }
}
