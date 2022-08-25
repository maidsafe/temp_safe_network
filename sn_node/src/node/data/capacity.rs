// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{Prefix, XorName};
use sn_interface::messaging::data::StorageLevel;

use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};

// The number of separate copies of a chunk which should be maintained.
pub(crate) const MIN_LEVEL_WHEN_FULL: u8 = 9; // considered full when >= 90 %.

/// A util for sharing the
/// info on data capacity among the
/// chunk storing nodes in the section.
#[derive(Default)]
pub(crate) struct Capacity {
    adult_levels: BTreeMap<XorName, StorageLevel>,
}

impl Capacity {
    /// Whether the adult is considered full.
    /// This happens when it has reported at least `MIN_LEVEL_WHEN_FULL`.
    pub(crate) fn is_full(&self, adult: &XorName) -> Option<bool> {
        let level = self.adult_levels.get(adult)?.value();
        Some(level >= MIN_LEVEL_WHEN_FULL)
    }

    pub(super) fn add_new_adult(&mut self, adult: XorName) {
        info!("Adding new adult:{adult} to Capacity tracker");

        if let Some(old_entry) = self.adult_levels.insert(adult, StorageLevel::zero()) {
            let _level = old_entry.value();
            warn!("Throwing old storage level for Adult {adult}:{_level}");
        }
    }

    /// Avg usage by nodes in the section, a value between 0 and 10.
    pub(super) fn avg_usage(&self) -> u8 {
        let mut total = 0_usize;
        // not sure if necessary, but now we'll be working with an isolated snapshot:
        let levels = self.adult_levels.values().collect_vec();
        let num_adults = levels.len();
        if num_adults == 0 {
            return 0; // avoid divide by zero
        }
        for v in levels {
            total += v.value() as usize;
        }
        (total / num_adults) as u8
    }

    /// Storage levels of nodes in the section.
    pub(super) fn levels(&self) -> BTreeMap<XorName, StorageLevel> {
        let mut map = BTreeMap::new();
        for (name, level) in self.adult_levels.iter() {
            let _prev = map.insert(*name, *level);
        }
        map
    }

    /// Nodes and storage levels of nodes matching the prefix.
    pub(super) fn levels_matching(&self, prefix: Prefix) -> BTreeMap<XorName, StorageLevel> {
        self.levels()
            .iter()
            .filter(|(name, _)| prefix.matches(name))
            .map(|(name, level)| (*name, *level))
            .collect()
    }

    /// Full chunk storing nodes in the section (considered full when at >= `MIN_LEVEL_WHEN_FULL`).
    pub(super) fn full_adults(&self) -> BTreeSet<XorName> {
        let mut set = BTreeSet::new();
        for (name, level) in self.adult_levels.iter() {
            if level.value() >= MIN_LEVEL_WHEN_FULL {
                let _changed = set.insert(*name);
            }
        }
        set
    }

    pub(super) fn set_adult_levels(&mut self, levels: BTreeMap<XorName, StorageLevel>) {
        for (name, level) in levels {
            let _changed = self.set_adult_level(name, level);
        }
    }

    /// Returns whether the level changed or not.
    pub(super) fn set_adult_level(&mut self, adult: XorName, new_level: StorageLevel) -> bool {
        {
            if let Some(level) = self.adult_levels.get_mut(&adult) {
                let current_level = { level.value() };
                info!("Current level: {}", current_level);
                if new_level.value() > current_level {
                    *level = new_level;
                    info!("Old value overwritten.");
                    return true; // value changed
                }
                return false; // no change
            }
        }

        info!("No current level, aqcuiring top level write lock..");
        // locks to prevent racing
        info!("Top level write lock acquired.");
        // checking the value again, if there was a concurrent insert..
        if let Some(level) = self.adult_levels.get_mut(&adult) {
            info!("Oh wait, a value was just recorded..");
            let current_level = { level.value() };
            info!("Current level: {}", current_level);
            if new_level.value() > current_level {
                *level = new_level;
                info!("Old value overwritten.");
                return true; // value changed
            }
            false // no change
        } else {
            let _level = self.adult_levels.insert(adult, new_level);
            info!("New value inserted.");
            true // value changed
        }
    }

    /// Registered holders not present in provided list of members
    /// will be removed from `adult_levels` and no longer tracked for liveness.
    pub(super) fn retain_members_only(&mut self, members: &BTreeSet<XorName>) {
        let absent_adults: Vec<_> = self
            .adult_levels
            .iter()
            .filter(|(key, _)| !members.contains(key))
            .map(|(key, _)| *key)
            .collect();

        for adult in &absent_adults {
            let _level = self.adult_levels.remove(adult);
        }
    }
}
