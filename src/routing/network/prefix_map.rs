// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Container that acts as a map whose keys are Prefixes.

use dashmap::{self, mapref::multiple::RefMulti, DashMap};
use serde::{Deserialize, Serialize};
use std::iter::Iterator;
use xor_name::{Prefix, XorName};

/// Container that acts as a map whose keys are prefixes.
///
/// It automatically prunes redundant entries. That is, when the prefix of an entry is fully
/// covered by other prefixes, that entry is removed. For example, when there is entry with
/// prefix (00) and we insert entries with (000) and (001), the (00) prefix becomes fully
/// covered and is automatically removed.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PrefixMap<T>(DashMap<Prefix, T>);

impl<T> PrefixMap<T>
where
    T: Clone,
{
    /// Create empty `PrefixMap`.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Inserts new entry into the map. Replaces previous entry at the same prefix.
    /// Removes those ancestors of the inserted prefix that are now fully covered by their
    /// descendants.
    /// Does not insert anything if any descendant of the prefix of `entry` is already present in
    /// the map.
    /// Returns a boolean indicating whether anything changed.
    pub(crate) fn insert(&mut self, prefix: Prefix, entry: T) -> bool {
        // Don't insert if any descendant is already present in the map.
        if self.descendants(&prefix).next().is_some() {
            return false;
        }

        let _ = self.0.insert(prefix, entry);

        let parent_prefix = prefix.popped();
        self.prune(parent_prefix);
        true
    }

    /// Get the entry at `prefix`, if any.
    pub(crate) fn get(&self, prefix: &Prefix) -> Option<(Prefix, T)> {
        self.0
            .get(prefix)
            .map(|entry| (*entry.key(), entry.value().clone()))
    }

    /// Get the entry at the prefix that matches `name`. In case of multiple matches, returns the
    /// one with the longest prefix.
    pub(crate) fn get_matching(&self, name: &XorName) -> Option<(Prefix, T)> {
        let max = self
            .0
            .iter()
            .filter(|item| item.key().matches(name))
            .max_by_key(|item| item.key().bit_count())?;
        Some((*max.key(), max.value().clone()))
    }

    /// Get the entry at the prefix that matches `name`. In case of multiple matches, returns the
    /// one with the longest prefix. If there are no prefixes matching the given `name`, return
    /// a prefix matching the opposite to 1st bit of `name`. If the map is empty, return None.
    pub(crate) fn try_get_matching(&self, name: &XorName) -> Option<(Prefix, T)> {
        if let Some((prefix, t)) = self
            .iter()
            .filter(|e| e.key().matches(name))
            .max_by_key(|e| e.key().bit_count())
            .map(|e| {
                let (prefix, t) = e.pair();
                (*prefix, t.clone())
            })
        {
            Some((prefix, t))
        } else {
            self.iter()
                .filter(|e| e.key().matches(&name.with_bit(0, !name.bit(0))))
                .max_by_key(|e| e.key().bit_count())
                .map(|e| {
                    let (prefix, t) = e.pair();
                    (*prefix, t.clone())
                })
        }
    }

    /// Get the entry at the prefix that matches `prefix`. In case of multiple matches, returns the
    /// one with the longest prefix.
    pub(crate) fn get_matching_prefix(&self, prefix: &Prefix) -> Option<(Prefix, T)> {
        self.get_matching(&prefix.name())
    }

    /// Returns an owning iterator over the entries
    pub(crate) fn iter(&self) -> impl Iterator<Item = RefMulti<'_, Prefix, T>> {
        self.0.iter()
    }

    /// Returns an iterator over all entries whose prefixes are descendants (extensions) of
    /// `prefix`.
    pub(crate) fn descendants<'a>(
        &'a self,
        prefix: &'a Prefix,
    ) -> impl Iterator<Item = RefMulti<'a, Prefix, T>> + 'a {
        self.0
            .iter()
            .filter(move |p| p.key().is_extension_of(prefix))
    }

    /// Remove `prefix` and any of its ancestors if they are covered by their descendants.
    /// For example, if `(00)` and `(01)` are both in the map, we can remove `(0)` and `()`.
    pub(crate) fn prune(&self, mut prefix: Prefix) {
        // TODO: can this be optimized?

        loop {
            {
                let descendants: Vec<_> = self.descendants(&prefix).collect();
                let descendant_prefixes: Vec<&Prefix> =
                    descendants.iter().map(|item| item.key()).collect();
                if prefix.is_covered_by(descendant_prefixes) {
                    let _ = self.0.remove(&prefix);
                }
            }

            if prefix.is_empty() {
                break;
            } else {
                prefix = prefix.popped();
            }
        }
    }
}

// We have to impl this manually since the derive would require T: Default, which is not necessary.
// See rust-lang/rust#26925
impl<T> Default for PrefixMap<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

// Need to impl this manually, because the derived one would use `PartialEq` of `Entry` which
// compares only the prefixes.
impl<T> PartialEq for PrefixMap<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(other.0.iter())
                .all(|(lhs, rhs)| lhs.pair() == rhs.pair())
    }
}

impl<T> Eq for PrefixMap<T> where T: Eq {}

impl<T> From<PrefixMap<T>> for DashMap<Prefix, T> {
    fn from(map: PrefixMap<T>) -> Self {
        map.0
    }
}

impl<T> IntoIterator for PrefixMap<T> {
    type Item = T;
    type IntoIter = PrefixMapIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        PrefixMapIterator(self.0.into_iter())
    }
}

/// An owning iterator over a [`PrefixMap`].
///
/// This struct is created by [`PrefixMap::into_iter`].
#[derive(custom_debug::Debug)]
pub struct PrefixMapIterator<T>(#[debug(skip)] dashmap::iter::OwningIter<Prefix, T>);

impl<T> Iterator for PrefixMapIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, t)| t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;
    use rand::Rng;

    #[test]
    fn insert_existing_prefix() {
        let mut map = PrefixMap::new();
        assert!(map.insert(prefix("0"), 1));
        assert!(map.insert(prefix("0"), 2));
        assert_eq!(map.get(&prefix("0")), Some((prefix("0"), 2)));
    }

    #[test]
    fn insert_direct_descendants_of_existing_prefix() {
        let mut map = PrefixMap::new();
        assert!(map.insert(prefix("0"), 0));

        // Insert the first sibling. Parent remain in the map.
        assert!(map.insert(prefix("00"), 1));
        assert_eq!(map.get(&prefix("00")), Some((prefix("00"), 1)));
        assert_eq!(map.get(&prefix("01")), None);
        assert_eq!(map.get(&prefix("0")), Some((prefix("0"), 0)));

        // Insert the other sibling. Parent is removed because it is now fully covered by its
        // descendants.
        assert!(map.insert(prefix("01"), 2));
        assert_eq!(map.get(&prefix("00")), Some((prefix("00"), 1)));
        assert_eq!(map.get(&prefix("01")), Some((prefix("01"), 2)));
        assert_eq!(map.get(&prefix("0")), None);
    }

    #[tokio::test]
    async fn return_opposite_prefix_if_none_matching() {
        let mut rng = rand::thread_rng();

        let mut map = PrefixMap::new();
        let _ = map.insert(prefix("0"), 1);

        // There are no matching prefixes, so return None.
        assert_eq!(
            map.get_matching(&prefix("1").substituted_in(rng.gen())),
            None
        );

        // There are no matching prefixes, so return an opposite prefix.
        assert_eq!(
            map.try_get_matching(&prefix("1").substituted_in(rng.gen())),
            Some((prefix("0"), 1))
        );

        let _ = map.insert(prefix("1"), 1);
        assert_eq!(
            map.try_get_matching(&prefix("1").substituted_in(rng.gen())),
            Some((prefix("1"), 1))
        );
    }

    #[test]
    fn insert_indirect_descendants_of_existing_prefix() {
        let mut map = PrefixMap::new();
        assert!(map.insert(prefix("0"), 0));

        assert!(map.insert(prefix("000"), 1));
        assert_eq!(map.get(&prefix("000")), Some((prefix("000"), 1)));
        assert_eq!(map.get(&prefix("001")), None);
        assert_eq!(map.get(&prefix("00")), None);
        assert_eq!(map.get(&prefix("01")), None);
        assert_eq!(map.get(&prefix("0")), Some((prefix("0"), 0)));

        assert!(map.insert(prefix("001"), 2));
        assert_eq!(map.get(&prefix("000")), Some((prefix("000"), 1)));
        assert_eq!(map.get(&prefix("001")), Some((prefix("001"), 2)));
        assert_eq!(map.get(&prefix("00")), None);
        assert_eq!(map.get(&prefix("01")), None);
        assert_eq!(map.get(&prefix("0")), Some((prefix("0"), 0)));

        assert!(map.insert(prefix("01"), 3));
        assert_eq!(map.get(&prefix("000")), Some((prefix("000"), 1)));
        assert_eq!(map.get(&prefix("001")), Some((prefix("001"), 2)));
        assert_eq!(map.get(&prefix("00")), None);
        assert_eq!(map.get(&prefix("01")), Some((prefix("01"), 3)));
        // (0) is now fully covered and so was removed
        assert_eq!(map.get(&prefix("0")), None);
    }

    #[test]
    fn insert_ancestor_of_existing_prefix() {
        let mut map = PrefixMap::new();
        let _ = map.insert(prefix("00"), 1);

        assert!(!map.insert(prefix("0"), 2));
        assert_eq!(map.get(&prefix("0")), None);
        assert_eq!(map.get(&prefix("00")), Some((prefix("00"), 1)));
    }

    #[test]
    fn get_matching() {
        let mut rng = rand::thread_rng();

        let mut map = PrefixMap::new();
        let _ = map.insert(prefix("0"), 0);
        let _ = map.insert(prefix("1"), 1);
        let _ = map.insert(prefix("10"), 10);

        assert_eq!(
            map.get_matching(&prefix("0").substituted_in(rng.gen())),
            Some((prefix("0"), 0))
        );

        assert_eq!(
            map.get_matching(&prefix("11").substituted_in(rng.gen())),
            Some((prefix("1"), 1))
        );

        assert_eq!(
            map.get_matching(&prefix("10").substituted_in(rng.gen())),
            Some((prefix("10"), 10))
        );
    }

    #[test]
    fn get_matching_prefix() {
        let mut map = PrefixMap::new();
        let _ = map.insert(prefix("0"), 0);
        let _ = map.insert(prefix("1"), 1);
        let _ = map.insert(prefix("10"), 10);

        assert_eq!(
            map.get_matching_prefix(&prefix("0")),
            Some((prefix("0"), 0))
        );

        assert_eq!(
            map.get_matching_prefix(&prefix("11")),
            Some((prefix("1"), 1))
        );

        assert_eq!(
            map.get_matching_prefix(&prefix("10")),
            Some((prefix("10"), 10))
        );

        assert_eq!(
            map.get_matching_prefix(&prefix("101")),
            Some((prefix("10"), 10))
        );
    }

    #[test]
    fn serialize_transparent() -> Result<()> {
        let mut map = PrefixMap::new();
        let _ = map.insert(prefix("0"), 0);
        let _ = map.insert(prefix("1"), 1);
        let _ = map.insert(prefix("10"), 10);

        let copy_map: DashMap<_, _> = map.clone().into();
        let serialized_copy_map = rmp_serde::to_vec(&copy_map)?;

        assert_eq!(rmp_serde::to_vec(&map)?, serialized_copy_map);
        let _ = rmp_serde::from_read::<_, PrefixMap<i32>>(&*serialized_copy_map)?;
        Ok(())
    }

    fn prefix(s: &str) -> Prefix {
        s.parse().expect("")
    }
}
