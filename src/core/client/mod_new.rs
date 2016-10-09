// Copyright 2016 MaidSafe.net limited.
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

#[cfg(feature = "use-mock-routing")]
mod non_networking_test_framework;
mod routing_el;
mod user_account;

use core::{CoreError, CoreEvent, CoreMsgTx, HeadFuture, utility};
use core::translated_events::NetworkEvent;
use maidsafe_utilities::thread::{self, Joiner};
use routing::{AppendWrapper, Authority, Data, DataIdentifier, FullId, MessageId, StructuredData,
              XorName};
#[cfg(not(feature = "use-mock-routing"))]
use routing::Client as Routing;
use routing::TYPE_TAG_SESSION_PACKET;
use routing::client_errors::MutationError;
use rust_sodium::crypto::{box_, sign};
use rust_sodium::crypto::hash::sha256;
#[cfg(feature = "use-mock-routing")]
use self::non_networking_test_framework::RoutingMock as Routing;
use self::user_account::Account;
use std::collections::HashMap;
use std::time::Duration;

pub type ReturnType = Future<Item = CoreEvent, Error = CoreError>;

const CONNECTION_TIMEOUT_SECS: u64 = 60;

/// The main self-authentication client instance that will interface all the request from high
/// level API's to the actual routing layer and manage all interactions with it. This is
/// essentially a non-blocking Client with upper layers having an option to either block and wait
/// on the returned ResponseGetters for receiving network response or spawn a new thread. The Client
/// itself is however well equipped for parallel and non-blocking PUTs and GETS.
pub struct Client {
    routing: Routing,
    heads: HashMap<MessageId, HeadFuture>,
    account: Option<Account>,
    session_packet_id: Option<XorName>,
    session_pkt_keys: Option<SessionPacketEncryptionKeys>,
    client_manager_addr: Option<XorName>,
    stats: Stats,
    _joiner: Joiner,
}

impl Client {
    /// This is a getter-only Gateway function to the Maidsafe network. It will create an
    /// unregistered random client, which can do very limited set of operations - eg., a
    /// Network-Get
    pub fn create_unregistered_client(core_tx: CoreMsgTx) -> Result<Client, CoreError> {
        trace!("Creating unregistered client.");

        let (routing_tx, routing_rx) = mpsc::channel();
        let routing = try!(Routing::new(routing_tx, None));

        trace!("Waiting to get connected to the Network...");
        match routing_rx.recv_timeout(Duration::secs(CONNECTION_TIMEOUT_SECS)) {
            Ok(NetworkEvent::Connected) => (),
            x => {
                warn!("Could not connect to the Network. Unexpected: {:?}", x);
                return Err(CoreError::OperationAborted);
            }
        }
        trace!("Connected to the Network.");

        let joiner = thread::named("Routing Event Loop",
                                   move || routing_el::run(routing_rx, core_tx));

        Ok(Client {
            routing: routing,
            heads: HashMap::with_capacity(10),
            account: None,
            session_packet_id: None,
            session_pkt_keys: None,
            client_manager_addr: None,
            stats: Default::default(),
            _joiner: joiner,
        })
    }

    /// This is one of the two main Gateway functions to the Maidsafe network, the other being the
    /// log_in. This will help create a fresh account for the user in the SAFE-network.
    pub fn create_account(acc_locator: &str,
                          acc_password: &str,
                          core_tx: CoreMsgTx)
                          -> Result<Client, CoreError> {
        trace!("Creating an account.");

        let (password, keyword, pin) = utility::derive_secrets(acc_locator, acc_password);

        let account = Account::new(None, None);
        let id_packet = FullId::with_keys((account.get_maid().public_keys().1,
                                           account.get_maid().secret_keys().1.clone()),
                                          (account.get_maid().public_keys().0,
                                           account.get_maid().secret_keys().0.clone()));

        let (routing_tx, routing_rx) = mpsc::channel();
        let routing = try!(Routing::new(routing_tx, Some(id_packet)));

        trace!("Waiting to get connected to the Network...");
        match routing_rx.recv_timeout(Duration::secs(CONNECTION_TIMEOUT_SECS)) {
            Ok(NetworkEvent::Connected) => (),
            x => {
                warn!("Could not connect to the Network. Unexpected: {:?}", x);
                return Err(CoreError::OperationAborted);
            }
        }
        trace!("Connected to the Network.");

        let session_pkt_id = try!(Account::generate_network_id(&keyword, &pin));
        let session_pkt_keys = SessionPacketEncryptionKeys::new(password, pin);
        let account_sd = try!(StructuredData::new(TYPE_TAG_SESSION_PACKET,
                                         *session_packet_id,
                                         0,
                                         try!(account.encrypt(session_pkt_keys.get_password(),
                                                              session_pkt_keys.get_pin())),
                                         vec![account.get_public_maid().public_keys().0.clone()],
                                         Vec::new(),
                                         Some(&account.get_maid().secret_keys().0)));

        let Digest(digest) = sha256::hash(&(account.get_maid().public_keys().0).0);
        let client_manager_addr = XorName(digest);

        let dst = Authority::ClientManager(client_manager_addr);
        let msg_id = MessageId::new();
        try!(routing.send_put_request(dst, Data::Structured(account_sd), msg_id));
        match routing_rx.recv_timeout(Duration::secs(CONNECTION_TIMEOUT_SECS)) {
            Ok(Event::Response { response: Response::PutSuccess(_, id), .. }) => (),
            x => {
                warn!("Could not put session packet to the Network. Unexpected: {:?}",
                      x);
                return Err(CoreError::OperationAborted);
            }
        }

        let joiner = thread::named("Routing Event Loop",
                                   move || routing_el::run(routing_rx, core_tx));

        Ok(Client {
            routing: routing,
            heads: HashMap::with_capacity(10),
            account: Some(account),
            session_packet_id: Some(session_pkt_id),
            session_pkt_keys: Some(session_pkt_keys),
            client_manager_addr: Some(client_manager_addr),
            stats: Default::default(),
            _joiner: joiner,
        })
    }

    /// Remove `head` of the future chain, the `tail` of which is probaly being processed in event
    /// loop.
    pub fn remove_head(&mut self, id: &MessageId) -> Option<HeadFuture> {
        self.heads.remove(id)
    }

    // TODO All these return the same future from all branches. So convert to impl Trait when it
    // arrives in stable. Change from `Box<ReturnType>` -> `impl ReturnType`.
    /// Get data from the network.
    pub fn get(&mut self, data_id: DataIdentifier, opt_dst: Option<Authority>) -> Box<ReturnType> {
        trace!("GET for {:?}", data_id);
        self.stats.issued_gets += 1;

        let (head, rx) = futures::oneshot();
        rx.map_err(|e| CoreError::OperationAborted);

        if let DataIdentifier::Immutable(..) = data_id {
            if let Some(data) = self.cache.borrow().get(data_id.name()) {
                trace!("ImmutableData found in cache.");
                head.complete(CoreEvent::Get(Ok(data)));
                return rx;
            }
        }

        let dst = match opt_dst {
            Some(auth) => auth,
            None => Authority::NaeManager(*data_id.name()),
        };

        let msg_id = MessageId::new();
        if let Err(e) = self.routing.send_get_request(dst, data_id, msg_id) {
            head.complete(CoreEvent::Get(Err(From::from(e))));
        } else {
            let _ = self.heads.insert(msg_id, head);
        }

        rx
    }

    /// Put data onto the network.
    pub fn put(&mut self, data: Data, opt_dst: Option<Authority>) -> Box<ReturnType> {
        trace!("PUT for {:?}", data);
        self.issued_puts += 1;

        let (head, rx) = futures::oneshot();
        rx.map_err(|e| CoreError::OperationAborted);

        let dst = match opt_dst {
            Some(auth) => auth,
            None => {
                let cm_addr = match self.cm_addr() {
                    Ok(addr) => addr,
                    Err(e) => {
                        head.complete(CoreEvent::Mutation(Err(e)));
                        return rx;
                    }
                };
                Authority::ClientManager(try!(self.cm_addr()))
            }
        };

        let msg_id = MessageId::new();
        if let Err(e) = self.routing.send_put_request(dst, data, msg_id) {
            head.complete(CoreEvent::Get(Err(From::from(e))));
        } else {
            let _ = self.heads.insert(msg_id, head);
        }

        rx
    }

    /// Return the amount of calls that were done to `get`
    pub fn issued_gets(&self) -> u64 {
        self.stats.issued_gets
    }

    /// Return the amount of calls that were done to `put`
    pub fn issued_puts(&self) -> u64 {
        self.stats.issued_puts
    }

    /// Return the amount of calls that were done to `post`
    pub fn issued_posts(&self) -> u64 {
        self.stats.issued_posts
    }

    /// Return the amount of calls that were done to `delete`
    pub fn issued_deletes(&self) -> u64 {
        self.stats.issued_deletes
    }

    /// Return the amount of calls that were done to `append`
    pub fn issued_appends(&self) -> u64 {
        self.stats.issued_appends
    }

    /// Get the default address where the PUTs will go to for this client
    pub fn cm_addr(&self) -> Result<XorName, CoreError> {
        self.client_manager_addr.ok_or(CoreError::OperationForbiddenForClient)
    }

    #[cfg(all(test, feature = "use-mock-routing"))]
    pub fn set_network_limits(&mut self, max_ops_count: Option<u64>) {
        self.routing.set_network_limits(max_ops_count);
    }
}

// ------------------------------------------------------------
// Helper Struct
// ------------------------------------------------------------

struct SessionPacketEncryptionKeys {
    pin: Vec<u8>,
    password: Vec<u8>,
}

impl SessionPacketEncryptionKeys {
    fn new(password: Vec<u8>, pin: Vec<u8>) -> SessionPacketEncryptionKeys {
        SessionPacketEncryptionKeys {
            pin: pin,
            password: password,
        }
    }

    fn get_password(&self) -> &[u8] {
        &self.password[..]
    }

    fn get_pin(&self) -> &[u8] {
        &self.pin[..]
    }
}

struct Stats {
    issued_gets: u64,
    issued_puts: u64,
    issued_posts: u64,
    issued_deletes: u64,
    issued_appends: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            issued_gets: 0,
            issued_puts: 0,
            issued_posts: 0,
            issued_deletes: 0,
            issued_appends: 0,
        }
    }
}

// ------------------------------------------------------------
