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

use auth::AppKeys;
use core::Client;
use rust_sodium::crypto::{box_, secretbox, sign};
use std::sync::mpsc;
use super::{App, AppContext};

// Create registered app.
pub fn create_app() -> App {
    let app_keys = gen_app_keys();
    unwrap!(App::from_keys(app_keys, |_network_event| ()))
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

// Generate random `AppKeys`.
fn gen_app_keys() -> AppKeys {
    let owner_key = sign::gen_keypair().0;
    let enc_key = secretbox::gen_key();
    let (sign_pk, sign_sk) = sign::gen_keypair();
    let (enc_pk, enc_sk) = box_::gen_keypair();

    AppKeys {
        owner_key: owner_key,
        enc_key: enc_key,
        sign_pk: sign_pk,
        sign_sk: sign_sk,
        enc_pk: enc_pk,
        enc_sk: enc_sk,
    }
}
