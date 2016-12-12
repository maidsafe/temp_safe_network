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

pub mod ffi;

mod errors;
mod object_cache;
#[cfg(test)]
mod test_util;

use core::{self, Client, ClientKeys, CoreMsg, CoreMsgTx, MDataInfo, NetworkEvent, NetworkTx};
use futures::Future;
use futures::stream::Stream;
use futures::sync::mpsc as futures_mpsc;
use ipc::{AccessContainer, AppKeys, AuthGranted, ContainerPermissions};
use ipc::req::ffi::Permission;
use maidsafe_utilities::thread::{self, Joiner};
use rust_sodium::crypto::secretbox;
use self::errors::AppError;
use self::object_cache::ObjectCache;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::mpsc as std_mpsc;
use tokio_core::reactor::{Core, Handle};

macro_rules! try_tx {
    ($result:expr, $tx:ident) => {
        match $result {
            Ok(res) => res,
            Err(e) => return unwrap!($tx.send(Err(AppError::from(e)))),
        }
    }
}

type AppFuture<T> = Future<Item = T, Error = AppError>;

/// Handle to an application instance.
pub struct App {
    core_tx: Mutex<CoreMsgTx<AppContext>>,
    _core_joiner: Joiner,
}

impl App {
    /// Create unregistered app.
    pub fn unregistered<N>(network_observer: N) -> Result<Self, AppError>
        where N: FnMut(Result<NetworkEvent, AppError>) + Send + 'static
    {
        Self::new(network_observer, |el_h, core_tx, net_tx| {
            let client = Client::unregistered(el_h, core_tx, net_tx)?;
            let context = AppContext::unregistered();
            Ok((client, context))
        })
    }

    /// Create registered app.
    pub fn registered<N>(auth_granted: AuthGranted, network_observer: N) -> Result<Self, AppError>
        where N: FnMut(Result<NetworkEvent, AppError>) + Send + 'static
    {
        let AuthGranted {
            app_keys: AppKeys { owner_key, enc_key, enc_pk, enc_sk, sign_pk, sign_sk },
            access_container,
            ..
        } = auth_granted;

        let client_keys = ClientKeys {
            sign_pk: sign_pk,
            sign_sk: sign_sk,
            enc_pk: enc_pk,
            enc_sk: enc_sk,
        };

        Self::new(network_observer, move |el_h, core_tx, net_tx| {
            let client = Client::from_keys(client_keys, owner_key, el_h, core_tx, net_tx)?;
            let context = AppContext::registered(enc_key, access_container);
            Ok((client, context))
        })
    }

    fn new<N, F>(mut network_observer: N, setup: F) -> Result<Self, AppError>
        where N: FnMut(Result<NetworkEvent, AppError>) + Send + 'static,
              F: FnOnce(Handle, CoreMsgTx<AppContext>, NetworkTx)
                        -> Result<(Client, AppContext), AppError> + Send + 'static
    {
        let (tx, rx) = std_mpsc::sync_channel(0);

        let joiner = thread::named("App Event Loop", move || {
            let el = try_tx!(Core::new(), tx);
            let el_h = el.handle();

            let (core_tx, core_rx) = futures_mpsc::unbounded();
            let (net_tx, net_rx) = futures_mpsc::unbounded();

            el_h.spawn(net_rx.map(move |event| network_observer(Ok(event)))
                .for_each(|_| Ok(())));

            let core_tx_clone = core_tx.clone();

            let (client, context) = try_tx!(setup(el_h, core_tx_clone, net_tx), tx);
            unwrap!(tx.send(Ok(core_tx)));

            core::run(el, client, context, core_rx);
        });

        let core_tx = rx.recv()??;

        Ok(App {
            core_tx: Mutex::new(core_tx),
            _core_joiner: joiner,
        })
    }

    /// Send a message to app's event loop
    pub fn send<F>(&self, f: F) -> Result<(), AppError>
        where F: FnOnce(&Client, &AppContext) -> Option<Box<Future<Item=(), Error=()>>>
                 + Send + 'static
    {
        let msg = CoreMsg::new(f);
        let mut core_tx = unwrap!(self.core_tx.lock());
        core_tx.send(msg).map_err(AppError::from)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let mut core_tx = match self.core_tx.lock() {
            Ok(core_tx) => core_tx,
            Err(err) => {
                info!("Unexpected error in drop: {:?}", err);
                return;
            }
        };

        let msg = CoreMsg::build_terminator();
        if let Err(err) = core_tx.send(msg) {
            info!("Unexpected error in drop: {:?}", err);
        }
    }
}

/// Application context (data associated with the app).
#[derive(Clone)]
pub struct AppContext {
    inner: Rc<Inner>,
}

enum Inner {
    Unauthorised(Unauthorised),
    Authorised(Authorised),
}

struct Unauthorised {
    object_cache: ObjectCache,
}

struct Authorised {
    object_cache: ObjectCache,
    sym_enc_key: secretbox::Key,
    _access_container: AccessContainer,
    _access_info: Vec<(MDataInfo, ContainerPermissions)>,
}

impl AppContext {
    fn unregistered() -> Self {
        AppContext {
            inner: Rc::new(Inner::Unauthorised(Unauthorised { object_cache: ObjectCache::new() })),
        }
    }

    fn registered(sym_enc_key: secretbox::Key, access_container: AccessContainer) -> Self {
        AppContext {
            inner: Rc::new(Inner::Authorised(Authorised {
                object_cache: ObjectCache::new(),
                sym_enc_key: sym_enc_key,
                _access_container: access_container,
                _access_info: Vec::new(),
            })),
        }
    }

    /// Object cache
    pub fn object_cache(&self) -> &ObjectCache {
        match *self.inner {
            Inner::Unauthorised(ref context) => &context.object_cache,
            Inner::Authorised(ref context) => &context.object_cache,
        }
    }

    /// Symmetric encryption/decryption key.
    pub fn sym_enc_key(&self) -> Result<&secretbox::Key, AppError> {
        Ok(&self.as_authorised()?.sym_enc_key)
    }

    /// Refresh access info by fetching it from the network.
    pub fn refresh_access_info(&self, _client: &Client) -> Box<AppFuture<()>> {
        unimplemented!()
    }

    /// Fetch mutable data info under the given name from the access container.
    pub fn get_mdata_info<T: Into<String>>(&self,
                                           _client: &Client,
                                           _name: T)
                                           -> Box<AppFuture<MDataInfo>> {
        unimplemented!()
    }

    /// Check the given permission for the given directory.
    pub fn is_permitted<T: Into<String>>(&self,
                                         _client: &Client,
                                         _dir_name: T,
                                         _permission: Permission)
                                         -> Box<AppFuture<bool>> {
        unimplemented!()
    }

    fn as_authorised(&self) -> Result<&Authorised, AppError> {
        match *self.inner {
            Inner::Authorised(ref context) => Ok(context),
            Inner::Unauthorised(_) => Err(AppError::Forbidden),
        }
    }
}
