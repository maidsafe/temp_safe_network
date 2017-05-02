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
pub use self::data::{Data, DataId, ImmutableDataId, MutableDataId};
use GROUP_SIZE;
use accumulator::Accumulator;
use chunk_store::{Chunk, ChunkId, ChunkStore};
use error::InternalError;
use maidsafe_utilities::serialisation;
use routing::{Authority, EntryAction, ImmutableData, MessageId, MutableData, PermissionSet,
              TYPE_TAG_SESSION_PACKET, User, Value, XorName};
use routing::ClientError;
use rust_sodium::crypto::sign;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::From;
use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use utils::{self, HashMap, HashSet};
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

impl Chunk<DataId> for ImmutableData {
    type Id = ImmutableDataId;
}

impl ChunkId<DataId> for ImmutableDataId {
    type Chunk = ImmutableData;
    fn to_key(&self) -> DataId {
        DataId::Immutable(*self)
    }
}

impl Chunk<DataId> for MutableData {
    type Id = MutableDataId;
}

impl ChunkId<DataId> for MutableDataId {
    type Chunk = MutableData;
    fn to_key(&self) -> DataId {
        DataId::Mutable(*self)
    }
}

pub struct DataManager {
    chunk_store: ChunkStore<DataId>,
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

    // When a new node joins a group, the other members of the group send it "refresh"
    // messages which contains info about the data fragments that this node needs to fetch
    // from them.
    //
    // Note: refresh is a message whose source is individual node, so its accumulation
    // must be performed manually.
    pub fn handle_refresh(&mut self,
                          routing_node: &mut RoutingNode,
                          src: XorName,
                          serialised_refresh: &[u8])
                          -> Result<(), InternalError> {
        let fragments: Vec<FragmentInfo> = serialisation::deserialise(serialised_refresh)?;
        for fragment in fragments {
            if self.cache
                   .register_needed_fragment_with_another_holder(fragment.clone(), src) {
                continue;
            }

            if let Some(holders) = self.refresh_accumulator
                   .add(fragment.clone(), src)
                   .cloned() {
                self.refresh_accumulator.delete(&fragment);

                let needed = match fragment {
                    FragmentInfo::ImmutableData(name) => {
                        !self.chunk_store.has(&ImmutableDataId(name))
                    }
                    FragmentInfo::MutableDataShell { name, tag, version, .. } => {
                        match self.chunk_store.get(&MutableDataId(name, tag)) {
                            Err(_) => true,
                            Ok(data) => data.version() < version,
                        }
                    }
                    FragmentInfo::MutableDataEntry {
                        name,
                        tag,
                        ref key,
                        version,
                        ..
                    } => {
                        if let Ok(data) = self.chunk_store.get(&MutableDataId(name, tag)) {
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

                if needed {
                    for holder in holders {
                        self.cache
                            .insert_needed_fragment(fragment.clone(), holder);
                    }
                }
            }
        }

        self.request_needed_fragments(routing_node)
    }

    // When a node in a group receives request to mutate some data it holds, it first sends
    // "group refresh" message to all the other members of the group. Only when the message
    // accumulates, the node applies the mutation to the data and updates it in the chunk
    // store.
    //
    // Note: group refresh is a message whose source is group, so the accumulation is
    // performed automatically by routing.
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

            if self.commit_pending_mutation(routing_node, src, dst, mutation, message_id)? {
                match mutation_type {
                    PendingMutationType::PutIData => self.immutable_data_count += 1,
                    PendingMutationType::PutMData => self.mutable_data_count += 1,
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
                        .register_needed_data_with_another_holder(&data_id, *node);
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
        self.update_request_stats(&src);

        if let Ok(data) = self.chunk_store.get(&ImmutableDataId(name)) {
            trace!("As {:?} sending data {:?} to {:?}", dst, data, src);
            routing_node
                .send_get_idata_response(dst, src, Ok(data), msg_id)?;
        } else {
            trace!("DM sending get_idata_failure of {:?}", name);
            routing_node
                .send_get_idata_response(dst, src, Err(ClientError::NoSuchData), msg_id)?;
        }

        Ok(())
    }

    pub fn handle_get_idata_success(&mut self,
                                    routing_node: &mut RoutingNode,
                                    src: XorName,
                                    data: ImmutableData)
                                    -> Result<(), InternalError> {
        let valid = if let Some(fragment) = self.cache.stop_needed_fragment_request(&src) {
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

        let data_id = data.id();

        if self.chunk_store.has(&data_id) {
            return Ok(()); // data is already there.
        }

        self.clean_chunk_store();
        self.chunk_store.put(&data_id, &data)?;

        self.immutable_data_count += 1;
        log_status!(self);

        Ok(())
    }

    pub fn handle_get_idata_failure(&mut self,
                                    routing_node: &mut RoutingNode,
                                    src: XorName)
                                    -> Result<(), InternalError> {
        if self.cache.stop_needed_fragment_request(&src).is_none() {
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
        if self.chunk_store.has(&data.id()) {
            trace!("DM sending PutIData success for data {:?}, it already exists.",
                   data.name());
            routing_node
                .send_put_idata_response(dst, src, Ok(()), msg_id)?;
            return Ok(());
        }

        self.clean_chunk_store();

        if self.chunk_store_full() {
            let err = ClientError::NetworkFull;
            routing_node
                .send_put_idata_response(dst, src, Err(err.clone()), msg_id)?;
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
        let data_id = data.id();
        let rejected = if self.chunk_store.has(&data_id) {
            trace!("DM sending PutMData failure for data {:?}, it already exists.",
                   data_id);
            routing_node
                .send_put_mdata_response(dst, src, Err(ClientError::DataExists), msg_id)?;
            true
        } else {
            self.clean_chunk_store();

            if self.chunk_store_full() {
                let err = ClientError::NetworkFull;
                routing_node
                    .send_put_mdata_response(dst, src, Err(err), msg_id)?;
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
        self.update_request_stats(&src);
        let res = self.fetch_mdata(name, tag).map(|data| data.shell());
        routing_node
            .send_get_mdata_shell_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_get_mdata_shell_success(&mut self,
                                          routing_node: &mut RoutingNode,
                                          src: XorName,
                                          mut shell: MutableData)
                                          -> Result<(), InternalError> {
        let valid = if let Some(fragment) = self.cache.stop_needed_fragment_request(&src) {
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

        let data_id = shell.id();
        let new = match self.chunk_store.get(&data_id) {
            Ok(ref old_data) if old_data.version() >= shell.version() => {
                // The data in the chunk store is already more recent than the
                // shell we received. Ignore it.
                return Ok(());
            }
            Ok(ref old_data) => {
                // The shell is more recent than the data in the chunk store.
                // Repace the data with the shell, but keep the entries.
                for (key, value) in old_data.entries() {
                    shell.mutate_entry_without_validation(key.clone(), value.clone());
                }

                false
            }
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
        self.chunk_store.put(&data_id, &shell)?;

        if new {
            self.mutable_data_count += 1;
            log_status!(self);
        }

        Ok(())
    }

    pub fn handle_get_mdata_shell_failure(&mut self,
                                          routing_node: &mut RoutingNode,
                                          src: XorName)
                                          -> Result<(), InternalError> {
        if self.cache.stop_needed_fragment_request(&src).is_none() {
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
        self.update_request_stats(&src);
        let res = self.fetch_mdata(name, tag).map(|data| data.version());
        routing_node
            .send_get_mdata_version_response(dst, src, res, msg_id)?;
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
        self.update_request_stats(&src);
        let res = self.fetch_mdata(name, tag)
            .map(|data| data.entries().clone());
        routing_node
            .send_list_mdata_entries_response(dst, src, res, msg_id)?;
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
        self.update_request_stats(&src);
        let res = self.fetch_mdata(name, tag)
            .map(|data| data.entries().keys().cloned().collect());
        routing_node
            .send_list_mdata_keys_response(dst, src, res, msg_id)?;
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
        self.update_request_stats(&src);
        let res = self.fetch_mdata(name, tag)
            .map(|data| data.entries().values().cloned().collect());
        routing_node
            .send_list_mdata_values_response(dst, src, res, msg_id)?;
        Ok(())
    }

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
    pub fn handle_get_mdata_value(&mut self,
                                  routing_node: &mut RoutingNode,
                                  src: Authority<XorName>,
                                  dst: Authority<XorName>,
                                  name: XorName,
                                  tag: u64,
                                  key: Vec<u8>,
                                  msg_id: MessageId)
                                  -> Result<(), InternalError> {
        self.update_request_stats(&src);
        let res = self.fetch_mdata(name, tag)
            .and_then(|data| data.get(&key).cloned().ok_or(ClientError::NoSuchEntry));
        routing_node
            .send_get_mdata_value_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_get_mdata_value_success(&mut self,
                                          routing_node: &mut RoutingNode,
                                          src: XorName,
                                          value: Value)
                                          -> Result<(), InternalError> {
        let info = if let Some(fragment) = self.cache.stop_needed_fragment_request(&src) {
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

        let data_id = MutableDataId(name, tag);
        match self.chunk_store.get(&data_id) {
            Ok(mut data) => {
                if data.mutate_entry_without_validation(key, value) {
                    self.clean_chunk_store();
                    self.chunk_store.put(&data_id, &data)?;
                }
            }
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
                                          src: XorName)
                                          -> Result<(), InternalError> {
        if self.cache.stop_needed_fragment_request(&src).is_none() {
            return Err(InternalError::InvalidMessage);
        }

        self.request_needed_fragments(routing_node)
    }

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
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
        self.update_request_stats(&src);
        let res = self.fetch_mdata(name, tag)
            .map(|data| data.permissions().clone());
        routing_node
            .send_list_mdata_permissions_response(dst, src, res, msg_id)?;
        Ok(())
    }

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
    pub fn handle_list_mdata_user_permissions(&mut self,
                                              routing_node: &mut RoutingNode,
                                              src: Authority<XorName>,
                                              dst: Authority<XorName>,
                                              name: XorName,
                                              tag: u64,
                                              user: User,
                                              msg_id: MessageId)
                                              -> Result<(), InternalError> {
        self.update_request_stats(&src);
        let res = self.fetch_mdata(name, tag)
            .and_then(|data| data.user_permissions(&user).map(|p| *p));
        routing_node
            .send_list_mdata_user_permissions_response(dst, src, res, msg_id)?;
        Ok(())
    }

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
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

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
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

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
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
        let data_id = DataId::mutable(name, tag);

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
                                       if !utils::verify_mdata_owner(data, &client_name) {
                                           return Err(ClientError::AccessDenied);
                                       }

                                       data.change_owner(new_owner, version)
                                   })
    }

    pub fn handle_node_added(&mut self,
                             routing_node: &mut RoutingNode,
                             node_name: &XorName)
                             -> Result<(), InternalError> {
        if self.cache
               .prune_needed_fragments(routing_node.routing_table()?) {
            let _ = self.request_needed_fragments(routing_node);
        }

        let mut refresh = Vec::new();
        let mut has_pruned_data = false;

        for fragment in self.our_fragments() {
            let data_id = fragment.data_id();

            // Only retain fragments for which we're still in the close group.
            match routing_node
                      .routing_table()?
                      .other_closest_names(data_id.name(), GROUP_SIZE) {
                None => {
                    trace!("No longer a DM for {:?}", data_id);

                    match data_id {
                        DataId::Immutable(idata_id) => {
                            if self.chunk_store.has(&idata_id) &&
                               !self.cache.is_in_unneeded(&idata_id) {
                                self.immutable_data_count -= 1;
                                has_pruned_data = true;
                                self.cache.add_as_unneeded(idata_id);
                            }
                        }
                        DataId::Mutable(mdata_id) => {
                            if self.chunk_store.has(&mdata_id) {
                                self.mutable_data_count -= 1;
                                has_pruned_data = true;
                                let _ = self.chunk_store.delete(&mdata_id);
                            }
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

        Ok(())
    }

    /// Get all names and hashes of all data. // [TODO]: Can be optimised - 2016-04-23 09:11pm
    /// Send to all members of group of data.
    pub fn handle_node_lost(&mut self,
                            routing_node: &mut RoutingNode,
                            node_name: &XorName)
                            -> Result<(), InternalError> {
        let pruned_unneeded_chunks = self.cache
            .prune_unneeded_chunks(routing_node.routing_table()?);
        if pruned_unneeded_chunks != 0 {
            self.immutable_data_count += pruned_unneeded_chunks;
            log_status!(self);
        }

        if self.cache
               .prune_needed_fragments(routing_node.routing_table()?) {
            let _ = self.request_needed_fragments(routing_node);
        }

        let mut refreshes = HashMap::default();
        for fragment in self.our_fragments() {
            match routing_node
                      .routing_table()?
                      .other_closest_names(fragment.name(), GROUP_SIZE) {
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

        Ok(())
    }

    pub fn check_timeouts(&mut self, routing_node: &mut RoutingNode) {
        let _ = self.request_needed_fragments(routing_node);
    }

    fn fetch_mdata(&self, name: XorName, tag: u64) -> Result<MutableData, ClientError> {
        let data_id = MutableDataId(name, tag);
        if let Ok(data) = self.chunk_store.get(&data_id) {
            Ok(data)
        } else if tag == TYPE_TAG_SESSION_PACKET {
            Err(ClientError::NoSuchAccount)
        } else {
            Err(ClientError::NoSuchData)
        }
    }

    fn update_request_stats(&mut self, src: &Authority<XorName>) {
        if let Authority::Client { .. } = *src {
            self.client_get_requests += 1;
            log_status!(self);
        }
    }

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
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
        let res = self.fetch_mdata(name, tag)
            .and_then(|mut data| {
                          f(&mut data)?;
                          Ok(data)
                      });

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
                                            DataId::mutable(name, tag),
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

    fn commit_pending_mutation(&mut self,
                               routing_node: &mut RoutingNode,
                               src: Authority<XorName>,
                               dst: Authority<XorName>,
                               mutation: PendingMutation,
                               msg_id: MessageId)
                               -> Result<bool, InternalError> {
        let mutation_type = mutation.mutation_type();
        let data_id = mutation.data_id();

        let res = match mutation {
            PendingMutation::PutIData(data) => self.chunk_store.put(&data.id(), &data),
            PendingMutation::PutMData(data) |
            PendingMutation::MutateMDataEntries(data) |
            PendingMutation::SetMDataUserPermissions(data) |
            PendingMutation::DelMDataUserPermissions(data) |
            PendingMutation::ChangeMDataOwner(data) => self.chunk_store.put(&data.id(), &data),
        };

        let res = if let Err(error) = res {
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

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
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
                routing_node
                    .send_put_idata_response(dst, src, res, msg_id)?
            }
            PendingMutationType::PutMData => {
                trace!("DM sending PutMData {} for data {:?}", res_string, data_id);
                routing_node
                    .send_put_mdata_response(dst, src, res, msg_id)?
            }
            PendingMutationType::MutateMDataEntries => {
                trace!("DM sending MutateMDataEntries {} for data {:?}",
                       res_string,
                       data_id);
                routing_node
                    .send_mutate_mdata_entries_response(dst, src, res, msg_id)?
            }
            PendingMutationType::SetMDataUserPermissions => {
                trace!("DM sending SetMDataUserPermissions {} for data {:?}",
                       res_string,
                       data_id);
                routing_node
                    .send_set_mdata_user_permissions_response(dst, src, res, msg_id)?
            }
            PendingMutationType::DelMDataUserPermissions => {
                trace!("DM sending DelMDataUserPermissions {} for data {:?}",
                       res_string,
                       data_id);
                routing_node
                    .send_del_mdata_user_permissions_response(dst, src, res, msg_id)?
            }
            PendingMutationType::ChangeMDataOwner => {
                trace!("DM sending ChangeMDataOwner {} for data {:?}",
                       res_string,
                       data_id);
                routing_node
                    .send_change_mdata_owner_response(dst, src, res, msg_id)?
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
                if let Err(error) = self.chunk_store.delete(&data_id) {
                    error!("DM failed to delete unneeded chunk {:?}: {:?}",
                           data_id,
                           error);
                    break;
                }
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
        routing_node
            .send_refresh_request(src, dst, serialised_refresh, MessageId::new())?;
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
                routing_node
                    .send_refresh_request(Authority::NaeManager(name),
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

        // Set of holders we already sent a request to. Used to prevent sending
        // multiple request to a single holder.
        let mut busy_holders = HashSet::default();

        for (fragment, holders) in candidates {
            for holder in holders {
                if let Some(group) = routing_node.close_group(*fragment.name(), GROUP_SIZE) {
                    if group.contains(&holder) {
                        if !busy_holders.insert(holder) {
                            continue;
                        }

                        let dst = Authority::ManagedNode(holder);
                        let msg_id = MessageId::new();

                        self.cache
                            .start_needed_fragment_request(&fragment, &holder);

                        match fragment {
                            FragmentInfo::ImmutableData(name) => {
                                routing_node
                                    .send_get_idata_request(src, dst, name, msg_id)?;
                            }
                            FragmentInfo::MutableDataShell { name, tag, .. } => {
                                routing_node
                                    .send_get_mdata_shell_request(src, dst, name, tag, msg_id)?;
                            }
                            FragmentInfo::MutableDataEntry { name, tag, ref key, .. } => {
                                routing_node
                                    .send_get_mdata_value_request(src,
                                                                  dst,
                                                                  name,
                                                                  tag,
                                                                  key.clone(),
                                                                  msg_id)?;
                            }
                        }

                        break;
                    }
                }
            }
        }

        self.cache.print_stats();
        Ok(())
    }

    // Get all fragments for the data we have in the chunk store.
    fn chunk_store_fragments(&self) -> HashSet<FragmentInfo> {
        let mut result = HashSet::default();

        for data_id in self.chunk_store.keys() {
            match data_id {
                DataId::Immutable(data_id) => {
                    let _ = result.insert(FragmentInfo::ImmutableData(*data_id.name()));
                }
                DataId::Mutable(data_id) => {
                    let data = if let Ok(data) = self.chunk_store.get(&data_id) {
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
        let mut result = self.cache.needed_fragments();
        result.extend(self.chunk_store_fragments());
        result
    }
}

#[cfg(feature = "use-mock-crust")]
impl DataManager {
    pub fn get_stored_ids(&self) -> Vec<(DataId, u64)> {
        self.chunk_store
            .keys()
            .into_iter()
            .filter(|data_id| match *data_id {
                        DataId::Immutable(ref id) => !self.cache.is_in_unneeded(id),
                        DataId::Mutable(_) => true,
                    })
            .filter_map(|data_id| {
                            self.get_version(&data_id)
                                .map(|version| (data_id, version))
                        })
            .collect()
    }

    fn get_version(&self, data_id: &DataId) -> Option<u64> {
        match *data_id {
            DataId::Immutable(_) => Some(0),
            DataId::Mutable(ref data_id) => {
                if let Ok(data) = self.chunk_store.get(data_id) {
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
    pub fn put_into_chunk_store<T, I>(&mut self, data: T)
        where T: Chunk<DataId, Id = I> + Data<Id = I>,
              I: ChunkId<DataId>
    {
        unwrap!(self.chunk_store.put(&data.id(), &data))
    }

    /// For testing only - retrieve data with the given ID from the chunk store.
    pub fn get_from_chunk_store<T: ChunkId<DataId>>(&self, data_id: &T) -> Option<T::Chunk> {
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
