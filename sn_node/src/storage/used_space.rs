// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tracing::info;

#[derive(Clone, Debug)]
/// Tracking used space
pub struct UsedSpace {
    /// The minimum space that the network requires of the node, to allocate for storage.
    ///
    /// If this value is lower than what is currently used by the elders, the node will sooner or later risk getting kicked out
    /// (as it will fill up or fail to serve requested data).
    min_capacity: usize,
    /// The maximum (inclusive) allocated space for storage
    ///
    /// This is used by a node operator to prevent the node to fill up
    /// actual disk space beyond what the operator deems convenient.
    max_capacity: usize,
    used_space: Arc<AtomicUsize>,
}

impl UsedSpace {
    /// Create new `UsedSpace` tracker
    pub fn new(min_capacity: usize, max_capacity: usize) -> Self {
        Self {
            min_capacity,
            max_capacity,
            used_space: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) fn increase(&self, size: usize) {
        let _ = self.used_space.fetch_add(size, Ordering::Relaxed);
    }

    pub(crate) fn decrease(&self, size: usize) {
        let _ = self.used_space.fetch_sub(size, Ordering::Relaxed);
    }

    /// This prevents the node to fill up actual disk space
    /// beyond what a node operator deems convenient.
    pub(crate) fn can_add(&self, size: usize) -> bool {
        let current_used_space = self.used_space.load(Ordering::Relaxed);
        current_used_space + size <= self.max_capacity
    }

    /// Checks if we've reached the minimum expected capacity.
    pub(crate) fn has_reached_min_capacity(&self) -> bool {
        let current_used_space = self.used_space.load(Ordering::Relaxed);
        current_used_space >= self.min_capacity
    }

    #[allow(unused)]
    pub(crate) fn ratio(&self) -> f64 {
        let used = self.used_space.load(Ordering::Relaxed);
        let min_capacity = self.min_capacity;
        let used_space_ratio = used as f64 / min_capacity as f64;
        info!("Used space: {:?}", used);
        info!("Min capacity: {:?}", min_capacity);
        info!("Used space ratio: {:?}", used_space_ratio);
        used_space_ratio
    }
}
