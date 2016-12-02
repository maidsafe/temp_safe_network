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
use std::sync::mpsc;
use super::App;
use super::object_cache::ObjectCache;

// Create registered app.
pub fn create_app() -> App {
    unimplemented!()
}

// Run the given closure inside the app's event loop. The return value of
// the closure is returned immediately.
pub fn run_now<F, R>(app: &App, f: F) -> R
    where F: FnOnce(&Client, &ObjectCache) -> R + Send + 'static,
          R: Send + 'static
{
    let (tx, rx) = mpsc::channel();

    unwrap!(app.send(move |client, object_cache| {
        unwrap!(tx.send(f(client, object_cache)));
        None
    }));

    unwrap!(rx.recv())
}


/*
pub fn create_unregistered_session() -> Session {
    unwrap!(Session::unregistered(|_| ()))
}

// Run the given closure inside the session event loop. The closure should
// return a future which will then be driven to completion and its result
// returned.
pub fn run<F, I, R, E>(session: &Session, f: F) -> R
    where F: FnOnce(&Client, &ObjectCache) -> I + Send + 'static,
          I: IntoFuture<Item = R, Error = E> + 'static,
          R: Send + 'static,
          E: Debug
{
    let (tx, rx) = mpsc::channel();

    unwrap!(session.send(move |client, object_cache| {
        let future = f(client, object_cache)
            .into_future()
            .map_err(|err| panic!("{:?}", err))
            .map(move |result| unwrap!(tx.send(result)))
            .into_box();

        Some(future)
    }));

    unwrap!(rx.recv())
}

*/
