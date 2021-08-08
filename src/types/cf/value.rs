// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use std::sync::Arc;
use tokio::sync::RwLock;

///
#[derive(Debug)]
pub struct CFValue<V> {
    value: RwLock<Arc<V>>,
}

impl<V> CFValue<V> {
    ///
    pub fn new(v: V) -> Self {
        Self {
            value: RwLock::new(Arc::new(v)),
        }
    }

    ///
    pub async fn clone(&self) -> V
    where
        V: Clone,
    {
        let c = self.value.read().await;
        c.as_ref().clone()
    }

    ///
    pub async fn get(&self) -> Arc<V> {
        let c = self.value.read().await;
        c.clone()
    }

    ///
    pub async fn set(&self, item: V) {
        let mut c = self.value.write().await;
        *c = Arc::new(item);
    }
}
