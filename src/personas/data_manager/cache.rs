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
use super::data::{DataId, ImmutableDataId, MutableDataId};
use super::mutation::Mutation;
use GROUP_SIZE;
use routing::{Authority, MessageId, MutableData, RoutingTable, Value, XorName};
use std::collections::VecDeque;
use std::collections::hash_map::Entry;
use std::time::Duration;
use utils::{self, HashMap, HashSet, Instant, SecureHash};

/// The timeout for cached data from requests; if no consensus is reached, the data is dropped.
const PENDING_WRITE_TIMEOUT_SECS: u64 = 60;
/// The timeout for retrieving data fragments from individual peers.
const FRAGMENT_REQUEST_TIMEOUT_SECS: u64 = 60;
/// The timeout after which cached mutable data entries expire.
const MDATA_ENTRY_TIMEOUT_SECS: u64 = 60;

type CachedMDataEntries = HashMap<Vec<u8>, (Value, Instant)>;

pub struct Cache {
    /// Immutable data chunks we are no longer responsible for. These can be deleted
    /// from the chunk store.
    unneeded_chunks: UnneededChunks,

    /// Maps data fragment holders to the list of data fragments we need from them.
    /// Also tracks any ongoing requests to retrieve those fragments.
    fragment_holders: HashMap<XorName, FragmentHolder>,
    /// Maps fragments to the list of holders that have it and tracks whether there
    /// is an ongoing request to retrieve it.
    fragment_index: HashMap<FragmentInfo, FragmentState>,

    /// Maps data identifiers to the list of pending writes that affect that chunk.
    pending_writes: HashMap<DataId, Vec<PendingWrite>>,
    /// Mutable data entries that arrived before the data shell.
    mdata_entries: HashMap<(XorName, u64), CachedMDataEntries>,

    total_needed_fragments_count: usize,
    requested_needed_fragments_count: usize,
    logging_time: Instant,
}

impl Cache {
    /// Returns data fragments we need but have not requested yet.
    pub fn unrequested_needed_fragments(&mut self) -> Vec<(FragmentInfo, Vec<XorName>)> {
        self.reset_expired_requests();

        let mut result = HashMap::default();

        for (holder_name, holder) in &self.fragment_holders {
            if holder.is_requested() {
                continue;
            }

            for fragment in &holder.fragments {
                if let FragmentInfo::ImmutableData(ref name) = *fragment {
                    if self.unneeded_chunks.contains(name) {
                        continue;
                    }
                }

                if self.fragment_index
                       .get(fragment)
                       .map_or(false, |state| state.requested) {
                    continue;
                }

                result
                    .entry(fragment.clone())
                    .or_insert_with(Vec::new)
                    .push(*holder_name);
            }
        }

        result.into_iter().collect()
    }

    /// Returns all data fragments we need.
    pub fn needed_fragments(&self) -> HashSet<FragmentInfo> {
        self.fragment_index
            .keys()
            .filter(|fragment| if let FragmentInfo::ImmutableData(ref name) = **fragment {
                        !self.unneeded_chunks.contains(name)
                    } else {
                        true
                    })
            .cloned()
            .collect()
    }

    /// Insert new needed fragment and register it with the given holder.
    pub fn insert_needed_fragment(&mut self, fragment: FragmentInfo, holder: XorName) {
        if self.fragment_holders
               .entry(holder)
               .or_insert_with(FragmentHolder::new)
               .fragments
               .insert(fragment.clone()) {
            let _ = self.fragment_index
                .entry(fragment.clone())
                .or_insert_with(FragmentState::new)
                .holders
                .insert(holder);
        }
    }

    /// Register the given fragment with the new holder, but only if we already
    /// have it registered with some other holder(s).
    pub fn register_needed_fragment_with_another_holder(&mut self,
                                                        fragment: FragmentInfo,
                                                        holder: XorName)
                                                        -> bool {
        if let Some(state) = self.fragment_index.get_mut(&fragment) {
            if self.fragment_holders
                   .entry(holder)
                   .or_insert_with(FragmentHolder::new)
                   .fragments
                   .insert(fragment) {
                let _ = state.holders.insert(holder);
                return true;
            }
        }

        false
    }

    /// Register all existing needed fragments belonging to the given data with the new holder.
    pub fn register_needed_data_with_another_holder(&mut self, data_id: &DataId, holder: XorName) {
        let fragments: Vec<_> = self.fragment_index
            .keys()
            .filter_map(|fragment| if fragment.data_id() == *data_id {
                            Some(fragment.clone())
                        } else {
                            None
                        })
            .collect();

        if fragments.is_empty() {
            return;
        }

        let mut holder = self.fragment_holders
            .entry(holder)
            .or_insert_with(FragmentHolder::new);

        for fragment in fragments {
            let _ = holder.fragments.insert(fragment);
        }
    }

    pub fn start_needed_fragment_request(&mut self, fragment: &FragmentInfo, holder: &XorName) {
        if let Some(holder) = self.fragment_holders.get_mut(holder) {
            holder.start_request(fragment.clone());

            if let Some(state) = self.fragment_index.get_mut(fragment) {
                state.requested = true;
            }
        }
    }

    pub fn stop_needed_fragment_request(&mut self, holder_name: &XorName) -> Option<FragmentInfo> {
        let mut result = None;
        let mut remove = false;

        if let Some(holder) = self.fragment_holders.get_mut(holder_name) {
            if let Some(fragment) = holder.stop_request() {
                if let Some(state) = self.fragment_index.get_mut(&fragment) {
                    state.requested = false;
                    let _ = state.holders.remove(holder_name);
                }

                result = Some(fragment);
                remove = holder.fragments.is_empty();
            }
        }

        if remove {
            let _ = self.fragment_holders.remove(holder_name);
        }

        result
    }

    /// Removes needed fragments that are no longer valid due to churn.
    /// Returns whether any of the pruned fragments had a request ongoing.
    pub fn prune_needed_fragments(&mut self, routing_table: &RoutingTable<XorName>) -> bool {
        let mut lost_holders = Vec::new();
        let mut result = false;

        for (holder_name, holder) in &mut self.fragment_holders {
            let (lost, retained) = holder
                .fragments
                .drain()
                .partition(|fragment| {
                               routing_table
                                   .other_closest_names(fragment.name(), GROUP_SIZE)
                                   .map_or(true, |group| !group.contains(&holder_name))
                           });

            holder.fragments = retained;

            for fragment in &lost {
                if holder.stop_request_for(fragment) {
                    result = true;
                    break;
                }
            }

            for fragment in &lost {
                let remove = if let Some(state) = self.fragment_index.get_mut(fragment) {
                    let _ = state.holders.remove(holder_name);
                    state.holders.is_empty()
                } else {
                    false
                };

                if remove {
                    let _ = self.fragment_index.remove(fragment);
                }
            }

            if holder.fragments.is_empty() {
                lost_holders.push(*holder_name);
            }
        }

        for holder in &lost_holders {
            let _ = self.fragment_holders.remove(holder);
        }

        result
    }

    // Removes the given fragment.
    pub fn remove_needed_fragment(&mut self, fragment: &FragmentInfo) {
        if let Some(state) = self.fragment_index.remove(fragment) {
            let mut empty_holders = Vec::new();

            for holder_name in state.holders {
                if let Some(holder) = self.fragment_holders.get_mut(&holder_name) {
                    holder.stop_request_for(fragment);

                    let _ = holder.fragments.remove(fragment);
                    if holder.fragments.is_empty() {
                        empty_holders.push(holder_name);
                    }
                }
            }

            for holder_name in empty_holders {
                let _ = self.fragment_holders.remove(&holder_name);
            }
        }
    }

    pub fn is_in_unneeded(&self, data_id: &ImmutableDataId) -> bool {
        self.unneeded_chunks.contains(data_id.name())
    }

    pub fn add_as_unneeded(&mut self, data_id: ImmutableDataId) {
        self.unneeded_chunks.push(*data_id.name());
    }

    pub fn prune_unneeded_chunks(&mut self, routing_table: &RoutingTable<XorName>) -> u64 {
        let before = self.unneeded_chunks.len();

        self.unneeded_chunks
            .retain(|name| !routing_table.is_closest(name, GROUP_SIZE));

        (before - self.unneeded_chunks.len()) as u64
    }

    /// Inserts the given mutation as a pending write. If the mutation doesn't
    /// conflict with any existing pending mutations and is accepted (`rejected`
    /// is false), returns `MutationVote` to send to the other member of the group.
    /// Otherwise, returns `None`.
    pub fn insert_pending_write(&mut self,
                                mutation: Mutation,
                                src: Authority<XorName>,
                                dst: Authority<XorName>,
                                msg_id: MessageId,
                                rejected: bool)
                                -> Option<MutationVote> {
        let hash = utils::secure_hash(&mutation);
        let data_id = mutation.data_id();

        let mut writes = self.pending_writes
            .entry(data_id)
            .or_insert_with(Vec::new);

        if !rejected &&
           writes
               .iter()
               .any(|other| !other.rejected && other.mutation.conflicts_with(&mutation)) {
            return None;
        } else {
            writes.push(PendingWrite {
                            hash: hash,
                            mutation: mutation,
                            timestamp: Instant::now(),
                            src: src,
                            dst: dst,
                            message_id: msg_id,
                            rejected: rejected,
                        });
        }

        if rejected {
            None
        } else {
            Some(MutationVote {
                     data_id: data_id,
                     hash: hash,
                 })
        }
    }

    /// Removes and returns the pending write for the specified data identifier
    /// and having the specified hash.
    pub fn take_pending_write(&mut self,
                              data_id: &DataId,
                              hash: &SecureHash)
                              -> Option<PendingWrite> {
        // If there is more than one matching pending write, remove all of them,
        // but return only the non-rejected one.
        if let Entry::Occupied(mut entry) = self.pending_writes.entry(*data_id) {
            let mut accepted = Vec::new();
            let mut rejected = Vec::new();

            while let Some(index) = entry.get().iter().position(|write| write.hash == *hash) {
                let write = entry.get_mut().remove(index);
                if write.rejected {
                    rejected.push(write)
                } else {
                    accepted.push(write)
                }
            }

            if entry.get().is_empty() {
                let _ = entry.remove();
            }

            accepted
                .into_iter()
                .next()
                .or_else(|| rejected.into_iter().next())
        } else {
            None
        }
    }

    /// Removes and returns all timed out pending writes.
    pub fn remove_expired_pending_writes(&mut self) -> Vec<PendingWrite> {
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

    pub fn pop_unneeded_chunk(&mut self) -> Option<ImmutableDataId> {
        self.unneeded_chunks.pop().map(ImmutableDataId)
    }

    pub fn insert_mdata_entry(&mut self, name: XorName, tag: u64, key: Vec<u8>, value: Value) {
        self.remove_expired_mdata_entries();
        let _ = self.mdata_entries
            .entry((name, tag))
            .or_insert_with(HashMap::default)
            .insert(key, (value, Instant::now()));
    }

    pub fn take_mdata_entries(&mut self, name: XorName, tag: u64) -> HashMap<Vec<u8>, Value> {
        let result = self.mdata_entries
            .remove(&(name, tag))
            .unwrap_or_else(HashMap::default)
            .into_iter()
            .map(|(key, (value, _))| (key, value))
            .collect();

        self.remove_expired_mdata_entries();

        result
    }

    fn reset_expired_requests(&mut self) {
        for holder in self.fragment_holders.values_mut() {
            holder.reset_expired_request()
        }
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

        let new_total = self.fragment_index.len();
        let new_requested = self.fragment_holders
            .iter()
            .filter(|&(_, holder)| holder.is_requested())
            .count();

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

#[cfg(all(test, feature = "use-mock-routing"))]
impl Cache {
    /// Clear the cache.
    pub fn clear(&mut self) {
        self.unneeded_chunks.clear();
        self.fragment_holders.clear();
        self.fragment_index.clear();
        self.pending_writes.clear();
        self.mdata_entries.clear();
        self.total_needed_fragments_count = 0;
        self.requested_needed_fragments_count = 0;
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            unneeded_chunks: UnneededChunks::new(),
            fragment_holders: HashMap::default(),
            fragment_index: HashMap::default(),
            pending_writes: HashMap::default(),
            mdata_entries: HashMap::default(),
            logging_time: Instant::now(),
            total_needed_fragments_count: 0,
            requested_needed_fragments_count: 0,
        }
    }
}


// A pending write to the chunk store. This is cached in memory until the group either reaches
// consensus and stores the chunk, or it times out and is dropped.
pub struct PendingWrite {
    pub mutation: Mutation,
    pub hash: SecureHash,
    timestamp: Instant,
    pub src: Authority<XorName>,
    pub dst: Authority<XorName>,
    pub message_id: MessageId,
    pub rejected: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MutationVote {
    pub data_id: DataId,
    pub hash: SecureHash,
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
            FragmentInfo::ImmutableData(ref name) |
            FragmentInfo::MutableDataShell { ref name, .. } |
            FragmentInfo::MutableDataEntry { ref name, .. } => name,
        }
    }

    pub fn data_id(&self) -> DataId {
        match *self {
            FragmentInfo::ImmutableData(name) => DataId::Immutable(ImmutableDataId(name)),
            FragmentInfo::MutableDataShell { name, tag, .. } |
            FragmentInfo::MutableDataEntry { name, tag, .. } => {
                DataId::Mutable(MutableDataId(name, tag))
            }
        }
    }
}

struct FragmentHolder {
    request: Option<FragmentRequest>,
    fragments: HashSet<FragmentInfo>,
}

impl FragmentHolder {
    fn new() -> Self {
        FragmentHolder {
            request: None,
            fragments: HashSet::default(),
        }
    }

    fn is_requested(&self) -> bool {
        self.request.is_some()
    }

    fn start_request(&mut self, fragment: FragmentInfo) {
        self.request = Some(FragmentRequest::new(fragment));
    }

    fn stop_request(&mut self) -> Option<FragmentInfo> {
        self.request
            .take()
            .map(|request| {
                     let _ = self.fragments.remove(&request.fragment);
                     request.fragment
                 })
    }

    // If there is an ongoing request for the given fragment, stops it and returns true,
    // otherwise does nothing and returns false.
    fn stop_request_for(&mut self, fragment: &FragmentInfo) -> bool {
        if self.request
               .as_ref()
               .map_or(false, |request| request.fragment == *fragment) {
            self.request = None;
            true
        } else {
            false
        }
    }

    fn reset_expired_request(&mut self) {
        if self.request
               .as_ref()
               .map_or(false, |request| request.is_expired()) {
            self.request = None;
        }
    }
}

struct FragmentRequest {
    fragment: FragmentInfo,
    timestamp: Instant,
}

impl FragmentRequest {
    fn new(fragment: FragmentInfo) -> Self {
        FragmentRequest {
            fragment: fragment,
            timestamp: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.timestamp.elapsed().as_secs() > FRAGMENT_REQUEST_TIMEOUT_SECS
    }
}

struct FragmentState {
    holders: HashSet<XorName>,
    requested: bool,
}

impl FragmentState {
    fn new() -> Self {
        FragmentState {
            holders: HashSet::default(),
            requested: false,
        }
    }
}

// Structure that holds data chunk IDs in order but also allows efficient
// lookup.
struct UnneededChunks {
    queue: VecDeque<XorName>,
    set: HashSet<XorName>,
}

impl UnneededChunks {
    fn new() -> Self {
        UnneededChunks {
            queue: VecDeque::new(),
            set: HashSet::default(),
        }
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn contains(&self, name: &XorName) -> bool {
        self.set.contains(name)
    }

    fn push(&mut self, name: XorName) {
        if self.set.insert(name) {
            self.queue.push_back(name);
        }
    }

    fn pop(&mut self) -> Option<XorName> {
        if let Some(name) = self.queue.pop_front() {
            let _ = self.set.remove(&name);
            Some(name)
        } else {
            None
        }
    }

    fn retain<F>(&mut self, f: F)
        where F: FnMut(&XorName) -> bool
    {
        let (retained, removed) = self.queue.drain(..).partition(f);
        self.queue = retained;
        for name in removed {
            let _ = self.set.remove(&name);
        }
    }

    #[cfg(all(test, feature = "use-mock-routing"))]
    fn clear(&mut self) {
        self.queue.clear();
        self.set.clear();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand;

    #[test]
    fn unrequested_needed_fragments() {
        let mut cache = Cache::default();

        let holder0 = rand::random();
        let holder1 = rand::random();

        let fragment0 = FragmentInfo::ImmutableData(rand::random());

        assert!(cache.unrequested_needed_fragments().is_empty());

        // Insert a single fragment. It should be present in the collection.
        cache.insert_needed_fragment(fragment0.clone(), holder0);
        let result = cache.unrequested_needed_fragments();
        assert_eq!(result.len(), 1);
        assert_eq!(first(result), (fragment0.clone(), vec![holder0]));

        // Insert the same fragment with the same holder again. The collection
        // should not change.
        cache.insert_needed_fragment(fragment0.clone(), holder0);
        let result = cache.unrequested_needed_fragments();
        assert_eq!(result.len(), 1);
        assert_eq!(first(result), (fragment0.clone(), vec![holder0]));

        // Insert the same fragment but with different holder. It should be present
        // in the collection only once, but with both holders.
        cache.insert_needed_fragment(fragment0.clone(), holder1);
        let result = cache.unrequested_needed_fragments();
        assert_eq!(result.len(), 1);
        let item = first(result);
        assert_eq!(item.0, fragment0);
        assert_eq!(item.1.len(), 2);
        assert!(item.1.contains(&holder0));
        assert!(item.1.contains(&holder1));

        // Start request against one holder. The fragment should not appear among
        // the unrequested fragments even though this fragment is still unrequested
        // in different holder.
        cache.start_needed_fragment_request(&fragment0, &holder0);
        assert!(cache.unrequested_needed_fragments().is_empty());

        // Stop the request. The fragment should be present in the collection again,
        // with the other holder.
        assert_eq!(unwrap!(cache.stop_needed_fragment_request(&holder0)),
                   fragment0);
        let result = cache.unrequested_needed_fragments();
        assert_eq!(result.len(), 1);
        assert_eq!(first(result), (fragment0.clone(), vec![holder1]));

        // Remove the fragment. It should remove it from all holders.
        cache.remove_needed_fragment(&fragment0);
        assert!(cache.unrequested_needed_fragments().is_empty());
    }

    fn first<I: IntoIterator<Item = T>, T>(i: I) -> T {
        unwrap!(i.into_iter().next())
    }
}
