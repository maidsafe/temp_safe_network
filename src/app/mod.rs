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

#![allow(unsafe_code)]

pub mod low_level_api;

mod errors;
mod object_cache;
#[cfg(test)]
mod test_util;

use auth::AppAccessToken;
use core::{self, Client, CoreMsg, CoreMsgTx, NetworkEvent};
use futures::Future;
use futures::stream::Stream;
use futures::sync::mpsc as futures_mpsc;
use maidsafe_utilities::thread::{self, Joiner};
use self::errors::AppError;
use self::object_cache::ObjectCache;
use std::os::raw::c_void;
use std::sync::Mutex;
use std::sync::mpsc as std_mpsc;
use tokio_core::reactor::Core;
use util::ffi::{self, OpaqueCtx};

macro_rules! try_tx {
    ($result:expr, $tx:ident) => {
        match $result {
            Ok(res) => res,
            Err(e) => return unwrap!($tx.send(Err(AppError::from(e)))),
        }
    }
}

/// Handle to an application instance.
pub struct App {
    _app_type: AppType,
    core_tx: Mutex<CoreMsgTx<ObjectCache>>,
    _core_joiner: Joiner,
}

impl App {
    /// Create unregistered app.
    pub fn unregistered<N>(mut network_observer: N) -> Result<Self, AppError>
        where N: FnMut(Result<NetworkEvent, AppError>) + Send + 'static
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

            let client = try_tx!(Client::unregistered(el_h, core_tx_clone, net_tx), tx);
            let object_cache = ObjectCache::new();
            unwrap!(tx.send(Ok(core_tx)));

            core::run(el, client, object_cache, core_rx);
        });

        let core_tx = rx.recv()??;

        Ok(App {
            _app_type: AppType::Unregistered,
            core_tx: Mutex::new(core_tx),
            _core_joiner: joiner,
        })
    }

    /// Send a message to app's event loop
    pub fn send<F>(&self, f: F) -> Result<(), AppError>
        where F: FnOnce(&Client, &ObjectCache) -> Option<Box<Future<Item=(), Error=()>>>
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

#[allow(unused)] // <-- TODO: remove this
enum AppType {
    Unregistered,
    FromKeys(AppAccessToken),
}

// ---------- FFI --------------------

/// Create unregistered app.
#[no_mangle]
pub unsafe extern "C" fn app_unregistered(user_data: *mut c_void,
                                          network_observer_cb: unsafe extern "C" fn(*mut c_void,
                                                                                    i32,
                                                                                    i32),
                                          o_app: *mut *mut App)
                                          -> i32 {
    ffi::catch_unwind_error_code(|| -> Result<_, AppError> {
        let user_data = OpaqueCtx(user_data);

        let app = App::unregistered(move |event| {
            match event {
                Ok(event) => network_observer_cb(user_data.0, 0, event.into()),
                Err(err) => network_observer_cb(user_data.0, ffi_error_code!(err), 0),
            }
        })?;

        *o_app = Box::into_raw(Box::new(app));

        Ok(())
    })
}
