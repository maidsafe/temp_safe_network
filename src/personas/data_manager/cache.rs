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
use super::data::{Data, DataId};
use GROUP_SIZE;
use routing::{Authority, ImmutableData, MessageId, MutableData, RoutingTable, Value, XorName};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use utils::{self, SecureHash};

/// The timeout for cached data from requests; if no consensus is reached, the data is dropped.
const PENDING_WRITE_TIMEOUT_SECS: u64 = 60;
/// The timeout for retrieving data fragments from individual peers.
const FRAGMENT_REQUEST_TIMEOUT_SECS: u64 = 60;
/// The timeout after which cached mutable data entries expire.
const MDATA_ENTRY_TIMEOUT_SECS: u64 = 60;

pub struct Cache {
    /// Chunks we are no longer responsible for. These can be deleted from the chunk store.
    unneeded_chunks: VecDeque<DataId>,
    /// Maps the peers to the data fragments we need from them and tracks any ongoing
    /// requests to retrieve those fragments.
    needed_fragments: HashMap<XorName, HashMap<FragmentInfo, FragmentRequest>>,
    /// Maps data identifiers to the list of pending writes that affect that chunk.
    pending_writes: HashMap<DataId, Vec<PendingWrite>>,
    /// Mutable data entries that arrived before the data shell.
    mdata_entries: HashMap<(XorName, u64), HashMap<Vec<u8>, (Value, Instant)>>,

    total_needed_fragments_count: usize,
    requested_needed_fragments_count: usize,
    logging_time: Instant,
}

impl Cache {
    /// Returns data fragments we need but have not requested yet.
    pub fn unrequested_needed_fragments(&mut self) -> Vec<(XorName, FragmentInfo)> {
        // Reset expired requests
        for request in self.needed_fragments
                .iter_mut()
                .flat_map(|(_, fragments)| fragments.values_mut()) {
            request.stop_if_expired();
        }

        // TODO (adam): should we exclude fragments of uneeded chunks?

        // Return all fragments that do not already have request ongoing.
        let mut result = HashMap::new();
        for (holder, fragments) in &self.needed_fragments {
            for fragment in fragments.keys() {
                let _ = result.insert(fragment.clone(), *holder);
            }
        }

        for (_, fragments) in &self.needed_fragments {
            for (fragment, request) in fragments {
                if request.is_ongoing() {
                    let _ = result.remove(fragment);
                }
            }
        }

        result
            .into_iter()
            .map(|(fragment, holder)| (holder, fragment))
            .collect()
    }

    /// Returns all data fragments we need.
    pub fn needed_fragments(&self) -> HashSet<FragmentInfo> {
        self.needed_fragments
            .values()
            .flat_map(|fragments| fragments.keys().cloned())
            .filter(|fragment| !self.unneeded_chunks.contains(&fragment.data_id()))
            .collect()
    }

    /// Insert new needed fragment and register it with the given holder.
    /// Returns true if the fragment hasn't been previously registered with the holder,
    /// false otherwise.
    pub fn insert_needed_fragment(&mut self, fragment: FragmentInfo, holder: XorName) -> bool {
        self.needed_fragments
            .entry(holder)
            .or_insert_with(HashMap::new)
            .insert(fragment, FragmentRequest::new())
            .is_none()
    }

    /// Register the given fragment with the new holder, but only if we already
    /// have it registered with some other holder(s).
    pub fn register_needed_fragment_with_holder(&mut self,
                                                fragment: FragmentInfo,
                                                holder: XorName)
                                                -> bool {
        if self.needed_fragments
               .values()
               .any(|fragments| fragments.contains_key(&fragment)) {
            self.insert_needed_fragment(fragment, holder)
        } else {
            false
        }
    }

    /// Register all existing needed fragments belonging to the given data with the new holder.
    pub fn register_needed_data_with_holder(&mut self, data_id: &DataId, holder: XorName) -> bool {
        let fragments: Vec<_> = self.needed_fragments
            .values()
            .flat_map(HashMap::keys)
            .filter(|fragment| fragment.data_id() == *data_id)
            .cloned()
            .collect();

        let mut result = false;

        for fragment in fragments {
            if self.insert_needed_fragment(fragment, holder) {
                result = true;
            }
        }

        result
    }

    pub fn start_needed_fragment_request(&mut self,
                                         fragment: &FragmentInfo,
                                         holder: &XorName,
                                         message_id: MessageId) {
        if let Some(request) = self.needed_fragments
               .get_mut(holder)
               .and_then(|fragments| fragments.get_mut(fragment)) {
            request.start(message_id);
        }
    }

    pub fn stop_needed_fragment_request(&mut self,
                                        holder: &XorName,
                                        message_id: MessageId)
                                        -> Option<FragmentInfo> {
        let mut remove_holder = false;

        let result = {
            let fragments = match self.needed_fragments.get_mut(holder) {
                Some(fragments) => fragments,
                None => return None,
            };


            if let Some(fragment) = fragments
                   .iter()
                   .find(|&(_, request)| request.message_id() == Some(message_id))
                   .map(|(fragment, _)| fragment.clone()) {
                let _ = fragments.remove(&fragment);
                remove_holder = fragments.is_empty();
                Some(fragment)
            } else {
                None
            }
        };

        if remove_holder {
            let _ = self.needed_fragments.remove(&holder);
        }

        result
    }

    /// Removes needed fragments that are no longer valid due to churn.
    /// Returns whether any of the pruned fragments had a request ongoing.
    pub fn prune_needed_fragments(&mut self, routing_table: &RoutingTable<XorName>) -> bool {
        let mut empty_holders = Vec::new();
        let mut result = false;

        for (holder, fragments) in &mut self.needed_fragments {
            let lost_fragments: Vec<_> = fragments
                .iter()
                .filter(|&(fragment, _)| {
                            routing_table
                                .other_closest_names(fragment.name(), GROUP_SIZE)
                                .map_or(true, |group| !group.contains(&holder))
                        })
                .map(|(fragment, request)| (fragment.clone(), *request))
                .collect();

            for (fragment, request) in lost_fragments {
                let _ = fragments.remove(&fragment);

                if request.is_ongoing() {
                    result = true;
                }
            }

            if fragments.is_empty() {
                empty_holders.push(*holder);
            }
        }

        for holder in empty_holders {
            let _ = self.needed_fragments.remove(&holder);
        }

        result
    }

    // Removes the given fragment from all holders.
    pub fn remove_needed_fragment(&mut self, fragment: &FragmentInfo) {
        let mut empty_holders = Vec::new();

        for (holder, fragments) in &mut self.needed_fragments {
            let _ = fragments.remove(fragment);

            if fragments.is_empty() {
                empty_holders.push(*holder);
            }
        }

        for holder in empty_holders {
            let _ = self.needed_fragments.remove(&holder);
        }
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
            self.unneeded_chunks
                .retain(|data_id| !pruned_unneeded_chunks.contains(data_id));
        }

        pruned_unneeded_chunks.len() as u64
    }

    /// Inserts the given mutation as a pending write. If it is the first for that
    /// data identifier, it returns a refresh message to send to ourselves as a group.
    pub fn insert_pending_write(&mut self,
                                mutation: PendingMutation,
                                src: Authority<XorName>,
                                dst: Authority<XorName>,
                                msg_id: MessageId,
                                rejected: bool)
                                -> Option<DataInfo> {
        let hash = utils::secure_hash(&mutation);

        let mut writes = self.pending_writes
            .entry(mutation.data_id())
            .or_insert_with(Vec::new);
        let result = if !rejected && writes.iter().all(|pending_write| pending_write.rejected) {
            Some(DataInfo {
                     data_id: mutation.data_id(),
                     hash: hash,
                 })
        } else {
            None
        };

        let pending_write = PendingWrite {
            hash: hash,
            mutation: mutation,
            timestamp: Instant::now(),
            src: src,
            dst: dst,
            message_id: msg_id,
            rejected: rejected,
        };
        writes.insert(0, pending_write);

        result
    }

    /// Removes and returns all pending writes for the specified data identifier from the cache.
    pub fn take_pending_writes(&mut self, data_id: &DataId) -> Vec<PendingWrite> {
        self.pending_writes
            .remove(data_id)
            .unwrap_or_else(Vec::new)
    }

    /// Removes and returns all timed out pending writes.
    pub fn remove_expired_writes(&mut self) -> Vec<PendingWrite> {
        let timeout = Duration::from_secs(PENDING_WRITE_TIMEOUT_SECS);
        let expired_writes: Vec<_> = self.pending_writes
            .iter_mut()
            .flat_map(|(_, writes)| {
                          writes
                              .iter()
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

    pub fn insert_mdata_entry(&mut self, name: XorName, tag: u64, key: Vec<u8>, value: Value) {
        self.remove_expired_mdata_entries();
        let _ = self.mdata_entries
            .entry((name, tag))
            .or_insert_with(HashMap::new)
            .insert(key, (value, Instant::now()));
    }

    pub fn take_mdata_entries(&mut self, name: XorName, tag: u64) -> HashMap<Vec<u8>, Value> {
        let result = self.mdata_entries
            .remove(&(name, tag))
            .unwrap_or_else(HashMap::new)
            .into_iter()
            .map(|(key, (value, _))| (key, value))
            .collect();

        self.remove_expired_mdata_entries();

        result
    }

    fn remove_expired_mdata_entries(&mut self) {
        let mut remove = Vec::new();

        for (data_id, entries) in &mut self.mdata_entries {
            let expired_keys: Vec<_> = entries
                .iter()
                .filter_map(|(key, &(_, instant))| if instant.elapsed().as_secs() >
                                                      MDATA_ENTRY_TIMEOUT_SECS {
                                Some(key.clone())
                            } else {
                                None
                            })
                .collect();

            for key in expired_keys {
                let _ = entries.remove(&key);
            }

            if entries.is_empty() {
                remove.push(*data_id);
            }
        }

        for data_id in remove {
            let _ = self.mdata_entries.remove(&data_id);
        }
    }

    pub fn print_stats(&mut self) {
        if self.logging_time.elapsed().as_secs() < STATUS_LOG_INTERVAL {
            return;
        }
        self.logging_time = Instant::now();

        let mut new_total = 0;
        let mut new_requested = 0;

        for request in self.needed_fragments.values().flat_map(HashMap::values) {
            new_total += 1;
            if request.is_ongoing() {
                new_requested += 1;
            }
        }

        if new_total != self.total_needed_fragments_count ||
           new_requested != self.requested_needed_fragments_count {
            self.total_needed_fragments_count = new_total;
            self.requested_needed_fragments_count = new_requested;

            info!("Cache Stats: {} requested / {} total needed fragments.",
                  new_requested,
                  new_total);
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            unneeded_chunks: VecDeque::new(),
            needed_fragments: HashMap::new(),
            pending_writes: HashMap::new(),
            mdata_entries: HashMap::new(),
            logging_time: Instant::now(),
            total_needed_fragments_count: 0,
            requested_needed_fragments_count: 0,
        }
    }
}

// A pending write to the chunk store. This is cached in memory until the group either reaches
// consensus and stores the chunk, or it times out and is dropped.
pub struct PendingWrite {
    pub mutation: PendingMutation,
    pub hash: SecureHash,
    timestamp: Instant,
    pub src: Authority<XorName>,
    pub dst: Authority<XorName>,
    pub message_id: MessageId,
    pub rejected: bool,
}

#[derive(Serialize)]
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

    pub fn fragment_infos(&self) -> Vec<FragmentInfo> {
        match *self {
            PendingMutation::PutIData(ref data) => vec![FragmentInfo::ImmutableData(*data.name())],
            PendingMutation::PutMData(ref data) |
            PendingMutation::MutateMDataEntries(ref data) |
            PendingMutation::SetMDataUserPermissions(ref data) |
            PendingMutation::DelMDataUserPermissions(ref data) |
            PendingMutation::ChangeMDataOwner(ref data) => FragmentInfo::mutable_data(data),
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

/// Information about a data fragment:
/// - immutable data,
/// - mutable data shell or,
/// - mutable data entry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum FragmentInfo {
    ImmutableData(XorName),
    MutableDataShell {
        name: XorName,
        tag: u64,
        version: u64,
        hash: SecureHash,
    },
    MutableDataEntry {
        name: XorName,
        tag: u64,
        key: Vec<u8>,
        version: u64,
        hash: SecureHash,
    },
}

impl FragmentInfo {
    /// Create `FragmentInfo` for the shell of the given mutable data.
    pub fn mutable_data_shell(data: &MutableData) -> Self {
        let hash = utils::mdata_shell_hash(data);

        FragmentInfo::MutableDataShell {
            name: *data.name(),
            tag: data.tag(),
            version: data.version(),
            hash: hash,
        }
    }

    /// Create `FragmentInfo` for the given mutable data entry.
    pub fn mutable_data_entry(data: &MutableData, key: Vec<u8>, value: &Value) -> Self {
        let hash = utils::mdata_value_hash(value);

        FragmentInfo::MutableDataEntry {
            name: *data.name(),
            tag: data.tag(),
            key: key,
            version: value.entry_version,
            hash: hash,
        }
    }

    // Get all fragments for the given mutable data.
    pub fn mutable_data(data: &MutableData) -> Vec<Self> {
        let mut result = Vec::with_capacity(1 + data.entries().len());

        result.push(Self::mutable_data_shell(data));

        for (key, value) in data.entries() {
            result.push(Self::mutable_data_entry(data, key.clone(), value));
        }

        result
    }

    pub fn name(&self) -> &XorName {
        match *self {
            FragmentInfo::ImmutableData(ref name) => name,
            FragmentInfo::MutableDataShell { ref name, .. } => name,
            FragmentInfo::MutableDataEntry { ref name, .. } => name,
        }
    }

    pub fn data_id(&self) -> DataId {
        match *self {
            FragmentInfo::ImmutableData(name) => DataId::Immutable(name),
            FragmentInfo::MutableDataShell { name, tag, .. } |
            FragmentInfo::MutableDataEntry { name, tag, .. } => DataId::Mutable(name, tag),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct FragmentRequest(Option<(Instant, MessageId)>);

impl FragmentRequest {
    fn new() -> Self {
        FragmentRequest(None)
    }

    fn start(&mut self, message_id: MessageId) {
        self.0 = Some((Instant::now(), message_id));
    }

    fn stop_if_expired(&mut self) {
        if self.is_expired() {
            self.0 = None
        }
    }

    fn is_ongoing(&self) -> bool {
        self.0.is_some()
    }

    fn is_expired(&self) -> bool {
        self.0
            .map(|(instant, _)| instant.elapsed().as_secs() > FRAGMENT_REQUEST_TIMEOUT_SECS)
            .unwrap_or(false)
    }

    fn message_id(&self) -> Option<MessageId> {
        self.0.map(|(_, message_id)| message_id)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct DataInfo {
    pub data_id: DataId,
    pub hash: SecureHash,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;

    #[test]
    fn needed_fragments() {
        let mut cache = Cache::default();

        let holder0 = rand::random();
        let holder1 = rand::random();

        let fragment0 = FragmentInfo::ImmutableData(rand::random());

        assert!(cache.needed_fragments().is_empty());
        assert!(cache.unrequested_needed_fragments().is_empty());

        // Insert a single fragment. Should be present in both collections.
        assert!(cache.insert_needed_fragment(fragment0.clone(), holder0));

        let fragments = cache.needed_fragments();
        assert_eq!(fragments.len(), 1);
        assert_eq!(unwrap!(fragments.into_iter().next()), fragment0);

        let fragments = cache.unrequested_needed_fragments();
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].0, holder0);
        assert_eq!(fragments[0].1, fragment0);

        // Insert the same fragment with the same holder again. The collections
        // should not change.
        assert!(!cache.insert_needed_fragment(fragment0.clone(), holder0));

        assert_eq!(cache.needed_fragments().len(), 1);
        assert_eq!(cache.unrequested_needed_fragments().len(), 1);

        // Insert the same fragment but with different holder. The collections
        // should still include it only once.
        assert!(cache.insert_needed_fragment(fragment0.clone(), holder1));
        assert_eq!(cache.unrequested_needed_fragments().len(), 1);

        // Start request against one holder. The fragment should not appear among the unrequested
        // fragments even though this fragment is still unrequested in different holder.
        cache.start_needed_fragment_request(&fragment0, &holder0, MessageId::new());
        assert_eq!(cache.unrequested_needed_fragments().len(), 0);
    }
}
