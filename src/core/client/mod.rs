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

mod account;
#[cfg(feature = "use-mock-routing")]
mod mock_routing;
mod routing_el;

use core::{CoreError, CoreEvent, CoreMsgTx, HeadFuture, utility};
use futures::{self, Future};
use lru_cache::LruCache;
use maidsafe_utilities::thread::{self, Joiner};
use routing::{Authority, Data, DataIdentifier, Event, FullId, MessageId, Response, StructuredData,
              TYPE_TAG_SESSION_PACKET, XorName};
#[cfg(not(feature = "use-mock-routing"))]
use routing::Client as Routing;
use rust_sodium::crypto::hash::sha256::{self, Digest};
use self::account::Account;
#[cfg(feature = "use-mock-routing")]
use self::mock_routing::MockRouting as Routing;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

pub type ReturnType = Future<Item = CoreEvent, Error = CoreError>;

const CONNECTION_TIMEOUT_SECS: u64 = 60;
const ACC_PKT_TIMEOUT_SECS: u64 = 60;
const IMMUT_DATA_CACHE_SIZE: usize = 300;

/// The main self-authentication client instance that will interface all the request from high
/// level API's to the actual routing layer and manage all interactions with it. This is
/// essentially a non-blocking Client with upper layers having an option to either block and wait
/// on the returned ResponseGetters for receiving network response or spawn a new thread. The Client
/// itself is however well equipped for parallel and non-blocking PUTs and GETS.
pub struct Client {
    routing: Routing,
    heads: HashMap<MessageId, HeadFuture>,
    cache: Rc<RefCell<LruCache<XorName, Data>>>,
    client_type: ClientType,
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
        match routing_rx.recv_timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS)) {
            Ok(Event::Connected) => (),
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
            cache: Rc::new(RefCell::new(LruCache::new(IMMUT_DATA_CACHE_SIZE))),
            client_type: ClientType::Unregistered,
            stats: Default::default(),
            _joiner: joiner,
        })
    }

    /// This is a Gateway function to the Maidsafe network. This will help create a fresh acc for
    /// the user in the SAFE-network.
    pub fn create_account(acc_locator: &str,
                          acc_password: &str,
                          core_tx: CoreMsgTx)
                          -> Result<Client, CoreError> {
        trace!("Creating an acc.");

        let (password, keyword, pin) = utility::derive_secrets(acc_locator, acc_password);

        let acc = Account::new(None, None);
        let id_packet = FullId::with_keys((acc.get_maid().public_keys().1,
                                           acc.get_maid().secret_keys().1.clone()),
                                          (acc.get_maid().public_keys().0,
                                           acc.get_maid().secret_keys().0.clone()));

        let (routing_tx, routing_rx) = mpsc::channel();
        let routing = try!(Routing::new(routing_tx, Some(id_packet)));

        trace!("Waiting to get connected to the Network...");
        match routing_rx.recv_timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS)) {
            Ok(Event::Connected) => (),
            x => {
                warn!("Could not connect to the Network. Unexpected: {:?}", x);
                return Err(CoreError::OperationAborted);
            }
        }
        trace!("Connected to the Network.");

        let acc_loc = try!(Account::generate_network_id(&keyword, &pin));
        let user_cred = UserCred::new(password, pin);
        let acc_sd = try!(StructuredData::new(TYPE_TAG_SESSION_PACKET,
                                              acc_loc,
                                              0,
                                              try!(acc.encrypt(&user_cred.password,
                                                               &user_cred.pin)),
                                              vec![acc.get_public_maid().public_keys().0.clone()],
                                              Vec::new(),
                                              Some(&acc.get_maid().secret_keys().0)));

        let Digest(digest) = sha256::hash(&(acc.get_maid().public_keys().0).0);
        let cm_addr = Authority::ClientManager(XorName(digest));

        let msg_id = MessageId::new();
        try!(routing.send_put_request(cm_addr.clone(), Data::Structured(acc_sd), msg_id));
        match routing_rx.recv_timeout(Duration::from_secs(ACC_PKT_TIMEOUT_SECS)) {
            Ok(Event::Response { response: Response::PutSuccess(_, id), .. }) if id == msg_id => (),
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
            cache: Rc::new(RefCell::new(LruCache::new(IMMUT_DATA_CACHE_SIZE))),
            client_type: ClientType::reg(acc, acc_loc, user_cred, cm_addr),
            stats: Default::default(),
            _joiner: joiner,
        })
    }

    /// This is a Gateway function to the Maidsafe network. This will help login to an already
    /// existing account of the user in the SAFE-network.
    pub fn log_in(acc_locator: &str,
                  acc_password: &str,
                  core_tx: CoreMsgTx)
                  -> Result<Client, CoreError> {
        trace!("Attempting to log into an acc.");

        let (password, keyword, pin) = utility::derive_secrets(acc_locator, acc_password);

        let acc_loc = try!(Account::generate_network_id(&keyword, &pin));
        let user_cred = UserCred::new(password, pin);
        let acc_sd_id = DataIdentifier::Structured(acc_loc, TYPE_TAG_SESSION_PACKET);

        let msg_id = MessageId::new();
        let dst = Authority::NaeManager(*acc_sd_id.name());

        let acc_sd = {
            trace!("Creating throw-away routing getter for account packet.");
            let (routing_tx, routing_rx) = mpsc::channel();
            let mut routing = try!(Routing::new(routing_tx, None));

            trace!("Waiting to get connected to the Network...");
            match routing_rx.recv_timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS)) {
                Ok(Event::Connected) => (),
                x => {
                    warn!("Could not connect to the Network. Unexpected: {:?}", x);
                    return Err(CoreError::OperationAborted);
                }
            }
            trace!("Connected to the Network.");

            try!(routing.send_get_request(dst, acc_sd_id, msg_id));
            match routing_rx.recv_timeout(Duration::from_secs(ACC_PKT_TIMEOUT_SECS)) {
                Ok(Event::Response { response:
                    Response::GetSuccess(Data::Structured(data), id), .. }) => {
                    if id == msg_id {
                        data
                    } else {
                        return Err(CoreError::OperationAborted);
                    }
                }
                x => {
                    warn!("Could not fetch account packet from the Network. Unexpected: {:?}",
                          x);
                    return Err(CoreError::OperationAborted);
                }
            }
        };

        let acc = try!(Account::decrypt(acc_sd.get_data(), &user_cred.password, &user_cred.pin));
        let id_packet = FullId::with_keys((acc.get_maid().public_keys().1,
                                           acc.get_maid().secret_keys().1.clone()),
                                          (acc.get_maid().public_keys().0,
                                           acc.get_maid().secret_keys().0.clone()));

        let Digest(digest) = sha256::hash(&(acc.get_maid().public_keys().0).0);
        let cm_addr = Authority::ClientManager(XorName(digest));

        trace!("Creating an actual routing...");
        let (routing_tx, routing_rx) = mpsc::channel();
        let routing = try!(Routing::new(routing_tx, Some(id_packet)));

        trace!("Waiting to get connected to the Network...");
        match routing_rx.recv_timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS)) {
            Ok(Event::Connected) => (),
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
            cache: Rc::new(RefCell::new(LruCache::new(IMMUT_DATA_CACHE_SIZE))),
            client_type: ClientType::reg(acc, acc_loc, user_cred, cm_addr),
            stats: Default::default(),
            _joiner: joiner,
        })
    }

    /// Remove `head` of the future chain, the `tail` of which is probaly being processed in event
    /// loop.
    pub fn remove_head(&mut self, id: &MessageId) -> Option<HeadFuture> {
        self.heads.remove(id)
    }

    /// Get data from the network. If the data exists locally in the cache (for ImmutableData) then
    /// it will immediately be returned without making an actual network request.
    pub fn get(&mut self, data_id: DataIdentifier, opt_dst: Option<Authority>) -> Box<ReturnType> {
        trace!("GET for {:?}", data_id);
        self.stats.issued_gets += 1;

        let (head, oneshot) = futures::oneshot();
        let rx = oneshot.map_err(|e| CoreError::OperationAborted);

        let rx: Box<ReturnType> = if let DataIdentifier::Immutable(..) = data_id {
            if let Some(data) = self.cache.borrow_mut().get_mut(data_id.name()) {
                trace!("ImmutableData found in cache.");
                head.complete(CoreEvent::Get(Ok(data.clone())));
                return Box::new(rx);
            }

            let cache = self.cache.clone();
            Box::new(rx.map(move |event| {
                match event {
                    CoreEvent::Get(Ok(ref data @ Data::Immutable(_))) => {
                        let _ = cache.borrow_mut().insert(*data.name(), data.clone());
                    }
                    _ => (),
                }
                event
            }))
        } else {
            Box::new(rx)
        };

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

    // TODO All these return the same future from all branches. So convert to impl Trait when it
    // arrives in stable. Change from `Box<ReturnType>` -> `impl ReturnType`.
    /// Put data onto the network.
    pub fn put(&mut self, data: Data, opt_dst: Option<Authority>) -> Box<ReturnType> {
        trace!("PUT for {:?}", data);
        self.stats.issued_puts += 1;

        let (head, oneshot) = futures::oneshot();
        let rx = Box::new(oneshot.map_err(|_| CoreError::OperationAborted));

        let dst = match opt_dst {
            Some(auth) => auth,
            None => {
                match self.cm_addr() {
                    Ok(addr) => addr.clone(),
                    Err(e) => {
                        head.complete(CoreEvent::Mutation(Err(e)));
                        return rx;
                    }
                }
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

    /// Post data onto the network.
    pub fn post(&mut self, data: Data, opt_dst: Option<Authority>) -> Box<ReturnType> {
        trace!("Post for {:?}", data);
        self.stats.issued_posts += 1;

        let (head, oneshot) = futures::oneshot();
        let rx = Box::new(oneshot.map_err(|e| CoreError::OperationAborted));

        let dst = match opt_dst {
            Some(auth) => auth,
            None => Authority::NaeManager(*data.name()),
        };

        let msg_id = MessageId::new();
        if let Err(e) = self.routing.send_post_request(dst, data, msg_id) {
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
    pub fn cm_addr(&self) -> Result<&Authority, CoreError> {
        self.client_type.cm_addr()
    }

    #[cfg(all(test, feature = "use-mock-routing"))]
    pub fn set_network_limits(&mut self, max_ops_count: Option<u64>) {
        self.routing.set_network_limits(max_ops_count);
    }
}

// ------------------------------------------------------------
// Helper Struct
// ------------------------------------------------------------

struct UserCred {
    pin: Vec<u8>,
    password: Vec<u8>,
}

impl UserCred {
    fn new(password: Vec<u8>, pin: Vec<u8>) -> UserCred {
        UserCred {
            pin: pin,
            password: password,
        }
    }
}

enum ClientType {
    Unregistered,
    Registered {
        acc: Account,
        acc_loc: XorName,
        user_cred: UserCred,
        cm_addr: Authority,
    },
}

impl ClientType {
    fn reg(acc: Account, acc_loc: XorName, user_cred: UserCred, cm_addr: Authority) -> Self {
        ClientType::Registered {
            acc: acc,
            acc_loc: acc_loc,
            user_cred: user_cred,
            cm_addr: cm_addr,
        }
    }

    fn acc(&self) -> Result<&Account, CoreError> {
        match *self {
            ClientType::Registered { ref acc, .. } => Ok(acc),
            ClientType::Unregistered => Err(CoreError::OperationForbiddenForClient),
        }
    }

    fn acc_loc(&self) -> Result<XorName, CoreError> {
        match *self {
            ClientType::Registered { acc_loc, .. } => Ok(acc_loc),
            ClientType::Unregistered => Err(CoreError::OperationForbiddenForClient),
        }
    }

    fn user_cred(&self) -> Result<&UserCred, CoreError> {
        match *self {
            ClientType::Registered { ref user_cred, .. } => Ok(user_cred),
            ClientType::Unregistered => Err(CoreError::OperationForbiddenForClient),
        }
    }

    fn cm_addr(&self) -> Result<&Authority, CoreError> {
        match *self {
            ClientType::Registered { ref cm_addr, .. } => Ok(cm_addr),
            ClientType::Unregistered => Err(CoreError::OperationForbiddenForClient),
        }
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
