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

mod account;
#[cfg(all(test, feature = "use-mock-routing"))]
mod tests;

use self::account::Account;
pub use self::account::DEFAULT_ACCOUNT_SIZE;
use GROUP_SIZE;
use error::InternalError;
use itertools::Itertools;
use maidsafe_utilities::serialisation;
use routing::{Authority, ImmutableData, MessageId, MutableData, RoutingTable,
              TYPE_TAG_SESSION_PACKET, XorName};
use routing::ClientError;
use rust_sodium::crypto::sign;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::convert::From;
use utils;
use vault::RoutingNode;

pub struct MaidManager {
    accounts: HashMap<XorName, Account>,
    request_cache: HashMap<MessageId, CachedRequest>,
}

impl MaidManager {
    pub fn new() -> MaidManager {
        MaidManager {
            accounts: HashMap::new(),
            request_cache: HashMap::new(),
        }
    }

    pub fn handle_refresh(&mut self,
                          routing_node: &mut RoutingNode,
                          serialised_msg: &[u8])
                          -> Result<(), InternalError> {
        match serialisation::deserialise::<Refresh>(serialised_msg)? {
            Refresh::Update(maid_name, account) => {
                if routing_node.close_group(maid_name, GROUP_SIZE).is_none() {
                    return Ok(());
                }
                let account_count = self.accounts.len();
                match self.accounts.entry(maid_name) {
                    Entry::Vacant(entry) => {
                        let _ = entry.insert(account);
                        info!("Managing {} client accounts.", account_count + 1);
                    }
                    Entry::Occupied(mut entry) => {
                        if entry.get().version < account.version {
                            trace!("Client account {:?}: {:?}", maid_name, account);
                            let _ = entry.insert(account);
                        }
                    }
                }
            }
            Refresh::Delete(maid_name) => {
                let _ = self.accounts.remove(&maid_name);
                info!("Managing {} client accounts.", self.accounts.len());
            }
        }
        Ok(())
    }

    pub fn handle_get_account_info(&mut self,
                                   routing_node: &mut RoutingNode,
                                   src: Authority<XorName>,
                                   dst: Authority<XorName>,
                                   msg_id: MessageId)
                                   -> Result<(), InternalError> {
        let res = self.get_account(&dst).map(|account| account.info);
        routing_node.send_get_account_info_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_put_idata(&mut self,
                            routing_node: &mut RoutingNode,
                            src: Authority<XorName>,
                            dst: Authority<XorName>,
                            data: ImmutableData,
                            msg_id: MessageId)
                            -> Result<(), InternalError> {
        if !data.validate_size() {
            routing_node.send_put_idata_response(dst, src, Err(ClientError::DataTooLarge), msg_id)?;
            return Ok(());
        }

        if let Err(err) = self.increment_mutation_counter(&utils::client_name(&src)) {
            routing_node.send_put_idata_response(dst, src, Err(err), msg_id)?;
            return Ok(());
        }

        {
            // Forwarding the request to NAE Manager.
            let src = dst;
            let dst = Authority::NaeManager(*data.name());
            trace!("MM forwarding PutIData request to {:?}", dst);
            routing_node.send_put_idata_request(src, dst, data, msg_id)?;
        }

        if let Some(prior) = self.request_cache.insert(msg_id,
                                                       CachedRequest {
                                                           src: src,
                                                           dst: dst,
                                                           tag: None,
                                                       }) {
            error!("Overwrote existing cached request with {:?} from {:?} to {:?}",
                   msg_id,
                   prior.src,
                   prior.dst);
        }

        Ok(())
    }

    pub fn handle_put_idata_success(&mut self,
                                    routing_node: &mut RoutingNode,
                                    msg_id: MessageId)
                                    -> Result<(), InternalError> {
        let CachedRequest { src, dst, .. } = self.remove_cached_request(msg_id)?;

        // Send success response back to client
        let client_name = utils::client_name(&src);
        let account = if let Some(account) = self.accounts.get(&client_name) {
            account.clone()
        } else {
            error!("Account for {:?} not found.", client_name);
            return Err(InternalError::NoSuchAccount);
        };

        self.send_refresh(routing_node, &client_name, account, MessageId::zero());
        let _ = routing_node.send_put_idata_response(dst, src, Ok(()), msg_id);

        Ok(())
    }

    pub fn handle_put_idata_failure(&mut self,
                                    routing_node: &mut RoutingNode,
                                    error: ClientError,
                                    msg_id: MessageId)
                                    -> Result<(), InternalError> {
        let CachedRequest { src, dst, .. } = self.remove_cached_request(msg_id)?;

        if !self.handle_put_failure(routing_node, &src) {
            return Ok(());
        }

        // Send failure response back to client
        routing_node.send_put_idata_response(dst, src, Err(error), msg_id)?;

        Ok(())
    }

    pub fn handle_put_mdata(&mut self,
                            routing_node: &mut RoutingNode,
                            src: Authority<XorName>,
                            dst: Authority<XorName>,
                            data: MutableData,
                            msg_id: MessageId,
                            requester: sign::PublicKey)
                            -> Result<(), InternalError> {
        if let Err(err) = data.validate() {
            routing_node.send_put_mdata_response(dst, src, Err(err), msg_id)?;
            return Ok(());
        }

        let client_name = utils::client_name(&src);

        // If the type_tag is `TYPE_TAG_SESSION_PACKET`, the account must not exist, else it must
        // exist.
        if data.tag() == TYPE_TAG_SESSION_PACKET {
            if dst.name() != client_name {
                trace!("Cannot create account for {:?} as {:?}.", src, dst);
                let err = ClientError::InvalidOperation;
                routing_node.send_put_mdata_response(dst, src, Err(err.clone()), msg_id)?;
                return Err(From::from(err));
            }

            if self.accounts.contains_key(&client_name) {
                let err = ClientError::AccountExists;
                routing_node.send_put_mdata_response(dst, src, Err(err.clone()), msg_id)?;
                return Err(From::from(err));
            }

            // Create the account.
            let _ = self.accounts.insert(client_name, Account::default());
            info!("Managing {} client accounts.", self.accounts.len());
        }

        if let Err(err) = self.increment_mutation_counter(&client_name) {
            // Undo the account creation
            if data.tag() == TYPE_TAG_SESSION_PACKET {
                let _ = self.accounts.remove(&client_name);
            }

            routing_node.send_put_mdata_response(dst, src, Err(err.clone()), msg_id)?;
            return Err(From::from(err));
        }

        let tag = data.tag();

        {
            // Forwarding the request to NAE Manager.
            let src = dst;
            let dst = Authority::NaeManager(*data.name());
            trace!("MM forwarding PutMData request to {:?}", dst);
            routing_node.send_put_mdata_request(src, dst, data, msg_id, requester)?;
        }

        if let Some(prior) = self.request_cache.insert(msg_id,
                                                       CachedRequest {
                                                           src: src,
                                                           dst: dst,
                                                           tag: Some(tag),
                                                       }) {
            error!("Overwrote existing cached request with {:?} from {:?} to {:?}",
                   msg_id,
                   prior.src,
                   prior.dst);
        }

        Ok(())
    }

    pub fn handle_put_mdata_success(&mut self,
                                    routing_node: &mut RoutingNode,
                                    msg_id: MessageId)
                                    -> Result<(), InternalError> {
        let CachedRequest { src, dst, .. } = self.remove_cached_request(msg_id)?;

        // Send success response back to client
        let client_name = utils::client_name(&src);
        let account = if let Some(account) = self.accounts.get(&client_name) {
            account.clone()
        } else {
            error!("Account for {:?} not found.", client_name);
            return Err(InternalError::NoSuchAccount);
        };

        self.send_refresh(routing_node, &client_name, account, MessageId::zero());
        let _ = routing_node.send_put_mdata_response(dst, src, Ok(()), msg_id);
        Ok(())
    }

    pub fn handle_put_mdata_failure(&mut self,
                                    routing_node: &mut RoutingNode,
                                    error: ClientError,
                                    msg_id: MessageId)
                                    -> Result<(), InternalError> {
        let CachedRequest { src, dst, tag } = self.remove_cached_request(msg_id)?;

        if !self.handle_put_failure(routing_node, &src) {
            return Ok(());
        }

        let error = match (tag, error) {
            (Some(TYPE_TAG_SESSION_PACKET), ClientError::DataExists) => {
                // We wouldn't have forwarded two `Put` requests for the same account, so
                // it must have been created via another client manager.
                let client_name = utils::client_name(&src);
                let _ = self.accounts.remove(&client_name);
                let refresh = Refresh::Delete(client_name);
                if let Ok(serialised_refresh) = serialisation::serialise(&refresh) {
                    trace!("MM sending delete refresh for account {}", src.name());
                    let _ = routing_node.send_refresh_request(dst, dst, serialised_refresh, msg_id);
                }

                ClientError::AccountExists
            }
            (_, error) => error,
        };

        // Send failure response back to client
        routing_node.send_put_mdata_response(dst, src, Err(error), msg_id)?;
        Ok(())
    }

    pub fn handle_mutate_mdata_entries(&mut self,
                                       routing_node: &mut RoutingNode,
                                       src: Authority<XorName>,
                                       dst: Authority<XorName>,
                                       msg_id: MessageId,
                                       requester: sign::PublicKey)
                                       -> Result<(), InternalError> {
        if let Err(err) = self.prepare_mdata_mutation(&src, &dst, Some(requester)) {
            routing_node.send_mutate_mdata_entries_response(dst, src, Err(err.clone()), msg_id)?;
            Err(From::from(err))
        } else {
            Ok(())
        }
    }

    pub fn handle_set_mdata_user_permissions(&mut self,
                                             routing_node: &mut RoutingNode,
                                             src: Authority<XorName>,
                                             dst: Authority<XorName>,
                                             msg_id: MessageId,
                                             requester: sign::PublicKey)
                                             -> Result<(), InternalError> {
        if let Err(err) = self.prepare_mdata_mutation(&src, &dst, Some(requester)) {
            routing_node.send_set_mdata_user_permissions_response(dst,
                                                                  src,
                                                                  Err(err.clone()),
                                                                  msg_id)?;
            Err(From::from(err))
        } else {
            Ok(())
        }
    }

    pub fn handle_del_mdata_user_permissions(&mut self,
                                             routing_node: &mut RoutingNode,
                                             src: Authority<XorName>,
                                             dst: Authority<XorName>,
                                             msg_id: MessageId,
                                             requester: sign::PublicKey)
                                             -> Result<(), InternalError> {
        if let Err(err) = self.prepare_mdata_mutation(&src, &dst, Some(requester)) {
            routing_node.send_del_mdata_user_permissions_response(dst,
                                                                  src,
                                                                  Err(err.clone()),
                                                                  msg_id)?;
            Err(From::from(err))
        } else {
            Ok(())
        }
    }

    pub fn handle_change_mdata_owner(&mut self,
                                     routing_node: &mut RoutingNode,
                                     src: Authority<XorName>,
                                     dst: Authority<XorName>,
                                     msg_id: MessageId)
                                     -> Result<(), InternalError> {
        if let Err(err) = self.prepare_mdata_mutation(&src, &dst, None) {
            routing_node.send_change_mdata_owner_response(dst, src, Err(err.clone()), msg_id)?;
            Err(From::from(err))
        } else {
            Ok(())
        }
    }

    pub fn handle_list_auth_keys_and_version(&mut self,
                                             routing_node: &mut RoutingNode,
                                             src: Authority<XorName>,
                                             dst: Authority<XorName>,
                                             msg_id: MessageId)
                                             -> Result<(), InternalError> {
        let res = self.get_account(&dst)
            .map(|account| (account.auth_keys.clone(), account.version));
        routing_node.send_list_auth_keys_and_version_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_ins_auth_key(&mut self,
                               routing_node: &mut RoutingNode,
                               src: Authority<XorName>,
                               dst: Authority<XorName>,
                               key: sign::PublicKey,
                               version: u64,
                               msg_id: MessageId)
                               -> Result<(), InternalError> {
        let res = self.mutate_account(&src, &dst, version, |account| {
            let _ = account.auth_keys.insert(key);
            Ok(())
        });
        routing_node.send_ins_auth_key_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_del_auth_key(&mut self,
                               routing_node: &mut RoutingNode,
                               src: Authority<XorName>,
                               dst: Authority<XorName>,
                               key: sign::PublicKey,
                               version: u64,
                               msg_id: MessageId)
                               -> Result<(), InternalError> {
        let res = self.mutate_account(&src,
                                      &dst,
                                      version,
                                      |account| if account.auth_keys.remove(&key) {
                                          Ok(())
                                      } else {
                                          Err(ClientError::NoSuchKey)
                                      });
        routing_node.send_del_auth_key_response(dst, src, res, msg_id)?;
        Ok(())
    }

    pub fn handle_node_added(&mut self,
                             routing_node: &mut RoutingNode,
                             node_name: &XorName,
                             routing_table: &RoutingTable<XorName>) {
        // Remove all accounts which we are no longer responsible for.
        let not_close = |name: &&XorName| !routing_table.is_closest(*name, GROUP_SIZE);
        let accounts_to_delete = self.accounts.keys().filter(not_close).cloned().collect_vec();
        // Remove all requests from the cache that we are no longer responsible for.
        let msg_ids_to_delete = self.request_cache
            .iter()
            .filter(|&(_, entry)| accounts_to_delete.contains(&entry.src.name()))
            .map(|(msg_id, _)| *msg_id)
            .collect_vec();
        for msg_id in msg_ids_to_delete {
            let _ = self.request_cache.remove(&msg_id);
        }
        if !accounts_to_delete.is_empty() {
            info!("Managing {} client accounts.",
                  self.accounts.len() - accounts_to_delete.len());
        }
        for maid_name in accounts_to_delete {
            trace!("No longer a MM for {}", maid_name);
            let _ = self.accounts.remove(&maid_name);
        }
        // Send refresh messages for the remaining accounts.
        for (maid_name, account) in &self.accounts {
            self.send_refresh(routing_node,
                              maid_name,
                              account.clone(),
                              MessageId::from_added_node(*node_name));
        }
    }

    pub fn handle_node_lost(&mut self, routing_node: &mut RoutingNode, node_name: &XorName) {
        for (maid_name, account) in &self.accounts {
            self.send_refresh(routing_node,
                              maid_name,
                              account.clone(),
                              MessageId::from_lost_node(*node_name));
        }
    }

    fn get_account(&self, dst: &Authority<XorName>) -> Result<&Account, ClientError> {
        let client_name = utils::client_manager_name(dst);
        if let Some(account) = self.accounts.get(&client_name) {
            Ok(account)
        } else {
            Err(ClientError::NoSuchAccount)
        }
    }

    fn mutate_account<F>(&mut self,
                         src: &Authority<XorName>,
                         dst: &Authority<XorName>,
                         version: u64,
                         f: F)
                         -> Result<(), ClientError>
        where F: FnOnce(&mut Account) -> Result<(), ClientError>
    {
        let client_name = utils::client_name(src);
        let client_manager_name = utils::client_manager_name(dst);

        if client_name != client_manager_name {
            // TODO (adam): is this the right error to return here?
            return Err(ClientError::AccessDenied);
        }

        if let Some(account) = self.accounts.get_mut(&client_manager_name) {
            if version == account.version + 1 {
                f(account)?;
                account.version = version;
                Ok(())
            } else {
                Err(ClientError::InvalidSuccessor)
            }
        } else {
            Err(ClientError::NoSuchAccount)
        }
    }

    fn increment_mutation_counter(&mut self, client_name: &XorName) -> Result<(), ClientError> {
        if let Some(account) = self.accounts.get_mut(client_name) {
            account.increment_mutation_counter()
        } else {
            Err(ClientError::NoSuchAccount)
        }
    }

    fn prepare_mdata_mutation(&mut self,
                              src: &Authority<XorName>,
                              dst: &Authority<XorName>,
                              requester: Option<sign::PublicKey>)
                              -> Result<(), ClientError> {
        let client_manager_name = utils::client_manager_name(dst);

        let account = if let Some(account) = self.accounts.get_mut(&client_manager_name) {
            account
        } else {
            return Err(ClientError::NoSuchAccount);
        };

        let client_key = utils::client_key(src);

        if let Some(requester) = requester {
            if requester != *client_key {
                return Err(ClientError::AccessDenied);
            }
        }

        let client_name = utils::client_name_from_key(&client_key);

        if client_name == client_manager_name || account.auth_keys.contains(&client_key) {
            account.increment_mutation_counter()
        } else {
            Err(ClientError::AccessDenied)
        }
    }

    fn handle_put_failure(&mut self,
                          routing_node: &mut RoutingNode,
                          src: &Authority<XorName>)
                          -> bool {

        let client_name = utils::client_name(src);

        let account = if let Some(account) = self.accounts.get_mut(&client_name) {
            // Refund account
            let _ = account.decrement_mutation_counter();
            account.clone()
        } else {
            return false;
        };

        self.send_refresh(routing_node, &client_name, account, MessageId::zero());

        true
    }

    fn send_refresh(&self,
                    routing_node: &mut RoutingNode,
                    maid_name: &XorName,
                    account: Account,
                    msg_id: MessageId) {
        let src = Authority::ClientManager(*maid_name);
        let refresh = Refresh::Update(*maid_name, account);
        if let Ok(serialised_refresh) = serialisation::serialise(&refresh) {
            trace!("MM sending refresh for account {}", src.name());
            let _ = routing_node.send_refresh_request(src, src, serialised_refresh, msg_id);
        }
    }

    fn remove_cached_request(&mut self, msg_id: MessageId) -> Result<CachedRequest, InternalError> {
        self.request_cache
            .remove(&msg_id)
            .ok_or_else(move || InternalError::FailedToFindCachedRequest(msg_id))
    }
}

#[cfg(feature = "use-mock-crust")]
impl MaidManager {
    pub fn get_mutation_count(&self, client_name: &XorName) -> Option<u64> {
        self.accounts.get(client_name).map(|account| account.info.mutations_done)
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Clone)]
enum Refresh {
    Update(XorName, Account),
    Delete(XorName),
}

// Entry in the request cache.
struct CachedRequest {
    src: Authority<XorName>,
    dst: Authority<XorName>,

    // Some(type_tag) if the request is for mutable data. None otherwise.
    tag: Option<u64>,
}
