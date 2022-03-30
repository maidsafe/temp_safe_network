// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::sync::Arc;
use sysinfo::{LoadAvg, RefreshKind, System, SystemExt};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub(super) struct LoadSampling {
    system: Arc<RwLock<System>>,
    load_sample: Arc<RwLock<LoadAvg>>,
    cores: usize,
}

impl LoadSampling {
    pub(super) fn new() -> Self {
        let cores = num_cpus::get_physical();
        debug!("num physical cores: {}", cores);

        let mut system = System::new_with_specifics(RefreshKind::new());
        system.refresh_cpu();

        let load_sample = Arc::new(RwLock::new(normalize(system.load_average(), cores)));

        Self {
            system: Arc::new(RwLock::new(system)),
            load_sample,
            cores,
        }
    }

    pub(super) async fn sample(&self) -> LoadAvg {
        {
            self.system.write().await.refresh_cpu();
        }

        let value = normalize(self.system.read().await.load_average(), self.cores);
        *self.load_sample.write().await = value.clone();

        value
    }

    #[allow(unused)]
    pub(super) async fn value(&self) -> LoadAvg {
        (*self.load_sample.read().await).clone()
    }
}

fn normalize(load: LoadAvg, cores: usize) -> LoadAvg {
    // Normalize the reading (e.g. `load=4` when `cores=4` => `normalized_load=1`)
    let cores = cores as f64;
    LoadAvg {
        one: 100.0 * f64::max(0.2, load.one) / cores,
        five: 100.0 * f64::max(0.2, load.five) / cores,
        fifteen: 100.0 * f64::max(0.2, load.fifteen) / cores,
    }
}
