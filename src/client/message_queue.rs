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

/// MessageQueue gets and collects messages/responses from routing. It also maintains local caching
/// of previously fetched ImmutableData (because the very nature of such data implies Immutability)
/// enabling fast re-retrieval and avoiding networking.
pub struct MessageQueue {
    local_cache  : ::lru_time_cache::LruCache<::routing::NameType, ::routing::data::Data>,
    message_queue: ::lru_time_cache::LruCache<::routing::NameType, ::routing::data::Data>,
}

impl MessageQueue {
    /// Create a new instance of MessageQueue
    pub fn new(notifier          : ::client::misc::ResponseNotifier,
               bootstrap_notifier: ::client::misc::BootstrapNotifier, // TODO Improve this design
               receiver          : ::std::sync::mpsc::Receiver<::routing::event::Event>) -> (::std::sync::Arc<::std::sync::Mutex<MessageQueue>>,
                                                                                             ::client::misc::RAIIThreadJoiner) {
        let message_queue = ::std::sync::Arc::new(::std::sync::Mutex::new(MessageQueue {
            local_cache  : ::lru_time_cache::LruCache::with_capacity(1000),
            message_queue: ::lru_time_cache::LruCache::with_capacity(1000),
        }));

        let message_queue_cloned = message_queue.clone();
        let receiver_joiner = eval_result!(::std::thread::Builder::new().name("MessageReceiverThread".to_string()).spawn(move || {
            for it in receiver.iter() {
                match it {
                    ::routing::event::Event::Response { response, .. } => {
                        match response {
                            ::routing::ExternalResponse::Get(data, _, _) => {
                                let data_name = data.name();
                                eval_result!(message_queue_cloned.lock()).message_queue.insert(data_name.clone(), data);

                                let &(ref lock, ref condition_var) = &*notifier;
                                let mut fetched_location = eval_result!(lock.lock());
                                *fetched_location = Some(data_name);
                                condition_var.notify_all();
                            },
                            _ => debug!("Received External Response: {:?} ;; This is currently not supported.", response),
                        }
                    },
                    // TODO Improve this design
                    ::routing::event::Event::Bootstrapped => {
                        debug!("Bootstrapped");
                        let (ref lock, ref condition_var) = *bootstrap_notifier;
                        let mut mutex_guard = eval_result!(lock.lock());
                        *mutex_guard = true;
                        condition_var.notify_all();
                    },
                    ::routing::event::Event::Terminated => break,
                    _ => debug!("Received Event: {:?} ;; This is currently not supported.", it),
                }
            }

            debug!("Thread \"MessageReceiverThread\" terminated.");
        }));

        (message_queue, ::client::misc::RAIIThreadJoiner::new(receiver_joiner))
    }

    /// Check if data is already in local cache
    pub fn local_cache_check(&mut self, key: &::routing::NameType) -> bool {
        self.local_cache.contains_key(key)
    }

    /// Get data if already in local cache.
    pub fn local_cache_get(&mut self, key: &::routing::NameType) -> Result<::routing::data::Data, ::errors::ClientError> {
        Ok(try!(self.local_cache.get(key).ok_or(::errors::ClientError::VersionCacheMiss)).clone())
    }

    /// Put data into local cache
    pub fn local_cache_insert(&mut self, key: ::routing::NameType, value: ::routing::data::Data) {
        self.local_cache.insert(key, value);
    }

    /// Get data from cache filled by the response from routing
    pub fn get_response(&mut self, location: &::routing::NameType) -> Result<::routing::data::Data, ::errors::ClientError> {
        self.message_queue.remove(&location).ok_or(::errors::ClientError::RoutingMessageCacheMiss)
    }
}
