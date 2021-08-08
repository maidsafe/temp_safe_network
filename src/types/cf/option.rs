// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use dashmap::DashMap;
use std::sync::Arc;

///
#[derive(Debug)]
pub struct CFOption<V> {
    states: DashMap<usize, Arc<V>>,
}

impl<V> CFOption<V> {
    ///
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }

    ///
    pub fn get(&self) -> Option<Arc<V>> {
        self.states.get(&0).map(|r| r.value().clone())
    }

    ///
    pub fn set(&self, item: V) {
        let mut len = self.states.len();
        let mut res = self.states.insert(len, Arc::new(item));
        while let Some(previous) = res.take() {
            if let Some(item) = self.states.insert(len, previous) {
                len += 1;
                res = self.states.insert(len, item);
            }
        }
    }
}
