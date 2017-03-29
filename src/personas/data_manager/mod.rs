// Copyright 2015 MaidSafe.net limited.
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

mod cache;
mod data;
#[cfg(all(test, feature = "use-mock-routing"))]
mod tests;

use self::cache::{Cache, DataInfo, FragmentInfo};
use self::cache::{PendingMutation, PendingMutationType, PendingWrite};
pub use self::data::{Data, DataId};
use GROUP_SIZE;
use accumulator::Accumulator;
use chunk_store::ChunkStore;
use error::InternalError;
use maidsafe_utilities::serialisation;
use routing::{Authority, EntryAction, ImmutableData, MessageId, MutableData, PermissionSet,
              RoutingTable, User, Value, XorName};
use routing::ClientError;
use rust_sodium::crypto::sign;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::From;
use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use utils;
use vault::RoutingNode;

const MAX_FULL_PERCENT: u64 = 50;
/// The quorum for accumulating refresh messages.
const ACCUMULATOR_QUORUM: usize = GROUP_SIZE / 2 + 1;
/// The timeout for accumulating refresh messages.
const ACCUMULATOR_TIMEOUT_SECS: u64 = 180;
/// The interval for print status log.
const STATUS_LOG_INTERVAL: u64 = 120;

macro_rules! log_status {
    ($dm:expr) => {
        if $dm.logging_time.elapsed().as_secs() > STATUS_LOG_INTERVAL {
            $dm.logging_time = Instant::now();
            info!("{:?}", $dm);
        }
    }
}

pub struct DataManager {
    chunk_store: ChunkStore<DataId, Data>,
    /// Accumulates refresh messages and the peers we received them from.
    refresh_accumulator: Accumulator<FragmentInfo, XorName>,
    cache: Cache,
    immutable_data_count: u64,
    mutable_data_count: u64,
    client_get_requests: u64,
    logging_time: Instant,
}

impl DataManager {
    pub fn new(chunk_store_root: PathBuf, capacity: u64) -> Result<DataManager, InternalError> {
        let chunk_store = ChunkStore::new(chunk_store_root, capacity)?;

        Ok(DataManager {
               chunk_store: chunk_store,
               refresh_accumulator:
                   Accumulator::with_duration(ACCUMULATOR_QUORUM,
                                              Duration::from_secs(ACCUMULATOR_TIMEOUT_SECS)),
               cache: Default::default(),
               immutable_data_count: 0,
               mutable_data_count: 0,
               client_get_requests: 0,
               logging_time: Instant::now(),
           })
    }

    pub fn handle_refresh(&mut self,
                          routing_node: &mut RoutingNode,
                          src: XorName,
                          serialised_refresh: &[u8])
                          -> Result<(), InternalError> {
        let fragments: Vec<FragmentInfo> = serialisation::deserialise(serialised_refresh)?;
        for fragment in fragments {
            if self.cache
                   .register_needed_fragment_with_holder(fragment.clone(), src) {
                continue;
            }

            if let Some(holders) = self.refresh_accumulator
                   .add(fragment.clone(), src)
                   .cloned() {
                self.refresh_accumulator.delete(&fragment);

                let needed = match fragment {
                    FragmentInfo::ImmutableData(name) => {
                        !self.chunk_store.has(&DataId::Immutable(name))
                    }
                    FragmentInfo::MutableDataShell { name, tag, version, .. } => {
                        match self.chunk_store.get(&DataId::Mutable(name, tag)) {
                            Err(_) => true,
                            Ok(Data::Mutable(data)) => data.version() < version,
                            Ok(_) => unreachable!(),
                        }
                    }
                    FragmentInfo::MutableDataEntry {
                        name,
                        tag,
                        ref key,
                        version,
                        ..
                    } => {
                        if let Ok(Data::Mutable(data)) =
                            self.chunk_store.get(&DataId::Mutable(name, tag)) {
                            match data.get(key) {
                                None => true,
                                Some(value) if value.entry_version < version => true,
                                Some(_) => false,
                            }
                        } else {
                            true
                        }
                    }
                };

                if !needed {
                    continue;
                }

                for holder in holders {
                    self.cache
                        .insert_needed_fragment(fragment.clone(), holder);
                }
            }
        }

        self.request_needed_fragments(routing_node)
    }

    /// Handles an accumulated refresh message sent from the whole group.
    pub fn handle_group_refresh(&mut self,
                                routing_node: &mut RoutingNode,
                                serialised_refresh: &[u8])
                                -> Result<(), InternalError> {
        let DataInfo {
            data_id,
            hash: refresh_hash,
        } = serialisation::deserialise(serialised_refresh)?;
        let mut success = false;

        for PendingWrite {
                mutation,
                src,
                dst,
                message_id,
                hash,
                rejected,
                ..
            } in self.cache.take_pending_writes(&data_id) {
            if hash != refresh_hash {
                if !rejected {
                    trace!("{:?} did not accumulate. Sending failure", data_id);
                    let error = ClientError::from("Concurrent modification.");
                    self.send_mutation_response(routing_node,
                                                src,
                                                dst,
                                                mutation.mutation_type(),
                                                mutation.data_id(),
                                                Err(error),
                                                message_id)?;
                }

                continue;
            }

            let mutation_type = mutation.mutation_type();
            let fragments = mutation.fragment_infos();

            if self.handle_pending_mutation(routing_node, src, dst, mutation, message_id)? {
                match mutation_type {
                    PendingMutationType::PutIData |
                    PendingMutationType::PutMData => self.count_added_data(&data_id),
                    _ => (),
                }

                log_status!(self);
                success = true;
            }

            self.send_refresh(routing_node,
                              Authority::NaeManager(*data_id.name()),
                              fragments)?;
        }

        if !success {
            if let Some(group) = routing_node.close_group(*data_id.name(), GROUP_SIZE) {
                for node in &group {
                    let _ = self.cache
                        .register_needed_data_with_holder(&data_id, *node);
                }

                self.request_needed_fragments(routing_node)?;
            }
        }

        Ok(())
    }

    pub fn handle_get_idata(&mut self,
                            routing_node: &mut RoutingNode,
                            src: Authority<XorName>,
                            dst: Authority<XorName>,
                            name: XorName,
                            msg_id: MessageId)
                            -> Result<(), InternalError> {
        if let Authority::Client { .. } = src {
            self.client_get_requests += 1;
            log_status!(self);
        }

        if let Ok(Data::Immutable(data)) = self.chunk_store.get(&DataId::Immutable(name)) {
            trace!("As {:?} sending data {:?} to {:?}", dst, data, src);
            routing_node.send_get_idata_response(dst, src, Ok(data), msg_id)?;
        } else {
            trace!("DM sending get_idata_failure of {:?}", name);
            routing_node.send_get_idata_response(dst, src, Err(ClientError::NoSuchData), msg_id)?;
        }

        Ok(())
    }

    pub fn handle_get_idata_success(&mut self,
                                    routing_node: &mut RoutingNode,
                                    src: XorName,
                                    data: ImmutableData,
                                    msg_id: MessageId)
                                    -> Result<(), InternalError> {
        let valid = if let Some(fragment) = self.cache.stop_needed_fragment_request(&src, msg_id) {
            match fragment {
                FragmentInfo::ImmutableData(ref name) if *name == *data.name() => {
                    self.cache.remove_needed_fragment(&fragment);
                    true
                }
                _ => false,
            }
        } else {
            false
        };

        self.request_needed_fragments(routing_node)?;

        if !valid {
            return Ok(());
        }

        // If we're no longer in the close group, return.
        if !close_to_address(routing_node, data.name()) {
            return Ok(());
        }

        let data = Data::Immutable(data);
        let data_id = data.id();

        if self.chunk_store.has(&data_id) {
            return Ok(()); // data is already there.
        }

        self.clean_chunk_store();
        self.chunk_store.put(&data_id, &data)?;

        self.count_added_data(&data_id);
        log_status!(self);

        Ok(())
    }

    pub fn handle_get_idata_failure(&mut self,
                                    routing_node: &mut RoutingNode,
                                    src: XorName,
                                    msg_id: MessageId)
                                    -> Result<(), InternalError> {
        if self.cache
               .stop_needed_fragment_request(&src, msg_id)
               .is_none() {
            return Err(InternalError::InvalidMessage);
        }

        self.request_needed_fragments(routing_node)
    }

    pub fn handle_put_idata(&mut self,
                            routing_node: &mut RoutingNode,
                            src: Authority<XorName>,
                            dst: Authority<XorName>,
                            data: ImmutableData,
                            msg_id: MessageId)
                            -> Result<(), InternalError> {
        let data_id = DataId::Immutable(*data.name());
        if self.chunk_store.has(&data_id) {
            trace!("DM sending PutIData success for data {:?}, it already exists.",
                   data.name());
            routing_node.send_put_idata_response(dst, src, Ok(()), msg_id)?;
            return Ok(());
        }

        self.clean_chunk_store();

        if self.chunk_store_full() {
            let err = ClientError::NetworkFull;
            routing_node.send_put_idata_response(dst, src, Err(err.clone()), msg_id)?;
            return Err(From::from(err));
        }

        self.update_pending_writes(routing_node,
                                   PendingMutation::PutIData(data),
                                   src,
                                   dst,
                                   msg_id,
                                   false)
    }

    pub fn handle_put_mdata(&mut self,
                            routing_node: &mut RoutingNode,
                            src: Authority<XorName>,
                            dst: Authority<XorName>,
                            data: MutableData,
                            msg_id: MessageId,
                            _requester: sign::PublicKey)
                            -> Result<(), InternalError> {

        let data_id = DataId::Mutable(*data.name(), data.tag());
        let rejected = if self.chunk_store.has(&data_id) {
            trace!("DM sending PutMData failure for data {:?}, it already exists.",
                   data_id);
            routing_node.send_put_mdata_response(dst, src, Err(ClientError::DataExists), msg_id)?;
            true
        } else {
            self.clean_chunk_store();

            if self.chunk_store_full() {
                let err = ClientError::NetworkFull;
                routing_node.send_put_mdata_response(dst, src, Err(err), msg_id)?;
                return Ok(());
            }

            false
        };

        self.update_pending_writes(routing_node,
                                   PendingMutation::PutMData(data),
                                   src,
                                   dst,
                                   msg_id,
                                   rejected)
    }

    pub fn handle_get_mdata_shell(&mut self,
                                  routing_node: &mut RoutingNode,
                                  src: Authority<XorName>,
                                  dst: Authority<XorName>,
                                  name: XorName,
                                  tag: u64,
                                  msg_id: MessageId)
                                  -> Result<(), InternalError> {
        let res = self.read_mdata(&src, name, tag, |data| Ok(data.shell()));
        routing_node.send_get_mdata_shell_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_get_mdata_shell_success(&mut self,
                                          routing_node: &mut RoutingNode,
                                          src: XorName,
                                          mut shell: MutableData,
                                          msg_id: MessageId)
                                          -> Result<(), InternalError> {
        let valid = if let Some(fragment) = self.cache.stop_needed_fragment_request(&src, msg_id) {
            let actual_hash = utils::mdata_shell_hash(&shell);

            match fragment {
                FragmentInfo::MutableDataShell { hash, .. } if hash == actual_hash => {
                    self.cache.remove_needed_fragment(&fragment);
                    true
                }
                _ => false,
            }
        } else {
            false
        };

        self.request_needed_fragments(routing_node)?;

        if !valid {
            return Ok(());
        }

        // If we're no longer in the close group, return.
        if !close_to_address(routing_node, shell.name()) {
            return Ok(());
        }

        let data_id = DataId::mutable(&shell);
        let new = match self.chunk_store.get(&data_id) {
            Ok(Data::Mutable(ref old_data)) if old_data.version() >= shell.version() => {
                // The data in the chunk store is already more recent than the
                // shell we received. Ignore it.
                return Ok(());
            }
            Ok(Data::Mutable(ref old_data)) => {
                // The shell is more recent than the data in the chunk store.
                // Repace the data with the shell, but keep the entries.
                for (key, value) in old_data.entries() {
                    shell.mutate_entry_without_validation(key.clone(), value.clone());
                }

                false
            }
            Ok(_) => unreachable!(),
            Err(_) => {
                // If we have cached entries for this data, apply them.
                for (key, value) in self.cache.take_mdata_entries(*shell.name(), shell.tag()) {
                    // OK to ingore the return value here because as the shell has no
                    // entries, this call can never fail.
                    let _ = shell.mutate_entry_without_validation(key, value);
                }

                true
            }
        };

        self.clean_chunk_store();
        self.chunk_store.put(&data_id, &Data::Mutable(shell))?;

        if new {
            self.count_added_data(&data_id);
            log_status!(self);
        }

        Ok(())
    }

    pub fn handle_get_mdata_shell_failure(&mut self,
                                          routing_node: &mut RoutingNode,
                                          src: XorName,
                                          msg_id: MessageId)
                                          -> Result<(), InternalError> {
        if self.cache
               .stop_needed_fragment_request(&src, msg_id)
               .is_none() {
            return Err(InternalError::InvalidMessage);
        }

        self.request_needed_fragments(routing_node)
    }

    pub fn handle_get_mdata_version(&mut self,
                                    routing_node: &mut RoutingNode,
                                    src: Authority<XorName>,
                                    dst: Authority<XorName>,
                                    name: XorName,
                                    tag: u64,
                                    msg_id: MessageId)
                                    -> Result<(), InternalError> {
        let res = self.read_mdata(&src, name, tag, |data| Ok(data.version()));
        routing_node.send_get_mdata_version_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_list_mdata_entries(&mut self,
                                     routing_node: &mut RoutingNode,
                                     src: Authority<XorName>,
                                     dst: Authority<XorName>,
                                     name: XorName,
                                     tag: u64,
                                     msg_id: MessageId)
                                     -> Result<(), InternalError> {
        let res = self.read_mdata(&src, name, tag, |data| Ok(data.entries().clone()));
        routing_node.send_list_mdata_entries_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_list_mdata_keys(&mut self,
                                  routing_node: &mut RoutingNode,
                                  src: Authority<XorName>,
                                  dst: Authority<XorName>,
                                  name: XorName,
                                  tag: u64,
                                  msg_id: MessageId)
                                  -> Result<(), InternalError> {
        let res = self.read_mdata(&src,
                                  name,
                                  tag,
                                  |data| Ok(data.entries().keys().cloned().collect()));
        routing_node.send_list_mdata_keys_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_list_mdata_values(&mut self,
                                    routing_node: &mut RoutingNode,
                                    src: Authority<XorName>,
                                    dst: Authority<XorName>,
                                    name: XorName,
                                    tag: u64,
                                    msg_id: MessageId)
                                    -> Result<(), InternalError> {
        let res = self.read_mdata(&src,
                                  name,
                                  tag,
                                  |data| Ok(data.entries().values().cloned().collect()));
        routing_node.send_list_mdata_values_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_get_mdata_value(&mut self,
                                  routing_node: &mut RoutingNode,
                                  src: Authority<XorName>,
                                  dst: Authority<XorName>,
                                  name: XorName,
                                  tag: u64,
                                  key: Vec<u8>,
                                  msg_id: MessageId)
                                  -> Result<(), InternalError> {
        let res =
            self.read_mdata(&src,
                            name,
                            tag,
                            |data| data.get(&key).cloned().ok_or(ClientError::NoSuchEntry));
        routing_node.send_get_mdata_value_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_get_mdata_value_success(&mut self,
                                          routing_node: &mut RoutingNode,
                                          src: XorName,
                                          value: Value,
                                          msg_id: MessageId)
                                          -> Result<(), InternalError> {
        let info = if let Some(fragment) = self.cache.stop_needed_fragment_request(&src, msg_id) {
            let actual_hash = utils::mdata_value_hash(&value);

            match fragment {
                FragmentInfo::MutableDataEntry {
                    name,
                    tag,
                    ref key,
                    hash,
                    ..
                } if hash == actual_hash => {
                    self.cache.remove_needed_fragment(&fragment);
                    Some((name, tag, key.clone()))
                }
                _ => None,
            }
        } else {
            None
        };

        self.request_needed_fragments(routing_node)?;

        let (name, tag, key) = if let Some(info) = info {
            info
        } else {
            return Ok(());
        };

        // If we're no longer in the close group, return.
        if !close_to_address(routing_node, &name) {
            return Ok(());
        }

        let data_id = DataId::Mutable(name, tag);

        match self.chunk_store.get(&data_id) {
            Ok(Data::Mutable(mut data)) => {
                if data.mutate_entry_without_validation(key, value) {
                    self.clean_chunk_store();
                    self.chunk_store.put(&data_id, &Data::Mutable(data))?;
                }
            }
            Ok(_) => unreachable!(),
            Err(_) => {
                // We don't have the shell yet, so keep the entry around in the cache
                // until we receive the shell.
                self.cache.insert_mdata_entry(name, tag, key, value);
            }
        }

        Ok(())
    }

    pub fn handle_get_mdata_value_failure(&mut self,
                                          routing_node: &mut RoutingNode,
                                          src: XorName,
                                          msg_id: MessageId)
                                          -> Result<(), InternalError> {
        if self.cache
               .stop_needed_fragment_request(&src, msg_id)
               .is_none() {
            return Err(InternalError::InvalidMessage);
        }

        self.request_needed_fragments(routing_node)
    }

    pub fn handle_mutate_mdata_entries(&mut self,
                                       routing_node: &mut RoutingNode,
                                       src: Authority<XorName>,
                                       dst: Authority<XorName>,
                                       name: XorName,
                                       tag: u64,
                                       actions: BTreeMap<Vec<u8>, EntryAction>,
                                       msg_id: MessageId,
                                       requester: sign::PublicKey)
                                       -> Result<(), InternalError> {
        self.handle_mdata_mutation(routing_node,
                                   src,
                                   dst,
                                   name,
                                   tag,
                                   PendingMutationType::MutateMDataEntries,
                                   msg_id,
                                   |data| data.mutate_entries(actions, requester))
    }

    pub fn handle_list_mdata_permissions(&mut self,
                                         routing_node: &mut RoutingNode,
                                         src: Authority<XorName>,
                                         dst: Authority<XorName>,
                                         name: XorName,
                                         tag: u64,
                                         msg_id: MessageId)
                                         -> Result<(), InternalError> {
        let res = self.read_mdata(&src, name, tag, |data| Ok(data.permissions().clone()));
        routing_node.send_list_mdata_permissions_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_list_mdata_user_permissions(&mut self,
                                              routing_node: &mut RoutingNode,
                                              src: Authority<XorName>,
                                              dst: Authority<XorName>,
                                              name: XorName,
                                              tag: u64,
                                              user: User,
                                              msg_id: MessageId)
                                              -> Result<(), InternalError> {
        let res = self.read_mdata(&src,
                                  name,
                                  tag,
                                  |data| data.user_permissions(&user).map(|p| p.clone()));
        routing_node.send_list_mdata_user_permissions_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_set_mdata_user_permissions(&mut self,
                                             routing_node: &mut RoutingNode,
                                             src: Authority<XorName>,
                                             dst: Authority<XorName>,
                                             name: XorName,
                                             tag: u64,
                                             user: User,
                                             permissions: PermissionSet,
                                             version: u64,
                                             msg_id: MessageId,
                                             requester: sign::PublicKey)
                                             -> Result<(), InternalError> {
        self.handle_mdata_mutation(routing_node,
                                   src,
                                   dst,
                                   name,
                                   tag,
                                   PendingMutationType::SetMDataUserPermissions,
                                   msg_id,
                                   |data| {
                                       data.set_user_permissions(user,
                                                                 permissions,
                                                                 version,
                                                                 requester)
                                   })
    }

    pub fn handle_del_mdata_user_permissions(&mut self,
                                             routing_node: &mut RoutingNode,
                                             src: Authority<XorName>,
                                             dst: Authority<XorName>,
                                             name: XorName,
                                             tag: u64,
                                             user: User,
                                             version: u64,
                                             msg_id: MessageId,
                                             requester: sign::PublicKey)
                                             -> Result<(), InternalError> {
        self.handle_mdata_mutation(routing_node,
                                   src,
                                   dst,
                                   name,
                                   tag,
                                   PendingMutationType::DelMDataUserPermissions,
                                   msg_id,
                                   |data| data.del_user_permissions(&user, version, requester))
    }

    pub fn handle_change_mdata_owner(&mut self,
                                     routing_node: &mut RoutingNode,
                                     src: Authority<XorName>,
                                     dst: Authority<XorName>,
                                     name: XorName,
                                     tag: u64,
                                     new_owners: BTreeSet<sign::PublicKey>,
                                     version: u64,
                                     msg_id: MessageId)
                                     -> Result<(), InternalError> {
        let mutation_type = PendingMutationType::ChangeMDataOwner;
        let data_id = DataId::Mutable(name, tag);

        let new_owners_len = new_owners.len();
        let new_owner = match new_owners.into_iter().next() {
            Some(owner) if new_owners_len == 1 => owner,
            Some(_) | None => {
                // `new_owners` must have exactly 1 element.
                self.send_mutation_response(routing_node,
                                            src,
                                            dst,
                                            mutation_type,
                                            data_id,
                                            Err(ClientError::InvalidOwners),
                                            msg_id)?;
                return Ok(());
            }
        };

        let client_name = utils::client_name(&src);

        self.handle_mdata_mutation(routing_node,
                                   src,
                                   dst,
                                   name,
                                   tag,
                                   mutation_type,
                                   msg_id,
                                   |data| {
                                       verify_mdata_owner(data, &client_name)?;
                                       data.change_owner(new_owner, version)
                                   })
    }

    pub fn handle_node_added(&mut self,
                             routing_node: &mut RoutingNode,
                             node_name: &XorName,
                             routing_table: &RoutingTable<XorName>) {
        if self.cache.prune_needed_fragments(routing_table) {
            let _ = self.request_needed_fragments(routing_node);
        }

        let mut refresh = Vec::new();
        let mut has_pruned_data = false;

        for fragment in self.our_fragments() {
            let data_id = fragment.data_id();

            // Only retain fragments for which we're still in the close group.
            match routing_table.other_closest_names(data_id.name(), GROUP_SIZE) {
                None => {
                    trace!("No longer a DM for {:?}", data_id);
                    if self.chunk_store.has(&data_id) && !self.cache.is_in_unneeded(&data_id) {
                        self.count_removed_data(&data_id);
                        has_pruned_data = true;

                        if let DataId::Immutable(..) = data_id {
                            self.cache.add_as_unneeded(data_id);
                        } else {
                            let _ = self.chunk_store.delete(&data_id);
                        }
                    }
                }
                Some(close_group) => {
                    if close_group.contains(&node_name) {
                        refresh.push(fragment.clone());
                    }
                }
            }
        }

        if !refresh.is_empty() {
            let _ = self.send_refresh(routing_node, Authority::ManagedNode(*node_name), refresh);
        }

        if has_pruned_data {
            log_status!(self);
        }
    }

    /// Get all names and hashes of all data. // [TODO]: Can be optimised - 2016-04-23 09:11pm
    /// Send to all members of group of data.
    pub fn handle_node_lost(&mut self,
                            routing_node: &mut RoutingNode,
                            node_name: &XorName,
                            routing_table: &RoutingTable<XorName>) {
        let pruned_unneeded_chunks = self.cache.prune_unneeded_chunks(routing_table);
        if pruned_unneeded_chunks != 0 {
            self.immutable_data_count += pruned_unneeded_chunks;
            log_status!(self);
        }

        if self.cache.prune_needed_fragments(routing_table) {
            let _ = self.request_needed_fragments(routing_node);
        }

        let mut refreshes = HashMap::new();
        for fragment in self.our_fragments() {
            match routing_table.other_closest_names(fragment.name(), GROUP_SIZE) {
                None => {
                    error!("Moved out of close group of {:?} in a NodeLost event.",
                           node_name);
                }
                Some(close_group) => {
                    // If no new node joined the group due to this event, continue:
                    // If the group has fewer than GROUP_SIZE elements, the lost node was not
                    // replaced at all. Otherwise, if the group's last node is closer to the data
                    // than the lost node, the lost node was not in the group in the first place.
                    if let Some(&outer_node) = close_group.get(GROUP_SIZE - 2) {
                        if fragment.name().closer(node_name, outer_node) {
                            refreshes
                                .entry(*outer_node)
                                .or_insert_with(Vec::new)
                                .push(fragment.clone());
                        }
                    }
                }
            }
        }

        for (node_name, refresh) in refreshes {
            let _ = self.send_refresh(routing_node, Authority::ManagedNode(node_name), refresh);
        }
    }

    pub fn check_timeouts(&mut self, routing_node: &mut RoutingNode) {
        let _ = self.request_needed_fragments(routing_node);
    }

    fn read_mdata<F, R>(&mut self,
                        src: &Authority<XorName>,
                        name: XorName,
                        tag: u64,
                        f: F)
                        -> Result<R, ClientError>
        where F: FnOnce(&MutableData) -> Result<R, ClientError>
    {
        if let Authority::Client { .. } = *src {
            self.client_get_requests += 1;
            log_status!(self);
        }

        let data_id = DataId::Mutable(name, tag);
        if let Ok(Data::Mutable(data)) = self.chunk_store.get(&data_id) {
            f(&data)
        } else {
            Err(ClientError::NoSuchData)
        }
    }

    fn mutate_mdata<F>(&self, name: XorName, tag: u64, f: F) -> Result<MutableData, ClientError>
        where F: FnOnce(&mut MutableData) -> Result<(), ClientError>
    {
        let data_id = DataId::Mutable(name, tag);
        if let Ok(Data::Mutable(mut data)) = self.chunk_store.get(&data_id) {
            f(&mut data)?;
            Ok(data)
        } else {
            Err(ClientError::NoSuchData)
        }
    }

    fn handle_mdata_mutation<F>(&mut self,
                                routing_node: &mut RoutingNode,
                                src: Authority<XorName>,
                                dst: Authority<XorName>,
                                name: XorName,
                                tag: u64,
                                mutation_type: PendingMutationType,
                                msg_id: MessageId,
                                f: F)
                                -> Result<(), InternalError>
        where F: FnOnce(&mut MutableData) -> Result<(), ClientError>
    {
        let res = self.mutate_mdata(name, tag, f);

        match res {
            Ok(data) => {
                self.update_pending_writes(routing_node,
                                           create_pending_mutation_for_mdata(mutation_type, data),
                                           src,
                                           dst,
                                           msg_id,
                                           false)
            }
            Err(error) => {
                self.send_mutation_response(routing_node,
                                            src,
                                            dst,
                                            mutation_type,
                                            DataId::Mutable(name, tag),
                                            Err(error),
                                            msg_id)
            }
        }
    }

    fn update_pending_writes(&mut self,
                             routing_node: &mut RoutingNode,
                             mutation: PendingMutation,
                             src: Authority<XorName>,
                             dst: Authority<XorName>,
                             message_id: MessageId,
                             rejected: bool)
                             -> Result<(), InternalError> {
        for PendingWrite {
                mutation,
                src,
                dst,
                message_id,
                ..
            } in self.cache.remove_expired_writes() {
            let error = ClientError::from("Request expired.");
            trace!("{:?} did not accumulate. Sending failure",
                   mutation.data_id());
            self.send_mutation_response(routing_node,
                                        src,
                                        dst,
                                        mutation.mutation_type(),
                                        mutation.data_id(),
                                        Err(error),
                                        message_id)?;
        }

        let data_name = *mutation.data_id().name();
        if let Some(refresh) = self.cache
               .insert_pending_write(mutation, src, dst, message_id, rejected) {
            self.send_group_refresh(routing_node, data_name, refresh, message_id)?;
        }

        Ok(())
    }

    fn handle_pending_mutation(&mut self,
                               routing_node: &mut RoutingNode,
                               src: Authority<XorName>,
                               dst: Authority<XorName>,
                               mutation: PendingMutation,
                               msg_id: MessageId)
                               -> Result<bool, InternalError> {
        let mutation_type = mutation.mutation_type();
        let data = mutation.into_data();
        let data_id = data.id();

        let res = if let Err(error) = self.chunk_store.put(&data_id, &data) {
            trace!("DM failed to store {:?} in chunkstore: {:?}",
                   data_id,
                   error);
            Err(ClientError::from(format!("Failed to store chunk: {:?}", error)))
        } else {
            Ok(())
        };

        let success = res.is_ok();
        self.send_mutation_response(routing_node, src, dst, mutation_type, data_id, res, msg_id)?;
        Ok(success)
    }

    fn send_mutation_response(&self,
                              routing_node: &mut RoutingNode,
                              src: Authority<XorName>,
                              dst: Authority<XorName>,
                              mutation_type: PendingMutationType,
                              data_id: DataId,
                              res: Result<(), ClientError>,
                              msg_id: MessageId)
                              -> Result<(), InternalError> {
        let res_string = match res {
            Ok(_) => "success".to_string(),
            Err(ref error) => format!("failure ({:?})", error),
        };

        match mutation_type {
            PendingMutationType::PutIData => {
                trace!("DM sending PutIData {} for data {:?}", res_string, data_id);
                routing_node.send_put_idata_response(dst, src, res, msg_id)?
            }
            PendingMutationType::PutMData => {
                trace!("DM sending PutMData {} for data {:?}", res_string, data_id);
                routing_node.send_put_mdata_response(dst, src, res, msg_id)?
            }
            PendingMutationType::MutateMDataEntries => {
                trace!("DM sending MutateMDataEntries {} for data {:?}",
                       res_string,
                       data_id);
                routing_node.send_mutate_mdata_entries_response(dst, src, res, msg_id)?
            }
            PendingMutationType::SetMDataUserPermissions => {
                trace!("DM sending SetMDataUserPermissions {} for data {:?}",
                       res_string,
                       data_id);
                routing_node.send_set_mdata_user_permissions_response(dst, src, res, msg_id)?
            }
            PendingMutationType::DelMDataUserPermissions => {
                trace!("DM sending DelMDataUserPermissions {} for data {:?}",
                       res_string,
                       data_id);
                routing_node.send_del_mdata_user_permissions_response(dst, src, res, msg_id)?
            }
            PendingMutationType::ChangeMDataOwner => {
                trace!("DM sending ChangeMDataOwner {} for data {:?}",
                       res_string,
                       data_id);
                routing_node.send_change_mdata_owner_response(dst, src, res, msg_id)?
            }
        }

        Ok(())
    }

    /// Returns whether our data uses more than `MAX_FULL_PERCENT` percent of available space.
    fn chunk_store_full(&self) -> bool {
        self.chunk_store.used_space() > (self.chunk_store.max_space() / 100) * MAX_FULL_PERCENT
    }

    /// Removes data chunks we are no longer responsible for until the chunk store is not full
    /// anymore.
    fn clean_chunk_store(&mut self) {
        while self.chunk_store_full() {
            if let Some(data_id) = self.cache.pop_unneeded_chunk() {
                let _ = self.chunk_store.delete(&data_id);
            } else {
                break;
            }
        }
    }

    fn send_refresh(&self,
                    routing_node: &mut RoutingNode,
                    dst: Authority<XorName>,
                    refresh: Vec<FragmentInfo>)
                    -> Result<(), InternalError> {
        // FIXME - We need to handle >2MB chunks
        let src = Authority::ManagedNode(routing_node.name()?);
        let serialised_refresh = serialisation::serialise(&refresh)?;
        trace!("DM sending refresh to {:?}.", dst);
        routing_node.send_refresh_request(src, dst, serialised_refresh, MessageId::new())?;
        Ok(())
    }

    fn send_group_refresh(&self,
                          routing_node: &mut RoutingNode,
                          name: XorName,
                          refresh: DataInfo,
                          msg_id: MessageId)
                          -> Result<(), InternalError> {
        match serialisation::serialise(&refresh) {
            Ok(serialised_data) => {
                trace!("DM sending refresh data to group {:?}.", name);
                routing_node.send_refresh_request(Authority::NaeManager(name),
                                          Authority::NaeManager(name),
                                          serialised_data,
                                          msg_id)?;
                Ok(())
            }
            Err(error) => {
                warn!("Failed to serialise data: {:?}", error);
                Err(From::from(error))
            }
        }
    }

    fn request_needed_fragments(&mut self,
                                routing_node: &mut RoutingNode)
                                -> Result<(), InternalError> {
        let src = Authority::ManagedNode(routing_node.name()?);
        let candidates = self.cache.unrequested_needed_fragments();

        for (holder, fragment) in candidates {
            if let Some(group) = routing_node.close_group(*fragment.name(), GROUP_SIZE) {
                if group.contains(&holder) {
                    let dst = Authority::ManagedNode(holder);
                    let msg_id = MessageId::new();

                    match fragment {
                        FragmentInfo::ImmutableData(name) => {
                            routing_node.send_get_idata_request(src, dst, name, msg_id)?;
                        }
                        FragmentInfo::MutableDataShell { name, tag, .. } => {
                            routing_node.send_get_mdata_shell_request(src, dst, name, tag, msg_id)?;
                        }
                        FragmentInfo::MutableDataEntry { name, tag, ref key, .. } => {
                            routing_node.send_get_mdata_value_request(src,
                                                              dst,
                                                              name,
                                                              tag,
                                                              key.clone(),
                                                              msg_id)?;
                        }
                    }

                    self.cache
                        .start_needed_fragment_request(&fragment, &holder, msg_id);
                }
            }
        }

        self.cache.print_stats();
        Ok(())
    }

    // Get all fragments for the data we have in the chunk store.
    fn chunk_store_fragments(&self) -> HashSet<FragmentInfo> {
        let mut result = HashSet::new();

        for data_id in self.chunk_store.keys() {
            match data_id {
                DataId::Immutable(name) => {
                    let _ = result.insert(FragmentInfo::ImmutableData(name));
                }
                DataId::Mutable(..) => {
                    let data = if let Ok(Data::Mutable(data)) = self.chunk_store.get(&data_id) {
                        data
                    } else {
                        error!("Failed to get {:?} from chunk store.", data_id);
                        continue;
                    };

                    for fragment in FragmentInfo::mutable_data(&data) {
                        let _ = result.insert(fragment);
                    }
                }
            }
        }

        result
    }

    // Get all fragments we are responsible for (irrespective of whether we already
    // have them or not).
    fn our_fragments(&self) -> HashSet<FragmentInfo> {
        self.cache
            .needed_fragments()
            .union(&self.chunk_store_fragments())
            .cloned()
            .collect()
    }

    fn count_added_data(&mut self, data_id: &DataId) {
        match *data_id {
            DataId::Immutable(..) => self.immutable_data_count += 1,
            DataId::Mutable(..) => self.mutable_data_count += 1,
        }
    }

    fn count_removed_data(&mut self, data_id: &DataId) {
        match *data_id {
            DataId::Immutable(..) => self.immutable_data_count -= 1,
            DataId::Mutable(..) => self.mutable_data_count -= 1,
        }
    }
}

#[cfg(feature = "use-mock-crust")]
impl DataManager {
    pub fn get_stored_ids(&self) -> Vec<(DataId, u64)> {
        self.chunk_store
            .keys()
            .into_iter()
            .filter(|data_id| !self.cache.is_in_unneeded(data_id))
            .filter_map(|data_id| {
                            self.get_version(&data_id)
                                .map(|version| (data_id, version))
                        })
            .collect()
    }

    fn get_version(&self, data_id: &DataId) -> Option<u64> {
        match *data_id {
            DataId::Immutable(_) => Some(0),
            DataId::Mutable(..) => {
                if let Ok(Data::Mutable(data)) = self.chunk_store.get(data_id) {
                    Some(data.version())
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(all(test, feature = "use-mock-routing"))]
impl DataManager {
    /// For testing only - put the given data directly into the chunks store.
    pub fn put_into_chunk_store<T: Into<Data>>(&mut self, data: T) {
        let data = data.into();
        unwrap!(self.chunk_store.put(&data.id(), &data))
    }

    /// For testing only - retrieve data with the given ID from the chunk store.
    pub fn get_from_chunk_store(&self, data_id: &DataId) -> Option<Data> {
        self.chunk_store.get(data_id).ok()
    }
}

impl Debug for DataManager {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter,
               "This vault has received {} Client Get requests. Chunks stored: Immutable: {}, \
                Mutable: {}. Total stored: {} bytes.",
               self.client_get_requests,
               self.immutable_data_count,
               self.mutable_data_count,
               self.chunk_store.used_space())
    }
}

fn close_to_address(routing_node: &mut RoutingNode, address: &XorName) -> bool {
    routing_node.close_group(*address, GROUP_SIZE).is_some()
}

fn create_pending_mutation_for_mdata(mutation_type: PendingMutationType,
                                     data: MutableData)
                                     -> PendingMutation {
    match mutation_type {
        PendingMutationType::MutateMDataEntries => PendingMutation::MutateMDataEntries(data),
        PendingMutationType::SetMDataUserPermissions => {
            PendingMutation::SetMDataUserPermissions(data)
        }
        PendingMutationType::DelMDataUserPermissions => {
            PendingMutation::DelMDataUserPermissions(data)
        }
        PendingMutationType::ChangeMDataOwner => PendingMutation::ChangeMDataOwner(data),
        PendingMutationType::PutIData |
        PendingMutationType::PutMData => unreachable!(),
    }
}

// Verify that the client with `client_name` is the owner of `data`.
fn verify_mdata_owner(data: &MutableData, client_name: &XorName) -> Result<(), ClientError> {
    if data.owners()
           .iter()
           .map(|owner_key| utils::client_name_from_key(owner_key))
           .any(|name| name == *client_name) {
        Ok(())
    } else {
        Err(ClientError::AccessDenied)
    }
}
