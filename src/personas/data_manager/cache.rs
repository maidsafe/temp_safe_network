// Copyright 2017 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use super::STATUS_LOG_INTERVAL;
use super::data::{Data, DataId, VersionedDataId};
use GROUP_SIZE;
use maidsafe_utilities::{self, serialisation};
use routing::{Authority, ImmutableData, MessageId, MutableData, RoutingTable, XorName};
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::hash_map::Entry;
use std::ops::Add;
use std::time::{Duration, Instant};

/// The timeout for cached data from requests; if no consensus is reached, the data is dropped.
const PENDING_WRITE_TIMEOUT_SECS: u64 = 60;
/// The timeout for retrieving data chunks from individual peers.
const GET_FROM_DATA_HOLDER_TIMEOUT_SECS: u64 = 60;

pub struct Cache {
    /// Chunks we are no longer responsible for. These can be deleted from the chunk store.
    unneeded_chunks: VecDeque<DataId>,
    /// Maps the peers to the set of data chunks that we need and we know they hold.
    data_holders: HashMap<XorName, HashSet<VersionedDataId>>,
    /// Maps the peers to the data chunks we requested from them, and the timestamp of the request.
    ongoing_gets: HashMap<XorName, (Instant, VersionedDataId, MessageId)>,
    /// Maps data identifiers to the list of pending writes that affect that chunk.
    pending_writes: HashMap<DataId, Vec<PendingWrite>>,
    ongoing_gets_count: usize,
    data_holder_items_count: usize,
    logging_time: Instant,
}

impl Cache {
    pub fn handle_get_idata_success(&mut self,
                                    src: XorName,
                                    data_name: &XorName,
                                    msg_id: MessageId) {
        let _ = self.remove_ongoing_get(src, msg_id);

        let vid = (DataId::Immutable(*data_name), 0);
        for (_, vids) in &mut self.data_holders {
            let _ = vids.remove(&vid);
        }
    }

    pub fn handle_get_idata_failure(&mut self, src: XorName, msg_id: MessageId) -> bool {
        self.remove_ongoing_get(src, msg_id)
    }

    pub fn needed_data(&mut self) -> Vec<(XorName, VersionedDataId)> {
        let empty_holders: Vec<_> = self.data_holders
            .iter()
            .filter(|&(_, vids)| vids.is_empty())
            .map(|(holder, _)| *holder)
            .collect();
        for holder in empty_holders {
            let _ = self.data_holders.remove(&holder);
        }

        let expired_gets: Vec<_> = self.ongoing_gets
            .iter()
            .filter(|&(_, &(ref timestamp, _, _))| {
                timestamp.elapsed().as_secs() > GET_FROM_DATA_HOLDER_TIMEOUT_SECS
            })
            .map(|(holder, _)| *holder)
            .collect();
        for holder in expired_gets {
            let _ = self.ongoing_gets.remove(&holder);
        }

        let mut outstanding_data_ids: HashSet<_> = self.ongoing_gets
            .values()
            .map(|&(_, (data_id, _), _)| data_id)
            .collect();

        let idle_holders: Vec<_> = self.data_holders
            .keys()
            .filter(|holder| !self.ongoing_gets.contains_key(holder))
            .cloned()
            .collect();

        let mut candidates = Vec::new();

        for idle_holder in idle_holders {
            if let Some(vids) = self.data_holders.get_mut(&idle_holder) {
                if let Some(&vid) = vids.iter()
                    .find(|&&(ref data_id, _)| !outstanding_data_ids.contains(data_id)) {
                    let _ = vids.remove(&vid);
                    let (data_id, _) = vid;
                    let _ = outstanding_data_ids.insert(data_id);

                    candidates.push((idle_holder, vid));
                }
            }
        }

        candidates
    }

    pub fn is_in_unneeded(&self, data_id: &DataId) -> bool {
        self.unneeded_chunks.contains(data_id)
    }

    pub fn add_as_unneeded(&mut self, data_id: DataId) {
        self.unneeded_chunks.push_back(data_id);
    }

    pub fn prune_unneeded_chunks(&mut self, routing_table: &RoutingTable<XorName>) -> u64 {
        let pruned_unneeded_chunks: HashSet<_> = self.unneeded_chunks
            .iter()
            .filter(|data_id| routing_table.is_closest(data_id.name(), GROUP_SIZE))
            .cloned()
            .collect();

        if !pruned_unneeded_chunks.is_empty() {
            self.unneeded_chunks.retain(|data_id| !pruned_unneeded_chunks.contains(data_id));
        }

        pruned_unneeded_chunks.len() as u64
    }

    pub fn add_records(&mut self, vid: VersionedDataId, holders: HashSet<XorName>) {
        for holder in holders {
            let _ = self.data_holders.entry(holder).or_insert_with(HashSet::new).insert(vid);
        }
    }

    pub fn register_data_with_holder(&mut self, src: &XorName, vid: &VersionedDataId) -> bool {
        if self.data_holders.values().any(|vids| vids.contains(vid)) {
            let _ = self.data_holders.entry(*src).or_insert_with(HashSet::new).insert(*vid);
            true
        } else {
            false
        }
    }

    /// Removes entries from `data_holders` that are no longer valid due to churn.
    pub fn prune_data_holders(&mut self, routing_table: &RoutingTable<XorName>) {
        let mut empty_holders = Vec::new();
        for (holder, vids) in &mut self.data_holders {
            let lost_vids: Vec<_> = vids.iter()
                .filter(|&&(ref data_id, _)| {
                    // The data needs to be removed if either we are not close to it anymore, i. e.
                    // other_closest_names returns None, or `holder` is not in it anymore.
                    routing_table.other_closest_names(data_id.name(), GROUP_SIZE)
                        .map_or(true, |group| !group.contains(&holder))
                })
                .cloned()
                .collect();

            for lost_vid in lost_vids {
                let _ = vids.remove(&lost_vid);
            }

            if vids.is_empty() {
                empty_holders.push(*holder);
            }
        }

        for holder in empty_holders {
            let _ = self.data_holders.remove(&holder);
        }
    }

    pub fn insert_into_ongoing_gets(&mut self,
                                    idle_holder: XorName,
                                    vid: VersionedDataId,
                                    msg_id: MessageId) {
        let _ = self.ongoing_gets.insert(idle_holder, (Instant::now(), vid, msg_id));
    }

    /// Remove entries from `ongoing_gets` that are no longer responsible for the data or that
    /// disconnected.
    pub fn prune_ongoing_gets(&mut self, routing_table: &RoutingTable<XorName>) -> bool {
        let lost_gets: Vec<_> = self.ongoing_gets
            .iter()
            .filter(|&(holder, &(_, (ref data_id, _), _))| {
                routing_table.other_closest_names(data_id.name(), GROUP_SIZE)
                    .map_or(true, |group| !group.contains(&holder))
            })
            .map(|(holder, _)| *holder)
            .collect();

        if !lost_gets.is_empty() {
            for holder in lost_gets {
                let _ = self.ongoing_gets.remove(&holder);
            }
            return true;
        }

        false
    }

    pub fn chain_records_in_cache<I>(&self, records_in_store: I) -> HashSet<VersionedDataId>
        where I: IntoIterator<Item = VersionedDataId>
    {
        let mut records: HashSet<_> = self.data_holders
            .values()
            .flat_map(|vids| vids.iter().cloned())
            .chain(self.ongoing_gets.values().map(|&(_, vid, _)| vid))
            .chain(records_in_store)
            .collect();

        for data_id in &self.unneeded_chunks {
            let _ = records.remove(&(*data_id, 0));
        }

        records
    }

    /// Inserts the given data as a pending write to the chunk store. If it is the first for that
    /// data identifier, it returns a refresh message to send to ourselves as a group.
    pub fn insert_pending_write(&mut self,
                                mutation: PendingMutation,
                                src: Authority<XorName>,
                                dst: Authority<XorName>,
                                msg_id: MessageId,
                                rejected: bool)
                                -> Option<RefreshData> {
        let hash_pair = match serialisation::serialise(&mutation) {
            Err(_) => return None,
            Ok(serialised) => serialised,
        };

        let hash = maidsafe_utilities::big_endian_sip_hash(&hash_pair);
        let (data_id, version) = mutation.versioned_data_id();
        let pending_write = PendingWrite {
            hash: hash,
            mutation: mutation,
            timestamp: Instant::now(),
            src: src,
            dst: dst,
            message_id: msg_id,
            rejected: rejected,
        };

        let mut writes = self.pending_writes.entry(data_id).or_insert_with(Vec::new);
        let result = if !rejected && writes.iter().all(|pending_write| pending_write.rejected) {
            Some(RefreshData {
                versioned_data_id: (data_id, version),
                hash: hash,
            })
        } else {
            None
        };

        writes.insert(0, pending_write);
        result
    }

    /// Removes and returns all pending writes for the specified data identifier from the cache.
    pub fn take_pending_writes(&mut self, data_id: &DataId) -> Vec<PendingWrite> {
        self.pending_writes.remove(data_id).unwrap_or_else(Vec::new)
    }

    /// Removes and returns all timed out pending writes.
    pub fn remove_expired_writes(&mut self) -> Vec<PendingWrite> {
        let timeout = Duration::from_secs(PENDING_WRITE_TIMEOUT_SECS);
        let expired_writes: Vec<_> = self.pending_writes
            .iter_mut()
            .flat_map(|(_, writes)| {
                writes.iter()
                    .position(|write| write.timestamp.elapsed() > timeout)
                    .map_or_else(Vec::new, |index| writes.split_off(index))
                    .into_iter()
            })
            .collect();

        let expired_keys: Vec<_> = self.pending_writes
            .iter_mut()
            .filter(|entry| entry.1.is_empty())
            .map(|(data_id, _)| *data_id)
            .collect();

        for data_id in expired_keys {
            let _ = self.pending_writes.remove(&data_id);
        }

        expired_writes
    }

    pub fn pop_unneeded_chunk(&mut self) -> Option<DataId> {
        self.unneeded_chunks.pop_front()
    }

    pub fn print_stats(&mut self) {
        if self.logging_time.elapsed().as_secs() < STATUS_LOG_INTERVAL {
            return;
        }
        self.logging_time = Instant::now();

        let new_ongoing_gets_count = self.ongoing_gets.len();
        let new_data_holder_items_count =
            self.data_holders.values().map(HashSet::len).fold(0, Add::add);

        if new_ongoing_gets_count != self.ongoing_gets_count ||
           new_data_holder_items_count != self.data_holder_items_count {
            self.ongoing_gets_count = new_ongoing_gets_count;
            self.data_holder_items_count = new_data_holder_items_count;

            info!("Cache Stats: Expecting {} Get responses. {} entries in data_holders.",
                  new_ongoing_gets_count,
                  new_data_holder_items_count);
        }
    }

    fn remove_ongoing_get(&mut self, src: XorName, msg_id: MessageId) -> bool {
        let mut remove = false;
        if let Entry::Occupied(entry) = self.ongoing_gets.entry(src) {
            remove = {
                let &(_, _, ongoing_msg_id) = entry.get();
                ongoing_msg_id == msg_id
            };

            if remove {
                let _ = entry.remove_entry();
            }
        }

        remove
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            unneeded_chunks: VecDeque::new(),
            data_holders: HashMap::new(),
            ongoing_gets: HashMap::new(),
            pending_writes: HashMap::new(),
            logging_time: Instant::now(),
            ongoing_gets_count: 0,
            data_holder_items_count: 0,
        }
    }
}

// A pending write to the chunk store. This is cached in memory until the group either reaches
// consensus and stores the chunk, or it times out and is dropped.
pub struct PendingWrite {
    pub mutation: PendingMutation,
    pub hash: u64,
    timestamp: Instant,
    pub src: Authority<XorName>,
    pub dst: Authority<XorName>,
    pub message_id: MessageId,
    pub rejected: bool,
}

#[derive(RustcEncodable)]
pub enum PendingMutation {
    PutIData(ImmutableData),
    PutMData(MutableData),
    MutateMDataEntries(MutableData),
    SetMDataUserPermissions(MutableData),
    DelMDataUserPermissions(MutableData),
    ChangeMDataOwner(MutableData),
}

impl PendingMutation {
    pub fn data_id(&self) -> DataId {
        match *self {
            PendingMutation::PutIData(ref data) => DataId::immutable(data),
            PendingMutation::PutMData(ref data) |
            PendingMutation::MutateMDataEntries(ref data) |
            PendingMutation::SetMDataUserPermissions(ref data) |
            PendingMutation::DelMDataUserPermissions(ref data) |
            PendingMutation::ChangeMDataOwner(ref data) => DataId::mutable(data),
        }
    }

    pub fn versioned_data_id(&self) -> VersionedDataId {
        match *self {
            PendingMutation::PutIData(ref data) => (DataId::immutable(data), 0),
            PendingMutation::PutMData(ref data) |
            PendingMutation::MutateMDataEntries(ref data) |
            PendingMutation::SetMDataUserPermissions(ref data) |
            PendingMutation::DelMDataUserPermissions(ref data) |
            PendingMutation::ChangeMDataOwner(ref data) => (DataId::mutable(data), data.version()),
        }
    }

    pub fn mutation_type(&self) -> PendingMutationType {
        match *self {
            PendingMutation::PutIData(_) => PendingMutationType::PutIData,
            PendingMutation::PutMData(_) => PendingMutationType::PutMData,
            PendingMutation::MutateMDataEntries(_) => PendingMutationType::MutateMDataEntries,
            PendingMutation::SetMDataUserPermissions(_) => {
                PendingMutationType::SetMDataUserPermissions
            }
            PendingMutation::DelMDataUserPermissions(_) => {
                PendingMutationType::DelMDataUserPermissions
            }
            PendingMutation::ChangeMDataOwner(_) => PendingMutationType::ChangeMDataOwner,
        }
    }

    pub fn into_data(self) -> Data {
        match self {
            PendingMutation::PutIData(data) => Data::Immutable(data),
            PendingMutation::PutMData(data) |
            PendingMutation::MutateMDataEntries(data) |
            PendingMutation::SetMDataUserPermissions(data) |
            PendingMutation::DelMDataUserPermissions(data) |
            PendingMutation::ChangeMDataOwner(data) => Data::Mutable(data),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PendingMutationType {
    PutIData,
    PutMData,
    MutateMDataEntries,
    SetMDataUserPermissions,
    DelMDataUserPermissions,
    ChangeMDataOwner,
}

/// A message from the group to itself to store the given data. If this accumulates, that means a
/// quorum of group members approves.
#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Copy, Clone)]
pub struct RefreshData {
    pub versioned_data_id: VersionedDataId,
    pub hash: u64,
}

/// A list of data held by the sender. Sent from node to node.
#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct RefreshDataList(pub Vec<VersionedDataId>);
