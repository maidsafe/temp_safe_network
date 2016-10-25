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

use core::{Client, CoreMsg, utility};
use core::futures::FutureExt;
use ffi::{App, Session};
use ffi::{helper, launcher_config};
use futures::{Future, IntoFuture};
use libc::c_void;
use std::ffi::CString;
use std::fmt::Debug;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

const RUN_TIMEOUT_MS: u64 = 10_000;

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

    unwrap!(Session::create_account(acc_locator, acc_password, move |_net_evt| ()))
}

// Run the given closure inside the session event loop.
pub fn run<F, I, R, E>(session: &Session, f: F) -> R
    where F: FnOnce(&Client) -> I + Send + 'static,
          I: IntoFuture<Item=R, Error=E> + 'static,
          R: Send + 'static,
          E: Debug
{
    let (tx, rx) = mpsc::channel();

    unwrap!(session.send(CoreMsg::new(move |client| {
        let future = f(client).into_future()
            .map_err(|err| panic!("{:?}", err))
            .map(move |result| unwrap!(tx.send(result)))
            .into_box();

        Some(future)
    })));

    unwrap!(rx.recv_timeout(Duration::from_millis(RUN_TIMEOUT_MS)))
}

pub fn create_app(session: &Session, has_safe_drive_access: bool) -> App {
    let app_name = "Test App".to_string();
    let app_id = unwrap!(utility::generate_random_string(10));
    let vendor = "Test Vendor".to_string();

    let (tx, rx) = mpsc::channel();

    unwrap!(session.send(CoreMsg::new(move |client| {
        let fut = launcher_config::app(client, app_name, app_id, vendor, has_safe_drive_access)
            .map(move |app| {
                unwrap!(tx.send(app));
            })
            .map_err(move |_| ())
            .into_box();
        Some(fut)
    })));

    unwrap!(rx.recv())
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

// RAII-style wrapper for a (pointer, length, capacity) tripple. Useful when we
// need to pass a String to a FFI function without giving up its ownership.
pub struct CUtf8 {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl CUtf8 {
    pub fn ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for CUtf8 {
    fn drop(&mut self) {
        unsafe {
            let _ = String::from_raw_parts(self.ptr, self.len, self.cap);
        }
    }
}

// Convert `String` to `CUtf8` wrapper.
pub fn string_to_c_utf8(s: String) -> CUtf8 {
    let (ptr, len, cap) = helper::string_to_c_utf8(s);
    CUtf8 {
        ptr: ptr,
        len: len,
        cap: cap,
    }
}
