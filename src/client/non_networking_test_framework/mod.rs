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

#![allow(unsafe_code, unused)] // TODO Remove the unused attribute later

use routing;
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

pub struct RoutingClientMock {
    callback_interface: ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>,
    network_delay_ms: u32,
}

impl RoutingClientMock {
    pub fn new(cb_interface: ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>,
               _: routing::types::Id) -> RoutingClientMock {
        RoutingClientMock {
            callback_interface: cb_interface,
            network_delay_ms: 1000,
        }
    }

    #[allow(dead_code)]
    pub fn set_network_delay_for_delay_simulation(&mut self, delay_ms: u32) {
        self.network_delay_ms = delay_ms;
    }

    pub fn get(&mut self, location: ::routing::NameType, request_for: ::client::DataRequest) -> Result<(), ::routing::error::ResponseError> {
        let delay_ms = self.network_delay_ms;
        let cb_interface = self.callback_interface.clone();
        let data_store = get_storage();

        ::std::thread::spawn(move || {
            ::std::thread::sleep_ms(delay_ms);
            match data_store.lock().unwrap().get(&location) {
                Some(raw_data) => {
                    if let Ok(data) = ::utility::deserialise::<::client::Data>(raw_data) {
                        if match (&data, request_for) {
                            (&::client::Data::ImmutableData(ref immut_data), ::client::DataRequest::ImmutableData(ref tag)) => immut_data.get_tag_type() == tag,
                            (&::client::Data::StructuredData(ref struct_data), ::client::DataRequest::StructuredData(ref tag)) => struct_data.get_tag_type() == *tag,
                            _ => false,
                        } {
                            cb_interface.lock().unwrap().handle_get_response(location, data);
                        }
                    }
                },
                None => (),
            };
        });

        Ok(())
    }

    pub fn put(&mut self, location: ::routing::NameType, data: ::client::Data) -> Result<(), ::routing::error::ResponseError> {
        let delay_ms = self.network_delay_ms;
        let cb_interface = self.callback_interface.clone();
        let data_store = get_storage();

        let mut data_store_mutex_guard = data_store.lock().unwrap();
        let success = if data_store_mutex_guard.contains_key(&location) {
            if let ::client::Data::ImmutableData(immut_data) = data {
                match ::utility::deserialise(data_store_mutex_guard.get(&location).unwrap()) {
                    Ok(::client::Data::ImmutableData(immut_data_stored)) => immut_data_stored.get_tag_type() == immut_data.get_tag_type(), // Immutable data is de-duplicated so always allowed
                    _ => false
                }
            } else {
                false
            }
        } else if let Ok(raw_data) = ::utility::serialise(&data) {
            data_store_mutex_guard.insert(location, raw_data);
            true
        } else {
            false
        };

        // ::std::thread::spawn(move || {
        //     ::std::thread::sleep_ms(delay_ms);
        //     if !success {
        //         cb_interface.lock().unwrap().handle_put_post_delete_error(location, ::routing::error::ResponseError::CouldNotPutData(data)); // TODO
        //     }
        // });

        Ok(())
    }

    pub fn post(&mut self, location: ::routing::NameType, data: ::client::Data) -> Result<(), ::routing::error::ResponseError> {
        let delay_ms = self.network_delay_ms;
        let cb_interface = self.callback_interface.clone();
        let data_store = get_storage();

        let mut data_store_mutex_guard = data_store.lock().unwrap();
        let success = if data_store_mutex_guard.contains_key(&location) {
            match (&data, ::utility::deserialise(data_store_mutex_guard.get(&location).unwrap())) {
                (&::client::Data::StructuredData(ref struct_data_new), Ok(::client::Data::StructuredData(ref struct_data_stored))) => {
                    if struct_data_new.get_version() != struct_data_stored.get_version() + 1 {
                        false
                    } else {
                        let mut count = 0usize;
                        if struct_data_stored.get_owners().iter().any(|key| { // This is more efficient than filter as it will stop whenever sign count reaches >= 50%
                            if struct_data_new.get_signatures().iter().any(|sig| ::sodiumoxide::crypto::sign::verify_detached(sig, &struct_data_new.data_to_sign(), key)) {
                                count += 1;
                            }

                            count >= struct_data_stored.get_owners().len() / 2 + struct_data_stored.get_owners().len() % 2
                        }) {
                            if let Ok(raw_data) = ::utility::serialise(&data) {
                                data_store_mutex_guard.insert(location, raw_data);
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                },
                _ => false,
            }
        } else {
            false
        };

        // ::std::thread::spawn(move || {
        //     ::std::thread::sleep_ms(delay_ms);
        //     if !success {
        //         cb_interface.lock().unwrap().handle_put_response(location, ::routing::error::ResponseError::CouldNotPutData(data)); // TODO
        //     }
        // });

        Ok(())
    }

    pub fn delete(&mut self, location: ::routing::NameType, data: ::client::Data) -> Result<(), ::routing::error::ResponseError> {
        let delay_ms = self.network_delay_ms;
        let cb_interface = self.callback_interface.clone();
        let data_store = get_storage();

        let mut data_store_mutex_guard = data_store.lock().unwrap();
        let success = if data_store_mutex_guard.contains_key(&location) {
            match (&data, ::utility::deserialise(data_store_mutex_guard.get(&location).unwrap())) {
                (&::client::Data::StructuredData(ref struct_data_new), Ok(::client::Data::StructuredData(ref struct_data_stored))) => {
                    let mut count = 0usize;
                    if struct_data_stored.get_owners().iter().any(|key| { // This is more efficient than filter as it will stop whenever sign count reaches >= 50%
                        if struct_data_new.get_signatures().iter().any(|sig| ::sodiumoxide::crypto::sign::verify_detached(sig, &struct_data_new.data_to_sign(), key)) {
                            count += 1;
                        }

                        count >= struct_data_stored.get_owners().len() / 2 + struct_data_stored.get_owners().len() % 2
                    }) {
                        let _ = data_store_mutex_guard.remove(&location);
                        true
                    } else {
                        false
                    }
                },
                _ => false,
            }
        } else {
            false
        };

        // ::std::thread::spawn(move || {
        //     ::std::thread::sleep_ms(delay_ms);
        //     if !success {
        //         cb_interface.lock().unwrap().handle_put_response(location, ::routing::error::ResponseError::CouldNotPutData(data)); // TODO
        //     }
        // });

        Ok(())
    }

    pub fn run(&mut self) {
        // let data_store = get_storage();
        // println!("Amount Of Chunks Stored: {:?}", data_store.lock().unwrap().len());
    }

    pub fn bootstrap(&mut self,
                     endpoints: Option<Vec<::routing::routing_client::Endpoint>>,
                     _: Option<u16>) -> Result<(), routing::error::RoutingError> {
        if let Some(vec_endpoints) = endpoints {
            for endpoint in vec_endpoints {
                println!("Endpoint: {:?}", endpoint);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use ::std::error::Error;

    use super::*;

    #[test]
    fn check_put_post_get_delete_for_immutable_data() {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(None), ::std::sync::Condvar::new()));
        let account_packet = ::client::user_account::Account::new(None, None);
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
        let orig_raw_data: Vec<u8> = (0u8..100u8).map(|_| ::rand::random::<u8>()).collect();
        let orig_immutable_data = ::client::ImmutableData::new(::client::ImmutableDataType::Normal, orig_raw_data.clone());
        let orig_data = ::client::Data::ImmutableData(orig_immutable_data.clone());

        // First PUT should succeed
        {
            match mock_routing.lock().unwrap().put(orig_immutable_data.name(), orig_data.clone()) {
                Ok(()) => (),
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // GET ImmutableData should pass
        {
            let mut mock_routing_guard = mock_routing.lock().unwrap();
            match mock_routing_guard.get(orig_immutable_data.name(), ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal)) {
                Ok(()) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(Some(notifier.clone()),
                                                                                             callback_interface.clone(),
                                                                                             orig_immutable_data.name(),
                                                                                             ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal));
                    match response_getter.get() {
                        Ok(data) => {
                            match data {
                                ::client::Data::ImmutableData(received_immutable_data) => assert_eq!(orig_immutable_data, received_immutable_data),
                                _ => panic!("Unexpected!"),
                            }
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // Subsequent PUTs for same ImmutableData should succeed - De-duplication
        {
            let put_result = mock_routing.lock().unwrap().put(orig_immutable_data.name(), orig_data.clone());
            match put_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // Construct Backup ImmutableData
        let new_immutable_data = ::client::ImmutableData::new(::client::ImmutableDataType::Backup, orig_raw_data);
        let new_data = ::client::Data::ImmutableData(new_immutable_data.clone());

        // Subsequent PUTs for same ImmutableData of different type should fail
        {
            let put_result = mock_routing.lock().unwrap().put(orig_immutable_data.name(), new_data);
            match put_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // POSTs for ImmutableData should fail
        {
            let post_result = mock_routing.lock().unwrap().post(orig_immutable_data.name(), orig_data.clone());
            match post_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in POST !!"),
            }
        }

        // DELETEs of ImmutableData should fail
        {
            let delete_result = mock_routing.lock().unwrap().delete(orig_immutable_data.name(), orig_data);
            match delete_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in DELETE !!"),
            }
        }

        // GET ImmutableData should pass
        {
            let mut mock_routing_mutex_guard = mock_routing.lock().unwrap();
            match mock_routing_mutex_guard.get(orig_immutable_data.name(), ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal)) {
                Ok(()) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(Some(notifier.clone()),
                                                                                             callback_interface.clone(),
                                                                                             orig_immutable_data.name(),
                                                                                             ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal));
                    match response_getter.get() {
                        Ok(data) => {
                            match data {
                                ::client::Data::ImmutableData(received_immutable_data) => assert_eq!(orig_immutable_data, received_immutable_data), // TODO Improve by directly assert_eq!(data)
                                _ => panic!("Unexpected!"),
                            }
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }
    }

    #[test]
    fn check_put_post_get_delete_for_structured_data() {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(None), ::std::sync::Condvar::new()));
        let account_packet = ::client::user_account::Account::new(None, None);
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
        let orig_data: Vec<u8> = (0u8..100u8).map(|_| ::rand::random::<u8>()).collect();
        let orig_immutable_data = ::client::ImmutableData::new(::client::ImmutableDataType::Normal, orig_data);
        let orig_data_immutable = ::client::Data::ImmutableData(orig_immutable_data.clone());

        // Construct StructuredData, 1st version, for this ImmutableData
        const TYPE_TAG: u64 = 999;
        let keyword = eval_result!(::utility::generate_random_string(10));
        let pin = ::utility::generate_random_pin();
        let user_id = ::client::user_account::Account::generate_network_id(&keyword, pin);
        let mut account_version = ::client::StructuredData::new(TYPE_TAG,
                                                                user_id.clone(),
                                                                0,
                                                                eval_result!(::utility::serialise(&vec![orig_immutable_data.name()])),
                                                                vec![account_packet.get_public_maid().public_keys().0.clone()],
                                                                Vec::new(),
                                                                &account_packet.get_maid().secret_keys().0);
        let mut data_account_version = ::client::Data::StructuredData(account_version.clone());

        // First PUT of StructuredData should succeed
        {
            match mock_routing.lock().unwrap().put(account_version.name(), data_account_version.clone()) {
                Ok(()) => (),
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // PUT for ImmutableData should succeed
        {
            match mock_routing.lock().unwrap().put(orig_immutable_data.name(), orig_data_immutable.clone()) {
                Ok(()) => (),
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        let mut received_structured_data: ::client::StructuredData;

        // GET StructuredData should pass
        {
            match mock_routing.lock().unwrap().get(account_version.name(), ::client::DataRequest::StructuredData(TYPE_TAG)) {
                Ok(()) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(Some(notifier.clone()),
                                                                                             callback_interface.clone(),
                                                                                             account_version.name(),
                                                                                             ::client::DataRequest::StructuredData(TYPE_TAG));
                    match response_getter.get() {
                        Ok(data) => {
                            match data {
                                ::client::Data::StructuredData(struct_data) => {
                                    received_structured_data = struct_data;
                                    assert!(account_version == received_structured_data);
                                },
                                _ => panic!("Unexpected!"),
                            }
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // GET ImmutableData from lastest version of StructuredData should pass
        {
            let mut location_vec = eval_result!(::utility::deserialise::<Vec<::routing::NameType>>(received_structured_data.get_data()));
            match mock_routing.lock().unwrap().get(eval_option!(location_vec.pop(), "Value must exist !"), ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal)) {
                Ok(()) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(Some(notifier.clone()),
                                                                                             callback_interface.clone(),
                                                                                             orig_immutable_data.name(),
                                                                                             ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal));
                    match response_getter.get() {
                        Ok(data) => {
                            match data {
                                ::client::Data::ImmutableData(received_immutable_data) => assert_eq!(orig_immutable_data, received_immutable_data),
                                _ => panic!("Unexpected!"),
                            }
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // Construct ImmutableData
        let new_data: Vec<u8> = (0u8..100u8).map(|_| ::rand::random::<u8>()).collect();
        let new_immutable_data = ::client::ImmutableData::new(::client::ImmutableDataType::Normal, new_data);
        let new_data_immutable = ::client::Data::ImmutableData(new_immutable_data.clone());

        // PUT for new ImmutableData should succeed
        {
            match mock_routing.lock().unwrap().put(new_immutable_data.name(), new_data_immutable) {
                Ok(()) => (),
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // Construct StructuredData, 2nd version, for this ImmutableData - IVALID Versioning
        let invalid_version_account_version = ::client::StructuredData::new(TYPE_TAG,
                                                                            user_id.clone(),
                                                                            0,
                                                                            ::utility::serialise(&vec![orig_immutable_data.name(), new_immutable_data.name()]).ok().unwrap(),
                                                                            vec![account_packet.get_public_maid().public_keys().0.clone()],
                                                                            Vec::new(),
                                                                            &account_packet.get_maid().secret_keys().0);
        let invalid_version_data_account_version = ::client::Data::StructuredData(invalid_version_account_version.clone());

        // Construct StructuredData, 2nd version, for this ImmutableData - IVALID Signature
        let invalid_signature_account_version = ::client::StructuredData::new(TYPE_TAG,
                                                                              user_id.clone(),
                                                                              1,
                                                                              ::utility::serialise(&vec![orig_immutable_data.name(), new_immutable_data.name()]).ok().unwrap(),
                                                                              vec![account_packet.get_public_maid().public_keys().0.clone()],
                                                                              Vec::new(),
                                                                              &account_packet.get_mpid().secret_keys().0);
        let invalid_signature_data_account_version = ::client::Data::StructuredData(invalid_signature_account_version.clone());

        // Construct StructuredData, 2nd version, for this ImmutableData - Valid
        account_version = ::client::StructuredData::new(TYPE_TAG,
                                                        user_id.clone(),
                                                        1,
                                                        ::utility::serialise(&vec![orig_immutable_data.name(), new_immutable_data.name()]).ok().unwrap(),
                                                        vec![account_packet.get_public_maid().public_keys().0.clone()],
                                                        Vec::new(),
                                                        &account_packet.get_maid().secret_keys().0);
        data_account_version = ::client::Data::StructuredData(account_version.clone());

        // Subsequent PUTs for same StructuredData should fail
        {
            let put_result = mock_routing.lock().unwrap().put(account_version.name(), data_account_version.clone());
            match put_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in PUT !!"),
            }
        }

        // Subsequent POSTSs for same StructuredData should fail if versioning is invalid
        {
            let post_result = mock_routing.lock().unwrap().post(invalid_version_account_version.name(), invalid_version_data_account_version.clone());
            match post_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in POST !!"),
            }
        }

        // Subsequent POSTSs for same StructuredData should fail if signature is invalid
        {
            let post_result = mock_routing.lock().unwrap().post(invalid_signature_account_version.name(), invalid_signature_data_account_version.clone());
            match post_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in POST !!"),
            }
        }

        // Subsequent POSTSs for existing StructuredData version should pass for valid update
        {
            let post_result = mock_routing.lock().unwrap().post(account_version.name(), data_account_version.clone());
            match post_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in POST !!"),
            }
        }

        // GET for new StructuredData version should pass
        {
            match mock_routing.lock().unwrap().get(account_version.name(), ::client::DataRequest::StructuredData(TYPE_TAG)) {
                Ok(()) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(Some(notifier.clone()),
                                                                                             callback_interface.clone(),
                                                                                             account_version.name(),
                                                                                             ::client::DataRequest::StructuredData(TYPE_TAG));
                    match response_getter.get() {
                        Ok(data) => {
                            match data {
                                ::client::Data::StructuredData(structured_data) => {
                                    received_structured_data = structured_data;
                                    assert!(received_structured_data == account_version);
                                },
                                _ => panic!("Unexpected!"),
                            }
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        let location_vec = eval_result!(::utility::deserialise::<Vec<::routing::NameType>>(received_structured_data.get_data()));
        assert_eq!(location_vec.len(), 2);

        // GET new ImmutableData should pass
        {
            let get_result = mock_routing.lock().unwrap().get(location_vec[1].clone(), ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal));
            match get_result {
                Ok(()) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(Some(notifier.clone()),
                                                                                             callback_interface.clone(),
                                                                                             location_vec[1].clone(),
                                                                                             ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal));
                    match response_getter.get() {
                        Ok(data) => {
                            match data {
                                ::client::Data::ImmutableData(received_immutable_data) => assert_eq!(new_immutable_data, received_immutable_data),
                                _ => panic!("Unexpected!"),
                            }
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // GET original ImmutableData should pass
        {
            let get_result = mock_routing.lock().unwrap().get(location_vec[0].clone(), ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal));
            match get_result {
                Ok(id) => {
                    let mut response_getter = ::client::response_getter::ResponseGetter::new(Some(notifier.clone()),
                                                                                             callback_interface.clone(),
                                                                                             location_vec[0].clone(),
                                                                                             ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal));
                    match response_getter.get() {
                        Ok(data) => {
                            match data {
                                ::client::Data::ImmutableData(received_immutable_data) => assert_eq!(orig_immutable_data, received_immutable_data),
                                _ => panic!("Unexpected!"),
                            }
                        },
                        Err(_) => panic!("Should have found data put before by a PUT"),
                    }
                },
                Err(_) => panic!("Failure in GET !!"),
            }
        }

        // DELETE of Structured Data should succeed
        {
            let delete_result = mock_routing.lock().unwrap().delete(account_version.name(), data_account_version.clone());
            match delete_result {
                Ok(()) => (),
                Err(_) => panic!("Failure in DELETE !!"),
            }
        }
    }
}
