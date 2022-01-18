// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use dashmap::DashMap;
use priority_queue::PriorityQueue;
use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use xor_name::XorName;

type Priority = u16;

/// An lru cache with a neat implementation to evict least recently used
/// element, by using a priority queue.
///
/// Implemented as a map of data and a priority queue. The cache stores the items inside a [`DashMap`]
/// to be able to retrieve them quickly. The key is kept in a [`PriorityQueue`], and every time it is accessed,
/// the priority is changed to a lower number.
///
/// At an insert of a new value, when the cache is full, the priority queue will simply be popped, and the least
/// recently used value will have the largest number, so it will be at the top of the queue.
#[derive(Clone, Debug)]
pub(crate) struct LruCache<T> {
    data: DashMap<XorName, Arc<T>>,
    queue: Arc<RwLock<PriorityQueue<XorName, Priority>>>,
    size: u16,
    start: Arc<AtomicU16>,
}

impl<T> LruCache<T> {
    pub(crate) fn new(size: u16) -> Self {
        Self {
            data: DashMap::new(),
            queue: Arc::new(RwLock::new(PriorityQueue::new())),
            size,
            start: Arc::new(AtomicU16::new(u16::MAX)),
        }
    }

    pub(crate) async fn insert(&self, key: &XorName, val: Arc<T>) {
        if self.data.contains_key(key) {
            return;
        }

        let _ = self.data.insert(*key, val);
        {
            let _ = self.queue.write().await.push(*key, self.priority().await);
        }

        let len = { self.queue.read().await.len() as u16 };
        if len > self.size {
            let mut write = self.queue.write().await;
            if let Some((evicted, _)) = write.pop() {
                let _ = self.data.remove(&evicted);
            }
        }
    }

    pub(crate) async fn get(&self, key: &XorName) -> Option<Arc<T>> {
        let exists = {
            let read_only = self.queue.read().await;
            read_only.get(key).is_some()
        };
        if exists {
            let _ = self
                .queue
                .write()
                .await
                .change_priority(key, self.priority().await);
        }
        self.data.get(key).as_deref().cloned()
    }

    pub(crate) async fn remove(&self, key: &XorName) {
        let _ = self.queue.write().await.remove(key);
        let _ = self.data.remove(key);
    }

    async fn priority(&self) -> Priority {
        let prio = self.start.fetch_sub(1, Ordering::SeqCst);
        if prio == 0 {
            // empty the cache when we overflow
            self.queue.write().await.clear();
            self.data.clear();
            self.start.fetch_sub(1, Ordering::SeqCst)
        } else {
            prio
        }
    }
}

#[cfg(test)]
mod test {
    use super::LruCache;

    use std::sync::Arc;
    use xor_name::XorName;

    #[tokio::test]
    async fn test_basic() {
        let cache = LruCache::new(3);

        let key_1 = &XorName::random();
        let key_2 = &XorName::random();
        let key_3 = &XorName::random();
        cache.insert(key_1, Arc::new("Strawberries")).await;
        cache.insert(key_2, Arc::new("Bananas")).await;
        cache.insert(key_3, Arc::new("Peaches")).await;

        let result_string = format!("{:?}", cache.get(key_2).await);
        let expected_string = format!("{:?}", Some("Bananas"));

        assert_eq!(result_string, expected_string);
    }

    #[tokio::test]
    async fn test_lru() {
        let cache = LruCache::new(3);

        let key_1 = &XorName::random();
        let key_2 = &XorName::random();
        let key_3 = &XorName::random();
        let key_4 = &XorName::random();
        cache.insert(key_1, Arc::new("Strawberries")).await;
        cache.insert(key_2, Arc::new("Bananas")).await;
        cache.insert(key_3, Arc::new("Peaches")).await;
        cache.insert(key_4, Arc::new("Blueberries")).await;

        let result_string = format!("{:?}", cache.get(key_1).await);
        let expected_string = format!("{:?}", None::<String>);

        assert_eq!(result_string, expected_string);
    }

    #[tokio::test]
    async fn test_remove() {
        let cache = LruCache::new(3);

        let key_1 = &XorName::random();
        let key_2 = &XorName::random();
        let key_3 = &XorName::random();
        cache.insert(key_1, Arc::new("Strawberries")).await;
        cache.insert(key_2, Arc::new("Bananas")).await;
        cache.insert(key_3, Arc::new("Peaches")).await;

        let result_string = format!("{:?}", cache.get(key_2).await);
        let expected_string = format!("{:?}", Some("Bananas"));

        assert_eq!(result_string, expected_string);

        cache.remove(key_2).await;

        let result_string = format!("{:?}", cache.get(key_2).await);
        let expected_string = format!("{:?}", None::<String>);

        assert_eq!(result_string, expected_string);
    }
}
