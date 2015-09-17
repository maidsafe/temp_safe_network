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

const EVENT_RECEIVER_THREAD_NAME: &'static str = "EventReceiverThread";

/// MessageQueue gets and collects messages/responses from routing. It also maintains local caching
/// of previously fetched ImmutableData (because the very nature of such data implies Immutability)
/// enabling fast re-retrieval and avoiding networking.
pub struct MessageQueue {
    local_cache          : ::lru_time_cache::LruCache<::routing::NameType, ::routing::data::Data>,
    data_senders         : ::std::collections::HashMap<::routing::NameType, Vec<::std::sync::mpsc::Sender<::translated_events::DataReceivedEvent>>>,
    error_senders        : Vec<::std::sync::mpsc::Sender<::translated_events::OperationFailureEvent>>,
    network_event_senders: Vec<::std::sync::mpsc::Sender<::translated_events::NetworkEvent>>,
    routing_message_cache: ::lru_time_cache::LruCache<::routing::NameType, ::routing::data::Data>,
}

impl MessageQueue {
    /// Create a new instance of MessageQueue. `data_senders` can be added later via function to
    /// add observer since one will not receive data until one asks for it. Thus there is enough
    /// chance to add an observer before requesting data.
    pub fn new(routing_event_receiver: ::std::sync::mpsc::Receiver<::routing::event::Event>,
               network_event_senders : Vec<::std::sync::mpsc::Sender<::translated_events::NetworkEvent>>,
               error_senders         : Vec<::std::sync::mpsc::Sender<::translated_events::OperationFailureEvent>>) -> (::std::sync::Arc<::std::sync::Mutex<MessageQueue>>,
                                                                                                                       ::client::misc::RAIIThreadJoiner) {
        let message_queue = ::std::sync::Arc::new(::std::sync::Mutex::new(MessageQueue {
            local_cache          : ::lru_time_cache::LruCache::with_capacity(1000),
            data_senders         : ::std::collections::HashMap::new(),
            error_senders        : error_senders,
            network_event_senders: network_event_senders,
            routing_message_cache: ::lru_time_cache::LruCache::with_capacity(1000),
        }));

        let message_queue_cloned = message_queue.clone();
        let receiver_joiner = eval_result!(::std::thread::Builder::new().name(EVENT_RECEIVER_THREAD_NAME.to_string()).spawn(move || {
            for it in routing_event_receiver.iter() {
                match it {
                    ::routing::event::Event::Response { response, .. } => {
                        match response {
                            ::routing::ExternalResponse::Get(data, _, _) => {
                                let data_name = data.name();
                                let mut dead_sender_positions = Vec::<usize>::new();
                                let mut queue_guard = eval_result!(message_queue_cloned.lock());
                                queue_guard.routing_message_cache.insert(data_name.clone(), data);
                                if let Some(mut specific_data_senders) = queue_guard.data_senders.get_mut(&data_name) {
                                    for it in specific_data_senders.iter().enumerate() {
                                        if it.1.send(::translated_events::DataReceivedEvent::DataReceived).is_err() {
                                            dead_sender_positions.push(it.0);
                                        }
                                    }

                                    MessageQueue::purge_dead_senders(&mut specific_data_senders, dead_sender_positions);
                                }
                            },
                            _ => debug!("Received External Response: {:?} ;; This is currently not supported.", response),
                        }
                    },
                    ::routing::event::Event::Bootstrapped => {
                        debug!("Routing Event Received: Bootstrapped");

                        let mut dead_sender_positions = Vec::<usize>::new();
                        let mut queue_guard = eval_result!(message_queue_cloned.lock());
                        for it in queue_guard.network_event_senders.iter().enumerate() {
                            if it.1.send(::translated_events::NetworkEvent::Bootstrapped).is_err() {
                                dead_sender_positions.push(it.0);
                            }
                        }

                        MessageQueue::purge_dead_senders(&mut queue_guard.network_event_senders, dead_sender_positions);
                    },
                    ::routing::event::Event::Disconnected => {
                        debug!("Routing Event Received: Disconnected");

                        let mut dead_sender_positions = Vec::<usize>::new();
                        let mut queue_guard = eval_result!(message_queue_cloned.lock());
                        for it in queue_guard.network_event_senders.iter().enumerate() {
                            if it.1.send(::translated_events::NetworkEvent::Disconnected).is_err() {
                                dead_sender_positions.push(it.0);
                            }
                        }

                        MessageQueue::purge_dead_senders(&mut queue_guard.network_event_senders, dead_sender_positions);
                    },
                    ::routing::event::Event::Terminated => {
                        debug!("Routing Event Received: Terminated");

                        let mut dead_sender_positions = Vec::<usize>::new();
                        let mut queue_guard = eval_result!(message_queue_cloned.lock());
                        for it in queue_guard.error_senders.iter().enumerate() {
                            if it.1.send(::translated_events::OperationFailureEvent::Terminated).is_err() {
                                dead_sender_positions.push(it.0);
                            }
                        }

                        MessageQueue::purge_dead_senders(&mut queue_guard.error_senders, dead_sender_positions);

                        dead_sender_positions = Vec::new();
                        for it in queue_guard.network_event_senders.iter().enumerate() {
                            if it.1.send(::translated_events::NetworkEvent::Terminated).is_err() {
                                dead_sender_positions.push(it.0);
                            }
                        }

                        MessageQueue::purge_dead_senders(&mut queue_guard.network_event_senders, dead_sender_positions);

                        for mut it in queue_guard.data_senders.iter_mut() {
                            dead_sender_positions = Vec::new();
                            for specific_senders in it.1.iter().enumerate() {
                                if specific_senders.1.send(::translated_events::DataReceivedEvent::Terminated).is_err() {
                                    dead_sender_positions.push(specific_senders.0);
                                }
                            }

                            MessageQueue::purge_dead_senders(&mut it.1, dead_sender_positions);
                        }

                        break;
                    },
                    _ => debug!("Received Routing Event: {:?} ;; This is currently not supported.", it),
                }
            }

            debug!("Thread \"{}\" terminated.", EVENT_RECEIVER_THREAD_NAME);
        }));

        (message_queue, ::client::misc::RAIIThreadJoiner::new(receiver_joiner))
    }

    /// Add observers for Data Recieve Events
    pub fn add_data_receive_event_observer(&mut self,
                                           data_name: ::routing::NameType,
                                           sender   : ::std::sync::mpsc::Sender<::translated_events::DataReceivedEvent>) {
        self.data_senders.entry(data_name).or_insert(Vec::new()).push(sender);
    }

    /// Add observers for Operation Failure Events like `PutFailure`, `PostFailure`, `DeleteFailure`,
    /// `Terminated`
    #[allow(dead_code)]
    pub fn add_operation_failure_event_observer(&mut self, sender: ::std::sync::mpsc::Sender<::translated_events::OperationFailureEvent>) {
        self.error_senders.push(sender);
    }

    /// Add observers for Network Events like `Bootstrapped`, `Disconnected`, `Terminated`
    #[allow(dead_code)]
    pub fn add_network_event_observer(&mut self, sender: ::std::sync::mpsc::Sender<::translated_events::NetworkEvent>) {
        self.network_event_senders.push(sender);
    }

    /// Check if data is already in local cache
    pub fn local_cache_check(&self, key: &::routing::NameType) -> bool {
        self.local_cache.contains_key(key)
    }

    /// Get data if already in local cache.
    pub fn local_cache_get(&mut self, key: &::routing::NameType) -> Result<::routing::data::Data, ::errors::ClientError> {
        self.local_cache.get(key).ok_or(::errors::ClientError::VersionCacheMiss).map(|val| val.clone())
    }

    /// Put data into local cache
    pub fn local_cache_insert(&mut self, key: ::routing::NameType, value: ::routing::data::Data) {
        self.local_cache.insert(key, value);
    }

    /// Get data from cache filled by the response from routing
    pub fn get_response(&mut self, location: &::routing::NameType) -> Result<::routing::data::Data, ::errors::ClientError> {
        self.routing_message_cache.get(location).ok_or(::errors::ClientError::RoutingMessageCacheMiss).map(|val| val.clone())
    }

    fn purge_dead_senders<T>(senders  : &mut Vec<::std::sync::mpsc::Sender<T>>,
                             positions: Vec<usize>) {
        let mut delta = 0;
        for val in positions {
            let _ = senders.remove(val - delta);
            delta += 1;
        }
    }
}
