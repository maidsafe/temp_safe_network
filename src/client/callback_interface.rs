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
// relating to use of the SAFE Network Software.                                                                 */

use routing;
use lru_time_cache;

use routing::error::ResponseError;

/// CallbackInterface is the concrete type that the routing layer will use to notify of network
/// responses. It is nonblocking in nature.
pub struct CallbackInterface {
    response_notifier: ::client::misc::ResponseNotifier,
    message_queue:     lru_time_cache::LruCache<routing::types::MessageId, Result<Vec<u8>, ResponseError>>,
    cache:             lru_time_cache::LruCache<routing::NameType, Vec<u8>>,
}

impl routing::client_interface::Interface for CallbackInterface {
  // fn handle_get(&mut self, type_id: u64, our_authority: Authority, from_authority: Authority,
  //               from_address: NameType, data: Vec<u8>)->Result<Action, ResponseError> {
  //   Err(ResponseError::InvalidRequest)
  // }

  // fn handle_put(&mut self, our_authority: Authority, from_authority: Authority,
  //               from_address: NameType, dest_address: DestinationAddress, data: Vec<u8>)->Result<Action, ResponseError> {
  //   ;
  //   Err(ResponseError::InvalidRequest)
  // }

  // fn handle_post(&mut self, our_authority: Authority, from_authority: Authority, from_address: NameType, data: Vec<u8>)->Result<Action, ResponseError> {
  //   ;
  //   Err(ResponseError::InvalidRequest)
  // }

    fn handle_get_response(&mut self,
                           message_id: routing::types::MessageId,
                           response: Result<Vec<u8>, ResponseError>) {
        self.message_queue.add(message_id, response);
        let &(ref lock, ref condition_var) = &*self.response_notifier;
        let mut fetched_id = lock.lock().unwrap();
        *fetched_id = message_id;
        condition_var.notify_all();
    }

    fn handle_put_response(&mut self,
                           message_id: routing::types::MessageId,
                           response: Result<Vec<u8>, ResponseError>) {
        self.message_queue.add(message_id, response);
        let &(ref lock, ref condition_var) = &*self.response_notifier;
        let mut fetched_id = lock.lock().unwrap();
        *fetched_id = message_id;
        condition_var.notify_all();
    }

  // fn handle_post_response(&mut self, from_authority: Authority, from_address: NameType, response: Result<Vec<u8>, ResponseError>) {
  //   ;
  // }

  // fn add_node(&mut self, node: NameType) { unimplemented!() }

  // fn drop_node(&mut self, node: NameType) { unimplemented!() }
}

impl CallbackInterface {
    /// Create a new instance of CallbackInterface
    pub fn new(notifier: ::client::misc::ResponseNotifier) -> CallbackInterface {
        CallbackInterface {
            response_notifier: notifier,
            message_queue:     lru_time_cache::LruCache::with_capacity(10000),
            cache:             lru_time_cache::LruCache::with_capacity(1000)
        }
    }

    /// Check if data is already in local cache
    pub fn cache_check(&mut self, name: &routing::NameType) -> bool {
        self.cache.check(name)
    }

    /// Get data if already in local cache.
    pub fn cache_get(&mut self, name: &routing::NameType)
            -> Option<Result<Vec<u8>, ResponseError>> {
        Some(Ok(self.cache.get(name).unwrap().clone()))
    }

    /// Put data into local cache
    pub fn cache_insert(&mut self, name: routing::NameType, data: Vec<u8>) {
        self.cache.insert(name, data);
    }

    /// Get data from cache filled by the response from routing
    pub fn get_response(&mut self, message_id: routing::types::MessageId)
            -> Option<Result<Vec<u8>, ResponseError>> {
        self.message_queue.remove(&message_id)
    }
}
