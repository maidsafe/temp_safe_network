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

use core::Client;
use core::utility::test_utils::random_client;
use ipc::{AccessContainer, AppKeys, AuthGranted, Config};
use rand;
use rust_sodium::crypto::{box_, secretbox, sign};
use std::sync::mpsc;
use super::{App, AppContext};

const ACCESS_CONTAINER_TAG: u64 = 1000;

// Create registered app.
pub fn create_app() -> App {
    let enc_key = secretbox::gen_key();
    let (sign_pk, sign_sk) = sign::gen_keypair();
    let (enc_pk, enc_sk) = box_::gen_keypair();

    // Create account and authorize the app key.
    let (tx, rx) = mpsc::channel();
    random_client(move |client| {
        let owner_key = unwrap!(client.owner_key());
        unwrap!(tx.send(owner_key));

        client.ins_auth_key(sign_pk, 1)
    });
    let owner_key = unwrap!(rx.recv());

    let app_keys = AppKeys {
        owner_key: owner_key,
        enc_key: enc_key,
        sign_pk: sign_pk,
        sign_sk: sign_sk,
        enc_pk: enc_pk,
        enc_sk: enc_sk,
    };

    let access_container = AccessContainer {
        id: rand::random(),
        tag: ACCESS_CONTAINER_TAG,
        nonce: secretbox::gen_nonce(),
    };

    let auth_granted = AuthGranted {
        app_keys: app_keys,
        bootstrap_config: Config,
        access_container: access_container,
    };

    unwrap!(App::authorised(auth_granted, |_network_event| ()))
}

/*
// Create unregistered app.
pub fn create_unregistered_app() -> App {
    unwrap!(App::unregistered(|_| ()))
}
*/

// Run the given closure inside the app's event loop. The return value of
// the closure is returned immediately.
pub fn run_now<F, R>(app: &App, f: F) -> R
    where F: FnOnce(&Client, &AppContext) -> R + Send + 'static,
          R: Send + 'static
{
    let (tx, rx) = mpsc::channel();

    unwrap!(app.send(move |client, context| {
        unwrap!(tx.send(f(client, context)));
        None
    }));

    unwrap!(rx.recv())
}

/*

// Run the given closure inside the app event loop. The closure should
// return a future which will then be driven to completion and its result
// returned.
pub fn run<F, I, R, E>(app: &App, f: F) -> R
    where F: FnOnce(&Client, &AppContext) -> I + Send + 'static,
          I: IntoFuture<Item = R, Error = E> + 'static,
          R: Send + 'static,
          E: Debug
{
    let (tx, rx) = mpsc::channel();

    unwrap!(app.send(move |client, app| {
        let future = f(client, app)
            .into_future()
            .map_err(|err| panic!("{:?}", err))
            .map(move |result| unwrap!(tx.send(result)))
            .into_box();

        Some(future)
    }));

    unwrap!(rx.recv())
}

*/
