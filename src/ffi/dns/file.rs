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

        let session = (*session).clone();
        let user_data = OpaqueCtx(user_data);

        ffi_try!(session.send(CoreMsg::new(move |client| {
            let client2 = client.clone();
            let client3 = client.clone();

            let fut = operations::get_service_home_dir_id(client,
                                                          &long_name,
                                                          service_name,
                                                          None)
                .map_err(FfiError::from)
                .and_then(move |dir_id| {
                    dir_helper::get_file_by_path(&client2,
                                                 Some(&dir_id),
                                                 &file_path)
                        .map_err(FfiError::from)
                })
                .and_then(move |file| {
                    FileDetails::new(file,
                                     client3,
                                     offset,
                                     length,
                                     include_metadata)
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
                                               o_cb: extern "C" fn(int32_t, *mut c_void, *mut FileMetadata))
                                               -> int32_t {
    helper::catch_unwind_i32(|| {
        let long_name = ffi_try!(helper::c_utf8_to_string(long_name, long_name_len));
        let service_name = ffi_try!(helper::c_utf8_to_string(service_name, service_name_len));
        let file_path = ffi_try!(helper::c_utf8_to_string(file_path, file_path_len));

        trace!("FFI get file metadata for file located at given path starting from home \
                directory of \"//{}.{}\".",
               service_name,
               long_name);

        let session = (*session).clone();
        let user_data = OpaqueCtx(user_data);

        ffi_try!(session.send(CoreMsg::new(move |client| {
            let client2 = client.clone();

            let fut = operations::get_service_home_dir_id(client,
                                                          &long_name,
                                                          service_name,
                                                          None)
                .map_err(FfiError::from)
                .and_then(move |dir_id| {
                    dir_helper::get_file_by_path(&client2,
                                                 Some(&dir_id),
                                                 &file_path)
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
    /*
    use core::utility;
    use dns::dns_operations::DnsOperations;
    use ffi::app::App;
    use ffi::test_utils;
    use nfs::AccessLevel;
    use nfs::helper::directory_helper::DirectoryHelper;
    use nfs::helper::file_helper::FileHelper;
    use nfs::metadata::directory_key::DirectoryKey;
    use rust_sodium::crypto::box_;

    fn create_public_file(app: &App, file_name: String, file_content: Vec<u8>) -> DirectoryKey {
        let dir_helper = DirectoryHelper::new(app.get_client());
        let mut file_helper = FileHelper::new(app.get_client());

        let app_dir_key = unwrap!(app.get_app_dir_key());
        let mut app_dir = unwrap!(dir_helper.get(&app_dir_key));

        let (file_dir, _) = unwrap!(dir_helper.create("public-dir".to_string(),
                                                      vec![0u8; 0],
                                                      false,
                                                      AccessLevel::Public,
                                                      Some(&mut app_dir)));
        let dir_key = file_dir.get_key().clone();

        let bin_metadata = vec![0u8; 0];
        let mut writer = unwrap!(file_helper.create(file_name, bin_metadata, file_dir));
        unwrap!(writer.write(&file_content));
        let _ = unwrap!(writer.close());

        dir_key
    }

    fn register_service(app: &App,
                        service_name: String,
                        public_name: String,
                        dir_key: DirectoryKey) {
        let (msg_public_key, msg_secret_key) = box_::gen_keypair();
        let services = vec![(service_name, dir_key)];
        let client = app.get_client();

        let public_signing_key = *unwrap!(unwrap!(client.lock()).get_public_signing_key());
        let secret_signing_key = unwrap!(unwrap!(client.lock()).get_secret_signing_key()).clone();
        let dns_operation = unwrap!(DnsOperations::new(client));

        unwrap!(dns_operation.register_dns(public_name,
                                           &msg_public_key,
                                           &msg_secret_key,
                                           &services,
                                           vec![public_signing_key],
                                           &secret_signing_key,
                                           None));
    }

    #[test]
    fn get_public_file() {
        let app = test_utils::create_app(false);

        let file_name = "index.html";
        let file_content = "<html><title>Home</title></html>";

        let public_directory_key = create_public_file(&app,
                                                      file_name.to_string(),
                                                      file_content.as_bytes().to_vec());
        let service_name = "www";
        let public_name = unwrap!(utility::generate_random_string(10));

        register_service(&app,
                         service_name.to_string(),
                         public_name.clone(),
                         public_directory_key);

        let _ = unwrap!(super::get_file(&app, &public_name, service_name, file_name, 0, 0, false));

        // Fetch the file using a new client
        let app2 = test_utils::create_app(false);

        let _ = unwrap!(super::get_file(&app2, &public_name, service_name, file_name, 0, 0, false));

        // Fetch the file using an unregisterd client
        let app3 = test_utils::create_unregistered_app();

        let _ = unwrap!(super::get_file(&app3, &public_name, service_name, file_name, 0, 0, false));
    }
    */
}
