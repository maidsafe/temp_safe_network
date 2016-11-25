// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

mod account;
#[cfg(feature = "use-mock-routing")]
mod mock_routing;
mod routing_el;

use core::{CoreError, CoreEvent, CoreFuture, CoreMsg, CoreMsgTx, DIR_TAG, FutureExt, NetworkEvent,
           NetworkTx, utility};
use futures::{self, Complete, Future};
use lru_cache::LruCache;
use maidsafe_utilities::thread::{self, Joiner};
use routing::{Authority, EntryAction, Event, FullId, ImmutableData, MessageId, MutableData,
              Response, TYPE_TAG_SESSION_PACKET, Value, XorName};
#[cfg(not(feature = "use-mock-routing"))]
use routing::Client as Routing;
use rust_sodium::crypto::{box_, sign};
use rust_sodium::crypto::hash::sha256::{self, Digest};
pub use self::account::{ClientKeys, Dir};
use self::account::Account;
#[cfg(feature = "use-mock-routing")]
use self::mock_routing::MockRouting as Routing;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use tokio_core::reactor::{Handle, Timeout};

const ACC_PKT_TIMEOUT_SECS: u64 = 60;
const CONNECTION_TIMEOUT_SECS: u64 = 10;
const IMMUT_DATA_CACHE_SIZE: usize = 300;
const REQUEST_TIMEOUT_SECS: u64 = 120;

macro_rules! oneshot {
    ($client:expr, $event:path) => {{
        let msg_id = MessageId::new();
        let (hook, oneshot) = futures::oneshot();
        let fut = oneshot.map_err(|_| CoreError::OperationAborted)
            .and_then(|event| match event {
                $event(res) => res,
                _ => Err(CoreError::ReceivedUnexpectedEvent),
            });

        (hook, $client.timeout(msg_id, fut), msg_id)
    }}
}

/// The main self-authentication client instance that will interface all the
/// request from high level API's to the actual routing layer and manage all
/// interactions with it. This is essentially a non-blocking Client with upper
/// layers having an option to either block and wait on the returned
/// ResponseGetters for receiving network response or spawn a new thread. The
/// Client itself is however well equipped for parallel and non-blocking PUTs
/// and GETS.
#[derive(Clone)]
pub struct Client {
    inner: Rc<RefCell<Inner>>,
}

struct Inner {
    el_handle: Handle,
    routing: Routing,
    hooks: HashMap<MessageId, Complete<CoreEvent>>,
    cache: LruCache<XorName, ImmutableData>,
    client_type: ClientType,
    timeout: Duration,
    joiner: Joiner,
}

impl Client {
    /// This is a getter-only Gateway function to the Maidsafe network. It will
    /// create an unregistered random client, which can do very limited set of
    /// operations - eg., a Network-Get
    pub fn unregistered<T>(el_handle: Handle,
                           core_tx: CoreMsgTx<T>,
                           net_tx: NetworkTx)
                           -> Result<Self, CoreError>
        where T: 'static
    {
        trace!("Creating unregistered client.");

        let (routing, routing_rx) = setup_routing(None)?;
        let net_tx_clone = net_tx.clone();
        let core_tx_clone = core_tx.clone();
        let joiner = spawn_routing_thread(routing_rx, core_tx_clone, net_tx_clone);

        Ok(Self::new(Inner {
            el_handle: el_handle,
            routing: routing,
            hooks: HashMap::with_capacity(10),
            cache: LruCache::new(IMMUT_DATA_CACHE_SIZE),
            client_type: ClientType::Unregistered,
            timeout: Duration::from_secs(REQUEST_TIMEOUT_SECS),
            joiner: joiner,
        }))
    }

    /// Creates an empty dir to hold configuration or user data
    fn create_empty_dir(routing: &Routing,
                        routing_rx: &Receiver<Event>,
                        owners: BTreeSet<sign::PublicKey>,
                        requester: sign::PublicKey)
                        -> Result<Dir, CoreError> {
        let dir = Dir::random(DIR_TAG);
        let dir_md = MutableData::new(dir.name,
                                      dir.type_tag,
                                      BTreeMap::new(),
                                      BTreeMap::new(),
                                      owners)?;

        let msg_id = MessageId::new();
        routing.put_mdata(Authority::NaeManager(dir.name), dir_md, msg_id, requester)?;

        match routing_rx.recv_timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS)) {
            Ok(Event::Response { response: Response::PutMData { ref res, msg_id: ref id }, .. })
                if *id == msg_id => {
                match *res {
                    Ok(..) => (),
                    Err(ref client_error) => {
                        return Err(CoreError::RoutingClientError(client_error.clone()));
                    }
                }
            }
            x => {
                warn!("Could not put MutableData to the Network. Unexpected: {:?}",
                      x);
                return Err(CoreError::OperationAborted);
            }
        }

        Ok(dir)
    }

    /// This is a Gateway function to the Maidsafe network. This will help
    /// create a fresh acc for the user in the SAFE-network.
    pub fn registered<T>(acc_locator: &str,
                         acc_password: &str,
                         el_handle: Handle,
                         core_tx: CoreMsgTx<T>,
                         net_tx: NetworkTx)
                         -> Result<Client, CoreError>
        where T: 'static
    {
        trace!("Creating an acc.");

        let (password, keyword, pin) = utility::derive_secrets(acc_locator, acc_password);

        let acc_loc = Account::generate_network_id(&keyword, &pin)?;
        let user_cred = UserCred::new(password, pin);

        let maid_keys = ClientKeys::new();
        let pub_key = maid_keys.sign_pk.clone();
        let full_id = Some(maid_keys.clone().into());

        let mut owners = BTreeSet::new();
        owners.insert(pub_key.clone());

        let (routing, routing_rx) = setup_routing(full_id)?;

        let user_root = Client::create_empty_dir(&routing, &routing_rx, owners.clone(), pub_key)?;
        let config_dir = Client::create_empty_dir(&routing, &routing_rx, owners.clone(), pub_key)?;

        let acc = Account::new(maid_keys, user_root, config_dir);

        let mut acc_data = BTreeMap::new();
        let _ = acc_data.insert("Login".as_bytes().to_owned(),
                                Value {
                                    content: acc.encrypt(&user_cred.password, &user_cred.pin)?,
                                    entry_version: 0,
                                });

        let acc_md = MutableData::new(acc_loc,
                                      TYPE_TAG_SESSION_PACKET,
                                      BTreeMap::new(),
                                      acc_data,
                                      owners)?;

        let Digest(digest) = sha256::hash(&pub_key.0);
        let cm_addr = Authority::ClientManager(XorName(digest));

        let msg_id = MessageId::new();
        routing.put_mdata(cm_addr.clone(), acc_md, msg_id, pub_key.clone())?;

        match routing_rx.recv_timeout(Duration::from_secs(ACC_PKT_TIMEOUT_SECS)) {
            Ok(Event::Response { response: Response::PutMData { ref res, msg_id: ref id }, .. })
                if *id == msg_id => {
                match *res {
                    Ok(..) => (),
                    Err(ref client_error) => {
                        return Err(CoreError::RoutingClientError(client_error.clone()));
                    }
                }
            }
            x => {
                warn!("Could not put session packet to the Network. Unexpected: {:?}",
                      x);
                return Err(CoreError::OperationAborted);
            }
        }

        let net_tx_clone = net_tx.clone();
        let core_tx_clone = core_tx.clone();
        let joiner = spawn_routing_thread(routing_rx, core_tx_clone, net_tx_clone);

        Ok(Self::new(Inner {
            el_handle: el_handle,
            routing: routing,
            hooks: HashMap::with_capacity(10),
            cache: LruCache::new(IMMUT_DATA_CACHE_SIZE),
            client_type: ClientType::reg(acc, acc_loc, user_cred, cm_addr),
            timeout: Duration::from_secs(REQUEST_TIMEOUT_SECS),
            joiner: joiner,
        }))
    }

    /// This is a Gateway function to the Maidsafe network. This will help
    /// login to an already existing account of the user in the SAFE-network.
    pub fn login<T>(acc_locator: &str,
                    acc_password: &str,
                    el_handle: Handle,
                    core_tx: CoreMsgTx<T>,
                    net_tx: NetworkTx)
                    -> Result<Client, CoreError>
        where T: 'static
    {
        trace!("Attempting to log into an acc.");

        let (password, keyword, pin) = utility::derive_secrets(acc_locator, acc_password);

        let acc_loc = Account::generate_network_id(&keyword, &pin)?;
        let user_cred = UserCred::new(password, pin);

        let msg_id = MessageId::new();
        let dst = Authority::NaeManager(acc_loc);

        let acc_content = {
            trace!("Creating throw-away routing getter for account packet.");
            let (routing, routing_rx) = setup_routing(None)?;

            routing.get_mdata_value(dst,
                                 acc_loc,
                                 TYPE_TAG_SESSION_PACKET,
                                 "Login".as_bytes().to_owned(),
                                 msg_id)?;

            match routing_rx.recv_timeout(Duration::from_secs(ACC_PKT_TIMEOUT_SECS)) {
                Ok(Event::Response { response:
                                     Response::GetMDataValue { msg_id: id, res }, .. }) => {
                    if id != msg_id {
                        return Err(CoreError::OperationAborted);
                    }
                    match res {
                        Ok(Value { content, .. }) => {
                            content
                        },
                        Err(client_error) => {
                            return Err(CoreError::RoutingClientError(client_error));
                        }
                    }
                }
                x => {
                    warn!("Could not fetch account packet from the Network. Unexpected: {:?}",
                          x);
                    return Err(CoreError::OperationAborted);
                }
            }
        };

        let acc = Account::decrypt(&acc_content, &user_cred.password, &user_cred.pin)?;
        let id_packet = acc.maid_keys.clone().into();

        let Digest(digest) = sha256::hash(&acc.maid_keys.sign_pk.0);
        let cm_addr = Authority::ClientManager(XorName(digest));

        trace!("Creating an actual routing...");
        let (routing, routing_rx) = setup_routing(Some(id_packet))?;
        let net_tx_clone = net_tx.clone();
        let core_tx_clone = core_tx.clone();
        let joiner = spawn_routing_thread(routing_rx, core_tx_clone, net_tx_clone);

        Ok(Self::new(Inner {
            el_handle: el_handle,
            routing: routing,
            hooks: HashMap::with_capacity(10),
            cache: LruCache::new(IMMUT_DATA_CACHE_SIZE),
            client_type: ClientType::reg(acc, acc_loc, user_cred, cm_addr),
            timeout: Duration::from_secs(REQUEST_TIMEOUT_SECS),
            joiner: joiner,
        }))
    }

    /// This is a Gateway function to the Maidsafe network. This will help
    /// login to an already existing account of the user in the SAFE-network.
    pub fn from_keys<T>(keys: ClientKeys,
                        owner: sign::PublicKey,
                        el_handle: Handle,
                        core_tx: CoreMsgTx<T>,
                        net_tx: NetworkTx)
                        -> Result<Client, CoreError>
        where T: 'static
    {
        trace!("Attempting to log into an acc using client keys.");

        let Digest(digest) = sha256::hash(&keys.sign_pk.0);
        let cm_addr = Authority::ClientManager(XorName(digest));

        trace!("Creating an actual routing...");
        let (routing, routing_rx) = setup_routing(Some(keys.clone().into()))?;
        let net_tx_clone = net_tx.clone();
        let core_tx_clone = core_tx.clone();
        let joiner = spawn_routing_thread(routing_rx, core_tx_clone, net_tx_clone);

        Ok(Self::new(Inner {
            el_handle: el_handle,
            routing: routing,
            hooks: HashMap::with_capacity(10),
            cache: LruCache::new(IMMUT_DATA_CACHE_SIZE),
            client_type: ClientType::from_keys(owner, cm_addr),
            timeout: Duration::from_secs(REQUEST_TIMEOUT_SECS),
            joiner: joiner,
        }))
    }

    fn new(inner: Inner) -> Self {
        Client { inner: Rc::new(RefCell::new(inner)) }
    }

    /// Set request timeout.
    pub fn set_timeout(&self, duration: Duration) {
        self.inner_mut().timeout = duration;
    }

    #[doc(hidden)]
    pub fn restart_routing<T>(&self, mut core_tx: CoreMsgTx<T>, mut net_tx: NetworkTx)
        where T: 'static
    {
        let opt_id = if let ClientType::Registered { ref acc, .. } = self.inner().client_type {
            Some(acc.maid_keys.clone().into())
        } else {
            None
        };

        let (routing, routing_rx) = match setup_routing(opt_id) {
            Ok(elt) => elt,
            Err(e) => {
                info!("Could not restart routing (will re-attempt, unless dropped): {:?}",
                      e);
                let msg = {
                    let core_tx = core_tx.clone();
                    let net_tx = net_tx.clone();
                    CoreMsg::new(move |client, _| {
                        client.restart_routing(core_tx, net_tx);
                        None
                    })
                };
                let _ = core_tx.send(msg);
                return;
            }
        };

        let _ = net_tx.send(NetworkEvent::Connected);

        let joiner = spawn_routing_thread(routing_rx, core_tx, net_tx);

        self.inner_mut().hooks.clear();
        self.inner_mut().routing = routing;
        self.inner_mut().joiner = joiner;
    }

    #[doc(hidden)]
    pub fn fire_hook(&self, id: &MessageId, event: CoreEvent) {
        // Using in `if` keeps borrow alive. Do not try to combine the 2 lines into one.
        let opt = self.inner_mut().hooks.remove(id);
        if let Some(hook) = opt {
            hook.complete(event);
        }
    }

    fn insert_hook(&self, msg_id: MessageId, hook: Complete<CoreEvent>) {
        let _ = self.inner_mut().hooks.insert(msg_id, hook);
    }

    /// Get immutable data from the network. If the data exists locally in the cache
    /// then it will be immediately be returned without making an actual network
    /// request.
    pub fn get_idata(&self,
                     name: XorName,
                     dst: Option<Authority>)
                     -> Box<CoreFuture<ImmutableData>> {
        trace!("GetIData for {:?}", name);

        let (hook, rx, msg_id) = oneshot!(self, CoreEvent::GetIData);

        // Check if the data is in the cache. If it is, return it immediately.
        // If not, retrieve it from the network and store it in the cache.
        let rx = {
            let data = self.inner_mut()
                .cache
                .get_mut(&name)
                .map(|data| data.clone());

            if let Some(data) = data {
                trace!("ImmutableData found in cache.");
                hook.complete(CoreEvent::GetIData(Ok(data)));
                return rx.into_box();
            }

            let inner = self.inner.clone();
            rx.map(move |data| {
                    let _ = inner.borrow_mut().cache.insert(*data.name(), data.clone());
                    data
                })
                .into_box()
        };

        let dst = dst.unwrap_or_else(|| Authority::NaeManager(name));
        let result = self.routing_mut().get_idata(dst, name, msg_id);
        if let Err(err) = result {
            hook.complete(CoreEvent::GetIData(Err(CoreError::from(err))));
        } else {
            let _ = self.insert_hook(msg_id, hook);
        }

        rx
    }

    // TODO All these return the same future from all branches. So convert to impl
    // Trait when it arrives in stable. Change from `Box<CoreFuture>` -> `impl
    // CoreFuture`.
    /// Put immutable data onto the network.
    pub fn put_idata(&self, data: ImmutableData, dst: Option<Authority>) -> Box<CoreFuture<()>> {
        trace!("PutIData for {:?}", data);

        let (hook, rx, msg_id) = oneshot!(self, CoreEvent::Mutation);

        let dst = match dst {
            Some(a) => Ok(a),
            None => self.inner().client_type.cm_addr().map(|a| a.clone()),
        };

        let dst = match dst {
            Ok(a) => a,
            Err(e) => {
                hook.complete(CoreEvent::Mutation(Err(e)));
                return rx;
            }
        };

        let result = self.routing_mut().put_idata(dst, data, msg_id);
        if let Err(e) = result {
            hook.complete(CoreEvent::Mutation(Err(CoreError::from(e))));
        } else {
            let _ = self.insert_hook(msg_id, hook);
        }

        rx
    }

    /// Put `MutableData` onto the network.
    pub fn put_mdata(&self, data: MutableData, dst: Option<Authority>) -> Box<CoreFuture<()>> {
        trace!("PutMData for {:?}", data);

        let (hook, rx, msg_id) = oneshot!(self, CoreEvent::Mutation);

        let dst = match dst {
            Some(a) => Ok(a),
            None => self.inner().client_type.cm_addr().map(|a| a.clone()),
        };

        let dst = match dst {
            Ok(a) => a,
            Err(e) => {
                hook.complete(CoreEvent::Mutation(Err(e)));
                return rx;
            }
        };

        let result = self.routing_mut()
            .put_mdata(dst, data, msg_id, fry!(self.public_signing_key()));
        if let Err(e) = result {
            hook.complete(CoreEvent::Mutation(Err(CoreError::from(e))));
        } else {
            let _ = self.insert_hook(msg_id, hook);
        }

        rx
    }

    /// Mutates `MutableData` entries in bulk.
    pub fn mutate_mdata_entries(&self,
                                data: XorName,
                                tag: u64,
                                actions: BTreeMap<Vec<u8>, EntryAction>,
                                dst: Option<Authority>)
                                -> Box<CoreFuture<()>> {
        trace!("PutMData for {:?}", data);

        let (hook, rx, msg_id) = oneshot!(self, CoreEvent::Mutation);

        let dst = match dst {
            Some(a) => Ok(a),
            None => self.inner().client_type.cm_addr().map(|a| a.clone()),
        };

        let dst = match dst {
            Ok(a) => a,
            Err(e) => {
                hook.complete(CoreEvent::Mutation(Err(e)));
                return rx;
            }
        };

        let result = self.routing_mut()
            .mutate_mdata_entries(dst,
                                  data,
                                  tag,
                                  actions,
                                  msg_id,
                                  fry!(self.public_signing_key()));

        if let Err(e) = result {
            hook.complete(CoreEvent::Mutation(Err(CoreError::from(e))));
        } else {
            let _ = self.insert_hook(msg_id, hook);
        }

        rx
    }

    /*
    /// Put data to the network, with recovery.
    ///
    /// 1. If a data with the same name didn't previously exist, this is the
    /// same as normal PUT.
    /// 2. If it existed, but was deleted, attempt to reclaim it.
    /// 3. Otherwise succeed only if there is owners match.
    ///
    /// Resolves to the current version of the data, or 0 if the data doesn't
    /// have version.
    pub fn put_recover(&self,
                       data: Data,
                       dst: Option<Authority>,
                       sign_sk: sign::SecretKey)
                       -> Box<CoreFuture<u64>> {
        let version = match data {
            Data::Structured(ref data) => data.get_version(),
            Data::PrivAppendable(ref data) => data.get_version(),
            Data::PubAppendable(ref data) => data.get_version(),
            _ => {
                // Don't do recovery for other types
                return self.put(data, dst).map(|_| 0).into_box();
            }
        };

        let self2 = self.clone();
        let self3 = self.clone();

        self.put(data.clone(), dst.clone())
            .map(move |_| version)
            .or_else(move |put_err| {
                debug!("PUT failed with {:?}. Attempting recovery.", put_err);

                // Only attempt recovery on these errors:
                match put_err {
                    CoreError::MutationFailure { reason: MutationError::InvalidSuccessor, .. } |
                    CoreError::MutationFailure { reason: MutationError::DataExists, .. } => (),
                    _ => return err!(put_err),
                }

                self2.get(data.identifier(), None)
                    .then(move |result| {
                        let owner_match = match (result, data) {
                            (Ok(Data::Structured(ref old)), Data::Structured(ref new))
                                if old.is_deleted() => {
                                // The existing data is deleted. Attempt reclaim.
                                let data = fry!(StructuredData::new(
                                    new.get_type_tag(),
                                    *new.name(),
                                    old.get_version() + 1,
                                    new.get_data().clone(),
                                    new.get_owner_keys().clone(),
                                    new.get_previous_owner_keys().clone(),
                                    Some(&sign_sk))
                                        .map_err(move |_| put_err));

                                let version = data.get_version();

                                return self3.put(Data::Structured(data), dst)
                                    .map(move |_| version)
                                    .into_box();
                            }
                            (Ok(Data::Structured(old)), Data::Structured(new)) => {
                                old.get_owner_keys() == new.get_owner_keys()
                            }
                            (Ok(Data::PrivAppendable(old)), Data::PrivAppendable(new)) => {
                                old.get_owner_keys() == new.get_owner_keys()
                            }
                            (Ok(Data::PubAppendable(old)), Data::PubAppendable(new)) => {
                                old.get_owner_keys() == new.get_owner_keys()
                            }
                            (Ok(old), _) => {
                                debug!("Address space already occupied by: {:?}.", old);
                                return err!(put_err);
                            }
                            (Err(get_err), _) => {
                                debug!("Address space is vacant but still unable to PUT due to \
                                        {:?}.",
                                       get_err);
                                return err!(put_err);
                            }
                        };

                        if owner_match {
                            debug!("PUT recovery successful !");
                            ok!(version)
                        } else {
                            debug!("Data exists but we are not the owner.");
                            err!(put_err)
                        }
                    })
                    .into_box()
            })
            .into_box()
    }

    /// Post data onto the network.
    pub fn post(&self, data: Data, dst: Option<Authority>) -> Box<CoreFuture<()>> {
        trace!("Post for {:?}", data);
        self.stats_mut().issued_posts += 1;

        let msg_id = MessageId::new();

        let (hook, oneshot) = futures::oneshot();
        let rx = self.build_mutation_future(msg_id, oneshot);

        let dst = dst.unwrap_or_else(|| Authority::NaeManager(*data.name()));
        let result = self.routing_mut().send_post_request(dst, data, msg_id);

        if let Err(e) = result {
            hook.complete(CoreEvent::Mutation(Err(From::from(e))));
        } else {
            let _ = self.insert_hook(msg_id, hook);
        }

        rx
    }

    /// Delete data from the network
    pub fn delete(&self, data: Data, dst: Option<Authority>) -> Box<CoreFuture<()>> {
        trace!("DELETE for {:?}", data);

        self.stats_mut().issued_deletes += 1;

        let msg_id = MessageId::new();

        let (hook, oneshot) = futures::oneshot();
        let rx = self.build_mutation_future(msg_id, oneshot);

        let dst = dst.unwrap_or_else(|| Authority::NaeManager(*data.name()));
        let result = self.routing_mut().send_delete_request(dst, data, msg_id);

        if let Err(e) = result {
            hook.complete(CoreEvent::Mutation(Err(From::from(e))));
        } else {
            let _ = self.insert_hook(msg_id, hook);
        }

        rx
    }

    /// A version of `delete` that returns success if the data was already not
    /// present on the network, or it was present but in a deleted state
    /// already.
    pub fn delete_recover(&self, data: Data, dst: Option<Authority>) -> Box<CoreFuture<()>> {
        trace!("DELETE with recovery for {:?}", data);

        self.delete(data, dst)
            .then(|result| {
                match result {
                    Ok(()) |
                    Err(CoreError::MutationFailure {
                        reason: MutationError::NoSuchData, ..
                    }) |
                    Err(CoreError::MutationFailure {
                        reason: MutationError::InvalidOperation, ..
                    }) => {
                        debug!("DELETE recovery successful !");
                        Ok(())
                    }
                    Err(err) => {
                        debug!("DELETE recovery failed: {:?}", err);
                        Err(err)
                    }
                }
            })
            .into_box()
    }

    /// Append request
    pub fn append(&self, appender: AppendWrapper, dst: Option<Authority>) -> Box<CoreFuture<()>> {
        trace!("APPEND for {:?}", appender);

        self.stats_mut().issued_appends += 1;

        let msg_id = MessageId::new();

        let (hook, oneshot) = futures::oneshot();
        let rx = self.build_mutation_future(msg_id, oneshot);

        let dst = match dst {
            Some(auth) => auth,
            None => {
                let append_to = match appender {
                    AppendWrapper::Pub { ref append_to, .. } |
                    AppendWrapper::Priv { ref append_to, .. } => *append_to,
                };
                Authority::NaeManager(append_to)
            }
        };

        let result = self.routing_mut().send_append_request(dst, appender, msg_id);

        if let Err(e) = result {
            hook.complete(CoreEvent::Mutation(Err(From::from(e))));
        } else {
            let _ = self.insert_hook(msg_id, hook);
        }

        rx
    }

    /// Get data from the network.
    pub fn get_account_info(&self, dst: Option<Authority>) -> Box<CoreFuture<(u64, u64)>> {
        trace!("Account info GET issued.");

        let msg_id = MessageId::new();

        let (hook, oneshot) = futures::oneshot();
        let rx = oneshot.map_err(|_| CoreError::OperationAborted)
            .and_then(|event| match event {
                CoreEvent::AccountInfo(res) => res,
                _ => Err(CoreError::ReceivedUnexpectedEvent),
            });
        let rx = self.timeout(msg_id, rx);

        let dst = match dst {
            Some(a) => Ok(a),
            None => self.inner().client_type.cm_addr().map(|a| a.clone()),
        };

        let dst = match dst {
            Ok(a) => a,
            Err(e) => {
                hook.complete(CoreEvent::Mutation(Err(e)));
                return rx;
            }
        };

        let result = self.routing_mut().send_get_account_info_request(dst, msg_id);

        if let Err(e) = result {
            hook.complete(CoreEvent::AccountInfo(Err(From::from(e))));
        } else {
            let _ = self.insert_hook(msg_id, hook);
        }

        rx
    }
*/
    /// Create an entry for the Root Directory ID for the user into the session
    /// packet, encrypt and store it. It will be retrieved when the user logs
    /// into their account.  Root directory ID is necessary to fetch all of the
    /// user's data as all further data is encoded as meta-information into the
    /// Root Directory or one of its subdirectories.
    pub fn set_user_root_dir(&self, dir: Dir) -> Box<CoreFuture<()>> {
        trace!("Setting user root Dir ID.");

        let mut inner = self.inner_mut();
        let mut account = fry!(inner.client_type.acc_mut());
        account.user_root = dir;

        self.update_session_packet()
    }

    /// Get User's Root Directory ID if available in session packet used for
    /// current login
    pub fn user_root_dir(&self) -> Option<Dir> {
        self.inner().client_type.acc().ok().and_then(|account| Some(account.user_root.clone()))
    }

    /// Create an entry for the Maidsafe configuration specific Root Directory
    /// ID into the session packet, encrypt and store it. It will be retrieved
    /// when the user logs into their account. Root directory ID is necessary
    /// to fetch all of configuration data as all further data is encoded as
    /// meta-information into the config Root Directory or one of its
    /// subdirectories.
    pub fn set_config_root_dir(&self, dir: Dir) -> Box<CoreFuture<()>> {
        trace!("Setting configuration root Dir ID.");

        let mut inner = self.inner_mut();
        let mut account = fry!(inner.client_type.acc_mut());
        account.config_root = dir;

        self.update_session_packet()
    }

    /// Get Maidsafe specific configuration's Root Directory ID if available in
    /// session packet used for current login
    pub fn config_root_dir(&self) -> Option<Dir> {
        self.inner().client_type.acc().ok().and_then(|account| Some(account.config_root.clone()))
    }

    /// Returns the public encryption key
    pub fn public_encryption_key(&self) -> Result<box_::PublicKey, CoreError> {
        let inner = self.inner();
        let account = inner.client_type.acc()?;
        Ok(account.maid_keys.enc_pk)
    }

    /// Returns the Secret encryption key
    pub fn secret_encryption_key(&self) -> Result<box_::SecretKey, CoreError> {
        let inner = self.inner();
        let account = inner.client_type.acc()?;
        Ok(account.maid_keys.enc_sk.clone())
    }

    /// Returns the Public Signing key
    pub fn public_signing_key(&self) -> Result<sign::PublicKey, CoreError> {
        let inner = self.inner();
        let account = inner.client_type.acc()?;
        Ok(account.maid_keys.sign_pk)
    }

    /// Returns the Secret Signing key
    pub fn secret_signing_key(&self) -> Result<sign::SecretKey, CoreError> {
        let inner = self.inner();
        let account = inner.client_type.acc()?;
        Ok(account.maid_keys.sign_sk.clone())
    }

    /// Returns the public and secret signing keys.
    pub fn signing_keypair(&self) -> Result<(sign::PublicKey, sign::SecretKey), CoreError> {
        let inner = self.inner();
        let account = inner.client_type.acc()?;
        Ok((account.maid_keys.sign_pk, account.maid_keys.sign_sk.clone()))
    }

    fn update_session_packet(&self) -> Box<CoreFuture<()>> {
        trace!("Updating session packet.");

        let inner = self.inner();

        let data_name = fry!(inner.client_type.acc_loc());
        let account = fry!(inner.client_type.acc());

        let encrypted_account = {
            let keys = fry!(inner.client_type.user_cred());
            fry!(account.encrypt(&keys.password, &keys.pin))
        };

        let mut actions = BTreeMap::new();
        let _ = actions.insert("Login".as_bytes().to_owned(),
                               EntryAction::Update(Value {
                                   content: encrypted_account,
                                   entry_version: 0,
                               }));

        self.mutate_mdata_entries(data_name, TYPE_TAG_SESSION_PACKET, actions, None)
    }

    #[allow(unused)] // TODO(nbaksalyar) remove this
    fn timeout<F, T>(&self, msg_id: MessageId, future: F) -> Box<CoreFuture<T>>
        where F: Future<Item = T, Error = CoreError> + 'static,
              T: 'static
    {
        let duration = self.inner().timeout;
        let timeout = match Timeout::new(duration, &self.inner().el_handle) {
            Ok(timeout) => timeout,
            Err(err) => {
                return err!(CoreError::Unexpected(format!("Timeout create error: {:?}", err)))
            }
        };

        let client = self.clone();
        let timeout = timeout.then(move |result| -> Result<T, _> {
            let _ = client.inner_mut().hooks.remove(&msg_id);

            match result {
                Ok(()) => Err(CoreError::RequestTimeout),
                Err(err) => Err(CoreError::Unexpected(format!("Timeout fire error {:?}", err))),
            }
        });

        future.select(timeout)
            .then(|result| match result {
                Ok((a, _)) => Ok(a),
                Err((a, _)) => Err(a),
            })
            .into_box()
    }

    #[allow(unused)] // TODO(nbaksalyar) remove this
    fn routing_mut(&self) -> RefMut<Routing> {
        RefMut::map(self.inner.borrow_mut(), |i| &mut i.routing)
    }

    fn inner(&self) -> Ref<Inner> {
        self.inner.borrow()
    }

    fn inner_mut(&self) -> RefMut<Inner> {
        self.inner.borrow_mut()
    }
}

#[cfg(all(test, feature = "use-mock-routing"))]
impl Client {
    #[doc(hidden)]
    pub fn set_network_limits(&self, max_ops_count: Option<u64>) {
        self.routing_mut().set_network_limits(max_ops_count);
    }

    #[doc(hidden)]
    pub fn simulate_network_disconnect(&self) {
        self.routing_mut().simulate_disconnect();
    }

    #[doc(hidden)]
    pub fn set_simulate_timeout(&self, enabled: bool) {
        self.routing_mut().set_simulate_timeout(enabled);
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client")
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

#[allow(unused)] // <- TODO(nbaksalyar) remove this
enum ClientType {
    Unregistered,
    Registered {
        acc: Account,
        acc_loc: XorName,
        user_cred: UserCred,
        cm_addr: Authority,
    },
    FromKeys {
        owner: sign::PublicKey,
        cm_addr: Authority,
    },
}

impl ClientType {
    fn from_keys(owner: sign::PublicKey, cm_addr: Authority) -> Self {
        ClientType::FromKeys {
            owner: owner,
            cm_addr: cm_addr,
        }
    }

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
            ClientType::FromKeys { .. } => Err(CoreError::OperationForbiddenForClient),
        }
    }

    fn acc_mut(&mut self) -> Result<&mut Account, CoreError> {
        match *self {
            ClientType::Registered { ref mut acc, .. } => Ok(acc),
            ClientType::Unregistered => Err(CoreError::OperationForbiddenForClient),
            ClientType::FromKeys { .. } => Err(CoreError::OperationForbiddenForClient),
        }
    }

    fn acc_loc(&self) -> Result<XorName, CoreError> {
        match *self {
            ClientType::Registered { acc_loc, .. } => Ok(acc_loc),
            ClientType::Unregistered => Err(CoreError::OperationForbiddenForClient),
            ClientType::FromKeys { .. } => Err(CoreError::OperationForbiddenForClient),
        }
    }

    fn user_cred(&self) -> Result<&UserCred, CoreError> {
        match *self {
            ClientType::Registered { ref user_cred, .. } => Ok(user_cred),
            ClientType::Unregistered => Err(CoreError::OperationForbiddenForClient),
            ClientType::FromKeys { .. } => Err(CoreError::OperationForbiddenForClient),
        }
    }

    fn cm_addr(&self) -> Result<&Authority, CoreError> {
        match *self {
            ClientType::Registered { ref cm_addr, .. } => Ok(cm_addr),
            ClientType::Unregistered => Err(CoreError::OperationForbiddenForClient),
            ClientType::FromKeys { ref cm_addr, .. } => Ok(cm_addr),
        }
    }
}

fn setup_routing(full_id: Option<FullId>) -> Result<(Routing, Receiver<Event>), CoreError> {
    let (routing_tx, routing_rx) = mpsc::channel();
    let routing = Routing::new(routing_tx, full_id)?;

    trace!("Waiting to get connected to the Network...");
    match routing_rx.recv_timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS)) {
        Ok(Event::Connected) => (),
        x => {
            warn!("Could not connect to the Network. Unexpected: {:?}", x);
            // TODO: we should return more descriptive error here
            return Err(CoreError::OperationAborted);
        }
    }
    trace!("Connected to the Network.");

    Ok((routing, routing_rx))
}

fn spawn_routing_thread<T>(routing_rx: Receiver<Event>,
                           core_tx: CoreMsgTx<T>,
                           net_tx: NetworkTx)
                           -> Joiner
    where T: 'static
{
    thread::named("Routing Event Loop",
                  move || routing_el::run(routing_rx, core_tx, net_tx))
}

#[cfg(test)]
mod tests {
    use core::{CoreError, DIR_TAG, utility};
    use core::utility::test_utils::{finish, random_client, setup_client};
    use futures::Future;
    use futures::sync::mpsc;
    use routing::{ClientError, ImmutableData};
    // use rust_sodium::crypto::secretbox;
    use super::*;
    use tokio_core::reactor::Core;

    #[test]
    fn unregistered_client() {
        let orig_data = ImmutableData::new(unwrap!(utility::generate_random_vector(30)));

        // Registered Client PUTs something onto the network
        {
            let orig_data = orig_data.clone();
            random_client(|client| client.put_idata(orig_data, None));
        }

        // Unregistered Client should be able to retrieve the data
        setup_client(|el_h, core_tx, net_tx| Client::unregistered(el_h, core_tx, net_tx),
                     move |client| {
            let client2 = client.clone();
            let client3 = client.clone();

            client.get_idata(*orig_data.name(), None)
                .then(move |res| {
                    let data = unwrap!(res);
                    assert_eq!(data, orig_data);
                    let dir = Dir::random(DIR_TAG);
                    client2.set_user_root_dir(dir)
                })
                .then(move |res| {
                    let e = match res {
                        Ok(_) => {
                            panic!("Unregistered client should not be allowed to set user root dir")
                        }
                        Err(e) => e,
                    };
                    match e {
                        CoreError::OperationForbiddenForClient => (),
                        _ => panic!("Unexpected {:?}", e),
                    }

                    let dir = Dir::random(DIR_TAG);
                    client3.set_config_root_dir(dir)
                })
                .then(|res| {
                    let e = match res {
                        Ok(_) => {
                            panic!("Unregistered client should not be allowed to set config root \
                                    dir")
                        }
                        Err(e) => e,
                    };
                    match e {
                        CoreError::OperationForbiddenForClient => (),
                        _ => panic!("Unexpected {:?}", e),
                    }
                    finish()
                })
        });
    }

    #[test]
    fn registered_client() {
        let el = unwrap!(Core::new());
        let (core_tx, _) = mpsc::unbounded();
        let (net_tx, _) = mpsc::unbounded();

        let sec_0 = unwrap!(utility::generate_random_string(10));
        let sec_1 = unwrap!(utility::generate_random_string(10));

        // Account creation for the 1st time - should succeed
        let _ = unwrap!(Client::registered::<()>(&sec_0,
                                                 &sec_1,
                                                 el.handle(),
                                                 core_tx.clone(),
                                                 net_tx.clone()));

        // Account creation - same secrets - should fail
        match Client::registered(&sec_0, &sec_1, el.handle(), core_tx, net_tx) {
            Ok(_) => panic!("Account name hijacking should fail"),
            Err(CoreError::RoutingClientError(ClientError::AccountExists)) => (),
            Err(err) => panic!("{:?}", err),
        }
    }

    #[test]
    fn login() {
        let sec_0 = unwrap!(utility::generate_random_string(10));
        let sec_1 = unwrap!(utility::generate_random_string(10));

        setup_client(|el_h, core_tx, net_tx| {
            match Client::login(&sec_0,
                                &sec_1,
                                el_h.clone(),
                                core_tx.clone(),
                                net_tx.clone()) {
                Err(CoreError::RoutingClientError(ClientError::NoSuchAccount)) => (),
                x => panic!("Unexpected Login outcome: {:?}", x),
            }
            Client::registered(&sec_0, &sec_1, el_h, core_tx, net_tx)
        },
                     |_| finish());

        setup_client(|el_h, core_tx, net_tx| Client::login(&sec_0, &sec_1, el_h, core_tx, net_tx),
                     |_| finish());
    }
    /*
    #[test]
    fn user_root_dir_creation() {
        let sec_0 = unwrap!(utility::generate_random_string(10));
        let sec_1 = unwrap!(utility::generate_random_string(10));

        let dir_id = (DataIdentifier::Structured(rand::random(),
                                                 ::UNVERSIONED_STRUCT_DATA_TYPE_TAG),
                      Some(secretbox::gen_key()));
        let dir_id_clone = dir_id.clone();

        setup_client(|el_h, core_tx, net_tx| {
                         Client::registered(&sec_0, &sec_1, el_h, core_tx, net_tx)
                     },
                     move |client| {
                         assert!(client.user_root_dir_id().is_none());
                         client.set_user_root_dir_id(dir_id_clone)
                     });

        setup_client(|el_h, core_tx, net_tx| Client::login(&sec_0, &sec_1, el_h, core_tx, net_tx),
                     move |client| {
                         let got_dir_id = unwrap!(client.user_root_dir_id());
                         assert_eq!(got_dir_id, dir_id);
                         finish()
                     });
    }

    #[test]
    fn config_root_dir_creation() {
        let sec_0 = unwrap!(utility::generate_random_string(10));
        let sec_1 = unwrap!(utility::generate_random_string(10));

        let dir_id = (DataIdentifier::Structured(rand::random(),
                                                 ::UNVERSIONED_STRUCT_DATA_TYPE_TAG),
                      Some(secretbox::gen_key()));
        let dir_id_clone = dir_id.clone();

        setup_client(|el_h, core_tx, net_tx| {
                         Client::registered(&sec_0, &sec_1, el_h, core_tx, net_tx)
                     },
                     move |client| {
                         assert!(client.config_root_dir_id().is_none());
                         client.set_config_root_dir_id(dir_id_clone)
                     });

        setup_client(|el_h, core_tx, net_tx| Client::login(&sec_0, &sec_1, el_h, core_tx, net_tx),
                     move |client| {
                         let got_dir_id = unwrap!(client.config_root_dir_id());
                         assert_eq!(got_dir_id, dir_id);
                         finish()
                     });
    }

    #[test]
    fn put_or_reclaim_structured_data() {
        random_client(|client| {
            let client2 = client.clone();
            let client3 = client.clone();
            let client4 = client.clone();

            let owner_keys = vec![unwrap!(client.public_signing_key())];
            let owner_keys2 = owner_keys.clone();
            let owner_keys3 = owner_keys.clone();
            let owner_keys4 = owner_keys.clone();

            let sign_sk = unwrap!(client.secret_signing_key());
            let sign_sk2 = sign_sk.clone();
            let sign_sk3 = sign_sk.clone();
            let sign_sk4 = sign_sk.clone();

            let tag = ::UNVERSIONED_STRUCT_DATA_TYPE_TAG;
            let name = rand::random();
            let value = unwrap!(utility::generate_random_vector(10));

            // PUT the data to the network.
            let data = unwrap!(StructuredData::new(tag,
                                                   name,
                                                   0,
                                                   value,
                                                   owner_keys,
                                                   vec![],
                                                   Some(&sign_sk)));

            client.put(Data::Structured(data), None)
                .then(move |result| {
                    unwrap!(result);

                    // DELETE it.
                    let data = unwrap!(StructuredData::new(tag,
                                                           name,
                                                           1,
                                                           vec![],
                                                           vec![],
                                                           owner_keys2,
                                                           Some(&sign_sk2)));
                    client2.delete(Data::Structured(data), None)
                })
                .then(move |result| {
                    unwrap!(result);

                    // Try to PUT new data under the same name. Should fail.
                    let value = unwrap!(utility::generate_random_vector(10));
                    let data = unwrap!(StructuredData::new(tag,
                                                           name,
                                                           0,
                                                           value,
                                                           owner_keys3,
                                                           vec![],
                                                           Some(&sign_sk3)));
                    client3.put(Data::Structured(data), None)
                })
                .then(move |result| {
                    match result {
                        Err(CoreError::MutationFailure {
                            reason: MutationError::InvalidSuccessor, ..
                        }) => (),
                        Ok(()) => panic!("Unexpected success"),
                        Err(err) => panic!("{:?}", err),
                    }

                    // Not try again, but using `put_or_reclaim`. Should succeed.
                    let value = unwrap!(utility::generate_random_vector(10));
                    let data = unwrap!(StructuredData::new(tag,
                                                           name,
                                                           0,
                                                           value,
                                                           owner_keys4,
                                                           vec![],
                                                           Some(&sign_sk4)));
                    client4.put_recover(Data::Structured(data), None, sign_sk4)
                })
                .map_err(|err| panic!("{:?}", err))
                .map(|ver| assert!(ver != 0))
        })
    }

    #[cfg(feature = "use-mock-routing")]
    #[test]
    fn restart_routing() {
        use core::NetworkEvent;
        use core::utility::test_utils::random_client_with_net_obs;
        use futures;
        use maidsafe_utilities::thread;
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();
        let (hook, keep_alive) = futures::oneshot();

        let _joiner = thread::named("Network Observer", move || {
            match unwrap!(rx.recv()) {
                NetworkEvent::Disconnected => (),
                x => panic!("Unexpected network event: {:?}", x),
            }
            match unwrap!(rx.recv()) {
                NetworkEvent::Connected => (),
                x => panic!("Unexpected network event: {:?}", x),
            }
            hook.complete(());
        });

        random_client_with_net_obs(move |net_event| unwrap!(tx.send(net_event)),
                                   move |client| {
                                       client.simulate_network_disconnect();
                                       keep_alive
                                   });
    }

    #[cfg(feature = "use-mock-routing")]
    #[test]
    fn timeout() {
        use std::time::Duration;

        // Get
        random_client(|client| {
            let client2 = client.clone();

            client.set_simulate_timeout(true);
            client.set_timeout(Duration::from_millis(250));

            client.get(DataIdentifier::Immutable(rand::random()), None)
                .then(|result| match result {
                    Ok(_) => panic!("Unexpected success"),
                    Err(CoreError::RequestTimeout) => Ok::<_, CoreError>(()),
                    Err(err) => panic!("Unexpected {:?}", err),
                })
                .then(move |result| {
                    unwrap!(result);

                    let data = unwrap!(utility::generate_random_vector(4));
                    let data = ImmutableData::new(data);
                    let data = Data::Immutable(data);

                    client2.put(data, None)
                })
                .then(|result| match result {
                    Ok(_) => panic!("Unexpected success"),
                    Err(CoreError::RequestTimeout) => Ok::<_, CoreError>(()),
                    Err(err) => panic!("Unexpected {:?}", err),
                })
        })
    }
*/
}
