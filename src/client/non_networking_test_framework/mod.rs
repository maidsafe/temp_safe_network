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

#![allow(unsafe_code)]

use routing;
use maidsafe_types::TypeTag;
use routing::sendable::Sendable;
use routing::client_interface::Interface;

use client::callback_interface;

type DataStore = ::std::sync::Arc<::std::sync::Mutex<::std::collections::BTreeMap<routing::NameType, Vec<u8>>>>;

struct PersistentStorageSimulation {
    data_store: DataStore,
}

fn get_storage() -> DataStore {
    static mut STORAGE: *const PersistentStorageSimulation = 0 as *const PersistentStorageSimulation;
    static mut ONCE: ::std::sync::Once = ::std::sync::ONCE_INIT;

    unsafe {
        ONCE.call_once(|| {
            STORAGE = ::std::mem::transmute(Box::new(
                    PersistentStorageSimulation {
                        data_store: ::std::sync::Arc::new(::std::sync::Mutex::new(::std::collections::BTreeMap::new())),
                    }
                    ));
        });

        (*STORAGE).data_store.clone()
    }
}

/// RoutingClient Mock mimics routing interface to store data locally for testing instead of actual
/// networking with vaults etc.
pub struct RoutingClientMock {
    callback_interface: ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>,
    msg_id: routing::types::MessageId,
    network_delay_ms: u32,
}

impl RoutingClientMock {
    /// Create a new instance of RoutingClientMock
    pub fn new(cb_interface: ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>,
               _: routing::types::Id) -> RoutingClientMock {
        RoutingClientMock {
            callback_interface: cb_interface,
            msg_id: 1,
            network_delay_ms: 1000,
        }
    }

    #[allow(dead_code)]
    pub fn set_network_delay_for_delay_simulation(&mut self, delay_ms: u32) {
        self.network_delay_ms = delay_ms;
    }

    pub fn get(&mut self, _type_id: u64, name: routing::NameType) -> Result<routing::types::MessageId, ::IoError> {
        self.msg_id += 1;
        let msg_id = self.msg_id;
        let delay_ms = self.network_delay_ms;
        let cb_interface = self.callback_interface.clone();
        let data_store = get_storage();

        ::std::thread::spawn(move || {
            ::std::thread::sleep_ms(delay_ms);
            match data_store.lock().unwrap().get(&name).clone() {
                Some(data) => cb_interface.lock().unwrap().handle_get_response(msg_id, Ok(data.clone())),
                None => cb_interface.lock().unwrap().handle_get_response(msg_id, Err(routing::error::ResponseError::NoData)),
            };
        });

        Ok(self.msg_id)
    }

    pub fn put<T>(&mut self, sendable: T) -> Result<routing::types::MessageId, ::IoError> where T: Sendable {
        self.msg_id += 1;
        let msg_id = self.msg_id;
        let delay_ms = self.network_delay_ms;
        let cb_interface = self.callback_interface.clone();
        let data_store = get_storage();

        let structured_data_type_id = ::maidsafe_types::data::StructuredDataTypeTag;
        let success: bool = if sendable.type_tag() != structured_data_type_id.type_tag() && data_store.lock().unwrap().contains_key(&sendable.name()) {
            false
        } else {
            data_store.lock().unwrap().insert(sendable.name(), sendable.serialised_contents());
            true
        };

        ::std::thread::spawn(move || {
            ::std::thread::sleep_ms(delay_ms);
            if success {
                cb_interface.lock().unwrap().handle_put_response(msg_id, Ok(Vec::<u8>::new()));
            } else {
                cb_interface.lock().unwrap().handle_put_response(msg_id, Err(routing::error::ResponseError::InvalidRequest));
            }
        });

        Ok(self.msg_id)
    }

    pub fn unauthorised_put(&mut self, _: routing::NameType, sendable: Box<Sendable>) -> Result<routing::types::MessageId, ::IoError> {
        self.msg_id += 1;
        let msg_id = self.msg_id;
        let delay_ms = self.network_delay_ms;
        let cb_interface = self.callback_interface.clone();
        let data_store = get_storage();

        let structured_data_type_id = ::maidsafe_types::data::StructuredDataTypeTag;
        let success: bool = if sendable.type_tag() != structured_data_type_id.type_tag() && data_store.lock().unwrap().contains_key(&sendable.name()) {
            false
        } else {
            data_store.lock().unwrap().insert(sendable.name(), sendable.serialised_contents());
            true
        };

        ::std::thread::spawn(move || {
            ::std::thread::sleep_ms(delay_ms);
            if success {
                cb_interface.lock().unwrap().handle_put_response(msg_id, Ok(Vec::<u8>::new()));
            } else {
                cb_interface.lock().unwrap().handle_put_response(msg_id, Err(routing::error::ResponseError::InvalidRequest));
            }
        });

        Ok(self.msg_id)
    }

    pub fn run(&mut self) {
        // let data_store = get_storage();
        // println!("Amount Of Chunks Stored: {:?}", data_store.lock().unwrap().len());
    }

    pub fn bootstrap(&mut self,
                     endpoints: Option<Vec<::routing::routing_client::Endpoint>>,
                     _: Option<u16>) -> Result<(), routing::error::RoutingError> {
        match endpoints {
            Some(vec_endpoints) => {
                for endpoint in vec_endpoints {
                    println!("Endpoint: {:?}", endpoint);
                }

                Ok(())
            },
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use ::std::error::Error;

    use maidsafe_types;
    use maidsafe_types::TypeTag;
    use routing::sendable::Sendable;

    use super::*;

    #[test]
    fn check_unauthorised_put() {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(0), ::std::sync::Condvar::new()));
        let account_packet = ::client::user_account::Account::new(None);
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(::client::callback_interface::CallbackInterface::new(notifier.clone())));

        let id_packet = ::routing::types::Id::with_keys(account_packet.get_maid().public_keys().clone(),
                                                      account_packet.get_maid().secret_keys().clone());

        let mock_routing = ::std::sync::Arc::new(::std::sync::Mutex::new(RoutingClientMock::new(callback_interface.clone(), id_packet)));
        let mock_routing_clone = mock_routing.clone();

        let mock_routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let mock_routing_stop_flag_clone = mock_routing_stop_flag.clone();

        struct RAIIThreadExit {
            routing_stop_flag: ::std::sync::Arc<::std::sync::Mutex<bool>>,
            join_handle: Option<::std::thread::JoinHandle<()>>,
        }

        impl Drop for RAIIThreadExit {
            fn drop(&mut self) {
                *self.routing_stop_flag.lock().unwrap() = true;
                self.join_handle.take().unwrap().join().unwrap();
            }
        }

        let _managed_thread = RAIIThreadExit {
            routing_stop_flag: mock_routing_stop_flag,
            join_handle: Some(::std::thread::spawn(move || {
                while !*mock_routing_stop_flag_clone.lock().unwrap() {
                    ::std::thread::sleep_ms(10);
                    mock_routing_clone.lock().unwrap().run();
                }
            })),
        };

        // First Unauthorised-PUT should succeed
        {
            let destination = account_packet.get_public_maid().name();
            let boxed_public_maid = Box::new(account_packet.get_public_maid().clone());
            match mock_routing.lock().unwrap().unauthorised_put(destination, boxed_public_maid) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => (),
                        Err(error) => panic!("Unauthorised-PUT Response Failure :: {:?}", error.description()),
                    }
                },
                Err(_) => panic!("Failure in Unauthorised-PUT !!"),
            }
        }

        // Subsequent Unauthorised-PUTs for same MAID-Keys should fail
        {
            let destination = account_packet.get_public_maid().name();
            let boxed_public_maid = Box::new(account_packet.get_public_maid().clone());
            let unauthorised_put_result = mock_routing.lock().unwrap().unauthorised_put(destination, boxed_public_maid);
            match unauthorised_put_result {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => panic!("Overwriting of Existing Data Should Not Be Allowed !!"),
                        Err(_) => (),
                    }
                },
                Err(_) => panic!("Failure in Unauthorised-PUT !!"),
            }
        }
    }

    #[test]
    fn check_put_and_get_for_immutable_data() {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(0), ::std::sync::Condvar::new()));
        let account_packet = ::client::user_account::Account::new(None);
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(::client::callback_interface::CallbackInterface::new(notifier.clone())));

        let id_packet = ::routing::types::Id::with_keys(account_packet.get_maid().public_keys().clone(),
                                                      account_packet.get_maid().secret_keys().clone());

        let mock_routing = ::std::sync::Arc::new(::std::sync::Mutex::new(RoutingClientMock::new(callback_interface.clone(), id_packet)));
        let mock_routing_clone = mock_routing.clone();

        let mock_routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let mock_routing_stop_flag_clone = mock_routing_stop_flag.clone();

        struct RAIIThreadExit {
            routing_stop_flag: ::std::sync::Arc<::std::sync::Mutex<bool>>,
            join_handle: Option<::std::thread::JoinHandle<()>>,
        }

        impl Drop for RAIIThreadExit {
            fn drop(&mut self) {
                *self.routing_stop_flag.lock().unwrap() = true;
                self.join_handle.take().unwrap().join().unwrap();
            }
        }

        let _managed_thread = RAIIThreadExit {
            routing_stop_flag: mock_routing_stop_flag,
            join_handle: Some(::std::thread::spawn(move || {
                while !*mock_routing_stop_flag_clone.lock().unwrap() {
                    ::std::thread::sleep_ms(10);
                    mock_routing_clone.lock().unwrap().run();
                }
            })),
        };

        // Construct ImmutableData
        let immutable_data_type_id = maidsafe_types::data::ImmutableDataTypeTag;
        let orig_data: Vec<u8> = (0u8..100u8).map(|_| ::rand::random::<u8>()).collect();
        let orig_immutable_data = maidsafe_types::ImmutableData::new(orig_data);

        // GET ImmutableData should fail
        {
            match mock_routing.lock().unwrap().get(immutable_data_type_id.type_tag(), orig_immutable_data.name()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => panic!("Should not have found data before a PUT"),
                        Err(_) => (),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // First PUT should succeed
        {
            match mock_routing.lock().unwrap().put(orig_immutable_data.clone()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => (),
                        Err(error) => panic!("PUT Response Failure :: {:?}", error.description()),
                    }
                },
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // GET ImmutableData should pass
        {
            match mock_routing.lock().unwrap().get(immutable_data_type_id.type_tag(), orig_immutable_data.name()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(data) => {
                            let mut decoder = ::cbor::Decoder::from_bytes(&data[..]);
                            let received_immutable_data: maidsafe_types::ImmutableData = decoder.decode().next().unwrap().unwrap();

                            assert_eq!(orig_immutable_data, received_immutable_data);
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }


        // Subsequent PUTs for same ImmutableData should fail
        {
            let put_result = mock_routing.lock().unwrap().put(orig_immutable_data.clone());
            match put_result {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => panic!("Second PUT for same ImmutableData should fail !!"),
                        Err(_) => (),
                    }
                },
                Err(_) => panic!("Failure in PUT !!"),
            }
        }
    }

    #[test]
    fn check_put_and_get_for_structured_data() {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(0), ::std::sync::Condvar::new()));
        let account_packet = ::client::user_account::Account::new(None);
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(::client::callback_interface::CallbackInterface::new(notifier.clone())));

        let id_packet = ::routing::types::Id::with_keys(account_packet.get_maid().public_keys().clone(),
                                                      account_packet.get_maid().secret_keys().clone());

        let mock_routing = ::std::sync::Arc::new(::std::sync::Mutex::new(RoutingClientMock::new(callback_interface.clone(), id_packet)));
        let mock_routing_clone = mock_routing.clone();

        let mock_routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let mock_routing_stop_flag_clone = mock_routing_stop_flag.clone();

        struct RAIIThreadExit {
            routing_stop_flag: ::std::sync::Arc<::std::sync::Mutex<bool>>,
            join_handle: Option<::std::thread::JoinHandle<()>>,
        }

        impl Drop for RAIIThreadExit {
            fn drop(&mut self) {
                *self.routing_stop_flag.lock().unwrap() = true;
                self.join_handle.take().unwrap().join().unwrap();
            }
        }

        let _managed_thread = RAIIThreadExit {
            routing_stop_flag: mock_routing_stop_flag,
            join_handle: Some(::std::thread::spawn(move || {
                while !*mock_routing_stop_flag_clone.lock().unwrap() {
                    ::std::thread::sleep_ms(10);
                    mock_routing_clone.lock().unwrap().run();
                }
            })),
        };

        // Construct ImmutableData
        let immutable_data_type_id = maidsafe_types::data::ImmutableDataTypeTag;
        let orig_data: Vec<u8> = (0u8..100u8).map(|_| ::rand::random::<u8>()).collect();
        let orig_immutable_data = maidsafe_types::ImmutableData::new(orig_data);

        // Construct StructuredData, 1st version, for this ImmutableData
        let structured_data_type_id = maidsafe_types::data::StructuredDataTypeTag;
        let keyword = (0..100).map(|_| ::rand::random::<char>()).collect();
        let pin = ::rand::random::<u32>() % 10000u32;
        let user_id = ::client::user_account::Account::generate_network_id(&keyword, pin);
        let mut account_version = maidsafe_types::StructuredData::new(user_id.clone(),
                                                                      account_packet.get_public_maid().name(),
                                                                      vec![orig_immutable_data.name()]);

        // GET StructuredData should fail
        {
            match mock_routing.lock().unwrap().get(structured_data_type_id.type_tag(), user_id.clone()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => panic!("Should not have found data before a PUT"),
                        Err(_) => (),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // First PUT of StructuredData should succeed
        {
            match mock_routing.lock().unwrap().put(account_version.clone()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => (),
                        Err(error) => panic!("PUT Response Failure :: {:?}", error.description()),
                    }
                },
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // PUT for ImmutableData should succeed
        {
            match mock_routing.lock().unwrap().put(orig_immutable_data.clone()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => (),
                        Err(error) => panic!("PUT Response Failure :: {:?}", error.description()),
                    }
                },
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        let mut received_structured_data: maidsafe_types::StructuredData;

        // GET StructuredData should pass
        {
            match mock_routing.lock().unwrap().get(structured_data_type_id.type_tag(), user_id.clone()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(data) => {
                            let mut decoder = ::cbor::Decoder::from_bytes(&data[..]);
                            received_structured_data = decoder.decode().next().unwrap().unwrap();

                            assert_eq!(account_version, received_structured_data);
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // GET ImmutableData from lastest version of StructuredData should pass
        {
            match mock_routing.lock().unwrap().get(immutable_data_type_id.type_tag(), received_structured_data.value().pop().unwrap()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(data) => {
                            let mut decoder = ::cbor::Decoder::from_bytes(&data[..]);
                            let received_immutable_data: maidsafe_types::ImmutableData = decoder.decode().next().unwrap().unwrap();

                            assert_eq!(orig_immutable_data, received_immutable_data);
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // Construct ImmutableData
        let new_data: Vec<u8> = (0u8..100u8).map(|_| ::rand::random::<u8>()).collect();
        let new_immutable_data = maidsafe_types::ImmutableData::new(new_data);

        // PUT for new ImmutableData should succeed
        {
            match mock_routing.lock().unwrap().put(new_immutable_data.clone()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => (),
                        Err(error) => panic!("PUT Response Failure :: {:?}", error.description()),
                    }
                },
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // Construct StructuredData, 2nd version, for this ImmutableData
        account_version = maidsafe_types::StructuredData::new(user_id.clone(),
                                                              account_packet.get_public_maid().name(),
                                                              vec![orig_immutable_data.name(), new_immutable_data.name()]);

        // Subsequent PUTs for new StructuredData version should pass
        {
            let put_result = mock_routing.lock().unwrap().put(account_version.clone());
            match put_result {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(_) => (),
                        Err(_) => panic!("StructuredData should be allowed to be overwritten !!"),
                    }
                },
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // GET for new StructuredData version should pass
        {
            match mock_routing.lock().unwrap().get(structured_data_type_id.type_tag(), user_id.clone()) {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(data) => {
                            let mut decoder = ::cbor::Decoder::from_bytes(&data[..]);
                            received_structured_data = decoder.decode().next().unwrap().unwrap();

                            assert_eq!(account_version, received_structured_data);
                        },
                        Err(_) => (),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        assert_eq!(received_structured_data.value().len(), 2);

        // GET new ImmutableData should pass
        {
            let get_result = mock_routing.lock().unwrap().get(immutable_data_type_id.type_tag(), received_structured_data.value()[1].clone());
            match get_result {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(data) => {
                            let mut decoder = ::cbor::Decoder::from_bytes(&data[..]);
                            let received_immutable_data: maidsafe_types::ImmutableData = decoder.decode().next().unwrap().unwrap();

                            assert_eq!(new_immutable_data, received_immutable_data);
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // GET original ImmutableData should pass
        {
            let get_result = mock_routing.lock().unwrap().get(immutable_data_type_id.type_tag(), received_structured_data.value()[0].clone());
            match get_result {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(notifier.clone(), callback_interface.clone(), Some(id), None);
                    match response_getter.get() {
                        Ok(data) => {
                            let mut decoder = ::cbor::Decoder::from_bytes(&data[..]);
                            let received_immutable_data: maidsafe_types::ImmutableData = decoder.decode().next().unwrap().unwrap();

                            assert_eq!(orig_immutable_data, received_immutable_data);
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }
    }
}
