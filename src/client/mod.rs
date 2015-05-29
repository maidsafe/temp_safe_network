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

use cbor;
use crypto;

use routing;
use maidsafe_types;
use maidsafe_types::TypeTag;
use routing::sendable::Sendable;

mod user_account;
mod callback_interface;

pub struct Client {
    account:             user_account::Account,
    routing:             ::std::sync::Arc<::std::sync::Mutex<routing::routing_client::RoutingClient<callback_interface::CallbackInterface>>>,
    response_notifier:   ::ResponseNotifier,
    callback_interface:  ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>,
    routing_stop_flag:   ::std::sync::Arc<::std::sync::Mutex<bool>>,
    routing_join_handle: Option<::std::thread::JoinHandle<()>>,
}

impl Client {
    pub fn create_account(keyword: &String, pin: u32, password: &[u8]) -> Result<Client, ::IoError> {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(0), ::std::sync::Condvar::new()));
        let account_packet = user_account::Account::new(None);
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(callback_interface::CallbackInterface::new(notifier.clone())));
        let client_id_packet = routing::routing_client::ClientIdPacket::new(account_packet.get_maid().public_keys().clone(),
                                                                            account_packet.get_maid().secret_keys().clone());

        let routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(routing::routing_client::RoutingClient::new(callback_interface.clone(), client_id_packet)));
        let cloned_routing_client = routing_client.clone();
        let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let routing_stop_flag_clone = routing_stop_flag.clone();

        let client = Client {
            account: account_packet,
            routing: routing_client,
            callback_interface: callback_interface,
            response_notifier: notifier,
            routing_stop_flag: routing_stop_flag,
            routing_join_handle: Some(::std::thread::spawn(move || {
                while !*routing_stop_flag_clone.lock().unwrap() {
                    ::std::thread::sleep_ms(10);
                    cloned_routing_client.lock().unwrap().run();
                }
            })),
        };

        {
            let destination = client.account.get_public_maid().name();
            let boxed_public_maid = Box::new(client.account.get_public_maid().clone());
            client.routing.lock().unwrap().unauthorised_put(destination, boxed_public_maid);
        }

        let encrypted_account = maidsafe_types::ImmutableData::new(client.account.encrypt(&password, pin).ok().unwrap());
        let put_res = client.routing.lock().unwrap().put(encrypted_account.clone());
        match put_res {
            Ok(id) => {
                {
                    let &(ref lock, ref condition_var) = &*client.response_notifier;
                    let mut mutex_guard = lock.lock().unwrap();
                    while *mutex_guard != id {
                        mutex_guard = condition_var.wait(mutex_guard).unwrap();
                    }

                    let mut cb_interface = client.callback_interface.lock().unwrap();
                    if cb_interface.get_response(id).is_err() {
                        return Err(::IoError::new(::std::io::ErrorKind::Other, "Session-Packet PUT-Response Failure !!"));
                    }
                }
                let account_version = maidsafe_types::StructuredData::new(user_account::Account::generate_network_id(&keyword, pin),
                                                                          client.account.get_public_maid().name(),
                                                                          vec![encrypted_account.name()]);
                let put_res = client.routing.lock().unwrap().put(account_version);
                match put_res {
                    Ok(id) => {
                        {
                            let &(ref lock, ref condition_var) = &*client.response_notifier;
                            let mut mutex_guard = lock.lock().unwrap();
                            while *mutex_guard != id {
                                mutex_guard = condition_var.wait(mutex_guard).unwrap();
                            }

                            let mut cb_interface = client.callback_interface.lock().unwrap();
                            if cb_interface.get_response(id).is_err() {
                                return Err(::IoError::new(::std::io::ErrorKind::Other, "Version-Packet PUT-Response Failure !!"));
                            }
                        }

                        Ok(client)
                    },
                    Err(io_error) => Err(io_error),
                }
            },
            Err(io_error) => Err(io_error),
        }
    }

    pub fn log_in(keyword: &String, pin: u32, password: &[u8]) -> Result<Client, ::IoError> {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(0), ::std::sync::Condvar::new()));
        let user_network_id = user_account::Account::generate_network_id(keyword, pin);
        let fake_account_packet = user_account::Account::new(None);
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(callback_interface::CallbackInterface::new(notifier.clone())));
        let fake_client_id_packet = routing::routing_client::ClientIdPacket::new(fake_account_packet.get_maid().public_keys().clone(),
                                                                                 fake_account_packet.get_maid().secret_keys().clone());

        let fake_routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(routing::routing_client::RoutingClient::new(callback_interface.clone(), fake_client_id_packet)));
        let cloned_fake_routing_client = fake_routing_client.clone();
        let fake_routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let fake_routing_stop_flag_clone = fake_routing_stop_flag.clone();

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
            routing_stop_flag: fake_routing_stop_flag,
            join_handle: Some(::std::thread::spawn(move || {
                while !*fake_routing_stop_flag_clone.lock().unwrap() {
                    ::std::thread::sleep_ms(10);
                    cloned_fake_routing_client.lock().unwrap().run();
                }
            })),
        };

        let get_result = fake_routing_client.lock().unwrap().get(100u64, user_network_id); // TODO(Spandan) Structured Data should implement trait TypeTag in maidsafe_types

        match get_result {
            Ok(id) => {
                let mut get_response_result: _;

                {
                    let &(ref lock, ref condition_var) = &*notifier;
                    let mut mutex_guard = lock.lock().unwrap();
                    while *mutex_guard != id {
                        mutex_guard = condition_var.wait(mutex_guard).unwrap();
                    }

                    let mut cb_interface = callback_interface.lock().unwrap();
                    get_response_result = cb_interface.get_response(id);
                }

                match get_response_result {
                    Ok(raw_data) => {
                        let mut decoder = cbor::Decoder::from_bytes(raw_data);
                        let account_version: maidsafe_types::StructuredData = decoder.decode().next().unwrap().unwrap();

                        match account_version.value().pop() {
                            Some(latest_version) => {
                                let immutable_data_type_id: maidsafe_types::ImmutableDataTypeTag = unsafe { ::std::mem::uninitialized() };
                                let get_result = fake_routing_client.lock().unwrap().get(immutable_data_type_id.type_tag(), latest_version);
                                match get_result {
                                    Ok(id) => {
                                        {
                                            let &(ref lock, ref condition_var) = &*notifier;
                                            let mut mutex_guard = lock.lock().unwrap();
                                            while *mutex_guard != id {
                                                mutex_guard = condition_var.wait(mutex_guard).unwrap();
                                            }

                                            let mut cb_interface = callback_interface.lock().unwrap();
                                            get_response_result = cb_interface.get_response(id);
                                        }

                                        match get_response_result {
                                            Ok(raw_data) => {
                                                let mut decoder = cbor::Decoder::from_bytes(raw_data);
                                                let encrypted_account_packet: maidsafe_types::ImmutableData = decoder.decode().next().unwrap().unwrap();
                                                let account_packet = user_account::Account::decrypt(&encrypted_account_packet.value()[..], &password, pin).ok().unwrap();

                                                let client_id_packet = routing::routing_client::ClientIdPacket::new(account_packet.get_maid().public_keys().clone(),
                                                                                                                    account_packet.get_maid().secret_keys().clone());

                                                let routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(routing::routing_client::RoutingClient::new(callback_interface.clone(), client_id_packet)));
                                                let cloned_routing_client = routing_client.clone();
                                                let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
                                                let routing_stop_flag_clone = routing_stop_flag.clone();

                                                let client = Client {
                                                    account: account_packet,
                                                    routing: routing_client,
                                                    callback_interface: callback_interface,
                                                    response_notifier: notifier,
                                                    routing_stop_flag: routing_stop_flag,
                                                    routing_join_handle: Some(::std::thread::spawn(move || {
                                                        while !*routing_stop_flag_clone.lock().unwrap() {
                                                            ::std::thread::sleep_ms(10);
                                                            cloned_routing_client.lock().unwrap().run();
                                                        }
                                                    })),
                                                };

                                                Ok(client)
                                            },
                                            Err(_) => Err(::IoError::new(::std::io::ErrorKind::Other, "Session Packet (ImmutableData) GET-Response Failure !!")),
                                        }
                                    },
                                    Err(io_error) => Err(io_error),
                                }
                            },
                            None => Err(::IoError::new(::std::io::ErrorKind::Other, "No Session Packet information in retrieved StructuredData !!")),
                        }
                    },
                    Err(_) => Err(::IoError::new(::std::io::ErrorKind::Other, "StructuredData GET-Response Failure !!")),
                }
            },
            Err(io_error) => Err(io_error),
        }
    }

    pub fn put<T>(&mut self, sendable: T) -> Result<::WaitCondition, ::IoError> where T: Sendable {
        match self.routing.lock().unwrap().put(sendable) {
            Ok(id)      => Ok((id, self.response_notifier.clone())),
            Err(io_err) => Err(io_err),
        }
    }

    pub fn get(&mut self, name: routing::NameType) -> Result<::WaitCondition, ::IoError> {
        match self.routing.lock().unwrap().get(101u64, name) { //TODO(Spandan) Change this to accept proper type_tag/type_id
            Ok(id)      => Ok((id, self.response_notifier.clone())),
            Err(io_err) => Err(io_err),
        }
    }

    pub fn get_network_response_callback(&self) -> ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>> {
        self.callback_interface.clone()
    }

    pub fn get_response(&mut self, message_id: routing::types::MessageId) -> Result<Vec<u8>, routing::error::ResponseError> {
         self.callback_interface.lock().unwrap().get_response(message_id)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        *self.routing_stop_flag.lock().unwrap() = true;
        self.routing_join_handle.take().unwrap().join().unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use routing;

    #[test]
    fn account_creation() {
        // let keyword = "Spandan".to_string();
        // let password = "Sharma".as_bytes();
        // let pin = 1234u32;
        // let mut result = Client::create_account(&keyword, pin, &password);
        // assert!(result.is_ok());
    }
}
