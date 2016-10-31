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

use core::{self, Client, CoreError, CoreMsg, CoreMsgTx, FutureExt, NetworkEvent, NetworkTx,
           utility};
use futures::{Future, IntoFuture};
use futures::stream::Stream;
use rust_sodium::crypto::sign;
use std::{iter, u8};
use std::fmt::Debug;
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

// Create random registered client and run it inside an event loop. Use this to
// create Client automatically and randomly,
pub fn random_client<Run, I, E>(r: Run)
    where Run: FnOnce(&Client) -> I + Send + 'static,
          I: IntoFuture<Item = (), Error = E> + 'static,
          E: Debug
{
    let n = |net_event| panic!("Unexpected NetworkEvent occurred: {:?}", net_event);
    random_client_with_net_obs(n, r)
}

// Create random registered client and run it inside an event loop. Use this to
// create Client automatically and randomly,
pub fn random_client_with_net_obs<NetObs, Run, I, E>(n: NetObs, r: Run)
    where NetObs: FnMut(NetworkEvent) + 'static,
          Run: FnOnce(&Client) -> I + Send + 'static,
          I: IntoFuture<Item = (), Error = E> + 'static,
          E: Debug
{
    let c = |core_tx, net_tx| {
        let acc_locator = unwrap!(utility::generate_random_string(10));
        let acc_password = unwrap!(utility::generate_random_string(10));
        Client::registered(&acc_locator, &acc_password, core_tx, net_tx)
    };
    setup_client_with_net_obs(c, n, r)
}

// Helper to create a client and run it in an event loop. Useful when we need
// to supply credentials explicitly or when Client is to be constructed as
// unregistered or as a result of successful login. Use this to create Client
// manually,
pub fn setup_client<Create, Run, I, E>(c: Create, r: Run)
    where Create: FnOnce(CoreMsgTx<()>, NetworkTx) -> Result<Client, CoreError>,
          Run: FnOnce(&Client) -> I + Send + 'static,
          I: IntoFuture<Item = (), Error = E> + 'static,
          E: Debug
{
    let n = |net_event| panic!("Unexpected NetworkEvent occurred: {:?}", net_event);
    setup_client_with_net_obs(c, n, r)
}

// Helper to create a client and run it in an event loop. Useful when we need
// to supply credentials explicitly or when Client is to be constructed as
// unregistered or as a result of successful login. Use this to create Client
// manually,
pub fn setup_client_with_net_obs<Create, NetObs, Run, I, E>(c: Create, mut n: NetObs, r: Run)
    where Create: FnOnce(CoreMsgTx<()>, NetworkTx) -> Result<Client, CoreError>,
          NetObs: FnMut(NetworkEvent) + 'static,
          Run: FnOnce(&Client) -> I + Send + 'static,
          I: IntoFuture<Item = (), Error = E> + 'static,
          E: Debug
{
    let el = unwrap!(Core::new());
    let el_h = el.handle();

    let (core_tx, core_rx) = unwrap!(channel::channel(&el_h));
    let (net_tx, net_rx) = unwrap!(channel::channel(&el_h));
    let client = unwrap!(c(core_tx.clone(), net_tx));

    let net_fut = net_rx.for_each(move |net_event| Ok(n(net_event)))
        .map_err(|e| panic!("Network event stream error: {:?}", e));
    el_h.spawn(net_fut);

    let core_tx_clone = core_tx.clone();
    unwrap!(core_tx.send(CoreMsg::new(move |client, &()| {
        let fut = r(client).into_future()
            .map_err(|e| panic!("{:?}", e))
            .map(move |()| unwrap!(core_tx_clone.send(CoreMsg::build_terminator())))
            .into_box();

        Some(fut)
    })));

    core::run(el, client, (), core_rx);
}

/// Convenience for creating a blank runner.
pub fn finish() -> Result<(), ()> {
    Ok(())
}
