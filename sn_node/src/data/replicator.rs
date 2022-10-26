// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_interface::types::{DataAddress, Peer};

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};

pub(crate) struct ReplicationJob {
    pub(crate) data_address: DataAddress,
    pub(crate) recipients: BTreeSet<Peer>,
}

#[derive(Debug)]
pub(crate) struct ReplicationQueue {
    // the data, and all peers who we will send it to
    queue: Mutex<BTreeMap<DataAddress, BTreeSet<Peer>>>,
}

impl ReplicationQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) fn enqueue(&self, recipient: Peer, complementing_data: Vec<DataAddress>) {
        for data in complementing_data {
            trace!("data being enqueued for replication {:?}", data);
            // -> lock <-
            let mut guarded_queue = match self.queue.lock() {
                Ok(queue) => queue,
                Err(error) => {
                    error!("Apparently, something went wrong on another thread: {error}");
                    error.into_inner() // we can continue, because what else can we do (this part is probably not even reachable with current code)
                }
            };
            if let Some(peers_set) = guarded_queue.get_mut(&data) {
                debug!("data already queued, adding peer");
                let _existed = peers_set.insert(recipient);
            } else {
                let mut peers_set = BTreeSet::new();
                let _existed = peers_set.insert(recipient);
                let _existed = guarded_queue.insert(data, peers_set);
            };
            // -> end of block, lock released <-
        }
    }

    // Choose a random enqueued item
    pub(crate) async fn pop_random(&self) -> Option<ReplicationJob> {
        use rand::seq::IteratorRandom;
        let mut rng = rand::rngs::OsRng;

        // -> lock <-
        let mut guarded_queue = match self.queue.lock() {
            Ok(queue) => queue,
            Err(error) => {
                error!("Apparently, something went wrong on another thread: {error}");
                error.into_inner() // we can continue, because what else can we do (this part is probably not even reachable with current code)
            }
        };

        let random_queued_data = guarded_queue
            .iter()
            .choose(&mut rng)
            .map(|(address, _)| *address);

        let data_address = match random_queued_data {
            None => return None,
            Some(address) => address,
        };

        let recipients = match guarded_queue.remove(&data_address) {
            None => return None,
            Some(target_peers) => target_peers,
        };

        if recipients.is_empty() {
            // strange case, even reachable?
            return None;
        }

        Some(ReplicationJob {
            data_address,
            recipients,
        })
    }
}
