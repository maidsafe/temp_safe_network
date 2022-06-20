// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod event_rates;
mod load_sampling;

use self::event_rates::EventRates;
use self::load_sampling::LoadSampling;

use std::time::Duration;
use tokio::time::MissedTickBehavior;

const INITIAL_MSGS_PER_S: f64 = 500.0;
const INITIAL_CMDS_PER_S: f64 = 250.0;

const INTERVAL_ONE_MINUTE: u8 = 1;
const INTERVAL_FIFTEEN_MINUTES: u8 = 15;

/// Used to provide rate limits for the system handling of internal cmds and network msgs.
///
/// This component measures CPU loads and local activity per s, using 15 min moving average.
/// Specifically this measures cmds handled, and msgs received.
///
/// The rate limit values are updated every minute.
#[derive(Debug, Clone)]
pub(crate) struct RateLimits {
    cmd_rates: EventRates,
    msg_rates: EventRates,
    load_sampling: LoadSampling,
}

impl RateLimits {
    pub(crate) fn new() -> Self {
        let instance = Self {
            cmd_rates: EventRates::new(
                "cmds".to_string(),
                INITIAL_CMDS_PER_S,
                INTERVAL_FIFTEEN_MINUTES,
            ),
            msg_rates: EventRates::new(
                "msgs".to_string(),
                INITIAL_MSGS_PER_S,
                INTERVAL_FIFTEEN_MINUTES,
            ),
            load_sampling: LoadSampling::new(),
        };

        // start background sampler
        let clone = instance.clone();
        let _ = tokio::task::spawn_local(async move {
            clone.run_sampler().await;
        });

        instance
    }

    #[allow(unused)]
    pub(crate) async fn increment_cmds(&self) {
        self.cmd_rates.increment().await;
    }

    #[cfg(feature = "back-pressure")]
    pub(crate) async fn increment_msgs(&self) {
        self.msg_rates.increment().await;
    }

    #[allow(unused)]
    pub(crate) async fn max_cmds_per_s(&self) -> f64 {
        self.cmd_rates.max_events_per_s().await
    }

    #[cfg(feature = "back-pressure")]
    pub(crate) async fn max_msgs_per_s(&self) -> f64 {
        self.msg_rates.max_events_per_s().await
    }

    async fn run_sampler(&self) {
        let mut interval =
            tokio::time::interval(Duration::from_secs(60 * INTERVAL_ONE_MINUTE as u64));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            // sampling is done every 1 minute
            let _instant = interval.tick().await;

            let load = &self.load_sampling.sample().await;

            debug!("Load sample {:?}", load);

            // Since the rate limiters were instantiated with a 15 min period,
            // we pass in the measured avg load during past 15 min.
            self.cmd_rates.update(load.fifteen).await;
            self.msg_rates.update(load.fifteen).await;
        }
    }
}
