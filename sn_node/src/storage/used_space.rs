// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_interface::network_knowledge::recommended_section_size;
use std::sync::{
    atomic::{AtomicU8, AtomicUsize, Ordering},
    Arc,
};
use tracing::info;

/// The StorageLevel is an n-level scale of used space,
/// where each level represents a x% increase of usage.
///
/// We add a new node for every level of used space increment.
/// The `x` is given by 100 / recommended_section_size, which gives
/// the number of increments we need to grow from a newly split section
/// (at recommended_section_size) to 2 * recommended_section_size, where we will split again.
/// We expect the variant `Updated` to contain a value between x and x * recommended_section_size,
/// where every step is a x% increase of used space.
/// With recommended_section_size = 14, we get x = 7 %.
#[derive(Debug)]
pub enum StorageLevel {
    /// Contains the %-points of the new level.
    /// This is a value between x and x * recommended_section_size, where x = 100 / recommended_section_size.
    Updated(u8),
    NoChange,
}

/// Tracking used space
#[derive(Clone, Debug)]
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
    /// The level is bumped every x% increase, where x = 100 / recommended_section_size.
    ///
    /// We add a new node for every level of used space increment.
    last_seen_level: Arc<AtomicU8>,
    /// Counts the number of bytes stored to disk.
    used_space: Arc<AtomicUsize>,
}

impl UsedSpace {
    /// Create new `UsedSpace` tracker
    pub fn new(min_capacity: usize, max_capacity: usize) -> Self {
        Self {
            min_capacity,
            max_capacity,
            used_space: Arc::new(AtomicUsize::new(0)),
            last_seen_level: Arc::new(AtomicU8::new(0)),
        }
    }

    /// Returns whether a new storage level has been passed.
    ///
    /// A storage level is an increase with x%-points.
    /// We only report a passing once per level. So if it drops,
    /// and then rises again, we don't report the same passing a second time.
    pub(crate) fn increase(&self, size: usize) -> StorageLevel {
        let _ = self.used_space.fetch_add(size, Ordering::Relaxed);
        let used_space = self.used_space.load(Ordering::Relaxed);
        let used_space_ratio = used_space as f64 / self.min_capacity as f64;
        let current_level = to_storage_level(used_space_ratio);

        // we do a relaxed check here, because this is not important to be exact, we will update soon enough
        if self.last_seen_level.load(Ordering::Relaxed) > current_level {
            // current level is not higher than what we have previously had
            return StorageLevel::NoChange;
        }

        // we do an exact check here, as to not report a change more than once
        let updated = self
            .last_seen_level
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |last_seen| {
                if current_level > last_seen {
                    Some(current_level)
                } else {
                    None
                }
            })
            .is_ok();

        if updated {
            info!("Used space: {:?}", used_space);
            info!("Min capacity: {:?}", self.min_capacity);
            info!("Used space ratio: {:?}", used_space_ratio);
            StorageLevel::Updated(current_level)
        } else {
            StorageLevel::NoChange
        }
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

    pub(crate) fn ratio(&self) -> f64 {
        let used = self.used_space.load(Ordering::Relaxed);
        let min_capacity = self.min_capacity;
        used as f64 / min_capacity as f64
    }
}

/// We expect it to return a value between 0-(100 / recommended_section_size),
/// where every step is a x% increase of the value.
/// This gives an equal number of increments as the number of nodes necessary to
/// bring a newly split section up to the size where it splits again.
fn to_storage_level(value: f64) -> u8 {
    ((value * 100.0) as usize / recommended_section_size()) as u8
}

impl Default for UsedSpace {
    fn default() -> Self {
        use crate::node::cfg::config_handler::{DEFAULT_MAX_CAPACITY, DEFAULT_MIN_CAPACITY};
        Self {
            min_capacity: DEFAULT_MIN_CAPACITY,
            max_capacity: DEFAULT_MAX_CAPACITY,
            used_space: Arc::new(AtomicUsize::new(0)),
            last_seen_level: Arc::new(AtomicU8::new(0)),
        }
    }
}
