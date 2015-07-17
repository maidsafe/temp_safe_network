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

/// CallbackInterface is the concrete type that the routing layer will use to notify of network
/// responses. It is nonblocking in nature.
pub struct CallbackInterface {
    response_notifier: ::client::misc::ResponseNotifier,
    message_queue:     ::lru_time_cache::LruCache<::routing::NameType, ::client::Data>,
    local_cache:       ::lru_time_cache::LruCache<::routing::NameType, ::client::Data>,
}

//impl ::routing::client_interface::Interface for CallbackInterface {
//    fn handle_get_response(&mut self,
//                           message_id: ::routing::types::MessageId,
//                           response: Result<Vec<u8>, ::routing::error::ResponseError>) {
//        self.message_queue.add(message_id, response);
//        let &(ref lock, ref condition_var) = &*self.response_notifier;
//        let mut fetched_id = lock.lock().unwrap();
//        *fetched_id = message_id;
//        condition_var.notify_all();
//    }
//
//    fn handle_put_response(&mut self,
//                           message_id: ::routing::types::MessageId,
//                           response: Result<Vec<u8>, ::routing::error::ResponseError>) {
//        self.message_queue.add(message_id, response);
//        let &(ref lock, ref condition_var) = &*self.response_notifier;
//        let mut fetched_id = lock.lock().unwrap();
//        *fetched_id = message_id;
//        condition_var.notify_all();
//    }
//}

impl CallbackInterface {
    /// Create a new instance of CallbackInterface
    pub fn new(notifier: ::client::misc::ResponseNotifier) -> CallbackInterface {
        CallbackInterface {
            response_notifier: notifier,
            message_queue:     ::lru_time_cache::LruCache::with_capacity(1000),
            local_cache:       ::lru_time_cache::LruCache::with_capacity(1000)
        }
    }

    /// Check if data is already in local cache
    pub fn local_cache_check(&mut self, key: &::routing::NameType) -> bool {
        self.local_cache.contains_key(key)
    }

    /// Get data if already in local cache.
    pub fn local_cache_get(&mut self, key: &::routing::NameType) -> Result<::client::Data, ::errors::ClientError> {
        Ok(try!(self.local_cache.get(key).ok_or(::errors::ClientError::VersionCacheMiss)).clone())
    }

    /// Put data into local cache
    pub fn local_cache_insert(&mut self, key: ::routing::NameType, value: ::client::Data) {
        self.local_cache.insert(key, value);
    }

    /// Get data from cache filled by the response from routing
    pub fn get_response(&mut self, location: &::routing::NameType) -> Result<::client::Data, ::errors::ClientError> {
        Ok(try!(self.message_queue.remove(&location).ok_or(::errors::ClientError::RoutingMessageCacheMiss)))
    }

    pub fn handle_get_response(&mut self, original_requested_location: ::routing::NameType, response: ::client::Data) {
        self.message_queue.insert(original_requested_location.clone(), response);
        let &(ref lock, ref condition_var) = &*self.response_notifier;
        let mut fetched_location = lock.lock().unwrap();
        *fetched_location = Some(original_requested_location);
        condition_var.notify_all();
    }
}
