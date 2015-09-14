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

mod misc;
mod user_account;
mod message_queue;

#[cfg(feature = "use-mock-routing")]
mod non_networking_test_framework;

#[cfg(feature = "use-mock-routing")]
type Routing = non_networking_test_framework::RoutingMock;
#[cfg(not(feature = "use-mock-routing"))]
type Routing = ::routing::routing_client::RoutingClient;

const LOGIN_PACKET_TYPE_TAG: u64 = ::CLIENT_STRUCTURED_DATA_TAG - 1;

/// The main self-authentication client instance that will interface all the request from high
/// level API's to the actual routing layer and manage all interactions with it. This is
/// essentially a non-blocking Client with upper layers having an option to either block and wait
/// on the returned ResponseGetter for receiving network response or spawn a new thread. The Client
/// itself is however well equipped for parallel and non-blocking PUTs and GETS.
pub struct Client {
    account            : Option<user_account::Account>,
    routing            : Routing,
    _raii_joiner       : misc::RAIIThreadJoiner,
    message_queue      : ::std::sync::Arc<::std::sync::Mutex<message_queue::MessageQueue>>,
    response_notifier  : misc::ResponseNotifier,
    session_packet_id  : Option<::routing::NameType>,
    session_packet_keys: Option<SessionPacketEncryptionKeys>,
}

impl Client {
    /// This is a getter-only Gateway function to the Maidsafe network. It will create an
    /// unregistered random clinet, which can do very limited set of operations - eg., a
    /// Network-Get
    pub fn create_unregistered_client() -> Client {
        debug!("Creating unregistered client ...");

        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(None), ::std::sync::Condvar::new()));
        // TODO stabilise this design
        let bootstrap_notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(false), ::std::sync::Condvar::new()));
        let (sender, receiver) = ::std::sync::mpsc::channel();

        let routing = Client::get_new_routing(sender, None);
        let (message_queue, raii_joiner) = message_queue::MessageQueue::new(notifier.clone(), bootstrap_notifier.clone(), receiver);

        // TODO stabilise this design
        {
            debug!("Bootstrapping ...");
            let (ref lock, ref condition_var) = *bootstrap_notifier;
            let mut mutex_guard = eval_result!(lock.lock());
            while !*mutex_guard {
                mutex_guard = eval_result!(condition_var.wait(mutex_guard));
            }
            debug!("Bootstrapped");
        }

        Client {
            account            : None,
            routing            : routing,
            _raii_joiner       : raii_joiner,
            message_queue      : message_queue,
            response_notifier  : notifier,
            session_packet_id  : None,
            session_packet_keys: None,
        }
    }

    /// This is one of the two Gateway functions to the Maidsafe network, the other being the
    /// log_in. This will help create a fresh account for the user in the SAFE-network.
    pub fn create_account(keyword: String, pin: String, password: String) -> Result<Client, ::errors::ClientError> {
        debug!("Creating account for supplied credentials ...");

        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(None), ::std::sync::Condvar::new()));
        // TODO stabilise this design
        let bootstrap_notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(false), ::std::sync::Condvar::new()));
        let account_packet = user_account::Account::new(None, None);
        let id_packet = ::routing::id::Id::with_keys((account_packet.get_maid().public_keys().0.clone(),
                                                      account_packet.get_maid().secret_keys().0.clone()),
                                                     (account_packet.get_maid().public_keys().1.clone(),
                                                      account_packet.get_maid().secret_keys().1.clone()));
        let (sender, receiver) = ::std::sync::mpsc::channel();

        let routing = Client::get_new_routing(sender, Some(id_packet));
        let (message_queue, raii_joiner) = message_queue::MessageQueue::new(notifier.clone(), bootstrap_notifier.clone(), receiver);

        // TODO stabilise this design
        {
            debug!("Bootstrapping ...");
            let (ref lock, ref condition_var) = *bootstrap_notifier;
            let mut mutex_guard = eval_result!(lock.lock());
            while !*mutex_guard {
                mutex_guard = eval_result!(condition_var.wait(mutex_guard));
            }
            debug!("Bootstrapped");
        }

        let client = Client {
            account            : Some(account_packet),
            routing            : routing,
            _raii_joiner       : raii_joiner,
            message_queue      : message_queue,
            response_notifier  : notifier,
            session_packet_id  : Some(try!(user_account::Account::generate_network_id(keyword.as_bytes(), pin.as_bytes()))),
            session_packet_keys: Some(SessionPacketEncryptionKeys::new(password, pin)),
        };

        {
            let account = try!(client.account.iter().next().ok_or(::errors::ClientError::from("Logic Error !! Report as bug.")));
            let session_packet_keys = try!(client.session_packet_keys.iter().next().ok_or(::errors::ClientError::from("Logic Error !! Report as bug.")));

            let account_version = try!(::routing::structured_data::StructuredData::new(LOGIN_PACKET_TYPE_TAG,
                                                                                       try!(client.session_packet_id.ok_or(::errors::ClientError::from("Logic Error !! Report as bug."))).clone(),
                                                                                       0,
                                                                                       try!(account.encrypt(session_packet_keys.get_password(),
                                                                                                            session_packet_keys.get_pin())),
                                                                                       vec![account.get_public_maid().public_keys().0.clone()],
                                                                                       Vec::new(),
                                                                                       Some(&account.get_maid().secret_keys().0)));
            client.put(::routing::data::Data::StructuredData(account_version), None);
        }

        Ok(client)
    }

    /// This is one of the two Gateway functions to the Maidsafe network, the other being the
    /// create_account. This will help log into an already created account for the user in the
    /// SAFE-network.
    pub fn log_in(keyword: String, pin: String, password: String) -> Result<Client, ::errors::ClientError> {
        debug!("Loging into account with supplied credentials ...");

        let mut unregistered_client = Client::create_unregistered_client();
        let user_id = try!(user_account::Account::generate_network_id(keyword.as_bytes(), pin.as_bytes()));

        let session_packet_request = ::routing::data::DataRequest::StructuredData(user_id.clone(), LOGIN_PACKET_TYPE_TAG);

        debug!("Fetch Session Packet using unregistered client ...");
        let response_getter = unregistered_client.get(session_packet_request, None);

        if let ::routing::data::Data::StructuredData(session_packet) = try!(response_getter.get()) {
            debug!("Session Packet fetched. Using Id's from fetched packet to construct a registered client ...");

            let decrypted_session_packet = try!(user_account::Account::decrypt(session_packet.get_data(), password.as_bytes(), pin.as_bytes()));
            let id_packet = ::routing::id::Id::with_keys((decrypted_session_packet.get_maid().public_keys().0.clone(),
                                                          decrypted_session_packet.get_maid().secret_keys().0.clone()),
                                                         (decrypted_session_packet.get_maid().public_keys().1.clone(),
                                                          decrypted_session_packet.get_maid().secret_keys().1.clone()));
            let (sender, receiver) = ::std::sync::mpsc::channel();

            let routing = Client::get_new_routing(sender, Some(id_packet));
            let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(None), ::std::sync::Condvar::new()));
            // TODO stabilise this design
            let bootstrap_notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(false), ::std::sync::Condvar::new()));
            let (message_queue, raii_joiner) = message_queue::MessageQueue::new(notifier.clone(), bootstrap_notifier.clone(), receiver);

            // TODO stabilise this design
            {
                debug!("Bootstrapping ...");
                let (ref lock, ref condition_var) = *bootstrap_notifier;
                let mut mutex_guard = eval_result!(lock.lock());
                while !*mutex_guard {
                    mutex_guard = eval_result!(condition_var.wait(mutex_guard));
                }
                debug!("Bootstrapped");
            }

            let client = Client {
                account            : Some(decrypted_session_packet),
                routing            : routing,
                _raii_joiner       : raii_joiner,
                message_queue      : message_queue,
                response_notifier  : notifier,
                session_packet_id  : Some(try!(user_account::Account::generate_network_id(keyword.as_bytes(), pin.as_bytes()))),
                session_packet_keys: Some(SessionPacketEncryptionKeys::new(password, pin)),
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
        if try!(self.account.iter_mut().next().ok_or(::errors::ClientError::OperationForbiddenForClient)).set_user_root_dir_id(root_dir_id) {
            self.update_session_packet()
        } else {
            Err(::errors::ClientError::RootDirectoryAlreadyExists)
        }
    }

    /// Get User's Root Directory ID if available in session packet used for current login
    pub fn get_user_root_directory_id(&self) -> Option<&::routing::NameType> {
        self.account.iter().next().and_then(|account| account.get_user_root_dir_id())
    }

    /// Create an entry for the Maidsafe configuration specific Root Directory ID into the
    /// session packet, encrypt and store it. It will be retireved when the user logs into
    /// his account. Root directory ID is necessary to fetch all of configuration data as all further
    /// data is encoded as meta-information into the config Root Directory or one of its subdirectories.
    pub fn set_configuration_root_directory_id(&mut self, root_dir_id: ::routing::NameType) -> Result<(), ::errors::ClientError> {
        if try!(self.account.iter_mut().next().ok_or(::errors::ClientError::OperationForbiddenForClient)).set_maidsafe_config_root_dir_id(root_dir_id) {
            self.update_session_packet()
        } else {
            Err(::errors::ClientError::RootDirectoryAlreadyExists)
        }
    }

    /// Get Maidsafe specific configuration's Root Directory ID if available in session packet used
    /// for current login
    pub fn get_configuration_root_directory_id(&self) -> Option<&::routing::NameType> {
        self.account.iter().next().and_then(|account| account.get_maidsafe_config_root_dir_id())
    }

    /// Combined Asymmectric and Symmetric encryption. The data is encrypted using random Key and
    /// IV with Xsalsa-symmetric encryption. Random IV ensures that same plain text produces different
    /// cipher-texts for each fresh symmetric encryption. The Key and IV are then asymmetrically
    /// enrypted using Public-MAID and the whole thing is then serialised into a single Vec<u8>.
    pub fn hybrid_encrypt(&self,
                          data_to_encrypt: &[u8],
                          nonce_opt: Option<&::sodiumoxide::crypto::box_::Nonce>) -> Result<Vec<u8>, ::errors::ClientError> {
        let account = try!(self.account.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient));

        let mut nonce_default = ::sodiumoxide::crypto::box_::Nonce([0u8; ::sodiumoxide::crypto::box_::NONCEBYTES]);
        let nonce = match nonce_opt {
            Some(nonce) => nonce,
            None => {
                let digest = ::sodiumoxide::crypto::hash::sha256::hash(&account.get_public_maid().name().0);
                let min_length = ::std::cmp::min(::sodiumoxide::crypto::box_::NONCEBYTES, digest.0.len());
                for it in digest.0.iter().take(min_length).enumerate() {
                    nonce_default.0[it.0] = *it.1;
                }
                &nonce_default
            },
        };

        ::utility::hybrid_encrypt(data_to_encrypt,
                                  &nonce,
                                  &account.get_public_maid().public_keys().1,
                                  &account.get_maid().secret_keys().1)
    }

    /// Reverse of hybrid_encrypt. Refer hybrid_encrypt.
    pub fn hybrid_decrypt(&self,
                          data_to_decrypt: &[u8],
                          nonce_opt: Option<&::sodiumoxide::crypto::box_::Nonce>) -> Result<Vec<u8>, ::errors::ClientError> {
        let account = try!(self.account.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient));

        let mut nonce_default = ::sodiumoxide::crypto::box_::Nonce([0u8; ::sodiumoxide::crypto::box_::NONCEBYTES]);
        let nonce = match nonce_opt {
            Some(nonce) => nonce,
            None => {
                let digest = ::sodiumoxide::crypto::hash::sha256::hash(&account.get_public_maid().name().0);
                let min_length = ::std::cmp::min(::sodiumoxide::crypto::box_::NONCEBYTES, digest.0.len());
                for it in digest.0.iter().take(min_length).enumerate() {
                    nonce_default.0[it.0] = *it.1;
                }
                &nonce_default
            },
        };

        ::utility::hybrid_decrypt(data_to_decrypt,
                                  &nonce,
                                  &account.get_public_maid().public_keys().1,
                                  &account.get_maid().secret_keys().1)
    }

    /// Get data from the network. This is non-blocking.
    pub fn get(&mut self,
               request_for : ::routing::data::DataRequest,
               opt_location: Option<::routing::authority::Authority>) -> response_getter::ResponseGetter {
        if let ::routing::data::DataRequest::ImmutableData(..) = request_for {
            let mut msg_queue = eval_result!(self.message_queue.lock());
            if msg_queue.local_cache_check(&request_for.name()) {
                return response_getter::ResponseGetter::new(None, self.message_queue.clone(), request_for)
            }
        }

        let location = match opt_location {
            Some(auth) => auth,
            None       => ::routing::authority::Authority::NaeManager(request_for.name()),
        };

        self.routing.get_request(location, request_for.clone());
        debug!("GET request posted to the network.");

        response_getter::ResponseGetter::new(Some(self.response_notifier.clone()), self.message_queue.clone(), request_for)
    }

    /// Put data onto the network. This is non-blocking.
    pub fn put(&self,
               data        : ::routing::data::Data,
               opt_location: Option<::routing::authority::Authority>) {
        let location = match opt_location {
            Some(auth) => auth,
            None       => ::routing::authority::Authority::ClientManager(data.name()),
        };

        debug!("Posting PUT request to the network ...");
        self.routing.put_request(location, data)
    }

    /// Post data onto the network
    pub fn post(&self,
                data        : ::routing::data::Data,
                opt_location: Option<::routing::authority::Authority>) {
        let location = match opt_location {
            Some(auth) => auth,
            None       => ::routing::authority::Authority::NaeManager(data.name()),
        };

        debug!("Posting POST request to the network ...");
        self.routing.post_request(location, data)
    }

    /// Delete data from the network
    pub fn delete(&self,
                  data        : ::routing::data::Data,
                  opt_location: Option<::routing::authority::Authority>) {
        let location = match opt_location {
            Some(auth) => auth,
            None       => ::routing::authority::Authority::ClientManager(data.name()),
        };

        debug!("Posting DELETE request to the network ...");
        self.routing.delete_request(location, data)
    }

    /// Returns the public encryption key
    pub fn get_public_encryption_key(&self) -> Result<&::sodiumoxide::crypto::box_::PublicKey, ::errors::ClientError> {
        let account = try!(self.account.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient));
        Ok(&account.get_maid().public_keys().1)
    }

    /// Returns the Secret encryption key
    pub fn get_secret_encryption_key(&self) -> Result<&::sodiumoxide::crypto::box_::SecretKey, ::errors::ClientError> {
        let account = try!(self.account.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient));
        Ok(&account.get_maid().secret_keys().1)
    }

    /// Returns the Public Signing key
    pub fn get_public_signing_key(&self) -> Result<&::sodiumoxide::crypto::sign::PublicKey, ::errors::ClientError> {
        let account = try!(self.account.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient));
        Ok(&account.get_maid().public_keys().0)
    }

    /// Returns the Secret Signing key
    pub fn get_secret_signing_key(&self) -> Result<&::sodiumoxide::crypto::sign::SecretKey, ::errors::ClientError> {
        let account = try!(self.account.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient));
        Ok(&account.get_maid().secret_keys().0)
    }

    fn get_new_routing(sender   : ::std::sync::mpsc::Sender<::routing::event::Event>,
                       id_packet: Option<::routing::id::Id>) -> Routing {
        Routing::new(sender, id_packet)
    }

    fn update_session_packet(&mut self) -> Result<(), ::errors::ClientError> {
        let session_packet_id = try!(self.session_packet_id.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient)).clone();
        let session_packet_request = ::routing::data::DataRequest::StructuredData(session_packet_id.clone(), LOGIN_PACKET_TYPE_TAG);

        let response_getter = self.get(session_packet_request, None);

        let account = try!(self.account.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient));
        let session_packet_keys = try!(self.session_packet_keys.iter().next().ok_or(::errors::ClientError::OperationForbiddenForClient));

        if let ::routing::data::Data::StructuredData(retrieved_session_packet) = try!(response_getter.get()) {
            let encrypted_account = try!(account.encrypt(session_packet_keys.get_password(),
                                                         session_packet_keys.get_pin()));

            let new_account_version = try!(::routing::structured_data::StructuredData::new(LOGIN_PACKET_TYPE_TAG,
                                                                                           session_packet_id,
                                                                                           retrieved_session_packet.get_version() + 1,
                                                                                           encrypted_account,
                                                                                           vec![account.get_public_maid().public_keys().0.clone()],
                                                                                           Vec::new(),
                                                                                           Some(&account.get_maid().secret_keys().0)));
            Ok(self.post(::routing::data::Data::StructuredData(new_account_version), None))
        } else {
            Err(::errors::ClientError::ReceivedUnexpectedData)
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        debug!("Client Destructor entered.");
        // Important, otherwise will be at the mercy of order of construction, ie., routing being
        // before _raii_joiner etc.
        self.routing.stop();
    }
}

/////////////////////////////////////////////////////////////////
/// Helper Struct
/////////////////////////////////////////////////////////////////

struct SessionPacketEncryptionKeys {
    pin     : Vec<u8>,
    password: Vec<u8>,
}

impl SessionPacketEncryptionKeys {
    fn new(password: String, pin: String) -> SessionPacketEncryptionKeys {
        SessionPacketEncryptionKeys {
            pin     : pin.into_bytes(),
            password: password.into_bytes(),
        }
    }

    fn get_password(&self) -> &[u8] {
        &self.password[..]
    }

    fn get_pin(&self) -> &[u8] {
        &self.pin[..]
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
        let pin = eval_result!(::utility::generate_random_string(10));
        let _ = eval_result!(Client::create_account(keyword, pin, password));
    }

    #[test]
    fn account_login() {
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = eval_result!(::utility::generate_random_string(10));

        // Creation should pass
        let _ = eval_result!(Client::create_account(keyword.clone(), pin.clone(), password.clone()));

        // Correct Credentials - Login Should Pass
        let _ = eval_result!(Client::log_in(keyword, pin, password));
    }

    #[test]
    fn unregistered_client() {
        let immut_data = ::routing::immutable_data::ImmutableData::new(::routing::immutable_data::ImmutableDataType::Normal,
                                                                       eval_result!(::utility::generate_random_vector(30)));
        let orig_data = ::routing::data::Data::ImmutableData(immut_data);

        // Registered Client PUTs something onto the network
        {
            let keyword = eval_result!(::utility::generate_random_string(10));
            let password = eval_result!(::utility::generate_random_string(10));
            let pin = eval_result!(::utility::generate_random_string(10));

            // Creation should pass
            let client = eval_result!(Client::create_account(keyword, pin, password));
            client.put(orig_data.clone(), None);
        }

        // Unregistered Client should be able to retrieve the data
        let mut unregistered_client = Client::create_unregistered_client();
        let request = ::routing::data::DataRequest::ImmutableData(orig_data.name(),
                                                                  ::routing::immutable_data::ImmutableDataType::Normal);
        let rxd_data = eval_result!(unregistered_client.get(request, None).get());

        assert_eq!(rxd_data, orig_data);

        // Operations Not Allowed for Unregistered Client
        let rand_name = ::routing::NameType::new(eval_result!(::utility::generate_random_array_u8_64()));

        match (unregistered_client.set_user_root_directory_id(rand_name.clone()),
               unregistered_client.set_configuration_root_directory_id(rand_name)) {
            (Err(::errors::ClientError::OperationForbiddenForClient),
             Err(::errors::ClientError::OperationForbiddenForClient)) => (),
            _ => panic!("Unexpected !!"),
        };
    }

    #[test]
    fn user_root_dir_id_creation() {
        // Construct Client
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = eval_result!(::utility::generate_random_string(10));

        let mut client = eval_result!(Client::create_account(keyword.clone(), pin.clone(), password.clone()));

        assert!(client.get_user_root_directory_id().is_none());
        assert!(client.get_configuration_root_directory_id().is_none());

        let root_dir_id = ::routing::NameType::new([99u8; 64]);
        eval_result!(client.set_user_root_directory_id(root_dir_id.clone()));

        // Correct Credentials - Login Should Pass
        let client = eval_result!(Client::log_in(keyword, pin, password));

        assert!(client.get_user_root_directory_id().is_some());
        assert!(client.get_configuration_root_directory_id().is_none());

        assert_eq!(client.get_user_root_directory_id(), Some(&root_dir_id));
    }

    #[test]
    fn maidsafe_config_root_dir_id_creation() {
        // Construct Client
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = eval_result!(::utility::generate_random_string(10));

        let mut client = eval_result!(Client::create_account(keyword.clone(), pin.clone(), password.clone()));

        assert!(client.get_user_root_directory_id().is_none());
        assert!(client.get_configuration_root_directory_id().is_none());

        let root_dir_id = ::routing::NameType::new([99u8; 64]);
        eval_result!(client.set_configuration_root_directory_id(root_dir_id.clone()));

        // Correct Credentials - Login Should Pass
        let client = eval_result!(Client::log_in(keyword, pin, password));

        assert!(client.get_user_root_directory_id().is_none());
        assert!(client.get_configuration_root_directory_id().is_some());

        assert_eq!(client.get_configuration_root_directory_id(), Some(&root_dir_id));
    }

    #[test]
    fn hybrid_encryption_decryption() {
        // Construct Client
        let keyword = eval_result!(::utility::generate_random_string(10));
        let password = eval_result!(::utility::generate_random_string(10));
        let pin = eval_result!(::utility::generate_random_string(10));

        let client = eval_result!(Client::create_account(keyword, pin, password));

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
