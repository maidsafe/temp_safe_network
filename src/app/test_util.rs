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

use core::{Client, FutureExt, utility};
use core::utility::test_utils::random_client;
use futures::{Future, IntoFuture};
use ipc::{AccessContInfo, AppKeys, AuthGranted, Config};
use rand;
use rust_sodium::crypto::{box_, secretbox, sign};
use std::sync::mpsc;
use super::{App, AppContext};
use super::errors::AppError;

const ACCESS_CONTAINER_TAG: u64 = 1000;

// Create registered app.
pub fn create_app() -> App {
    let app_id = unwrap!(utility::generate_random_string(10));

    let enc_key = secretbox::gen_key();
    let (sign_pk, sign_sk) = sign::gen_keypair();
    let (enc_pk, enc_sk) = box_::gen_keypair();

    // Create account and authorize the app key.
    let owner_key = random_client(move |client| {
        let owner_key = unwrap!(client.owner_key());
        client.ins_auth_key(sign_pk, 1).map(move |_| owner_key)
    });

    let app_keys = AppKeys {
        owner_key: owner_key,
        enc_key: enc_key,
        sign_pk: sign_pk,
        sign_sk: sign_sk,
        enc_pk: enc_pk,
        enc_sk: enc_sk,
    };

    let access_container = AccessContInfo {
        id: rand::random(),
        tag: ACCESS_CONTAINER_TAG,
        nonce: secretbox::gen_nonce(),
    };

    let auth_granted = AuthGranted {
        app_keys: app_keys,
        bootstrap_config: Config,
        access_container: access_container,
    };

    unwrap!(App::registered(app_id, auth_granted, |_network_event| ()))
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

// Run the given closure inside the app event loop. The closure should
// return a future which will then be driven to completion and its result
// returned.
pub fn run<F, I, T>(app: &App, f: F) -> T
    where F: FnOnce(&Client, &AppContext) -> I + Send + 'static,
          I: IntoFuture<Item = T, Error = AppError> + 'static,
          T: Send + 'static
{
    let (tx, rx) = mpsc::channel();

    unwrap!(app.send(move |client, app| {
        let future = f(client, app)
            .into_future()
            .then(move |result| {
                unwrap!(tx.send(unwrap!(result)));
                Ok(())
            })
            .into_box();

        Some(future)
    }));

    unwrap!(rx.recv())
}
