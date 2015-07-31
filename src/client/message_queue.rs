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

// TODO update doc
/// MessageQueue is the concrete type that the routing layer will use to notify of network
/// responses. It is nonblocking in nature.
pub struct MessageQueue {
    message_queue    : ::lru_time_cache::LruCache<::routing::NameType, ::client::Data>,
    local_cache      : ::lru_time_cache::LruCache<::routing::NameType, ::client::Data>,
    join_handle      : Option<::std::thread::JoinHandle<()>>,
}

impl MessageQueue {
    /// Create a new instance of MessageQueue
    pub fn new(notifier: ::client::misc::ResponseNotifier,
               receiver: ::std::sync::mpsc::Receiver<(::routing::NameType, ::client::Data)>) -> ::std::sync::Arc<::std::sync::Mutex<MessageQueue>> {
        let message_queue = ::std::sync::Arc::new(::std::sync::Mutex::new(MessageQueue {
            message_queue: ::lru_time_cache::LruCache::with_capacity(1000),
            local_cache  : ::lru_time_cache::LruCache::with_capacity(1000),
            join_handle  : None,
        }));

        let cloned_message_queue = message_queue.clone();
        message_queue.lock().unwrap().join_handle = Some(::std::thread::spawn(move || {
            for it in receiver.iter() {
                cloned_message_queue.lock().unwrap().message_queue.insert(it.0.clone(), it.1);

                let &(ref lock, ref condition_var) = &*notifier;
                let mut fetched_location = lock.lock().unwrap();
                *fetched_location = Some(it.0);
                condition_var.notify_all();
            }
        }));

        message_queue
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
}
