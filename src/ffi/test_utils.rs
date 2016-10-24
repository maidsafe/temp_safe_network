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

use core::{CoreMsg, utility};
use core::futures::FutureExt;
use ffi::{App, Session};
use ffi::launcher_config;
use futures::Future;
use std::ffi::CString;
use std::sync::mpsc;

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

    Session::create_account(acc_locator, acc_password)
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
