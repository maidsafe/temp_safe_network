// Copyright 2017 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

mod nfs;

use super::*;
use ffi_utils::test_utils::call_1;
use routing::ImmutableData;
use safe_core::ffi::AccountInfo;
use test_utils::create_app;

// Test account usage statistics before and after a mutation.
#[test]
fn account_info() {
    let app = create_app();
    let app = Box::into_raw(Box::new(app));

    let orig_stats: AccountInfo =
        unsafe { unwrap!(call_1(|ud, cb| app_account_info(app, ud, cb))) };
    assert!(orig_stats.mutations_available > 0);

    unsafe {
        unwrap!((*app).send(move |client, _| {
            client
                .put_idata(ImmutableData::new(vec![1, 2, 3]))
                .map_err(move |_| ())
                .into_box()
                .into()
        }));
    }

    let stats: AccountInfo = unsafe { unwrap!(call_1(|ud, cb| app_account_info(app, ud, cb))) };
    assert_eq!(stats.mutations_done, orig_stats.mutations_done + 1);
    assert_eq!(
        stats.mutations_available,
        orig_stats.mutations_available - 1
    );

    unsafe { app_free(app) };
}

// Test disconnection and reconnection with apps.
#[cfg(all(test, feature = "use-mock-routing"))]
#[test]
fn network_status_callback() {
    use App;
    use ffi_utils::test_utils::{UserData, call_0, call_1_with_custom, send_via_user_data_custom};
    use maidsafe_utilities::serialisation::serialise;
    use safe_core::ipc::BootstrapConfig;
    use std::os::raw::c_void;
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::time::Duration;

    {
        let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();

        let bootstrap_cfg = unwrap!(serialise(&BootstrapConfig::default()));
        let mut custom_ud: UserData = Default::default();
        let ptr: *const _ = &tx;
        custom_ud.custom = ptr as *mut c_void;

        let app: *mut App = unsafe {
            unwrap!(call_1_with_custom(&mut custom_ud, |ud, cb| {
                app_unregistered(
                    bootstrap_cfg.as_ptr(),
                    bootstrap_cfg.len(),
                    ud,
                    disconnect_cb,
                    cb,
                )
            }))
        };

        unsafe {
            unwrap!((*app).send(move |client, _| {
                client.simulate_network_disconnect();
                None
            }));
        }

        // disconnect_cb should be called.
        unwrap!(rx.recv_timeout(Duration::from_secs(15)));

        // Reconnect with the network
        unsafe { unwrap!(call_0(|ud, cb| app_reconnect(app, ud, cb))) };

        // This should time out.
        let result = rx.recv_timeout(Duration::from_secs(1));
        match result {
            Err(_) => (),
            _ => panic!("Disconnect callback was called"),
        }

        // The reconnection should be fine if we're already connected.
        unsafe { unwrap!(call_0(|ud, cb| app_reconnect(app, ud, cb))) };

        // disconnect_cb should be called.
        unwrap!(rx.recv_timeout(Duration::from_secs(15)));

        // This should time out.
        let result = rx.recv_timeout(Duration::from_secs(1));
        match result {
            Err(_) => (),
            _ => panic!("Disconnect callback was called"),
        }

        unsafe { app_free(app) };
    }

    extern "C" fn disconnect_cb(user_data: *mut c_void) {
        unsafe {
            send_via_user_data_custom(user_data, ());
        }
    }
}
