// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use dashmap::DashMap;
use itertools::Itertools;
use std::sync::Arc;

///
#[derive(Debug)]
pub struct CFSet<T> {
    states: DashMap<usize, Arc<T>>,
}

impl<T> CFSet<T> {
    ///
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }

    ///
    pub fn first(&self) -> Option<Arc<T>> {
        self.states.get(&0).map(|r| r.value().clone())
    }

    ///
    pub fn last(&self) -> Option<Arc<T>> {
        let key = self.states.len().checked_sub(1).unwrap_or_default();
        self.states.get(&key).map(|r| r.value().clone())
    }

    ///
    pub fn len(&self) -> usize {
        self.states.len()
    }

    ///
    pub fn values(&self) -> Vec<Arc<T>> {
        self.states.iter().map(|r| r.value().clone()).collect_vec()
    }

    ///
    pub fn push(&self, item: T) {
        let mut len = self.states.len();
        let mut res = self.states.insert(len, Arc::new(item));
        while let Some(previous) = res.take() {
            if let Some(item) = self.states.insert(len, previous) {
                len += 1;
                res = self.states.insert(len, item);
            }
        }
    }

    ///
    pub fn pop_front(&self) -> Option<Arc<T>> {
        let (_, value) = self.states.remove(&0)?;
        Some(value)
    }

    ///
    pub fn all<F>(&self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Arc<T>) -> bool,
    {
        self.states.iter().map(|r| r.value().clone()).all(f)
    }

    ///
    pub fn count<P>(&self, predicate: P) -> usize
    where
        Self: Sized,
        P: FnMut(&Arc<T>) -> bool,
    {
        self.states
            .iter()
            .map(|r| r.value().clone())
            .map(|v| v)
            .filter(predicate)
            .count()
    }
}
