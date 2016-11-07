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
use core::futures::FutureExt;
use ffi::{App, Session};
use ffi::launcher_config;
use ffi::object_cache::ObjectCache;
use futures::{Future, IntoFuture};
use std::ffi::CString;
use std::fmt::Debug;
use std::os::raw::c_void;
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

// Run the given closure inside the session event loop. The return value of
// the closure is returned immediately.
pub fn run_now<F, R>(session: &Session, f: F) -> R
    where F: FnOnce(&Client, &ObjectCache) -> R + Send + 'static,
          R: Send + 'static
{
    let (tx, rx) = mpsc::channel();

    unwrap!(session.send(move |client, object_cache| {
        unwrap!(tx.send(f(client, object_cache)));
        None
    }));

    unwrap!(rx.recv())
}

pub fn create_app(session: &Session, has_safe_drive_access: bool) -> App {
    let app_name = "Test App".to_string();
    let app_id = unwrap!(utility::generate_random_string(10));
    let vendor = "Test Vendor".to_string();

    run(session, move |client, _| {
        launcher_config::app(client, app_name, app_id, vendor, has_safe_drive_access)
    })
}

// Convert a `mpsc::Sender<T>` to a void ptr which can be passed as user data to
// ffi functions
pub fn sender_as_user_data<T>(tx: &Sender<T>) -> *mut c_void {
    let ptr: *const _ = tx;
    ptr as *mut c_void
}

// Send through a `mpsc::Sender` pointed to by the user data pointer.
pub unsafe fn send_via_user_data<T>(user_data: *mut c_void, value: T)
    where T: Send
{
    let tx = user_data as *mut Sender<T>;
    unwrap!((*tx).send(value));
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

// Call a FFI function and block until its callback gets called.
// Use this if the callback accepts no arguments in addition to user_data
// and error_code.
pub fn call_0<F>(f: F) -> Result<(), i32>
    where F: FnOnce(*mut c_void, unsafe extern "C" fn(*mut c_void, i32))
{
    let (tx, rx) = mpsc::channel::<i32>();
    f(sender_as_user_data(&tx), callback_0);

    let error = unwrap!(rx.recv());
    if error == 0 { Ok(()) } else { Err(error) }
}

// Call a FFI function and block until its callback gets called, then return
// the argument which were passed to that callback.
// Use this if the callback accepts one argument in addition to user_data
// and error_code.
pub unsafe fn call_1<F, T>(f: F) -> Result<T, i32>
    where F: FnOnce(*mut c_void, unsafe extern "C" fn(*mut c_void, i32, T))
{
    let (tx, rx) = mpsc::channel::<(i32, SendWrapper<T>)>();
    f(sender_as_user_data(&tx), callback_1::<T>);

    let (error, args) = unwrap!(rx.recv());
    if error == 0 { Ok(args.0) } else { Err(error) }
}

// Call a FFI function and block until its callback gets called, then return
// the arguments which were passed to that callback in a tuple.
// Use this if the callback accepts three arguments in addition to user_data and
// error_code.
pub unsafe fn call_3<F, T0, T1, T2>(f: F) -> Result<(T0, T1, T2), i32>
    where F: FnOnce(*mut c_void,
                    unsafe extern "C" fn(*mut c_void, i32, T0, T1, T2))
{
    let (tx, rx) = mpsc::channel::<(i32, SendWrapper<(T0, T1, T2)>)>();
    f(sender_as_user_data(&tx), callback_3::<T0, T1, T2>);

    let (error, args) = unwrap!(rx.recv());
    if error == 0 { Ok(args.0) } else { Err(error) }
}

// Call a FFI function and block until its callback gets called, then return
// the arguments which were passed to that callback converted to Vec<u8>.
// The callbacks must accept three arguments (in addition to user_data and
// error_code): pointer to the begining of the data (`*mut u8`), lengths
// (`usize`)
// and capacity (`usize`).
pub unsafe fn call_vec_u8<F>(f: F) -> Result<Vec<u8>, i32>
    where F: FnOnce(*mut c_void,
                    unsafe extern "C" fn(*mut c_void, i32, *mut u8, usize, usize))
{
    call_3(f).map(|(ptr, len, cap)| Vec::from_raw_parts(ptr, len, cap))
}

unsafe extern "C" fn callback_0(user_data: *mut c_void, error: i32) {
    send_via_user_data(user_data, error)
}

unsafe extern "C" fn callback_1<T>(user_data: *mut c_void, error: i32, arg: T) {
    send_via_user_data(user_data, (error, SendWrapper(arg)))
}

unsafe extern "C" fn callback_3<T0, T1, T2>(user_data: *mut c_void,
                                            error: i32,
                                            arg0: T0,
                                            arg1: T1,
                                            arg2: T2) {
    send_via_user_data(user_data, (error, SendWrapper((arg0, arg1, arg2))))
}

// Unsafe wrapper for passing non-Send types through mpsc channels.
// Use with caution!
struct SendWrapper<T>(T);
unsafe impl<T> Send for SendWrapper<T> {}
