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

use core::{Client, CoreMsg, FutureExt};
use ffi::{App, FfiError, FfiFuture, OpaqueCtx, Session};
use ffi::file_details::{FileDetails, FileMetadata};
use ffi::helper;
use ffi::object_cache::AppHandle;
use futures::Future;
use libc::{c_void, int32_t};
use nfs::NfsError;
use nfs::helper::{dir_helper, file_helper};
use nfs::helper::writer::Mode;
use std::ptr;
use time;

/// Delete a file.
#[no_mangle]
pub unsafe extern "C" fn nfs_delete_file(session: *const Session,
                                         app_handle: AppHandle,
                                         file_path: *const u8,
                                         file_path_len: usize,
                                         is_shared: bool,
                                         user_data: *mut c_void,
                                         o_cb: extern "C" fn(int32_t, *mut c_void))
                                         -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI delete file, given the path.");
        let file_path = ffi_try!(helper::c_utf8_to_str(file_path, file_path_len));

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = delete_file(&client, &app, file_path, is_shared)
                        .then(move |res| {
                            o_cb(ffi_result_code!(res), user_data.0);
                            Ok(())
                        })
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(ffi_error_code!(e), user_data.0);
                    None
                }
            }
        })));

        0
    })
}

/// Get file. The returned FileDetails pointer must be disposed of by calling
/// `file_details_drop` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn nfs_get_file(session: *const Session,
                                      app_handle: AppHandle,
                                      offset: i64,
                                      length: i64,
                                      file_path: *const u8,
                                      file_path_len: usize,
                                      is_path_shared: bool,
                                      include_metadata: bool,
                                      user_data: *mut c_void,
                                      o_cb: extern "C" fn(int32_t, *mut c_void, *mut FileDetails))
                                      -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI get file, given the path.");

        let file_path = ffi_try!(helper::c_utf8_to_str(file_path, file_path_len));

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = get_file(&client,
                                       &app,
                                       file_path,
                                       is_path_shared,
                                       offset,
                                       length,
                                       include_metadata)
                        .map(move |response| {
                            let details_handle = Box::into_raw(Box::new(response));
                            o_cb(0, user_data.0, details_handle);
                        })
                        .map_err(move |e| {
                            o_cb(ffi_error_code!(e), user_data.0, ptr::null_mut());
                        })
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(ffi_error_code!(e), user_data.0, ptr::null_mut());
                    None
                }
            }
        })));

        0
    })
}

/// Modify name, metadata or content of the file.
#[no_mangle]
pub unsafe extern "C" fn nfs_modify_file(session: *const Session,
                                         app_handle: AppHandle,
                                         file_path: *const u8,
                                         file_path_len: usize,
                                         is_shared: bool,
                                         new_name: *const u8,
                                         new_name_len: usize,
                                         new_metadata: *const u8,
                                         new_metadata_len: usize,
                                         new_content: *const u8,
                                         new_content_len: usize,
                                         user_data: *mut c_void,
                                         o_cb: extern "C" fn(int32_t, *mut c_void))
                                         -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI modify file, given the path.");

        let file_path = ffi_try!(helper::c_utf8_to_str(file_path, file_path_len));
        let new_name = ffi_try!(helper::c_utf8_to_opt_string(new_name, new_name_len));
        let new_metadata = helper::u8_ptr_to_opt_vec(new_metadata, new_metadata_len);
        let new_content = helper::u8_ptr_to_opt_vec(new_content, new_content_len);

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = modify_file(&client,
                                          &app,
                                          file_path,
                                          is_shared,
                                          new_name,
                                          new_metadata,
                                          new_content)
                        .then(move |res| {
                            o_cb(ffi_result_code!(res), user_data.0);
                            Ok(())
                        })
                        .into_box();

                    Some(fut)
                }
                Err(e) => {
                    o_cb(ffi_error_code!(e), user_data.0);
                    None
                }
            }
        })));
        0
    })
}

/// Move or copy a file.
#[no_mangle]
pub unsafe extern "C" fn nfs_move_file(session: *const Session,
                                       app_handle: AppHandle,
                                       src_path: *const u8,
                                       src_path_len: usize,
                                       is_src_path_shared: bool,
                                       dst_path: *const u8,
                                       dst_path_len: usize,
                                       is_dst_path_shared: bool,
                                       retain_src: bool,
                                       user_data: *mut c_void,
                                       o_cb: extern "C" fn(int32_t, *mut c_void))
                                       -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI move file, from {:?} to {:?}.", src_path, dst_path);

        let src_path = ffi_try!(helper::c_utf8_to_str(src_path, src_path_len));
        let dst_path = ffi_try!(helper::c_utf8_to_str(dst_path, dst_path_len));

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = move_file(&client,
                                        &app,
                                        src_path,
                                        is_src_path_shared,
                                        dst_path,
                                        is_dst_path_shared,
                                        retain_src)
                        .then(move |res| {
                            o_cb(ffi_result_code!(res), user_data.0);
                            Ok(())
                        })
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(ffi_error_code!(e), user_data.0);
                    None
                }
            }
        })));

        0
    })
}

/// Get file metadata. The returned pointer must be disposed of by calling
/// `file_metadata_drop` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn nfs_get_file_metadata(session: *const Session,
                                               app_handle: AppHandle,
                                               file_path: *const u8,
                                               file_path_len: usize,
                                               is_path_shared: bool,
                                               user_data: *mut c_void,
                                               o_cb: extern "C" fn(int32_t,
                                                                   *mut c_void,
                                                                   *mut FileMetadata))
                                               -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI get file metadata, given the path.");
        let file_path = ffi_try!(helper::c_utf8_to_str(file_path, file_path_len));

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = get_file_metadata(&client, &app, file_path, is_path_shared)
                        .map(move |metadata| {
                            let metadata_handle = Box::into_raw(Box::new(metadata));
                            o_cb(0, user_data.0, metadata_handle);
                        })
                        .map_err(move |e| {
                            o_cb(ffi_error_code!(e), user_data.0, ptr::null_mut());
                        })
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(ffi_error_code!(e), user_data.0, ptr::null_mut());
                    None
                }
            }
        })));
        0
    })
}

fn delete_file(client: &Client, app: &App, file_path: &str, is_shared: bool) -> Box<FfiFuture<()>> {
    let c2 = client.clone();
    helper::dir_and_file(&client.clone(), app, file_path, is_shared)
        .and_then(move |(mut dir, dir_meta, filename)| {
            file_helper::delete(c2, &filename, &dir_meta.id(), &mut dir).map_err(FfiError::from)
        })
        .into_box()
}

fn get_file(client: &Client,
            app: &App,
            file_path: &str,
            is_path_shared: bool,
            offset: i64,
            length: i64,
            include_metadata: bool)
            -> Box<FfiFuture<FileDetails>> {
    let c2 = client.clone();
    helper::dir_and_file(client, app, file_path, is_path_shared)
        .and_then(move |(dir, _dir_meta, filename)| {
            let file = fry!(dir.find_file(&filename).ok_or(FfiError::InvalidPath));
            FileDetails::new(file.clone(), c2, offset, length, include_metadata)
        })
        .into_box()
}

fn modify_file(client: &Client,
               app: &App,
               file_path: &str,
               is_shared: bool,
               new_name: Option<String>,
               new_metadata: Option<Vec<u8>>,
               new_content: Option<Vec<u8>>)
               -> Box<FfiFuture<()>> {
    if new_name.is_none() && new_metadata.is_none() && new_content.is_none() {
        return err!(FfiError::from("Optional parameters could not be parsed"));
    }

    let c2 = client.clone();

    let fut = helper::dir_and_file(client, app, file_path, is_shared)
        .and_then(move |(mut dir, dir_metadata, filename)| {
            let mut file = fry!(dir.find_file(&filename)
                .cloned()
                .ok_or(FfiError::InvalidPath));

            let mut metadata_updated = false;
            if let Some(name) = new_name {
                file.metadata_mut().set_name(name);
                metadata_updated = true;
            }
            if let Some(metadata) = new_metadata {
                file.metadata_mut().set_user_metadata(metadata);
                metadata_updated = true;
            }

            if metadata_updated {
                file.metadata_mut().set_modified_time(time::now_utc());
                file_helper::update_metadata(c2,
                                             &filename,
                                             file.clone(),
                                             &dir_metadata.id(),
                                             &mut dir)
                    .map_err(FfiError::from)
                    .map(move |_| (file, dir_metadata, dir))
                    .into_box()
            } else {
                ok!((file, dir_metadata, dir))
            }
        });

    if let Some(content) = new_content {
        let c2 = client.clone();

        fut.and_then(move |(file, dir_metadata, dir)| {
                file_helper::update_content(c2,
                                            file.clone(),
                                            Mode::Overwrite,
                                            dir_metadata.id(),
                                            dir)
                    .map_err(FfiError::from)
            })
            .and_then(move |writer| {
                writer.write(&content[..])
                    .and_then(move |_| writer.close())
                    .map_err(FfiError::from)
            })
            .map(|_| ())
            .into_box()
    } else {
        fut.map(|_| ()).into_box()
    }
}

fn move_file(client: &Client,
             app: &App,
             src_path: &str,
             is_src_path_shared: bool,
             dst_path: &str,
             is_dst_path_shared: bool,
             retain_src: bool)
             -> Box<FfiFuture<()>> {
    let c2 = client.clone();
    let c3 = client.clone();

    helper::dir_and_file(&client, app, src_path, is_src_path_shared)
        .join(helper::dir(&client, app, dst_path, is_dst_path_shared))
        .and_then(move |((mut src_dir, src_dir_meta, src_filename), (mut dst_dir, dst_dir_meta))| {
            if dst_dir.find_file(&src_filename).is_some() {
                return err!(FfiError::from(NfsError::FileAlreadyExistsWithSameName));
            }

            let file = match src_dir.find_file(&src_filename).cloned() {
                Some(file) => file,
                None => return err!(FfiError::PathNotFound),
            };

            let _ = fry!(dst_dir.upsert_file(file));

            let fut = dir_helper::update(c2.clone(), &dst_dir_meta.id(), &dst_dir)
                .map_err(FfiError::from);

            if !retain_src {
                let _ = fry!(src_dir.remove_file(&src_filename));

                fut.and_then(move |_| {
                        dir_helper::update(c3, &src_dir_meta.id(), &src_dir).map_err(FfiError::from)
                    })
                    .into_box()
            } else {
                fut.into_box()
            }
        })
        .into_box()
}

fn get_file_metadata(client: &Client,
                     app: &App,
                     file_path: &str,
                     is_path_shared: bool)
                     -> Box<FfiFuture<FileMetadata>> {
    helper::dir_and_file(client, app, file_path, is_path_shared)
        .and_then(move |(dir, _dir_meta, filename)| {
            let file = try!(dir.find_file(&filename).ok_or(FfiError::InvalidPath));
            FileMetadata::new(file.metadata())
        })
        .into_box()
}

#[cfg(test)]
mod tests {
    use core::{Client, CoreMsg, FutureExt};
    use ffi::{App, FfiError, FfiFuture, test_utils};
    use futures::Future;
    use nfs::helper::{dir_helper, file_helper};
    use std::{slice, str};
    use std::sync::mpsc;
    use std::time::Duration;

    fn create_test_file(client: &Client, app: &App, name: &str) -> Box<FfiFuture<()>> {
        let app_root_dir_id = unwrap!(app.app_dir());
        let c2 = client.clone();

        let name = name.to_owned();

        dir_helper::get(client.clone(), &app_root_dir_id)
            .then(move |res| {
                let app_root_dir = unwrap!(res);
                file_helper::create(c2, name, Vec::new(), app_root_dir_id, app_root_dir)
            })
            .then(move |res| {
                let writer = unwrap!(res);
                let data = vec![10u8; 20];
                writer.write(&data[..])
                    .then(move |result| {
                        let _ = unwrap!(result);
                        writer.close()
                    })
            })
            .map(|_| ())
            .map_err(FfiError::from)
            .into_box()
    }

    #[test]
    fn delete_file() {
        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);
        let app2 = app.clone();
        let app_dir_id = unwrap!(app.app_dir());
        let app_dir_id2 = app_dir_id.clone();

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();
            let c5 = client.clone();

            let fut = create_test_file(&client, &app, "test_file.txt")
                .then(move |res| {
                    let _ = unwrap!(res, "can't create file test_file.txt");
                    dir_helper::get(c2, &app_dir_id)
                })
                .then(move |res| {
                    let app_root_dir = unwrap!(res);
                    assert_eq!(app_root_dir.files().len(), 1);
                    assert!(app_root_dir.find_file("test_file.txt").is_some());
                    super::delete_file(&c3, &app, "/test_file.txt", false)
                })
                .then(move |res| {
                    assert!(res.is_ok(), "can't delete file test_file.txt");
                    dir_helper::get(c4, &app_dir_id2)
                })
                .then(move |res| {
                    let app_root_dir = unwrap!(res);
                    assert_eq!(app_root_dir.files().len(), 0);
                    super::delete_file(&c5, &app2, "/test_file.txt", false)
                })
                .then(move |res| {
                    assert!(res.is_err(),
                            "deleting file /test_file.txt should return an error");
                    unwrap!(tx2.send(()));
                    Ok(())
                })
                .into_box();
            Some(fut)
        })));

        let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
        unwrap!(sess.send(CoreMsg::build_terminator()));
    }

    #[test]
    fn get_file() {
        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);
        let app2 = app.clone();

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();

            let fut = create_test_file(&client.clone(), &app, "test_file.txt")
                .then(move |res| {
                    let _ = unwrap!(res);

                    super::get_file(&c2, &app, "/test_file.txt", false, 0, 0, true)
                })
                .then(move |res| {
                    let details = unwrap!(res);
                    unsafe {
                        let metadata = unwrap!(details.metadata.as_ref());
                        let name = slice::from_raw_parts(metadata.name, metadata.name_len);
                        let name = String::from_utf8(name.to_owned()).unwrap();
                        assert_eq!(name, "test_file.txt");
                    }
                    super::get_file(&c3, &app2, "/does_not_exist", false, 0, 0, true)
                })
                .then(move |res| {
                    assert!(res.is_err(),
                            "getting file /does_not_exist should return error");
                    unwrap!(tx2.send(()));
                    Ok(())
                })
                .into_box();
            Some(fut)
        })));

        let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
        unwrap!(sess.send(CoreMsg::build_terminator()));
    }

    #[test]
    fn file_rename() {
        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);
        let app_dir_id = unwrap!(app.app_dir());
        let app_dir_id2 = app_dir_id.clone();

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();

            let fut = create_test_file(&client.clone(), &app, "test_file.txt")
                .then(move |res| {
                    let _ = unwrap!(res, "can't create test_file.txt");
                    dir_helper::get(c2, &app_dir_id)
                })
                .then(move |res| {
                    let app_root_dir = unwrap!(res);
                    assert_eq!(app_root_dir.files().len(), 1);
                    assert!(app_root_dir.find_file("test_file.txt").is_some());

                    super::modify_file(&c3,
                                       &app,
                                       "/test_file.txt",
                                       false,
                                       Some("new_test_file.txt".to_string()),
                                       None,
                                       None)
                })
                .then(move |res| {
                    assert!(res.is_ok());
                    dir_helper::get(c4, &app_dir_id2)
                })
                .then(move |res| {
                    let app_root_dir = unwrap!(res);
                    assert_eq!(app_root_dir.files().len(), 1);
                    assert!(app_root_dir.find_file("test_file.txt").is_none());
                    assert!(app_root_dir.find_file("new_test_file.txt").is_some());

                    unwrap!(tx2.send(()));
                    Ok(())
                })
                .into_box();

            Some(fut)
        })));

        let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
        unwrap!(sess.send(CoreMsg::build_terminator()));
    }

    #[test]
    fn file_update_user_metadata() {
        const METADATA: &'static str = "user metadata";

        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);
        let app_dir_id = unwrap!(app.app_dir());
        let app_dir_id2 = app_dir_id.clone();

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();

            let fut = create_test_file(&client.clone(), &app, "test_file.txt")
                .then(move |res| {
                    let _ = unwrap!(res, "can't create test_file.txt");
                    dir_helper::get(c2, &app_dir_id)
                })
                .then(move |res| {
                    let app_root_dir = unwrap!(res);
                    let file = unwrap!(app_root_dir.find_file("test_file.txt"));
                    assert_eq!(file.metadata().user_metadata().len(), 0);

                    super::modify_file(&c3,
                                       &app,
                                       "/test_file.txt",
                                       false,
                                       None,
                                       Some(METADATA.as_bytes().to_vec()),
                                       None)
                })
                .then(move |res| {
                    assert!(res.is_ok());
                    dir_helper::get(c4, &app_dir_id2)
                })
                .then(move |res| {
                    let app_root_dir = unwrap!(res);
                    let file = unwrap!(app_root_dir.find_file("test_file.txt"));
                    assert!(file.metadata().user_metadata().len() > 0);
                    assert_eq!(file.metadata().user_metadata(), METADATA.as_bytes());

                    unwrap!(tx2.send(()));
                    Ok(())
                })
                .into_box();
            Some(fut)
        })));

        let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
        unwrap!(sess.send(CoreMsg::build_terminator()));
    }

    #[test]
    fn file_update_content() {
        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);
        let app2 = app.clone();
        let app_dir_id = unwrap!(app.app_dir());
        let app_dir_id2 = app_dir_id.clone();

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();
            let c5 = client.clone();
            let c6 = client.clone();
            let c7 = client.clone();

            let fut = create_test_file(&client.clone(), &app, "test_file.txt")
                .then(move |res| {
                    let _ = unwrap!(res, "can't create file /test_file.txt");
                    let content = "first".as_bytes().to_vec();
                    super::modify_file(&c2,
                                       &app,
                                       "/test_file.txt",
                                       false,
                                       None,
                                       None,
                                       Some(content))
                })
                .then(move |res| {
                    let _ = unwrap!(res, "can't modify file /test_file.txt");
                    dir_helper::get(c3, &app_dir_id)
                })
                .then(move |res| {
                    let app_root_dir = unwrap!(res);
                    let file = unwrap!(app_root_dir.find_file("test_file.txt"));

                    let reader = unwrap!(file_helper::read(c4, file));
                    let size = reader.size();

                    reader.read(0, size)
                })
                .then(move |res| {
                    let content = unwrap!(res);
                    let content = unwrap!(str::from_utf8(&content));
                    assert_eq!(content, "first");

                    let content = "second".as_bytes().to_vec();
                    super::modify_file(&c5,
                                       &app2,
                                       "/test_file.txt",
                                       false,
                                       None,
                                       None,
                                       Some(content))
                })
                .then(move |res| {
                    let _ = unwrap!(res, "can't modify /test_file.txt (2)");
                    dir_helper::get(c6, &app_dir_id2)
                })
                .then(move |res| {
                    let app_root_dir = unwrap!(res);
                    let file = unwrap!(app_root_dir.find_file("test_file.txt"));
                    let reader = unwrap!(file_helper::read(c7, file));
                    let size = reader.size();

                    reader.read(0, size)
                })
                .then(move |res| {
                    let content = unwrap!(res);
                    let content = unwrap!(str::from_utf8(&content));
                    assert_eq!(content, "second");

                    unwrap!(tx2.send(()));
                    Ok(())
                })
                .into_box();
            Some(fut)
        })));

        let _ = unwrap!(rx.recv_timeout(Duration::from_secs(15)));
        unwrap!(sess.send(CoreMsg::build_terminator()));
    }
}
