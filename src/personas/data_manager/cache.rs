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
use maidsafe_utilities::{self, serialisation};
use routing::{Authority, ImmutableData, MessageId, MutableData, RoutingTable, XorName};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

/// The timeout for cached data from requests; if no consensus is reached, the data is dropped.
const PENDING_WRITE_TIMEOUT_SECS: u64 = 60;
/// The timeout for retrieving data fragments from individual peers.
const FRAGMENT_REQUEST_TIMEOUT_SECS: u64 = 60;

pub struct Cache {
    /// Chunks we are no longer responsible for. These can be deleted from the chunk store.
    unneeded_chunks: VecDeque<DataId>,
    /// Maps the peers to the data fragments we need from them and tracks any ongoing
    /// requests to retrieve those fragments.
    needed_fragments: HashMap<XorName, HashMap<FragmentInfo, FragmentRequest>>,
    /// Maps data identifiers to the list of pending writes that affect that chunk.
    pending_writes: HashMap<DataId, Vec<PendingWrite>>,

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

        // TODO (adam): make sure that for each fragment, we return at most one
        // record. This is so we send the request for each fragment to at most
        // one peer.

        // Return all fragments that do not already have request ongoing.
        let mut result = Vec::new();
        for (holder, fragments) in self.needed_fragments.iter() {
            for (fragment, request) in fragments {
                if !request.is_ongoing() {
                    result.push((*holder, fragment.clone()));
                }
            }
        }

        result
    }

    /// Returns all data fragments we need.
    pub fn needed_fragments(&self) -> HashSet<FragmentInfo> {
        self.needed_fragments
            .values()
            .flat_map(|fragments| fragments.keys().cloned())
            .filter(|fragment| !self.unneeded_chunks.contains(&fragment.data_id()))
            .collect()
    }

    pub fn insert_needed_fragment(&mut self, fragment: FragmentInfo, holder: XorName) {
        let _ = self.needed_fragments
            .entry(holder)
            .or_insert_with(HashMap::new)
            .insert(fragment, FragmentRequest::new());
    }

    pub fn register_needed_fragment_with_holder(&mut self,
                                                fragment: FragmentInfo,
                                                holder: XorName)
                                                -> bool {
        if self.needed_fragments
            .values()
            .any(|nfs| nfs.contains_key(&fragment)) {
            self.insert_needed_fragment(fragment, holder);
            true
        } else {
            false
        }
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

    /// Removes needed fragments that are no longer valid due to churn.
    /// Returns whether any of the pruned fragments had a request ongoing.
    pub fn prune_needed_fragments(&mut self, routing_table: &RoutingTable<XorName>) -> bool {
        let mut empty_holders = Vec::new();
        let mut result = false;

        for (holder, fragments) in &mut self.needed_fragments {
            let lost_fragments: Vec<_> = fragments.iter()
                .filter(|&(fragment, request)| {
                    routing_table.other_closest_names(fragment.name(), GROUP_SIZE)
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

    pub fn remove_needed_fragment(&mut self, holder: &XorName, message_id: MessageId) -> bool {
        let mut remove_holder = false;

        let result = {
            let fragments = match self.needed_fragments.get_mut(holder) {
                Some(fragments) => fragments,
                None => return false,
            };


            if let Some(fragment) = fragments.iter()
                .find(|&(fragment, request)| request.message_id() == Some(message_id))
                .map(|(fragment, _)| fragment.clone()) {
                let _ = fragments.remove(&fragment);
                remove_holder = fragments.is_empty();
                true
            } else {
                false
            }
        };

        if remove_holder {
            let _ = self.needed_fragments.remove(&holder);
        }

        result
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

    /// Inserts the given mutation as a pending write. If it is the first for that
    /// data identifier, it returns a refresh message to send to ourselves as a group.
    pub fn insert_pending_write(&mut self,
                                mutation: PendingMutation,
                                src: Authority<XorName>,
                                dst: Authority<XorName>,
                                msg_id: MessageId,
                                rejected: bool)
                                -> Option<DataInfo> {
        let hash = match serialisation::serialise(&mutation) {
            Err(_) => return None,
            Ok(serialised) => serialised,
        };
        let hash = maidsafe_utilities::big_endian_sip_hash(&hash);

        let mut writes = self.pending_writes.entry(mutation.data_id()).or_insert_with(Vec::new);
        let result = if !rejected && writes.iter().all(|pending_write| pending_write.rejected) {
            let (data_id, version) = mutation.data_id_and_version();
            Some(DataInfo {
                data_id: data_id,
                version: version,
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

        let mut new_total = 0;
        let mut new_requested = 0;

        for (fragment, request) in self.needed_fragments.values().flat_map(HashMap::iter) {
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

    pub fn data_id_and_version(&self) -> (DataId, u64) {
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

    pub fn fragment_infos(&self) -> Vec<FragmentInfo> {
        unimplemented!()
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
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, RustcEncodable, RustcDecodable)]
pub enum FragmentInfo {
    ImmutableData(XorName),
    MutableDataShell {
        name: XorName,
        tag: u64,
        version: u64,
        hash: u64,
    },
    MutableDataEntry {
        name: XorName,
        tag: u64,
        key: Vec<u8>,
        version: u64,
        hash: u64,
    },
}

impl FragmentInfo {
    // Get all fragments for the given mutable data.
    pub fn mutable_data(data: &MutableData) -> Vec<Self> {
        unimplemented!()
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

#[derive(Clone, Copy, Debug, RustcEncodable, RustcDecodable)]
pub struct DataInfo {
    pub data_id: DataId,
    pub version: u64,
    pub hash: u64,
}

#[cfg(test)]
mod tests {}
