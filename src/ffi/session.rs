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

//! Session management

use core::{self, Client, CoreMsg, CoreMsgTx};
use core::futures::FutureExt;
use ffi::{FfiError, OpaqueCtx};
use ffi::object_cache::ObjectCache;
use futures::Future;
use libc::{c_void, int32_t, int64_t, uint64_t};
use maidsafe_utilities::thread::{self, Joiner};
use std::sync::{Arc, Mutex, mpsc};
use super::helper;
use tokio_core::channel;
use tokio_core::reactor::Core;

/// Represents user session on the SAFE network. There should be one session
/// per launcher.
pub struct Session {
    inner: Arc<Inner>,
}

struct Inner {
    // Channel to communicate with the core event loop
    pub core_tx: Mutex<CoreMsgTx>,
    object_cache: Arc<Mutex<ObjectCache>>,
    _core_joiner: Joiner,
}

impl Session {
    /// Send a message to the core event loop
    pub fn send(&self, msg: CoreMsg) -> Result<(), FfiError> {
        let core_tx = unwrap!(self.inner.core_tx.lock());
        core_tx.send(msg).map_err(FfiError::from)
    }

    /// Returns an object cache tied to the session
    pub fn object_cache(&self) -> Arc<Mutex<ObjectCache>> {
        self.inner.object_cache.clone()
    }

    /// Create unregistered client.
    pub fn unregistered() -> Self {
        let (tx, rx) = mpsc::sync_channel(0);

        let joiner = thread::named("Core Event Loop", move || {
            let el = unwrap!(Core::new(), "Failed to create the event loop");
            let el_h = el.handle();

            let (core_tx, core_rx) = unwrap!(channel::channel(&el_h));
            let (net_tx, _net_rx) = unwrap!(channel::channel(&el_h));
            let core_tx_clone = core_tx.clone();

            tx.send(core_tx).unwrap();

            let client = unwrap!(Client::unregistered(core_tx_clone, net_tx),
                                 "Failed to create client");
            core::run(el, client, core_rx);
        });

        let tx = unwrap!(rx.recv());

        Session {
            inner: Arc::new(Inner {
                core_tx: Mutex::new(tx),
                _core_joiner: joiner,
                object_cache: Arc::new(Mutex::new(ObjectCache::default())),
            }),
        }
    }

    /// Create new account.
    pub fn create_account<S>(locator: S, password: S) -> Self
        where S: Into<String>
    {
        let (tx, rx) = mpsc::sync_channel(0);

        let locator = locator.into();
        let password = password.into();

        let joiner = thread::named("Core Event Loop", move || {
            let el = unwrap!(Core::new(), "Failed to create the event loop");
            let el_h = el.handle();

            let (core_tx, core_rx) = unwrap!(channel::channel(&el_h));
            let (net_tx, _net_rx) = unwrap!(channel::channel(&el_h));
            let core_tx_clone = core_tx.clone();

            tx.send(core_tx).unwrap();

            let client = unwrap!(Client::registered(&locator, &password, core_tx_clone, net_tx),
                                 "Failed to create client");
            core::run(el, client, core_rx);
        });

        let tx = unwrap!(rx.recv());

        Session {
            inner: Arc::new(Inner {
                core_tx: Mutex::new(tx),
                _core_joiner: joiner,
                object_cache: Arc::new(Mutex::new(ObjectCache::default())),
            }),
        }
    }

    /// Log in to existing account.
    pub fn log_in<S>(locator: S, password: S) -> Self
        where S: Into<String>
    {
        let (tx, rx) = mpsc::sync_channel(0);

        let locator = locator.into();
        let password = password.into();

        let joiner = thread::named("Core Event Loop", move || {
            let el = unwrap!(Core::new(), "Failed to create the event loop");
            let el_h = el.handle();

            let (core_tx, core_rx) = unwrap!(channel::channel(&el_h));
            let (net_tx, _net_rx) = unwrap!(channel::channel(&el_h));
            let core_tx_clone = core_tx.clone();
            tx.send(core_tx).unwrap();

            let client = unwrap!(Client::login(&locator, &password, core_tx_clone, net_tx),
                                 "Failed to create client");
            core::run(el, client, core_rx);
        });

        let tx = unwrap!(rx.recv());

        Session {
            inner: Arc::new(Inner {
                core_tx: Mutex::new(tx),
                _core_joiner: joiner,
                object_cache: Arc::new(Mutex::new(ObjectCache::default())),
            }),
        }
    }

    // /// Get SAFEdrive directory key.
    // pub fn safe_drive_dir(&self) -> &Option<Dir> {
    //     &self.safe_drive_dir
    // }

    // TODO(nbaksalyar): uncomment after implemented in Core
    // fn register_network_event_observer(&mut self, callback: extern "C" fn(i32)) {
    //     unwrap!(self.network_event_observers.lock()).push(callback);

    //     if self.network_thread.is_none() {
    //         let callbacks = self.network_event_observers.clone();

    //         let (tx, rx) = mpsc::channel();
    //         let cloned_tx = tx.clone();
    //         self.client.borrow_mut().add_network_event_observer(tx);

    //         let joiner = thread::named("FfiNetworkEventObserver", move || {
    //             while let Ok(event) = rx.recv() {
    //                 if let NetworkEvent::Terminated = event {
    //                     trace!("FFI exiting the network event notifier thread.");
    //                     break;
    //                 }

    //                 let callbacks = &*unwrap!(callbacks.lock());
    //                 info!("Informing {:?} to {} FFI network event observers.",
    //                       event,
    //                       callbacks.len());
    //                 let event_ffi_val = event.into();

    //                 for cb in callbacks {
    //                     cb(event_ffi_val);
    //                 }
    //             }
    //         });

    //         self.network_thread = Some((cloned_tx, joiner));
    //     }
    // }

    fn account_info(&self,
                    user_data: OpaqueCtx,
                    callback: extern "C" fn(int32_t, *mut c_void, uint64_t, uint64_t))
                    -> Result<(), FfiError> {
        self.send(CoreMsg::new(move |client| {
            Some(client.get_account_info(None)
                .map_err(move |e| callback(ffi_error_code!(e), user_data.0, 0, 0))
                .map(move |(data_stored, space_available)| {
                    callback(0, user_data.0, data_stored, space_available);
                })
                .into_box())
        }))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        debug!("Session is now being dropped.");
        if let Err(e) = self.send(CoreMsg::build_terminator()) {
            info!("Unexpected error in drop: {:?}", e);
        }
    }
}

/// Create a session as an unregistered client. This or any one of the other
/// companion functions to get a session must be called before initiating any
/// operation allowed by this crate.
#[no_mangle]
pub unsafe extern "C" fn create_unregistered_client(session_handle: *mut *mut Session) -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI create unregistered client.");

        let session = Session::unregistered();
        *session_handle = Box::into_raw(Box::new(session));
        0
    })
}

/// Create a registered client. This or any one of the other companion
/// functions to get a session must be called before initiating any operation
/// allowed by this crate. `session_handle` is a pointer to a pointer and must
/// point to a valid pointer not junk, else the consequences are undefined.
#[no_mangle]
pub unsafe extern "C" fn create_account(account_locator: *const u8,
                                        account_locator_len: usize,
                                        account_password: *const u8,
                                        account_password_len: usize,
                                        session_handle: *mut *mut Session)
                                        -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI create a client account.");

        let acc_locator = ffi_try!(helper::c_utf8_to_str(account_locator, account_locator_len));
        let acc_password = ffi_try!(helper::c_utf8_to_str(account_password, account_password_len));
        let session = Session::create_account(acc_locator, acc_password);

        *session_handle = Box::into_raw(Box::new(session));
        0
    })
}

/// Log into a registered client. This or any one of the other companion
/// functions to get a session must be called before initiating any operation
/// allowed by this crate. `session_handle` is a pointer to a pointer and must
/// point to a valid pointer not junk, else the consequences are undefined.
#[no_mangle]
pub unsafe extern "C" fn log_in(account_locator: *const u8,
                                account_locator_len: usize,
                                account_password: *const u8,
                                account_password_len: usize,
                                session_handle: *mut *mut Session)
                                -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI login a registered client.");

        let acc_locator = ffi_try!(helper::c_utf8_to_str(account_locator, account_locator_len));
        let acc_password = ffi_try!(helper::c_utf8_to_str(account_password, account_password_len));
        let session = Session::log_in(acc_locator, acc_password);

        *session_handle = Box::into_raw(Box::new(session));
        0
    })
}

// /// Register an observer to network events like Connected, Disconnected etc.
// as provided by the
// /// core module
// #[no_mangle]
// pub unsafe extern "C" fn register_network_event_observer(session: *mut
// Session,
// callback: extern
// "C" fn(i32))
//                                                          -> int32_t {
//     helper::catch_unwind_i32(|| {
//         trace!("FFI register a network event observer.");
//         unwrap!(*session.register_network_event_observer(callback));
//         0
//     })
// }


/// Return the amount of calls that were done to `get`
#[no_mangle]
pub unsafe extern "C" fn client_issued_gets(session: *const Session,
                                            user_data: *mut c_void,
                                            o_cb: extern "C" fn(int32_t, *mut c_void, int64_t))
                                            -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued GETs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(0, user_data.0, client.issued_gets() as int64_t);
            None
        })));
        0
    })
}

/// Return the amount of calls that were done to `put`
#[no_mangle]
pub unsafe extern "C" fn client_issued_puts(session: *const Session,
                                            user_data: *mut c_void,
                                            o_cb: extern "C" fn(int32_t, *mut c_void, int64_t))
                                            -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued PUTs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(0, user_data.0, client.issued_puts() as int64_t);
            None
        })));
        0
    })
}

/// Return the amount of calls that were done to `post`
#[no_mangle]
pub unsafe extern "C" fn client_issued_posts(session: *const Session,
                                             user_data: *mut c_void,
                                             o_cb: extern "C" fn(int32_t, *mut c_void, int64_t))
                                             -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued POSTs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(0, user_data.0, client.issued_posts() as int64_t);
            None
        })));
        0
    })
}

/// Return the amount of calls that were done to `delete`
#[no_mangle]
pub unsafe extern "C" fn client_issued_deletes(session: *const Session,
                                               user_data: *mut c_void,
                                               o_cb: extern "C" fn(int32_t, *mut c_void, int64_t))
                                               -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued DELETEs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(0, user_data.0, client.issued_deletes() as int64_t);
            None
        })));
        0
    })
}

/// Return the amount of calls that were done to `append`
#[no_mangle]
pub unsafe extern "C" fn client_issued_appends(session: *const Session,
                                               user_data: *mut c_void,
                                               o_cb: extern "C" fn(int32_t, *mut c_void, int64_t))
                                               -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued APPENDs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(0, user_data.0, client.issued_appends() as int64_t);
            None
        })));
        0
    })
}

/// Get data from the network. This is non-blocking. `data_stored` means number
/// of chunks Put. `space_available` means number of chunks which can still be
/// Put.
#[no_mangle]
pub unsafe extern "C" fn get_account_info(session: *const Session,
                                          user_data: *mut c_void,
                                          o_cb: extern "C" fn(int32_t,
                                                              *mut c_void,
                                                              uint64_t,
                                                              uint64_t))
                                          -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI get account information.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).account_info(user_data, o_cb));
        0
    })
}

/// Discard and clean up the previously allocated session. Use this only if the
/// session is obtained from one of the session obtainment functions in this
/// crate (`create_account`, `log_in`, `create_unregistered_client`). Using
/// `session` after a call to this functions is undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn drop_session(session: *mut Session) {
    let _ = Box::from_raw(session);
}

#[cfg(test)]
mod tests {
    use ffi::test_utils;
    use std::ptr;
    use super::*;

    #[test]
    fn create_account_and_log_in() {
        let acc_locator = test_utils::generate_random_cstring(10);
        let acc_password = test_utils::generate_random_cstring(10);

        {
            let mut session_handle: *mut Session = ptr::null_mut();

            unsafe {
                let session_handle_ptr = &mut session_handle;

                assert_eq!(create_account(acc_locator.as_ptr() as *const u8,
                                          10,
                                          acc_password.as_ptr() as *const u8,
                                          10,
                                          session_handle_ptr),
                           0);
            }

            assert!(!session_handle.is_null());
            unsafe { drop_session(session_handle) };
        }

        {
            let mut session_handle: *mut Session = ptr::null_mut();

            unsafe {
                let session_handle_ptr = &mut session_handle;

                assert_eq!(log_in(acc_locator.as_ptr() as *const u8,
                                  10,
                                  acc_password.as_ptr() as *const u8,
                                  10,
                                  session_handle_ptr),
                           0);
            }

            assert!(!session_handle.is_null());
            unsafe { drop_session(session_handle) };
        }
    }
}
