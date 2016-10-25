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

//! File operations

use core::CoreMsg;
use core::futures::FutureExt;
use dns::operations;
use ffi::{FfiError, OpaqueCtx, Session};
use ffi::file_details::{FileDetails, FileMetadata};
use ffi::helper;
use futures::Future;
use libc::{c_void, int32_t};
use nfs::helper::dir_helper;
use std::ptr;

/// Get file.
#[no_mangle]
pub unsafe extern "C" fn dns_get_file(session: *const Session,
                                      long_name: *const u8,
                                      long_name_len: usize,
                                      service_name: *const u8,
                                      service_name_len: usize,
                                      file_path: *const u8,
                                      file_path_len: usize,
                                      offset: i64,
                                      length: i64,
                                      include_metadata: bool,
                                      user_data: *mut c_void,
                                      o_cb: extern "C" fn(int32_t, *mut c_void, *mut FileDetails))
                                      -> int32_t {
    helper::catch_unwind_i32(|| {
        let long_name = ffi_try!(helper::c_utf8_to_string(long_name, long_name_len));
        let service_name = ffi_try!(helper::c_utf8_to_string(service_name, service_name_len));
        let file_path = ffi_try!(helper::c_utf8_to_string(file_path, file_path_len));

        trace!("FFI get file located at given path starting from home directory of \"//{}.{}\".",
               service_name,
               long_name);

        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let client2 = client.clone();
            let client3 = client.clone();

            let fut = operations::get_service_home_dir_id(client, &long_name, service_name, None)
                .map_err(FfiError::from)
                .and_then(move |dir_id| {
                    dir_helper::get_file_by_path(&client2, Some(&dir_id), &file_path)
                        .map_err(FfiError::from)
                })
                .and_then(move |file| {
                    FileDetails::new(file, client3, offset, length, include_metadata)
                })
                .map(move |details| {
                    let details = Box::into_raw(Box::new(details));
                    o_cb(0, user_data.0, details);
                })
                .map_err(move |err| {
                    o_cb(ffi_error_code!(err), user_data.0, ptr::null_mut());
                })
                .into_box();

            Some(fut)
        })));

        0
    })
}

/// Get file metadata.
#[no_mangle]
pub unsafe extern "C" fn dns_get_file_metadata(session: *const Session,
                                               long_name: *const u8,
                                               long_name_len: usize,
                                               service_name: *const u8,
                                               service_name_len: usize,
                                               file_path: *const u8,
                                               file_path_len: usize,
                                               user_data: *mut c_void,
                                               o_cb: extern "C" fn(int32_t,
                                                                   *mut c_void,
                                                                   *mut FileMetadata))
                                               -> int32_t {
    helper::catch_unwind_i32(|| {
        let long_name = ffi_try!(helper::c_utf8_to_string(long_name, long_name_len));
        let service_name = ffi_try!(helper::c_utf8_to_string(service_name, service_name_len));
        let file_path = ffi_try!(helper::c_utf8_to_string(file_path, file_path_len));

        trace!("FFI get file metadata for file located at given path starting from home \
                directory of \"//{}.{}\".",
               service_name,
               long_name);

        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let client2 = client.clone();

            let fut = operations::get_service_home_dir_id(client, &long_name, service_name, None)
                .map_err(FfiError::from)
                .and_then(move |dir_id| {
                    dir_helper::get_file_by_path(&client2, Some(&dir_id), &file_path)
                        .map_err(FfiError::from)
                })
                .and_then(move |file| {
                    let metadata = try!(FileMetadata::new(file.metadata()));
                    let metadata = Box::into_raw(Box::new(metadata));
                    o_cb(0, user_data.0, metadata);
                    Ok(())
                })
                .map_err(move |err| {
                    o_cb(ffi_error_code!(err), user_data.0, ptr::null_mut());
                })
                .into_box();

            Some(fut)
        })));

        0
    })
}

#[cfg(test)]
mod tests {
    use core::utility;
    use dns::operations;
    use ffi::Session;
    use ffi::file_details::FileDetails;
    use ffi::test_utils;
    use futures::Future;
    use libc::{c_void, int32_t};
    use nfs::{Dir, DirId};
    use nfs::helper::{dir_helper, file_helper};
    use rust_sodium::crypto::box_;
    use std::sync::mpsc;
    use std::time::Duration;
    use super::*;

    fn create_public_file(session: &Session, file_name: String, file_content: Vec<u8>) -> DirId {
        test_utils::run(session, |client| {
            let client2 = client.clone();

            let dir = Dir::new();
            dir_helper::create(client.clone(), &dir, None)
                .then(move |result| {
                    let dir_data_id = unwrap!(result);
                    let dir_id = (dir_data_id, None);

                    file_helper::create(client2, file_name, Vec::new(), dir_id.clone(), dir)
                        .map(move |writer| (writer, dir_id))
                })
                .then(move |result| {
                    let (writer, dir_id) = unwrap!(result);
                    writer.write(&file_content).map(move |_| (writer, dir_id))
                })
                .then(move |result| {
                    let (writer, dir_id) = unwrap!(result);
                    writer.close().map(move |_| dir_id)
                })
        })
    }

    fn register_service(session: &Session,
                        long_name: String,
                        service_name: String,
                        service_dir_id: DirId) {
        test_utils::run(session, move |client| {
            let (msg_pk, msg_sk) = box_::gen_keypair();
            let services = vec![(service_name, service_dir_id)];

            let sign_pk = unwrap!(client.public_signing_key());
            let sign_sk = unwrap!(client.secret_signing_key()).clone();

            operations::register_dns(client,
                                     long_name,
                                     msg_pk,
                                     msg_sk,
                                     &services,
                                     vec![sign_pk],
                                     sign_sk,
                                     None)
        })
    }

    #[test]
    fn get_public_file() {
        let timeout = Duration::from_millis(10_000);
        let session = test_utils::create_session();

        let file_name = "index.html".to_string();
        let file_content = "<html><title>Home</title></html>";

        let public_dir_id = create_public_file(&session,
                                               file_name.clone(),
                                               file_content.as_bytes().to_vec());

        let long_name = unwrap!(utility::generate_random_string(10));
        let service_name = "www".to_string();

        register_service(&session,
                         long_name.clone(),
                         service_name.clone(),
                         public_dir_id);

        let (tx, rx) = mpsc::channel::<()>();

        let long_name = test_utils::string_to_c_utf8(long_name);
        let service_name = test_utils::string_to_c_utf8(service_name);
        let file_name = test_utils::string_to_c_utf8(file_name);

        extern "C" fn callback(error: int32_t,
                               user_data: *mut c_void,
                               _file_details_ptr: *mut FileDetails) {
            assert_eq!(error, 0);
            unsafe { test_utils::send_via_user_data(user_data) }
        }

        unsafe {
            let result = dns_get_file(&session,
                                      long_name.ptr(),
                                      long_name.len(),
                                      service_name.ptr(),
                                      service_name.len(),
                                      file_name.ptr(),
                                      file_name.len(),
                                      0,
                                      0,
                                      false,
                                      test_utils::sender_as_user_data(&tx),
                                      callback);
            assert_eq!(result, 0);
            let _ = unwrap!(rx.recv_timeout(timeout));
        };

        // Fetch the file using a new client
        let session2 = test_utils::create_session();

        unsafe {
            let result = dns_get_file(&session2,
                                      long_name.ptr(),
                                      long_name.len(),
                                      service_name.ptr(),
                                      service_name.len(),
                                      file_name.ptr(),
                                      file_name.len(),
                                      0,
                                      0,
                                      false,
                                      test_utils::sender_as_user_data(&tx),
                                      callback);
            assert_eq!(result, 0);
            let _ = unwrap!(rx.recv_timeout(timeout));
        };

        // Fetch the file using an unregisterd client
        let session3 = Session::unregistered();

        unsafe {
            let result = dns_get_file(&session3,
                                      long_name.ptr(),
                                      long_name.len(),
                                      service_name.ptr(),
                                      service_name.len(),
                                      file_name.ptr(),
                                      file_name.len(),
                                      0,
                                      0,
                                      false,
                                      test_utils::sender_as_user_data(&tx),
                                      callback);
            assert_eq!(result, 0);
            let _ = unwrap!(rx.recv_timeout(timeout));
        };
    }
}
