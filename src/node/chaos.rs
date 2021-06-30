// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Run code when a chaotic. Defaults to 20% of the time. Overall frequence can be set via "SAFE_CHAOS_LEVEL" env var.
#[macro_export]
macro_rules! with_chaos {
    ( $x:expr) => {{
        #[cfg(feature = "chaos")]
        {
            use rand::distributions::{Distribution, Uniform};
            use std::env;
            use tracing::{debug, warn};

            let mut rng = rand::thread_rng();
            // 20% chance of happening
            let chaos_trigger: usize = env::var("SAFE_CHAOS_LEVEL")
                .unwrap_or("20".to_string())
                .parse()
                .unwrap_or(20);
            let die = Uniform::from(1..100);
            let throw = die.sample(&mut rng);
            debug!(
                "Threshold for \"chaos\" to occur is < {}, we rolled: {}",
                chaos_trigger, throw
            );

            if throw <= chaos_trigger {
                // do the chaos
                warn!("Chaos!");
                $x
            }
        }
    }};
}
