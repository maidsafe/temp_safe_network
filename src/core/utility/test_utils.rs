// Copyright 2015 MaidSafe.net limited.
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

use el::client::Client;
use el::core_el::{self, CoreMsg, CoreMsgRx, CoreMsgTx, NetworkTx};
use el::errors::CoreError;
use el::futures::FutureExt;
use el::utility;
use futures::Future;
use rust_sodium::crypto::sign;
use std::iter;
use std::u8;
use tokio_core::channel;
use tokio_core::reactor::Core;

/// Generates random public keys
pub fn generate_public_keys(len: usize) -> Vec<sign::PublicKey> {
    (0..len).map(|_| sign::gen_keypair().0).collect()
}

/// Generates random secret keys
pub fn generate_secret_keys(len: usize) -> Vec<sign::SecretKey> {
    (0..len).map(|_| sign::gen_keypair().1).collect()
}

/// Generates public keys of maximum size
pub fn get_max_sized_public_keys(len: usize) -> Vec<sign::PublicKey> {
    iter::repeat(sign::PublicKey([u8::MAX; sign::PUBLICKEYBYTES])).take(len).collect()
}

/// Generates secret keys of maximum size
pub fn get_max_sized_secret_keys(len: usize) -> Vec<sign::SecretKey> {
    iter::repeat(sign::SecretKey([u8::MAX; sign::SECRETKEYBYTES])).take(len).collect()
}

// Create random registered client and run it inside an event loop.
pub fn register_and_run<F, R>(f: F)
    where F: FnOnce(&Client) -> R + Send + 'static,
          R: Future + 'static
{
    setup_client(|core_tx, net_tx| {
            let acc_locator = unwrap!(utility::generate_random_string(10));
            let acc_password = unwrap!(utility::generate_random_string(10));
            Client::registered(&acc_locator, &acc_password, core_tx, net_tx)
        })
        .run(f)
}

// TODO Expand this to take a callback when ffi is coded - that way disconnections can be tested.
// Helper to create a client and run it inside an event loop.
pub fn setup_client<F>(f: F) -> Env
    where F: FnOnce(CoreMsgTx, NetworkTx) -> Result<Client, CoreError>
{
    let el = unwrap!(Core::new());
    let el_h = el.handle();
    let (core_tx, core_rx) = unwrap!(channel::channel(&el_handle));
    let (net_tx, net_rx) = unwrap!(channel::channel(&el_handle));
    let net_fut = net_rx.for_each(|net_event| {
            debug!("Network event encountered: {:?}", net_event);
            Ok(())
        })
        .map_err(|e| debug!("Network event stream error: {:?}", e));
    el_h.spawn(net_fut);

    Env {
        client: unwrap!(f(core_tx.clone(), net_tx)),
        el: el,
        core_tx: core_tx,
        core_rx: core_rx,
    }
}

pub struct Env {
    client: Client,
    el: Core,
    core_tx: CoreMsgTx,
    core_rx: CoreMsgRx,
}

impl Env {
    // Spin up an event loop and execute the given closure on it. The closure
    // must return a future which will then be driven to completion.
    pub fn run<F, R>(self, f: F)
        where F: FnOnce(&Client) -> R + Send + 'static,
              R: Future + 'static
    {
        let core_tx = self.core_tx.clone();

        unwrap!(self.core_tx.send(CoreMsg::new(move |client| {
            let future = f(client)
                .then(move |_| {
                    // When the future completes, send terminator to the event loop
                    // to stop it.
                    unwrap!(core_tx.send(CoreMsg::build_terminator()));
                    Ok(())
                })
                .into_box();

            Some(future)
        })));

        core_el::run(self.el, self.client, self.core_rx);
    }

    // Return the client stored in this Env.
    pub fn unwrap(self) -> Client {
        self.client
    }
}
