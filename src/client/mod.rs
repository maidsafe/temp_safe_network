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
        let mut cloned_routing_client = routing_client.clone();
        let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let routing_stop_flag_clone = routing_stop_flag.clone();

        let mut client = Client {
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

  //pub fn log_in(keyword : &String, password : &[u8], pin : u32) -> Client {
  //  let mut fetched_encrypted : Vec<u8>;
  //  {
  //    let network_id = Account::generate_network_id(keyword, pin);
  //    let temp_account = Account::new();
  //    let temp_cvar = Arc::new((Mutex::new(false), Condvar::new()));
  //    let temp_facade = Arc::new(Mutex::new(callback_interface::CallbackInterface::new(temp_cvar.clone())));
  //    let mut temp_routing = RoutingClient::new(callback_interface::CallbackInterface::new(temp_cvar.clone()), temp_account.get_account().clone());
  //    let mut get_queue = temp_routing.get(102u64, NameType::new(network_id.0));
  //    let &(ref lock, ref condition_var) = &*temp_cvar;
  //    let mut fetched = lock.lock().unwrap();
  //    while !*fetched {
  //        fetched = condition_var.wait(fetched).unwrap();
  //    }
  //    {
  //      let &ref facade_lock = &*temp_facade;
  //      let mut facade = facade_lock.lock().unwrap();
  //      let fetched_ownership = facade.get_response(get_queue.ok().unwrap()).ok().unwrap();
  //      // fetched_ownership is serialised SDV, the encrypted account shall be the root of of it
  //      let mut d = Decoder::from_bytes(fetched_ownership);
  //      let ownership: maidsafe_types::StructuredData = d.decode().next().unwrap().unwrap();
  //      *fetched = false;
  //      get_queue = temp_routing.get(101u64, ownership.get_value()[0][0].clone());
  //    }
  //    while !*fetched {
  //        fetched = notifier.wait(fetched).unwrap();
  //    }
  //    {
  //      let &ref facade_lock = &*temp_facade;
  //      let mut facade = facade_lock.lock().unwrap();
  //      fetched_encrypted = facade.get_response(get_queue.ok().unwrap()).ok().unwrap();
  //    }
  //  }
  //  let existing_account = Account::decrypt(&fetched_encrypted[..], &password, pin).ok().unwrap();
  //  let notifier = Arc::new((Mutex::new(false), Condvar::new()));
  //  let facade = Arc::new(Mutex::new(callback_interface::CallbackInterface::new(notifier.clone())));
  //  Client { routing: RoutingClient::new(callback_interface::CallbackInterface::new(notifier.clone()), existing_account.get_account().clone()),
  //           account: existing_account, callback_interface: facade, response_notifier: notifier }
  //}
     pub fn put<T>(&mut self, sendable: T) where T: Sendable {
       let _ =  self.routing.lock().unwrap().put(sendable);
     }

     pub fn get_response(&mut self,
                         message_id: routing::types::MessageId) -> Result<Vec<u8>, routing::error::ResponseError> {
        self.callback_interface.lock().unwrap().get_response(message_id)
     }

    pub fn get(&mut self, data_name: routing::NameType) -> Result<::WaitCondition, ::IoError>  {
        match self.routing.lock().unwrap().get(0u64, data_name) {
            Ok(id)      => Ok((id, self.response_notifier.clone())),
            Err(io_err) => Err(io_err),
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        *self.routing_stop_flag.lock().unwrap() = true;
        self.routing_join_handle.take().unwrap().join();
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
