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

use core::{Client, utility};
use ffi::{App, Session};
use ffi::launcher_config;
use futures::{Future, IntoFuture};
use libc::c_void;
use std::ffi::CString;
use std::fmt::Debug;
use std::sync::mpsc::{self, Sender};

pub fn generate_random_cstring(len: usize) -> CString {
    let mut cstring_vec = unwrap!(utility::generate_random_vector::<u8>(len));

    // Avoid internal nulls and ensure valid ASCII (thus valid utf8)
    for it in &mut cstring_vec {
        *it %= 128;
        if *it == 0 {
            *it += 1;
        }
    }

    // Ok to unwrap, as we took care of removing all NULs above.
    unwrap!(CString::new(cstring_vec))
}

pub fn create_session() -> Session {
    let acc_locator = unwrap!(utility::generate_random_string(10));
    let acc_password = unwrap!(utility::generate_random_string(10));

    unwrap!(Session::create_account(acc_locator, acc_password, |_net_evt| ()))
}

pub fn create_unregistered_session() -> Session {
    unwrap!(Session::unregistered(|_| ()))
}

// Run the given closure inside the session event loop.
pub fn run<F, I, R, E>(session: &Session, f: F) -> R
    where F: FnOnce(&Client) -> I + Send + 'static,
          I: IntoFuture<Item=R, Error=E> + 'static,
          R: Send + 'static,
          E: Debug
{
    let (tx, rx) = mpsc::channel();

    unwrap!(session.send_fn(move |client| {
        let future = f(client).into_future()
            .map_err(|err| panic!("{:?}", err))
            .map(move |result| unwrap!(tx.send(result)));

        Some(future)
    }));

    unwrap!(rx.recv())
}

pub fn create_app(session: &Session, has_safe_drive_access: bool) -> App {
    let app_name = "Test App".to_string();
    let app_id = unwrap!(utility::generate_random_string(10));
    let vendor = "Test Vendor".to_string();

    run(session, move |client| {
        launcher_config::app(client, app_name, app_id, vendor, has_safe_drive_access)
    })
}

// Convert a `mpsc::Sender<()>` to a void ptr which can be passed as user data to
// ffi functions
pub fn sender_as_user_data(tx: &Sender<()>) -> *mut c_void {
    let ptr: *const _ = tx;
    ptr as *mut c_void
}

// Send `()` through a `mpsc::Sender` pointed to by the user data pointer.
pub unsafe fn send_via_user_data(user_data: *mut c_void) {
    let tx = user_data as *mut Sender<()>;
    unwrap!((*tx).send(()));
}

pub struct FfiStr {
    pub ptr: *const u8,
    pub len: usize,
}

pub fn as_raw_parts(s: &str) -> FfiStr {
    FfiStr {
        ptr: s.as_ptr(),
        len: s.len(),
    }
}
