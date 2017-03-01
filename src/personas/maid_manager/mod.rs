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

use self::account::Account;
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
    pub fn get_put_count(&self, client_name: &XorName) -> Option<u64> {
        self.accounts.get(client_name).map(|account| account.data_stored)
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

#[cfg(all(test, feature = "use-mock-routing"))]
mod tests {
    use super::*;
    use super::account::DEFAULT_ACCOUNT_SIZE;
    use rand;
    use routing::{AccountInfo, Request, Response};
    use test_utils;

    #[test]
    fn account_basics() {
        let (src, client_key) = test_utils::gen_client_authority();
        let dst = test_utils::gen_client_manager_authority(client_key);

        let mut node = RoutingNode::new();
        let mut mm = MaidManager::new();

        // Retrieving account info for non-existintg account fails.
        let res = get_account_info(&mut mm, &mut node, src, dst);
        assert_match!(res, Err(ClientError::NoSuchAccount));

        // Create the account by issuing a PutMData with a special tag.
        create_account(&mut mm, &mut node, src, dst);

        // Now retrieving account info succeeds.
        let account_info = unwrap!(get_account_info(&mut mm, &mut node, src, dst));

        assert_eq!(account_info.mutations_done, 1);
        assert_eq!(account_info.mutations_available, DEFAULT_ACCOUNT_SIZE - 1);
    }

    #[test]
    fn idata_basics() {
        let (src, client_key) = test_utils::gen_client_authority();
        let dst = test_utils::gen_client_manager_authority(client_key);

        let mut node = RoutingNode::new();
        let mut mm = MaidManager::new();

        // Create account and retrieve the current account info.
        create_account(&mut mm, &mut node, src, dst);
        let account_info_0 = unwrap!(get_account_info(&mut mm, &mut node, src, dst));

        // Put immutable data.
        let data = test_utils::gen_random_immutable_data(10, &mut rand::thread_rng());
        let msg_id = MessageId::new();
        unwrap!(mm.handle_put_idata(&mut node, src, dst, data.clone(), msg_id));

        // Verify it got forwarded to its NAE manager.
        let message = unwrap!(node.sent_requests.remove(&msg_id));

        assert_eq!(message.src, dst);
        assert_eq!(message.dst, Authority::NaeManager(*data.name()));

        assert_match!(
            message.request,
            Request::PutIData { data: request_data, .. } => {
                assert_eq!(request_data, data);
            });

        // Verify the mutation was accounted for.
        let account_info_1 = unwrap!(get_account_info(&mut mm, &mut node, src, dst));
        assert_eq!(account_info_1.mutations_done,
                   account_info_0.mutations_done + 1);
        assert_eq!(account_info_1.mutations_available,
                   account_info_0.mutations_available - 1);
    }

    #[test]
    fn mdata_basics() {
        let (src, client_key) = test_utils::gen_client_authority();
        let dst = test_utils::gen_client_manager_authority(client_key);

        let mut node = RoutingNode::new();
        let mut mm = MaidManager::new();

        // Create account and retrieve the current account info.
        create_account(&mut mm, &mut node, src, dst);
        let account_info_0 = unwrap!(get_account_info(&mut mm, &mut node, src, dst));

        // Put initial mutable data
        let tag = rand::random();
        let data = test_utils::gen_empty_mutable_data(tag, client_key, &mut rand::thread_rng());

        let msg_id = MessageId::new();
        unwrap!(mm.handle_put_mdata(&mut node, src, dst, data.clone(), msg_id, client_key));

        // Verify it got forwarded to its NAE manager.
        let message = unwrap!(node.sent_requests.remove(&msg_id));

        assert_eq!(message.src, dst);
        assert_eq!(message.dst, Authority::NaeManager(*data.name()));

        assert_match!(
            message.request,
            Request::PutMData { data: request_data, .. } => {
                assert_eq!(request_data, data);
            });

        // Verify the mutation was accounted for.
        let account_info_1 = unwrap!(get_account_info(&mut mm, &mut node, src, dst));
        assert_eq!(account_info_1.mutations_done,
                   account_info_0.mutations_done + 1);
        assert_eq!(account_info_1.mutations_available,
                   account_info_0.mutations_available - 1);
    }

    #[test]
    fn auth_keys() {
        let (owner_client, owner_key) = test_utils::gen_client_authority();
        let owner_client_manager = test_utils::gen_client_manager_authority(owner_key);
        let (_, app_key) = test_utils::gen_client_authority();

        let mut node = RoutingNode::new();
        let mut mm = MaidManager::new();

        // Create owner account
        create_account(&mut mm, &mut node, owner_client, owner_client_manager);

        // Retrieve initial auth keys - should be empty with version 0.
        let msg_id = MessageId::new();
        unwrap!(mm.handle_list_auth_keys_and_version(&mut node,
                                                     owner_client,
                                                     owner_client_manager,
                                                     msg_id));
        let (auth_keys, version) = assert_match!(
            unwrap!(node.sent_responses.remove(&msg_id)).response,
            Response::ListAuthKeysAndVersion { res: Ok(ok), .. } => ok);

        assert!(auth_keys.is_empty());
        assert_eq!(version, 0);

        // Attempt to insert new auth key with incorrect version fails.
        let msg_id = MessageId::new();
        unwrap!(mm.handle_ins_auth_key(&mut node,
                                       owner_client,
                                       owner_client_manager,
                                       app_key,
                                       0,
                                       msg_id));

        assert_match!(
            unwrap!(node.sent_responses.remove(&msg_id)).response,
            Response::InsAuthKey { res: Err(ClientError::InvalidSuccessor), .. });

        // Attempt to insert new auth key by non-owner fails.
        let (evil_client, _) = test_utils::gen_client_authority();
        let msg_id = MessageId::new();
        unwrap!(mm.handle_ins_auth_key(&mut node,
                                       evil_client,
                                       owner_client_manager,
                                       app_key,
                                       1,
                                       msg_id));

        assert_match!(
            unwrap!(node.sent_responses.remove(&msg_id)).response,
            Response::InsAuthKey { res: Err(ClientError::AccessDenied), .. });

        // Insert the auth key with proper version bump.
        let msg_id = MessageId::new();
        unwrap!(mm.handle_ins_auth_key(&mut node,
                                       owner_client,
                                       owner_client_manager,
                                       app_key,
                                       1,
                                       msg_id));

        assert_match!(
            unwrap!(node.sent_responses.remove(&msg_id)).response,
            Response::InsAuthKey { res: Ok(()), .. });

        // Retrieve the auth keys again - should contain one element and have
        // bumped version.
        let msg_id = MessageId::new();
        unwrap!(mm.handle_list_auth_keys_and_version(&mut node,
                                                     owner_client,
                                                     owner_client_manager,
                                                     msg_id));
        let (auth_keys, version) = assert_match!(
            unwrap!(node.sent_responses.remove(&msg_id)).response,
            Response::ListAuthKeysAndVersion { res: Ok(ok), .. } => ok);

        assert_eq!(auth_keys.len(), 1);
        assert!(auth_keys.contains(&app_key));
        assert_eq!(version, 1);
    }

    #[test]
    fn mutation_authorisation() {
        let (owner_client, owner_key) = test_utils::gen_client_authority();
        let owner_client_manager = test_utils::gen_client_manager_authority(owner_key);
        let (app_client, app_key) = test_utils::gen_client_authority();

        let mut node = RoutingNode::new();
        let mut mm = MaidManager::new();

        // Create owner account
        create_account(&mut mm, &mut node, owner_client, owner_client_manager);

        // Put a mutable data
        let tag = rand::random();
        let data = test_utils::gen_empty_mutable_data(tag, owner_key, &mut rand::thread_rng());
        let msg_id = MessageId::new();
        unwrap!(mm.handle_put_mdata(&mut node,
                                    owner_client,
                                    owner_client_manager,
                                    data,
                                    msg_id,
                                    owner_key));

        // Attemp to mutate by unauthorised client fails.
        let msg_id = MessageId::new();
        let _ = mm.handle_mutate_mdata_entries(&mut node,
                                               app_client,
                                               owner_client_manager,
                                               msg_id,
                                               app_key);
        assert_match!(
            unwrap!(node.sent_responses.remove(&msg_id)).response,
            Response::MutateMDataEntries { res: Err(ClientError::AccessDenied), .. });

        // Mutation by the owner succeeds.
        let msg_id = MessageId::new();
        let _ = mm.handle_mutate_mdata_entries(&mut node,
                                               owner_client,
                                               owner_client_manager,
                                               msg_id,
                                               owner_key);
        // Note: No response sent here means all is good (MM sends respone to
        // MutateMDataEntries request only in case of error).
        assert!(!node.sent_responses.contains_key(&msg_id));

        // Authorise the app.
        let msg_id = MessageId::new();
        let _ = mm.handle_ins_auth_key(&mut node,
                                       owner_client,
                                       owner_client_manager,
                                       app_key,
                                       1,
                                       msg_id);
        assert_match!(
            unwrap!(node.sent_responses.remove(&msg_id)).response,
            Response::InsAuthKey { res: Ok(()), .. });

        // Mutation by authorised app now succeeds.
        let msg_id = MessageId::new();
        let _ = mm.handle_mutate_mdata_entries(&mut node,
                                               app_client,
                                               owner_client_manager,
                                               msg_id,
                                               app_key);
        assert!(!node.sent_responses.contains_key(&msg_id));

        // Attempt to mutate by requester that doesn't match the source client
        // key fails.
        let msg_id = MessageId::new();
        let _ = mm.handle_mutate_mdata_entries(&mut node,
                                               app_client,
                                               owner_client_manager,
                                               msg_id,
                                               owner_key);
        let message = unwrap!(node.sent_responses.remove(&msg_id));
        assert_match!(
            message.response,
            Response::MutateMDataEntries { res: Err(ClientError::AccessDenied), .. });
    }

    fn create_account(mm: &mut MaidManager,
                      node: &mut RoutingNode,
                      src: Authority<XorName>,
                      dst: Authority<XorName>) {
        let client_key = assert_match!(src, Authority::Client { client_key, .. } => client_key);
        let account_packet = test_utils::gen_empty_mutable_data(TYPE_TAG_SESSION_PACKET,
                                                                client_key,
                                                                &mut rand::thread_rng());
        let msg_id = MessageId::new();
        unwrap!(mm.handle_put_mdata(node, src, dst, account_packet, msg_id, client_key));
    }

    fn get_account_info(mm: &mut MaidManager,
                        node: &mut RoutingNode,
                        src: Authority<XorName>,
                        dst: Authority<XorName>)
                        -> Result<AccountInfo, ClientError> {
        let msg_id = MessageId::new();
        unwrap!(mm.handle_get_account_info(node, src, dst, msg_id));

        assert_match!(
            unwrap!(node.sent_responses.remove(&msg_id)).response,
            Response::GetAccountInfo { res, .. } => res)
    }
}
