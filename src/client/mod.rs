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

/// ResponseGetter is a lazy evaluated response getter.
pub mod response_getter;

mod user_account;
mod message_queue;

#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
mod mock_routing_types;
#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
pub use self::mock_routing_types::*;

#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
mod non_networking_test_framework;
#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
type RoutingClient = ::std::sync::Arc<::std::sync::Mutex<non_networking_test_framework::RoutingClientMock>>;
#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
fn get_new_routing_client(id_packet: ::routing::types::Id) -> (RoutingClient, ::std::sync::mpsc::Receiver<(::routing::NameType, Data)>) {
    let (routing_client_mock, receiver) = non_networking_test_framework::RoutingClientMock::new(id_packet);
    (::std::sync::Arc::new(::std::sync::Mutex::new(routing_client_mock)), receiver)
}

#[cfg(feature = "USE_ACTUAL_ROUTING")]
type RoutingClient = ::std::sync::Arc<::std::sync::Mutex<::routing::routing_client::RoutingClient<message_queue::MessageQueue>>>;
#[cfg(feature = "USE_ACTUAL_ROUTING")]
fn get_new_routing_client(cb_interface: ::std::sync::Arc<::std::sync::Mutex<message_queue::MessageQueue>>, id_packet: ::routing::types::Id) -> RoutingClient {
    ::std::sync::Arc::new(::std::sync::Mutex::new(::routing::routing_client::RoutingClient::new(cb_interface, id_packet)))
}

mod misc {
    pub type ResponseNotifier = ::std::sync::Arc<(::std::sync::Mutex<Option<::routing::NameType>>, ::std::sync::Condvar)>;
}

const POLL_DURATION_IN_MILLISEC: u32 = 1;
const LOGIN_PACKET_TYPE_TAG: u64 = ::CLIENT_STRUCTURED_DATA_TAG - 1;

/// The main self-authentication client instance that will interface all the request from high
/// level API's to the actual routing layer and manage all interactions with it. This is
/// essentially a non-blocking Client with upper layers having an option to either block and wait
/// on the returned ResponseGetter for receiving network response or spawn a new thread. The Client
/// itself is however well equipped for parallel and non-blocking PUTs and GETS.
pub struct Client {
    account:             user_account::Account,
    session_packet_id:   ::routing::NameType,
    session_packet_keys: SessionPacketEncryptionKeys,
    routing:             RoutingClient,
    response_notifier:   misc::ResponseNotifier,
    message_queue:  ::std::sync::Arc<::std::sync::Mutex<message_queue::MessageQueue>>,
    routing_stop_flag:   ::std::sync::Arc<::std::sync::Mutex<bool>>,
    routing_join_handle: Option<::std::thread::JoinHandle<()>>,
}

impl Client {
    /// This is one of the two Gateway functions to the Maidsafe network, the other being the
    /// log_in. This will help create a fresh account for the user in the SAFE-network.
    pub fn create_account(keyword: &String, pin: u32, password_str: &String) -> Result<Client, ::errors::ClientError> {
        let password = password_str.as_bytes();

        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(None), ::std::sync::Condvar::new()));
        let account_packet = user_account::Account::new(None, None);
        let id_packet = ::routing::types::Id::with_keys(account_packet.get_maid().public_keys().clone(),
                                                        account_packet.get_maid().secret_keys().clone());

        let (routing_client, receiver) = get_new_routing_client(id_packet);
        let message_queue = message_queue::MessageQueue::new(notifier.clone(), receiver);
        let cloned_routing_client = routing_client.clone();
        let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let routing_stop_flag_clone = routing_stop_flag.clone();

        let mut client = Client {
            account            : account_packet,
            session_packet_id  : user_account::Account::generate_network_id(keyword, pin),
            session_packet_keys: SessionPacketEncryptionKeys::new(password, pin),
            routing            : routing_client,
            message_queue      : message_queue,
            response_notifier  : notifier,
            routing_stop_flag  : routing_stop_flag,
            routing_join_handle: Some(::std::thread::spawn(move || {
                                     let _ = cloned_routing_client.lock().unwrap().bootstrap(None, None);
                                     while !*routing_stop_flag_clone.lock().unwrap() {
                                         ::std::thread::sleep_ms(POLL_DURATION_IN_MILLISEC);
                                         cloned_routing_client.lock().unwrap().run();
                                     }
                                 })),
        };

        let account_version = ::client::StructuredData::new(LOGIN_PACKET_TYPE_TAG,
                                                            client.session_packet_id.clone(),
                                                            0,
                                                            try!(client.account.encrypt(password, pin)),
                                                            vec![client.account.get_public_maid().public_keys().0.clone()],
                                                            Vec::new(),
                                                            &client.account.get_maid().secret_keys().0);
        try!(client.put(account_version.name(), ::client::Data::StructuredData(account_version)));
        Ok(client)
    }

    /// This is one of the two Gateway functions to the Maidsafe network, the other being the
    /// create_account. This will help log into an already created account for the user in the
    /// SAFE-network.
    pub fn log_in(keyword: &String, pin: u32, password_str: &String) -> Result<Client, ::errors::ClientError> {
        let password = password_str.as_bytes();

        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(None), ::std::sync::Condvar::new()));
        let user_network_id = user_account::Account::generate_network_id(keyword, pin);
        let fake_account_packet = user_account::Account::new(None, None);
        let fake_id_packet = ::routing::types::Id::with_keys(fake_account_packet.get_maid().public_keys().clone(),
                                                             fake_account_packet.get_maid().secret_keys().clone());

        let (fake_routing_client, receiver) = get_new_routing_client(fake_id_packet);
        let message_queue = message_queue::MessageQueue::new(notifier.clone(), receiver);

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
                let _ = cloned_fake_routing_client.lock().unwrap().bootstrap(None, None);
                while !*fake_routing_stop_flag_clone.lock().unwrap() {
                    ::std::thread::sleep_ms(POLL_DURATION_IN_MILLISEC);
                    cloned_fake_routing_client.lock().unwrap().run();
                }
            })),
        };

        // TODO - Remove This thread::sleep Hack
        ::std::thread::sleep_ms(3000);

        let location_session_packet = ::client::StructuredData::compute_name(LOGIN_PACKET_TYPE_TAG, &user_network_id);
        try!(fake_routing_client.lock().unwrap().get(location_session_packet.clone(), ::client::DataRequest::StructuredData(LOGIN_PACKET_TYPE_TAG)));

        let mut response_getter = ::client::response_getter::ResponseGetter::new(Some(notifier.clone()),
                                                                                 message_queue.clone(),
                                                                                 location_session_packet,
                                                                                 ::client::DataRequest::StructuredData(LOGIN_PACKET_TYPE_TAG));
        if let ::client::Data::StructuredData(session_packet) = try!(response_getter.get()) {
            let decrypted_session_packet = try!(user_account::Account::decrypt(session_packet.get_data(), password, pin));
            let id_packet = ::routing::types::Id::with_keys(decrypted_session_packet.get_maid().public_keys().clone(),
                                                            decrypted_session_packet.get_maid().secret_keys().clone());

            let (routing_client, receiver) = get_new_routing_client(id_packet);
            let message_queue = message_queue::MessageQueue::new(notifier.clone(), receiver);
            let cloned_routing_client = routing_client.clone();
            let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
            let routing_stop_flag_clone = routing_stop_flag.clone();

            let client = Client {
                account: decrypted_session_packet,
                session_packet_id: user_account::Account::generate_network_id(keyword, pin),
                session_packet_keys: SessionPacketEncryptionKeys::new(password, pin),
                routing: routing_client,
                message_queue: message_queue,
                response_notifier: notifier,
                routing_stop_flag: routing_stop_flag,
                routing_join_handle: Some(::std::thread::spawn(move || {
                    let _ = cloned_routing_client.lock().unwrap().bootstrap(None, None);
                    while !*routing_stop_flag_clone.lock().unwrap() {
                        ::std::thread::sleep_ms(POLL_DURATION_IN_MILLISEC);
                        cloned_routing_client.lock().unwrap().run();
                    }
                })),
            };

            Ok(client)
        } else {
            Err(::errors::ClientError::ReceivedUnexpectedData)
        }
    }

    /// Create an entry for the Root Directory ID for the user into the session packet, encrypt and
    /// store it. It will be retireved when the user logs into his account. Root directory ID is
    /// necessary to fetch all of user's data as all further data is encoded as meta-information
    /// into the Root Directory or one of its subdirectories.
    pub fn set_user_root_directory_id(&mut self, root_dir_id: ::routing::NameType) -> Result<(), ::errors::ClientError> {
        if self.account.set_user_root_dir_id(root_dir_id) {
            self.update_session_packet()
        } else {
            Err(::errors::ClientError::RootDirectoryAlreadyExists)
        }
    }

    /// Get User's Root Directory ID if available in session packet used for current login
    pub fn get_user_root_directory_id(&self) -> Option<&::routing::NameType> {
        self.account.get_user_root_dir_id()
    }

    /// Create an entry for the Maidsafe configuration specific Root Directory ID into the
    /// session packet, encrypt and store it. It will be retireved when the user logs into
    /// his account. Root directory ID is necessary to fetch all of configuration data as all further
    /// data is encoded as meta-information into the config Root Directory or one of its subdirectories.
    pub fn set_configuration_root_directory_id(&mut self, root_dir_id: ::routing::NameType) -> Result<(), ::errors::ClientError> {
        if self.account.set_maidsafe_config_root_dir_id(root_dir_id) {
            self.update_session_packet()
        } else {
            Err(::errors::ClientError::RootDirectoryAlreadyExists)
        }
    }

    /// Get Maidsafe specific configuration's Root Directory ID if available in session packet used
    /// for current login
    pub fn get_configuration_root_directory_id(&self) -> Option<&::routing::NameType> {
        self.account.get_maidsafe_config_root_dir_id()
    }

    /// Combined Asymmectric and Symmetric encryption. The data is encrypted using random Key and
    /// IV with Xsalsa-symmetric encryption. Random IV ensures that same plain text produces different
    /// cipher-texts for each fresh symmetric encryption. The Key and IV are then asymmetrically
    /// enrypted using Public-MAID and the whole thing is then serialised into a single Vec<u8>.
    pub fn hybrid_encrypt(&self,
                          data_to_encrypt: &[u8],
                          nonce_opt: Option<&::sodiumoxide::crypto::box_::Nonce>) -> Result<Vec<u8>, ::errors::ClientError> {
        let mut nonce_default = ::sodiumoxide::crypto::box_::Nonce([0u8; ::sodiumoxide::crypto::box_::NONCEBYTES]);
        let nonce = match nonce_opt {
            Some(nonce) => nonce,
            None => {
                let digest = ::sodiumoxide::crypto::hash::sha256::hash(&self.account.get_public_maid().name().0);
                let min_length = ::std::cmp::min(::sodiumoxide::crypto::box_::NONCEBYTES, digest.0.len());
                for it in digest.0.iter().take(min_length).enumerate() {
                    nonce_default.0[it.0] = *it.1;
                }
                &nonce_default
            },
        };

        Ok(try!(::utility::hybrid_encrypt(data_to_encrypt,
                                          &nonce,
                                          &self.account.get_public_maid().public_keys().1,
                                          &self.account.get_maid().secret_keys().1)))
    }

    /// Reverse of hybrid_encrypt. Refer hybrid_encrypt.
    pub fn hybrid_decrypt(&self,
                          data_to_decrypt: &[u8],
                          nonce_opt: Option<&::sodiumoxide::crypto::box_::Nonce>) -> Result<Vec<u8>, ::errors::ClientError> {
        let mut nonce_default = ::sodiumoxide::crypto::box_::Nonce([0u8; ::sodiumoxide::crypto::box_::NONCEBYTES]);
        let nonce = match nonce_opt {
            Some(nonce) => nonce,
            None => {
                let digest = ::sodiumoxide::crypto::hash::sha256::hash(&self.account.get_public_maid().name().0);
                let min_length = ::std::cmp::min(::sodiumoxide::crypto::box_::NONCEBYTES, digest.0.len());
                for it in digest.0.iter().take(min_length).enumerate() {
                    nonce_default.0[it.0] = *it.1;
                }
                &nonce_default
            },
        };

        Ok(try!(::utility::hybrid_decrypt(data_to_decrypt,
                                          &nonce,
                                          &self.account.get_public_maid().public_keys().1,
                                          &self.account.get_maid().secret_keys().1)))
    }

    /// Get data onto the network. This is non-blocking.
    pub fn get(&mut self, location: ::routing::NameType, request_for: DataRequest) -> Result<response_getter::ResponseGetter, ::errors::ClientError> {
        if let ::client::DataRequest::ImmutableData(_) = request_for {
            let mut cb_interface = self.message_queue.lock().unwrap();
            if cb_interface.local_cache_check(&location) {
                return Ok(response_getter::ResponseGetter::new(None, self.message_queue.clone(), location, request_for));
            }
        }

        try!(self.routing.lock().unwrap().get(location.clone(), request_for.clone()));
        Ok(response_getter::ResponseGetter::new(Some(self.response_notifier.clone()), self.message_queue.clone(), location, request_for))
    }

    /// Put data onto the network. This is non-blocking.
    pub fn put(&mut self, location: ::routing::NameType, data: Data) -> Result<(), ::errors::ClientError> {
        Ok(try!(self.routing.lock().unwrap().put(location, data)))
    }

    /// Post data onto the network
    pub fn post(&mut self, location: ::routing::NameType, data: Data) -> Result<(), ::errors::ClientError> {
        Ok(try!(self.routing.lock().unwrap().post(location, data)))
    }

    /// Delete data from the network
    pub fn delete(&mut self, location: ::routing::NameType, data: Data) -> Result<(), ::errors::ClientError> {
        Ok(try!(self.routing.lock().unwrap().delete(location, data)))
    }

    /// Returns the public encryption key
    pub fn get_public_encryption_key(&self) -> &::sodiumoxide::crypto::box_::PublicKey {
        &self.account.get_maid().public_keys().1
    }

    /// Returns the Secret encryption key
    pub fn get_secret_encryption_key(&self) -> &::sodiumoxide::crypto::box_::SecretKey {
        &self.account.get_maid().secret_keys().1
    }

    /// Returns the Public Signing key
    pub fn get_public_signing_key(&self) -> &::sodiumoxide::crypto::sign::PublicKey {
        &self.account.get_maid().public_keys().0
    }

    /// Returns the Secret Signing key
    pub fn get_secret_signing_key(&self) -> &::sodiumoxide::crypto::sign::SecretKey {
        &self.account.get_maid().secret_keys().0
    }

    fn update_session_packet(&mut self) -> Result<(), ::errors::ClientError> {
        let encrypted_account = try!(self.account.encrypt(self.session_packet_keys.get_password(), self.session_packet_keys.get_pin()));
        let location = ::client::StructuredData::compute_name(LOGIN_PACKET_TYPE_TAG, &self.session_packet_id);
        if let ::client::Data::StructuredData(retrieved_session_packet) = try!(try!(self.get(location.clone(),
                                                                                             ::client::DataRequest::StructuredData(LOGIN_PACKET_TYPE_TAG))).get()) {
            let new_account_version = ::client::StructuredData::new(LOGIN_PACKET_TYPE_TAG,
                                                                    self.session_packet_id.clone(),
                                                                    retrieved_session_packet.get_version() + 1,
                                                                    encrypted_account,
                                                                    vec![self.account.get_public_maid().public_keys().0.clone()],
                                                                    Vec::new(),
                                                                    &self.account.get_maid().secret_keys().0);
            Ok(try!(self.post(location, ::client::Data::StructuredData(new_account_version))))
        } else {
            Err(::errors::ClientError::ReceivedUnexpectedData)
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        *self.routing_stop_flag.lock().unwrap() = true;
        self.routing_join_handle.take().unwrap().join().unwrap();
    }
}

/////////////////////////////////////////////////////////////////
/// Helper Struct
/////////////////////////////////////////////////////////////////

struct SessionPacketEncryptionKeys {
    password: Vec<u8>,
    pin: u32,
}

impl SessionPacketEncryptionKeys {
    fn new(password: &[u8], pin: u32) -> SessionPacketEncryptionKeys {
        let vec: Vec<u8> = password.iter().map(|a| *a).collect();
        SessionPacketEncryptionKeys {
            password: vec,
            pin: pin,
        }
    }

    fn get_password(&self) -> &[u8] {
        &self.password[..]
    }

    fn get_pin(&self) -> u32 {
        self.pin
    }
}

/////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn account_creation() {
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = ::utility::generate_random_pin();
        let _ = eval_result!(Client::create_account(&keyword, pin, &password));
    }

    #[test]
    fn account_login() {
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = ::utility::generate_random_pin();

        // Creation should pass
        let _ = eval_result!(Client::create_account(&keyword, pin, &password));

        // Correct Credentials - Login Should Pass
        let _ = eval_result!(Client::log_in(&keyword, pin, &password));
    }

    #[test]
    fn user_root_dir_id_creation() {
        // Construct Client
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = ::utility::generate_random_pin();

        let mut client = eval_result!(Client::create_account(&keyword, pin, &password));

        assert!(client.get_user_root_directory_id().is_none());
        assert!(client.get_configuration_root_directory_id().is_none());

        let root_dir_id = ::routing::NameType::new([99u8; 64]);
        eval_result!(client.set_user_root_directory_id(root_dir_id.clone()));

        // Correct Credentials - Login Should Pass
        let client = eval_result!(Client::log_in(&keyword, pin, &password));

        assert!(client.get_user_root_directory_id().is_some());
        assert!(client.get_configuration_root_directory_id().is_none());

        assert_eq!(client.get_user_root_directory_id(), Some(&root_dir_id));
    }

    #[test]
    fn maidsafe_config_root_dir_id_creation() {
        // Construct Client
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = ::utility::generate_random_pin();

        let mut client = eval_result!(Client::create_account(&keyword, pin, &password));

        assert!(client.get_user_root_directory_id().is_none());
        assert!(client.get_configuration_root_directory_id().is_none());

        let root_dir_id = ::routing::NameType::new([99u8; 64]);
        eval_result!(client.set_configuration_root_directory_id(root_dir_id.clone()));

        // Correct Credentials - Login Should Pass
        let client = eval_result!(Client::log_in(&keyword, pin, &password));

        assert!(client.get_user_root_directory_id().is_none());
        assert!(client.get_configuration_root_directory_id().is_some());

        assert_eq!(client.get_configuration_root_directory_id(), Some(&root_dir_id));
    }

    #[test]
    fn hybrid_encryption_decryption() {
        // Construct Client
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = ::utility::generate_random_pin();

        let client = eval_result!(Client::create_account(&keyword, pin, &password));

        // Identical Plain Texts
        let plain_text_original_0 = vec![123u8; 1000];
        let plain_text_original_1 = plain_text_original_0.clone();

        // Encrypt passing Nonce
        let nonce = ::sodiumoxide::crypto::box_::gen_nonce();
        let cipher_text_0 = eval_result!(client.hybrid_encrypt(&plain_text_original_0[..], Some(&nonce)));
        let cipher_text_1 = eval_result!(client.hybrid_encrypt(&plain_text_original_1[..], Some(&nonce)));

        // Encrypt without passing Nonce
        let cipher_text_2 = eval_result!(client.hybrid_encrypt(&plain_text_original_0[..], None));
        let cipher_text_3 = eval_result!(client.hybrid_encrypt(&plain_text_original_1[..], None));

        // Same Plain Texts
        assert_eq!(plain_text_original_0, plain_text_original_1);

        // Different Results because of random "iv"
        assert!(cipher_text_0 != cipher_text_1);
        assert!(cipher_text_0 != cipher_text_2);
        assert!(cipher_text_0 != cipher_text_3);
        assert!(cipher_text_2 != cipher_text_1);
        assert!(cipher_text_2 != cipher_text_3);

        // Decrypt with Nonce
        let plain_text_0 = eval_result!(client.hybrid_decrypt(&cipher_text_0, Some(&nonce)));
        let plain_text_1 = eval_result!(client.hybrid_decrypt(&cipher_text_1, Some(&nonce)));

        // Decrypt without Nonce
        let plain_text_2 = eval_result!(client.hybrid_decrypt(&cipher_text_2, None));
        let plain_text_3 = eval_result!(client.hybrid_decrypt(&cipher_text_3, None));

        // Decryption without passing Nonce for something encrypted with passing Nonce - Should Fail
        match client.hybrid_decrypt(&cipher_text_0, None) {
            Ok(_) => panic!("Should have failed !"),
            Err(::errors::ClientError::AsymmetricDecipherFailure) => (),
            Err(error) => panic!("{:?}", error),
        }
        // Decryption passing Nonce for something encrypted without passing Nonce - Should Fail
        match client.hybrid_decrypt(&cipher_text_3, Some(&nonce)) {
            Ok(_) => panic!("Should have failed !"),
            Err(::errors::ClientError::AsymmetricDecipherFailure) => (),
            Err(error) => panic!("{:?}", error),
        }

        // Should have decrypted to the same Plain Texts
        assert_eq!(plain_text_original_0, plain_text_0);
        assert_eq!(plain_text_original_1, plain_text_1);
        assert_eq!(plain_text_original_0, plain_text_2);
        assert_eq!(plain_text_original_1, plain_text_3);
    }
}
