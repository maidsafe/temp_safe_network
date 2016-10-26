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

use core::{self, Client, CoreMsg, CoreMsgTx, NetworkEvent};
use core::futures::FutureExt;
use ffi::{FfiError, OpaqueCtx};
use ffi::object_cache::ObjectCache;
use futures::{Future, IntoFuture};
use futures::stream::Stream;
use libc::{c_void, int32_t, int64_t, uint64_t};
use maidsafe_utilities::thread::{self, Joiner};
use std::sync::{Arc, Mutex, mpsc};
use super::helper;
use tokio_core::channel;
use tokio_core::reactor::Core;

macro_rules! try_tx {
    ($result:expr, $tx:ident) => {
        match $result {
            Ok(res) => res,
            Err(e) => { return unwrap!($tx.send(Err(FfiError::from(e)))); }
        }
    }
}

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

    /// Send the given closure to be executed on the core event loop.
    pub fn send_fn<F, I>(&self, f: F) -> Result<(), FfiError>
        where F: FnOnce(&Client) -> Option<I> + Send + 'static,
              I: IntoFuture<Item=(), Error=()> + 'static
    {
        self.send(CoreMsg::new(move |client| {
            f(client).map(|i| i.into_future().into_box())
        }))
    }

    /// Returns an object cache tied to the session
    pub fn object_cache(&self) -> Arc<Mutex<ObjectCache>> {
        self.inner.object_cache.clone()
    }

    /// Create unregistered client.
    pub fn unregistered<NetObs>(mut network_observer: NetObs) -> Result<Self, FfiError>
        where NetObs: FnMut(Result<NetworkEvent, FfiError>) + Send + 'static
    {
        let (tx, rx) = mpsc::sync_channel(0);

        let joiner = thread::named("Core Event Loop", move || {
            let el = try_tx!(Core::new(), tx);
            let el_h = el.handle();

            let (core_tx, core_rx) = try_tx!(channel::channel(&el_h), tx);
            let (net_tx, net_rx) = try_tx!(channel::channel(&el_h), tx);

            let net_obs_fut =
                net_rx.then(move |net_event| {
                        Ok(network_observer(net_event.map_err(FfiError::from)))
                    })
                    .for_each(|_| Ok(()));

            el_h.spawn(net_obs_fut);

            let core_tx_clone = core_tx.clone();

            let client = try_tx!(Client::unregistered(core_tx_clone, net_tx), tx);
            unwrap!(tx.send(Ok(core_tx)));

            core::run(el, client, core_rx);
        });

        let core_tx = try!(try!(rx.recv()));

        Ok(Session {
            inner: Arc::new(Inner {
                core_tx: Mutex::new(core_tx),
                _core_joiner: joiner,
                object_cache: Arc::new(Mutex::new(ObjectCache::default())),
            }),
        })
    }

    /// Create new account.
    pub fn create_account<S, NetObs>(locator: S,
                                     password: S,
                                     mut network_observer: NetObs)
                                     -> Result<Self, FfiError>
        where S: Into<String>,
              NetObs: FnMut(Result<NetworkEvent, FfiError>) + Send + 'static
    {
        let (tx, rx) = mpsc::sync_channel(0);

        let locator = locator.into();
        let password = password.into();

        let joiner = thread::named("Core Event Loop", move || {
            let el = try_tx!(Core::new(), tx);
            let el_h = el.handle();

            let (core_tx, core_rx) = try_tx!(channel::channel(&el_h), tx);
            let (net_tx, net_rx) = try_tx!(channel::channel(&el_h), tx);
            let core_tx_clone = core_tx.clone();

            let net_obs_fut =
                net_rx.then(move |net_event| {
                        Ok(network_observer(net_event.map_err(FfiError::from)))
                    })
                    .for_each(|_| Ok(()));
            el_h.spawn(net_obs_fut);

            let client = try_tx!(Client::registered(&locator, &password, core_tx_clone, net_tx),
                                 tx);

            unwrap!(tx.send(Ok(core_tx)));

            core::run(el, client, core_rx);
        });

        let core_tx = try!(try!(rx.recv()));

        Ok(Session {
            inner: Arc::new(Inner {
                core_tx: Mutex::new(core_tx),
                _core_joiner: joiner,
                object_cache: Arc::new(Mutex::new(ObjectCache::default())),
            }),
        })
    }

    /// Log in to existing account.
    pub fn log_in<S, NetObs>(locator: S,
                             password: S,
                             mut network_observer: NetObs)
                             -> Result<Self, FfiError>
        where S: Into<String>,
              NetObs: FnMut(Result<NetworkEvent, FfiError>) + Send + 'static
    {
        let (tx, rx) = mpsc::sync_channel(0);

        let locator = locator.into();
        let password = password.into();

        let joiner = thread::named("Core Event Loop", move || {
            let el = try_tx!(Core::new(), tx);
            let el_h = el.handle();

            let (core_tx, core_rx) = try_tx!(channel::channel(&el_h), tx);
            let (net_tx, net_rx) = try_tx!(channel::channel(&el_h), tx);
            let core_tx_clone = core_tx.clone();

            let net_obs_fut =
                net_rx.then(move |net_event| {
                        Ok(network_observer(net_event.map_err(FfiError::from)))
                    })
                    .for_each(|_| Ok(()));
            el_h.spawn(net_obs_fut);

            let client = try_tx!(Client::login(&locator, &password, core_tx_clone, net_tx),
                                 tx);

            unwrap!(tx.send(Ok(core_tx)));

            core::run(el, client, core_rx);
        });

        let core_tx = try!(try!(rx.recv()));

        Ok(Session {
            inner: Arc::new(Inner {
                core_tx: Mutex::new(core_tx),
                _core_joiner: joiner,
                object_cache: Arc::new(Mutex::new(ObjectCache::default())),
            }),
        })
    }

    // /// Get SAFEdrive directory key.
    // pub fn safe_drive_dir(&self) -> &Option<Dir> {
    //     &self.safe_drive_dir
    // }

    fn account_info(&self,
                    user_data: OpaqueCtx,
                    callback: unsafe extern "C" fn(*mut c_void, int32_t, uint64_t, uint64_t))
                    -> Result<(), FfiError> {
        self.send(CoreMsg::new(move |client| {
            Some(client.get_account_info(None)
                .map_err(move |e| unsafe { callback(user_data.0, ffi_error_code!(e), 0, 0) })
                .map(move |(data_stored, space_available)| {
                    unsafe { callback(user_data.0, 0, data_stored, space_available) }
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
pub unsafe extern "C" fn create_unregistered_client(user_data: *mut c_void,
                                                    obs_cb: unsafe extern "C" fn(*mut c_void,
                                                                                 int32_t,
                                                                                 int32_t),
                                                    session_handle: *mut *mut Session)
                                                    -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI create unregistered client.");
        let user_data = OpaqueCtx(user_data);
        let session = ffi_try!(Session::unregistered(move |net_event| {
            match net_event {
                Ok(event) => obs_cb(user_data.0, 0, event.into()),
                Err(e) => obs_cb(user_data.0, ffi_error_code!(e), 0),
            }
        }));
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
                                        session_handle: *mut *mut Session,
                                        user_data: *mut c_void,
                                        o_network_obs_cb: unsafe extern "C" fn(*mut c_void,
                                                                               int32_t,
                                                                               int32_t))
                                        -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI create a client account.");

        let acc_locator = ffi_try!(helper::c_utf8_to_str(account_locator, account_locator_len));
        let acc_password = ffi_try!(helper::c_utf8_to_str(account_password, account_password_len));
        let user_data = OpaqueCtx(user_data);
        let session =
            ffi_try!(Session::create_account(acc_locator, acc_password, move |net_event| {
                match net_event {
                    Ok(event) => o_network_obs_cb(user_data.0, 0, event.into()),
                    Err(e) => o_network_obs_cb(user_data.0, ffi_error_code!(e), 0),
                }
            }));

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
                                session_handle: *mut *mut Session,
                                user_data: *mut c_void,
                                o_network_obs_cb: unsafe extern "C" fn(*mut c_void,
                                                                       int32_t,
                                                                       int32_t))
                                -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI login a registered client.");

        let acc_locator = ffi_try!(helper::c_utf8_to_str(account_locator, account_locator_len));
        let acc_password = ffi_try!(helper::c_utf8_to_str(account_password, account_password_len));
        let user_data = OpaqueCtx(user_data);
        let session = ffi_try!(Session::log_in(acc_locator, acc_password, move |net_event| {
            match net_event {
                Ok(event) => o_network_obs_cb(user_data.0, 0, event.into()),
                Err(e) => o_network_obs_cb(user_data.0, ffi_error_code!(e), 0),
            }
        }));

        *session_handle = Box::into_raw(Box::new(session));
        0
    })
}

/// Return the amount of calls that were done to `get`
#[no_mangle]
pub unsafe extern "C" fn client_issued_gets(session: *const Session,
                                            user_data: *mut c_void,
                                            o_cb: unsafe extern "C" fn(*mut c_void,
                                                                       int32_t,
                                                                       int64_t))
                                            -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued GETs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(user_data.0, 0, client.issued_gets() as int64_t);
            None
        })));
        0
    })
}

/// Return the amount of calls that were done to `put`
#[no_mangle]
pub unsafe extern "C" fn client_issued_puts(session: *const Session,
                                            user_data: *mut c_void,
                                            o_cb: unsafe extern "C" fn(*mut c_void,
                                                                       int32_t,
                                                                       int64_t))
                                            -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued PUTs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(user_data.0, 0, client.issued_puts() as int64_t);
            None
        })));
        0
    })
}

/// Return the amount of calls that were done to `post`
#[no_mangle]
pub unsafe extern "C" fn client_issued_posts(session: *const Session,
                                             user_data: *mut c_void,
                                             o_cb: unsafe extern "C" fn(int32_t,
                                                                        *mut c_void,
                                                                        int64_t))
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
                                               o_cb: unsafe extern "C" fn(*mut c_void,
                                                                          int32_t,
                                                                          int64_t))
                                               -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued DELETEs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(user_data.0, 0, client.issued_deletes() as int64_t);
            None
        })));
        0
    })
}

/// Return the amount of calls that were done to `append`
#[no_mangle]
pub unsafe extern "C" fn client_issued_appends(session: *const Session,
                                               user_data: *mut c_void,
                                               o_cb: unsafe extern "C" fn(*mut c_void,
                                                                          int32_t,
                                                                          int64_t))
                                               -> i32 {
    helper::catch_unwind_i32(|| {
        trace!("FFI retrieve client issued APPENDs.");
        let user_data = OpaqueCtx(user_data);
        ffi_try!((*session).send(CoreMsg::new(move |client| {
            o_cb(user_data.0, 0, client.issued_appends() as int64_t);
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
                                          o_cb: unsafe extern "C" fn(*mut c_void,
                                                                     int32_t,
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
    use libc::c_void;
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
                                          session_handle_ptr,
                                          ptr::null_mut(),
                                          net_event_cb),
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
                                  session_handle_ptr,
                                  ptr::null_mut(),
                                  net_event_cb),
                           0);
            }

            assert!(!session_handle.is_null());
            unsafe { drop_session(session_handle) };
        }

        unsafe extern "C" fn net_event_cb(_user_data: *mut c_void, err_code: i32, _event: i32) {
            assert_eq!(err_code, 0);
        }
    }
}
