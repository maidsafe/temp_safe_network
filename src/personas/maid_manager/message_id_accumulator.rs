// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use lru_time_cache::LruCache;
use routing::{XorName, QUORUM_DENOMINATOR, QUORUM_NUMERATOR};
use std::collections::BTreeSet;
use std::time::Duration;

pub struct MessageIdAccumulator<K> {
    group_size: usize,
    map: LruCache<K, BTreeSet<XorName>>,
}

impl<K> MessageIdAccumulator<K>
where
    K: Clone + Ord,
{
    pub fn new(group_size: usize, duration: Duration) -> Self {
        MessageIdAccumulator {
            group_size,
            map: LruCache::with_expiry_duration(duration),
        }
    }

    pub fn add(&mut self, key: K, src_name: XorName) -> Option<K> {
        let done = {
            let src_list = self.map.entry(key.clone()).or_insert_with(Default::default);
            let _ = src_list.insert(src_name);
            src_list.len() * QUORUM_DENOMINATOR > self.group_size * QUORUM_NUMERATOR
        };

        if done {
            let _ = self.map.remove(&key);
            Some(key)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;
    use routing::XorName;

    #[test]
    fn smoke() {
        let mut accumulator = MessageIdAccumulator::new(8, Duration::from_secs(10));
        let msg_id = 0;
        let duplicate_sender = XorName(rand::random());
        assert_eq!(accumulator.add(msg_id, XorName(rand::random())), None);
        assert_eq!(accumulator.add(msg_id, duplicate_sender), None);
        assert_eq!(accumulator.add(msg_id, XorName(rand::random())), None);
        assert_eq!(accumulator.add(msg_id, XorName(rand::random())), None);
        assert_eq!(accumulator.add(msg_id, duplicate_sender), None);
        assert_eq!(
            accumulator.add(msg_id, XorName(rand::random())),
            Some(msg_id)
        );
    }
}
