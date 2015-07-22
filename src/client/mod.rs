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
mod callback_interface;

#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
mod mock_routing_types;
#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
pub use self::mock_routing_types::*;

#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
mod non_networking_test_framework;
#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
type RoutingClient = ::std::sync::Arc<::std::sync::Mutex<non_networking_test_framework::RoutingClientMock>>;
#[cfg(not(feature = "USE_ACTUAL_ROUTING"))]
fn get_new_routing_client(cb_interface: ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>, id_packet: ::routing::types::Id) -> RoutingClient {
    ::std::sync::Arc::new(::std::sync::Mutex::new(non_networking_test_framework::RoutingClientMock::new(cb_interface, id_packet)))
}

#[cfg(feature = "USE_ACTUAL_ROUTING")]
type RoutingClient = ::std::sync::Arc<::std::sync::Mutex<::routing::routing_client::RoutingClient<callback_interface::CallbackInterface>>>;
#[cfg(feature = "USE_ACTUAL_ROUTING")]
fn get_new_routing_client(cb_interface: ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>, id_packet: ::routing::types::Id) -> RoutingClient {
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
    callback_interface:  ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>,
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
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(callback_interface::CallbackInterface::new(notifier.clone())));
        let id_packet = ::routing::types::Id::with_keys(account_packet.get_maid().public_keys().clone(),
                                                        account_packet.get_maid().secret_keys().clone());

        let routing_client = get_new_routing_client(callback_interface.clone(), id_packet);
        let cloned_routing_client = routing_client.clone();
        let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let routing_stop_flag_clone = routing_stop_flag.clone();

        let mut client = Client {
            account: account_packet,
            session_packet_id: user_account::Account::generate_network_id(keyword, pin),
            session_packet_keys: SessionPacketEncryptionKeys::new(password, pin),
            routing: routing_client,
            callback_interface: callback_interface,
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
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(callback_interface::CallbackInterface::new(notifier.clone())));
        let fake_id_packet = ::routing::types::Id::with_keys(fake_account_packet.get_maid().public_keys().clone(),
                                                             fake_account_packet.get_maid().secret_keys().clone());

        let fake_routing_client = get_new_routing_client(callback_interface.clone(), fake_id_packet);
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
                                                                                 callback_interface.clone(),
                                                                                 location_session_packet,
                                                                                 ::client::DataRequest::StructuredData(LOGIN_PACKET_TYPE_TAG));
        if let ::client::Data::StructuredData(session_packet) = try!(response_getter.get()) {
            let decrypted_session_packet = try!(user_account::Account::decrypt(session_packet.get_data(), password, pin));
            let id_packet = ::routing::types::Id::with_keys(decrypted_session_packet.get_maid().public_keys().clone(),
                                                            decrypted_session_packet.get_maid().secret_keys().clone());

            let routing_client = get_new_routing_client(callback_interface.clone(), id_packet);
            let cloned_routing_client = routing_client.clone();
            let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
            let routing_stop_flag_clone = routing_stop_flag.clone();

            let client = Client {
                account: decrypted_session_packet,
                session_packet_id: user_account::Account::generate_network_id(keyword, pin),
                session_packet_keys: SessionPacketEncryptionKeys::new(password, pin),
                routing: routing_client,
                callback_interface: callback_interface,
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
            let mut cb_interface = self.callback_interface.lock().unwrap();
            if cb_interface.local_cache_check(&location) {
                return Ok(response_getter::ResponseGetter::new(None, self.callback_interface.clone(), location, request_for));
            }
        }

        try!(self.routing.lock().unwrap().get(location.clone(), request_for.clone()));
        Ok(response_getter::ResponseGetter::new(Some(self.response_notifier.clone()), self.callback_interface.clone(), location, request_for))
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
        let keyword = ::utility::generate_random_string(10).ok().unwrap();
        let password = ::utility::generate_random_string(10).ok().unwrap();
        let pin = ::utility::generate_random_pin();
        let result = Client::create_account(&keyword, pin, &password);
        assert!(result.is_ok());
    }

    #[test]
    fn account_login() {
        let keyword = ::utility::generate_random_string(10).ok().unwrap();
        let password = ::utility::generate_random_string(10).ok().unwrap();
        let pin = ::utility::generate_random_pin();

        let mut result: _;

        // Creation should pass
        result = Client::create_account(&keyword, pin, &password);
        assert!(result.is_ok());

        // Correct Credentials - Login Should Pass
        result = Client::log_in(&keyword, pin, &password);
        assert!(result.is_ok());
    }

    #[test]
    fn user_root_dir_id_creation() {
        // Construct Client
        let keyword = ::utility::generate_random_string(10).ok().unwrap();
        let password = ::utility::generate_random_string(10).ok().unwrap();
        let pin = ::utility::generate_random_pin();

        let result = Client::create_account(&keyword, pin, &password);
        assert!(result.is_ok());
        let mut client = result.ok().unwrap();

        assert!(client.get_user_root_directory_id().is_none());
        assert!(client.get_configuration_root_directory_id().is_none());

        let root_dir_id = ::routing::NameType::new([99u8; 64]);
        match client.set_user_root_directory_id(root_dir_id.clone()) {
            Ok(()) => {
                // Correct Credentials - Login Should Pass
                let result = Client::log_in(&keyword, pin, &password);
                assert!(result.is_ok());

                let client = result.ok().unwrap();

                assert!(client.get_user_root_directory_id().is_some());
                assert!(client.get_configuration_root_directory_id().is_none());

                assert_eq!(client.get_user_root_directory_id(), Some(&root_dir_id));
            },
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn maidsafe_config_root_dir_id_creation() {
        // Construct Client
        let keyword = ::utility::generate_random_string(10).ok().unwrap();
        let password = ::utility::generate_random_string(10).ok().unwrap();
        let pin = ::utility::generate_random_pin();

        let result = Client::create_account(&keyword, pin, &password);
        assert!(result.is_ok());
        let mut client = result.ok().unwrap();

        assert!(client.get_user_root_directory_id().is_none());
        assert!(client.get_configuration_root_directory_id().is_none());

        let root_dir_id = ::routing::NameType::new([99u8; 64]);
        match client.set_configuration_root_directory_id(root_dir_id.clone()) {
            Ok(()) => {
                // Correct Credentials - Login Should Pass
                let result = Client::log_in(&keyword, pin, &password);
                assert!(result.is_ok());

                let client = result.ok().unwrap();

                assert!(client.get_user_root_directory_id().is_none());
                assert!(client.get_configuration_root_directory_id().is_some());

                assert_eq!(client.get_configuration_root_directory_id(), Some(&root_dir_id));
            },
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn hybrid_encryption_decryption() {
        // Construct Client
        let keyword = ::utility::generate_random_string(10).ok().unwrap();
        let password = ::utility::generate_random_string(10).ok().unwrap();
        let pin = ::utility::generate_random_pin();

        let result = Client::create_account(&keyword, pin, &password);
        assert!(result.is_ok());
        let client = result.ok().unwrap();

        // Identical Plain Texts
        let plain_text_0 = vec![123u8; 1000];
        let plain_text_1 = plain_text_0.clone();

        // Encrypt passing Nonce
        let nonce = ::sodiumoxide::crypto::box_::gen_nonce();
        let hybrid_encrypt_0 = client.hybrid_encrypt(&plain_text_0[..], Some(&nonce));
        let hybrid_encrypt_1 = client.hybrid_encrypt(&plain_text_1[..], Some(&nonce));

        // Encrypt without passing Nonce
        let hybrid_encrypt_2 = client.hybrid_encrypt(&plain_text_0[..], None);
        let hybrid_encrypt_3 = client.hybrid_encrypt(&plain_text_1[..], None);

        assert!(hybrid_encrypt_0.is_ok());
        assert!(hybrid_encrypt_1.is_ok());
        assert!(hybrid_encrypt_2.is_ok());
        assert!(hybrid_encrypt_3.is_ok());

        // Same Plain Texts
        assert_eq!(plain_text_0, plain_text_1);

        let cipher_text_0 = hybrid_encrypt_0.ok().unwrap();
        let cipher_text_1 = hybrid_encrypt_1.ok().unwrap();
        let cipher_text_2 = hybrid_encrypt_2.ok().unwrap();
        let cipher_text_3 = hybrid_encrypt_3.ok().unwrap();

        // Different Results because of random "iv"
        assert!(cipher_text_0 != cipher_text_1);
        assert!(cipher_text_0 != cipher_text_2);
        assert!(cipher_text_0 != cipher_text_3);
        assert!(cipher_text_2 != cipher_text_1);
        assert!(cipher_text_2 != cipher_text_3);

        // Decrypt with Nonce
        let hybrid_decrypt_0 = client.hybrid_decrypt(&cipher_text_0, Some(&nonce));
        let hybrid_decrypt_1 = client.hybrid_decrypt(&cipher_text_1, Some(&nonce));

        // Decrypt without Nonce
        let hybrid_decrypt_2 = client.hybrid_decrypt(&cipher_text_2, None);
        let hybrid_decrypt_3 = client.hybrid_decrypt(&cipher_text_3, None);

        // Decryption without passing Nonce for something encrypted with passing Nonce - Should Fail
        let hybrid_decrypt_4 = client.hybrid_decrypt(&cipher_text_0, None);
        // Decryption passing Nonce for something encrypted without passing Nonce - Should Fail
        let hybrid_decrypt_5 = client.hybrid_decrypt(&cipher_text_3, Some(&nonce));

        assert!(hybrid_decrypt_0.is_ok());
        assert!(hybrid_decrypt_1.is_ok());
        assert!(hybrid_decrypt_2.is_ok());
        assert!(hybrid_decrypt_3.is_ok());

        // Should fail
        assert!(hybrid_decrypt_4.is_err());
        assert!(hybrid_decrypt_5.is_err());

        // Should have decrypted to the same Plain Texts
        assert_eq!(plain_text_0, hybrid_decrypt_0.ok().unwrap());
        assert_eq!(plain_text_1, hybrid_decrypt_1.ok().unwrap());
        assert_eq!(plain_text_0, hybrid_decrypt_2.ok().unwrap());
        assert_eq!(plain_text_1, hybrid_decrypt_3.ok().unwrap());
    }
}
