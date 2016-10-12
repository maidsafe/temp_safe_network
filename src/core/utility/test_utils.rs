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

use core::client::Client;
use core::core_el::{self, CoreMsg, CoreMsgRx, CoreMsgTx};
use core::errors::CoreError;
use core::futures::FutureExt;
use core::utility;
use futures::Future;
use rust_sodium::crypto::sign;
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
    ::std::iter::repeat(sign::PublicKey([::std::u8::MAX; sign::PUBLICKEYBYTES])).take(len).collect()
}

/// Generates secret keys of maximum size
pub fn get_max_sized_secret_keys(len: usize) -> Vec<sign::SecretKey> {
    ::std::iter::repeat(sign::SecretKey([::std::u8::MAX; sign::SECRETKEYBYTES])).take(len).collect()
}

// Create random registered client and run it inside an event loop.
pub fn register_and_run<F, R>(f: F)
    where F: FnOnce(&Client) -> R + Send + 'static,
          R: Future + 'static
{
    setup_client(|core_tx| {
        let acc_locator = unwrap!(utility::generate_random_string(10));
        let acc_password = unwrap!(utility::generate_random_string(10));
        Client::registered(&acc_locator, &acc_password, core_tx)
    }).run(f)
}

// Helper to create a client and run it inside an event loop.
pub fn setup_client<F>(f: F) -> Env
    where F: FnOnce(CoreMsgTx) -> Result<Client, CoreError>
{
    let core = unwrap!(Core::new());
    let (tx, rx) = unwrap!(channel::channel(&core.handle()));

    Env {
        client: unwrap!(f(tx.clone())),
        core: core,
        tx: tx,
        rx: rx,
    }
}

pub struct Env {
    client: Client,
    core: Core,
    tx: CoreMsgTx,
    rx: CoreMsgRx,
}

impl Env {
    // Spin up an event loop and execute the given closure on it. The closure
    // must return a future which will then be driven to completion.
    pub fn run<F, R>(self, f: F)
        where F: FnOnce(&Client) -> R + Send + 'static,
              R: Future + 'static
    {
        let tx = self.tx.clone();

        unwrap!(self.tx.send(CoreMsg::new(move |client| {
            let future = f(client)
                .then(move |_| {
                    // When the future completes, send terminator to the event loop
                    // to stop it.
                    unwrap!(tx.send(CoreMsg::build_terminator()));
                    Ok(())
                })
                .into_box();

            Some(future)
        })));

        core_el::run(self.core, self.client, self.rx);
    }

    // Return the client stored in this Env.
    pub fn unwrap(self) -> Client {
        self.client
    }
}
