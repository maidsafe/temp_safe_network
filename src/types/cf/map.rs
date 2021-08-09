// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use dashmap::{mapref::entry::Entry, DashMap};
use std::{collections::BTreeMap, hash::Hash, sync::Arc};

///
#[derive(Debug)]
pub struct CFMap<K, V>
where
    K: Eq + Hash,
{
    states: DashMap<K, Arc<V>>,
}

impl<K, V> Default for CFMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> CFMap<K, V> {
        CFMap::new()
    }
}

impl<K, V> CFMap<K, V>
where
    K: Eq + Hash,
{
    ///
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }

    ///
    pub fn from(map: BTreeMap<K, V>) -> Self {
        let states = DashMap::new();
        for (k, v) in map {
            let _ = states.insert(k, Arc::new(v));
        }
        Self { states }
    }

    ///
    pub async fn clone(&self) -> BTreeMap<K, V>
    where
        K: Clone + Ord,
        V: Clone,
    {
        let mut map = BTreeMap::new();
        for pair in self.states.iter() {
            let key = pair.key().clone();
            let value = pair.value().as_ref().clone();
            let _ = map.insert(key, value);
        }
        map
    }

    ///
    pub async fn get(&self, key: &K) -> Option<Arc<V>> {
        self.states.get(key).map(|r| r.value().clone())
    }

    ///
    pub fn len(&self) -> usize {
        self.states.len()
    }

    ///
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    ///
    pub async fn values(&self) -> Vec<Arc<V>> {
        let mut values = vec![];
        for pair in self.states.iter() {
            values.push(pair.value().clone())
        }
        values
    }

    ///
    pub async fn insert(&self, key: K, item: V) -> Option<Arc<V>> {
        self.states.insert(key, Arc::new(item))
    }

    ///
    pub async fn insert_if<FOcc>(&self, key: K, item: V, mut condition: FOcc) -> bool
    where
        FOcc: FnMut((&Arc<V>, &V)) -> bool,
    {
        match self.states.entry(key) {
            Entry::Vacant(entry) => {
                let _ = entry.insert(Arc::new(item));
            }
            Entry::Occupied(mut entry) => {
                let e = entry.get();
                if condition((e, &item)) {
                    let _ = entry.insert(Arc::new(item));
                } else {
                    return false;
                }
            }
        }
        true
    }

    ///
    pub fn retain(&self, mut f: impl FnMut(&K) -> bool) {
        self.states.retain(|key, _| f(key))
    }

    ///
    pub async fn any<F>(&self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut((&K, &Arc<V>)) -> bool,
    {
        let mut any = false;
        for pair in self.states.iter() {
            let value = pair.value();
            if f((pair.key(), value)) {
                any = true;
                break;
            }
        }

        any
    }

    ///
    pub async fn any_value<F>(&self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(&Arc<V>) -> bool,
    {
        let mut any = false;
        for pair in self.states.iter() {
            let value = pair.value();
            if f(value) {
                any = true;
                break;
            }
        }

        any
    }

    ///
    pub async fn all<F>(&self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(&Arc<V>) -> bool,
    {
        let mut all = true;
        for pair in self.states.iter() {
            let value = pair.value();
            if !f(value) {
                all = false;
                break;
            }
        }

        all
    }

    ///
    pub async fn count<P>(&self, mut predicate: P) -> usize
    where
        Self: Sized,
        P: FnMut(&Arc<V>) -> bool,
    {
        let mut count = 0;
        for pair in self.states.iter() {
            let value = pair.value();
            if predicate(value) {
                count += 1;
                break;
            }
        }

        count
    }
}
