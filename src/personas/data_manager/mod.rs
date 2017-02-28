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

use self::cache::{Cache, PendingMutation, PendingMutationType, PendingWrite, RefreshData,
                  RefreshDataList};
use self::data::{Data, DataId, VersionedDataId};
use GROUP_SIZE;
use accumulator::Accumulator;
use chunk_store::ChunkStore;
use error::InternalError;
use maidsafe_utilities::serialisation;
use routing::{Authority, EntryAction, ImmutableData, MessageId, MutableData, PermissionSet,
              RoutingTable, User, XorName};
use routing::ClientError;
use rust_sodium::crypto::sign;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::convert::From;
use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;
use std::time::{Duration, Instant};
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
    refresh_accumulator: Accumulator<VersionedDataId, XorName>,
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
                          serialised_data_list: &[u8])
                          -> Result<(), InternalError> {
        let RefreshDataList(data_list) = serialisation::deserialise(serialised_data_list)?;
        for vid in data_list {
            if self.cache.register_data_with_holder(&src, &vid) {
                continue;
            }

            if let Some(holders) = self.refresh_accumulator.add(vid, src).cloned() {
                self.refresh_accumulator.delete(&vid);

                let (data_id, version) = vid;
                let data_needed = match data_id {
                    DataId::Immutable(..) => !self.chunk_store.has(&data_id),
                    DataId::Mutable(..) => {
                        match self.chunk_store.get(&data_id) {
                            Err(_) => true,
                            Ok(Data::Mutable(data)) => data.version() < version,
                            Ok(_) => unreachable!(),
                        }
                    }
                };

                if !data_needed {
                    continue;
                }

                self.cache.add_records(vid, holders);
            }
        }

        self.send_gets_for_needed_data(routing_node)
    }

    /// Handles an accumulated refresh message sent from the whole group.
    pub fn handle_group_refresh(&mut self,
                                routing_node: &mut RoutingNode,
                                serialised_refresh: &[u8])
                                -> Result<(), InternalError> {
        let RefreshData { versioned_data_id: (data_id, version), hash: refresh_hash } =
            serialisation::deserialise(serialised_refresh)?;
        let mut success = false;

        for PendingWrite { mutation, src, dst, message_id, hash, rejected, .. } in
            self.cache.take_pending_writes(&data_id) {
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
            if self.handle_pending_mutation(routing_node, src, dst, mutation, message_id)? {
                match mutation_type {
                    PendingMutationType::PutIData |
                    PendingMutationType::PutMData => self.count_added_data(&data_id),
                    _ => (),
                }

                log_status!(self);
                success = true;
            }

            let data_list = vec![(data_id, version)];
            self.send_refresh(routing_node,
                              Authority::NaeManager(*data_id.name()),
                              data_list)?;
        }

        if !success {
            if let Some(group) = routing_node.close_group(*data_id.name(), GROUP_SIZE) {
                let vid = (data_id, version);
                for node in &group {
                    let _ = self.cache.register_data_with_holder(node, &vid);
                }

                self.send_gets_for_needed_data(routing_node)?;
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
        self.cache.handle_get_idata_success(src, data.name(), msg_id);
        self.send_gets_for_needed_data(routing_node)?;

        // If we're no longer in the close group, return.
        if !close_to_address(routing_node, data.name()) {
            return Ok(());
        }

        let data = Data::Immutable(data);
        let data_id = data.id();

        // TODO: Check that the data's hash actually agrees with an accumulated entry.
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
        if !self.cache.handle_get_idata_failure(src, msg_id) {
            return Err(InternalError::InvalidMessage);
        }

        self.send_gets_for_needed_data(routing_node)
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
                routing_node.send_put_mdata_response(dst, src, Err(err.clone()), msg_id)?;
                return Err(From::from(err));
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
        let res = self.read_mdata(&src,
                                  name,
                                  tag,
                                  |data| data.get(&key).cloned().ok_or(ClientError::NoSuchEntry));
        routing_node.send_get_mdata_value_response(dst, src, res, msg_id)?;
        Ok(())
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

        let num_owners_len = new_owners.len();
        let new_owner = match new_owners.into_iter().next() {
            Some(owner) if num_owners_len == 1 => owner,
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

        let requester = match src {
            Authority::Client { client_key, .. } => client_key,
            _ => {
                self.send_mutation_response(routing_node,
                                            src,
                                            dst,
                                            mutation_type,
                                            data_id,
                                            Err(ClientError::InvalidOperation),
                                            msg_id)?;
                return Ok(());
            }
        };

        self.handle_mdata_mutation(routing_node,
                                   src,
                                   dst,
                                   name,
                                   tag,
                                   mutation_type,
                                   msg_id,
                                   |data| data.change_owner(new_owner, version, requester))
    }

    pub fn handle_node_added(&mut self,
                             routing_node: &mut RoutingNode,
                             node_name: &XorName,
                             routing_table: &RoutingTable<XorName>) {
        self.cache.prune_data_holders(routing_table);
        if self.cache.prune_ongoing_gets(routing_table) {
            let _ = self.send_gets_for_needed_data(routing_node);
        }

        let vids = self.cache.chain_records_in_cache(self.chunk_store
            .keys()
            .into_iter()
            .filter_map(|data_id| self.to_versioned_data_id(data_id)));

        let mut has_pruned_data = false;
        // Only retain data for which we're still in the close group.
        let mut data_list = Vec::new();

        for (data_id, version) in vids {
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
                        data_list.push((data_id, version));
                    }
                }
            }
        }

        if !data_list.is_empty() {
            let _ = self.send_refresh(routing_node, Authority::ManagedNode(*node_name), data_list);
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

        self.cache.prune_data_holders(routing_table);

        if self.cache.prune_ongoing_gets(routing_table) {
            let _ = self.send_gets_for_needed_data(routing_node);
        }

        let vids = self.cache.chain_records_in_cache(self.chunk_store
            .keys()
            .into_iter()
            .filter_map(|data_id| self.to_versioned_data_id(data_id)));
        let mut data_lists = HashMap::new();

        for vid in vids {
            match routing_table.other_closest_names(vid.0.name(), GROUP_SIZE) {
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
                        if vid.0.name().closer(node_name, outer_node) {
                            data_lists.entry(*outer_node).or_insert_with(Vec::new).push(vid);
                        }
                    }
                }
            }
        }

        for (node_name, data_list) in data_lists {
            let _ = self.send_refresh(routing_node, Authority::ManagedNode(node_name), data_list);
        }
    }

    pub fn check_timeouts(&mut self, routing_node: &mut RoutingNode) {
        let _ = self.send_gets_for_needed_data(routing_node);
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
        for PendingWrite { mutation, src, dst, message_id, .. } in
            self.cache.remove_expired_writes() {
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
        if let Some(refresh_data) =
            self.cache.insert_pending_write(mutation, src, dst, message_id, rejected) {
            self.send_group_refresh(routing_node, data_name, refresh_data, message_id)?;
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
                    data_list: Vec<VersionedDataId>)
                    -> Result<(), InternalError> {
        // FIXME - We need to handle >2MB chunks
        let src = Authority::ManagedNode(routing_node.name()?);
        let serialised_list = serialisation::serialise(&RefreshDataList(data_list))?;
        trace!("DM sending refresh to {:?}.", dst);
        routing_node.send_refresh_request(src, dst, serialised_list, MessageId::new())?;
        Ok(())
    }

    fn send_group_refresh(&self,
                          routing_node: &mut RoutingNode,
                          name: XorName,
                          refresh_data: RefreshData,
                          msg_id: MessageId)
                          -> Result<(), InternalError> {
        match serialisation::serialise(&refresh_data) {
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

    fn send_gets_for_needed_data(&mut self,
                                 routing_node: &mut RoutingNode)
                                 -> Result<(), InternalError> {
        let src = Authority::ManagedNode(routing_node.name()?);
        let candidates = self.cache.needed_data();

        for (idle_holder, vid) in candidates {
            if let Some(group) = routing_node.close_group(*vid.0.name(), GROUP_SIZE) {
                if group.contains(&idle_holder) {
                    let (data_id, _) = vid;

                    // TODO: figure out how to get mutable data too.
                    if let DataId::Immutable(name) = data_id {
                        let msg_id = MessageId::new();
                        self.cache.insert_into_ongoing_gets(idle_holder, vid, msg_id);

                        let dst = Authority::ManagedNode(idle_holder);
                        let _ = routing_node.send_get_idata_request(src, dst, name, msg_id);
                    }
                }
            }
        }

        self.cache.print_stats();
        Ok(())
    }

    /// Returns the `VersionedDataId` for the given data identifier, or `None` if not stored.
    fn to_versioned_data_id(&self, data_id: DataId) -> Option<VersionedDataId> {
        match data_id {
            DataId::Immutable(_) => Some((data_id, 0)),
            DataId::Mutable(..) => {
                if let Ok(Data::Mutable(data)) = self.chunk_store.get(&data_id) {
                    Some((data_id, data.version()))
                } else {
                    error!("Failed to get {:?} from chunk store.", data_id);
                    None
                }
            }
        }
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

    /*
    #[cfg(feature = "use-mock-crust")]
    pub fn get_stored_names(&self) -> Vec<IdAndVersion> {
        let (front, back) = self.cache.unneeded_chunks.as_slices();
        self.chunk_store
            .keys()
            .into_iter()
            .filter(|data_id| !front.contains(data_id) && !back.contains(data_id))
            .filter_map(|data_id| self.to_id_and_version(data_id))
            .collect()
    }
    */
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

#[cfg(all(test, feature = "use-mock-routing"))]
mod tests {
    use super::*;
    use rand;
    use routing::{Authority, EntryActions, Request, Response};
    use std::env;
    use test_utils;

    const CHUNK_STORE_CAPACITY: u64 = 1024;
    const CHUNK_STORE_DIR: &'static str = "test_safe_vault_chunk_store";

    const TEST_TAG: u64 = 12345678;

    #[test]
    fn idata_basics() {
        let (client, client_key) = test_utils::gen_client_authority();
        let client_manager = test_utils::gen_client_manager_authority(client_key);

        let data = test_utils::gen_random_immutable_data(10, &mut rand::thread_rng());
        let nae_manager = Authority::NaeManager(*data.name());

        let mut node = RoutingNode::new();
        let mut dm = create_data_manager();

        // Get non-existent data fails.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_get_idata(&mut node, client, nae_manager, *data.name(), msg_id));

        let message = unwrap!(node.sent_responses.remove(&msg_id));
        assert_match!(
            message.response,
            Response::GetIData { res: Err(ClientError::NoSuchData), .. });

        // Put immutable data sends refresh to the NAE manager.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_put_idata(&mut node, client_manager, nae_manager, data.clone(), msg_id));

        let message = unwrap!(node.sent_requests.remove(&msg_id));
        let refresh = assert_match!(message.request, Request::Refresh(payload, _) => payload);
        assert_eq!(message.src, nae_manager);
        assert_eq!(message.dst, nae_manager);

        // Simulate receiving the refresh. This should result in the data being
        // put into the chunk store.
        unwrap!(dm.handle_group_refresh(&mut node, &refresh));

        // Get the data back and assert its the same data we put in originally.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_get_idata(&mut node, client, nae_manager, *data.name(), msg_id));

        let message = unwrap!(node.sent_responses.remove(&msg_id));
        let retrieved_data =
            assert_match!(message.response, Response::GetIData { res: Ok(data), .. } => data);
        assert_eq!(retrieved_data, data);
    }

    #[test]
    fn mdata_basics() {
        let mut rng = rand::thread_rng();

        let (client, client_key) = test_utils::gen_client_authority();
        let client_manager = test_utils::gen_client_manager_authority(client_key);

        let data = test_utils::gen_empty_mutable_data(TEST_TAG, client_key, &mut rng);
        let data_name = *data.name();
        let nae_manager = Authority::NaeManager(data_name);

        let mut node = RoutingNode::new();
        let mut dm = create_data_manager();

        // Attempt to list entries of non-existent data fails.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_list_mdata_entries(&mut node,
                                             client,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             msg_id));
        let message = unwrap!(node.sent_responses.remove(&msg_id));
        assert_match!(
            message.response,
            Response::ListMDataEntries { res: Err(ClientError::NoSuchData), .. });

        // Put mutable data sends refresh to the NAE manager.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_put_mdata(&mut node,
                                    client_manager,
                                    nae_manager,
                                    data,
                                    msg_id,
                                    client_key));

        let message = unwrap!(node.sent_requests.remove(&msg_id));
        let refresh = assert_match!(message.request, Request::Refresh(payload, _) => payload);

        // Simulate receiving the refresh. This should result in the data being
        // put into the chunk store.
        unwrap!(dm.handle_group_refresh(&mut node, &refresh));

        let message = unwrap!(node.sent_responses.remove(&msg_id));
        assert_match!(message.response, Response::PutMData { res: Ok(()), .. });

        // Now list the data entries - should successfuly respond with empty list.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_list_mdata_entries(&mut node,
                                             client,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             msg_id));

        let message = unwrap!(node.sent_responses.remove(&msg_id));
        let entries = assert_match!(
            message.response,
            Response::ListMDataEntries { res: Ok(entries), .. } => entries);
        assert!(entries.is_empty());
    }

    #[test]
    fn mdata_mutations() {
        let mut rng = rand::thread_rng();

        let (client, client_key) = test_utils::gen_client_authority();
        let client_manager = test_utils::gen_client_manager_authority(client_key);

        let data = test_utils::gen_empty_mutable_data(TEST_TAG, client_key, &mut rng);
        let data_name = *data.name();
        let nae_manager = Authority::NaeManager(data_name);

        let mut node = RoutingNode::new();
        let mut dm = create_data_manager();

        // Put the data.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_put_mdata(&mut node,
                                    client_manager,
                                    nae_manager,
                                    data,
                                    msg_id,
                                    client_key));
        let message = unwrap!(node.sent_requests.remove(&msg_id));
        let refresh = assert_match!(message.request, Request::Refresh(payload, _) => payload);
        unwrap!(dm.handle_group_refresh(&mut node, &refresh));

        // Initially, the entries should be empty.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_list_mdata_entries(&mut node,
                                             client,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             msg_id));

        let message = unwrap!(node.sent_responses.remove(&msg_id));
        let entries = assert_match!(
            message.response,
            Response::ListMDataEntries { res: Ok(entries), .. } => entries);
        assert!(entries.is_empty());

        // Mutate the entries and simulate refresh.
        let key_0 = test_utils::gen_random_vec(10, &mut rng);
        let value_0 = test_utils::gen_random_vec(10, &mut rng);

        let key_1 = test_utils::gen_random_vec(10, &mut rng);
        let value_1 = test_utils::gen_random_vec(10, &mut rng);

        let actions = EntryActions::new()
            .ins(key_0.clone(), value_0.clone(), 0)
            .ins(key_1.clone(), value_1.clone(), 0)
            .into();
        let msg_id = MessageId::new();
        unwrap!(dm.handle_mutate_mdata_entries(&mut node,
                                               client,
                                               nae_manager,
                                               data_name,
                                               TEST_TAG,
                                               actions,
                                               msg_id,
                                               client_key));

        let message = unwrap!(node.sent_requests.remove(&msg_id));
        let refresh = assert_match!(message.request, Request::Refresh(payload, _) => payload);
        unwrap!(dm.handle_group_refresh(&mut node, &refresh));

        let message = unwrap!(node.sent_responses.remove(&msg_id));
        assert_match!(message.response, Response::MutateMDataEntries { res: Ok(()), .. });

        // The data should now contain the previously inserted two entries.
        let msg_id = MessageId::new();
        unwrap!(dm.handle_list_mdata_entries(&mut node,
                                             client,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             msg_id));

        let message = unwrap!(node.sent_responses.remove(&msg_id));
        let entries = assert_match!(
            message.response,
            Response::ListMDataEntries { res: Ok(entries), .. } => entries);
        assert_eq!(entries.len(), 2);
        let retrieved_value_0 = unwrap!(entries.get(&key_0));
        let retrieved_value_1 = unwrap!(entries.get(&key_1));

        assert_eq!(retrieved_value_0.content, value_0);
        assert_eq!(retrieved_value_1.content, value_1);
    }

    fn create_data_manager() -> DataManager {
        unwrap!(DataManager::new(env::temp_dir().join(CHUNK_STORE_DIR), CHUNK_STORE_CAPACITY))
    }
}
