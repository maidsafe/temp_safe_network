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

//! Directory operations.

use core::{Client, CoreMsg};
use core::futures::FutureExt;
use ffi::{App, FfiError, FfiFuture, OpaqueCtx, Session, helper};
use ffi::dir_details::DirDetails;
use ffi::object_cache::AppHandle;
use futures::Future;
use libc::{c_void, int32_t};
use nfs::helper::dir_helper;
use std::{ptr, slice};
use time;

/// Create a new directory.
#[no_mangle]
pub unsafe extern "C" fn nfs_create_dir(session: *const Session,
                                        app_handle: AppHandle,
                                        dir_path: *const u8,
                                        dir_path_len: usize,
                                        user_metadata: *const u8,
                                        user_metadata_len: usize,
                                        is_private: bool,
                                        is_shared: bool,
                                        user_data: *mut c_void,
                                        o_cb: extern "C" fn(*mut c_void, int32_t))
                                        -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI create directory, given the path.");

        let dir_path = ffi_try!(helper::c_utf8_to_str(dir_path, dir_path_len));
        let user_metadata = slice::from_raw_parts(user_metadata, user_metadata_len);
        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = create_dir(&client,
                                         &app,
                                         dir_path,
                                         user_metadata,
                                         is_private,
                                         is_shared)
                        .then(move |result| Ok(o_cb(user_data.0, ffi_result_code!(result))))
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e));
                    None
                }
            }
        })));

        0
    })
}

/// Delete a directory.
#[no_mangle]
pub unsafe extern "C" fn nfs_delete_dir(session: *const Session,
                                        app_handle: AppHandle,
                                        dir_path: *const u8,
                                        dir_path_len: usize,
                                        is_shared: bool,
                                        user_data: *mut c_void,
                                        o_cb: unsafe extern "C" fn(*mut c_void, int32_t))
                                        -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI delete dir, given the path.");
        let dir_path = ffi_try!(helper::c_utf8_to_str(dir_path, dir_path_len));

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = delete_dir(&client, &app, dir_path, is_shared)
                        .then(move |result| Ok(o_cb(user_data.0, ffi_result_code!(result))))
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e));
                    None
                }
            }
        })));

        0
    })
}

/// Get directory
#[no_mangle]
pub unsafe extern "C" fn nfs_get_dir(session: *const Session,
                                     app_handle: AppHandle,
                                     dir_path: *const u8,
                                     dir_path_len: usize,
                                     is_shared: bool,
                                     user_data: *mut c_void,
                                     o_cb: extern "C" fn(*mut c_void,
                                                         int32_t,
                                                         details_handle: *mut DirDetails))
                                     -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI get dir, given the path.");
        let dir_path = ffi_try!(helper::c_utf8_to_str(dir_path, dir_path_len));

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = get_dir(&client, &app, dir_path, is_shared)
                        .map(move |details| {
                            let details_handle = Box::into_raw(Box::new(details));
                            o_cb(user_data.0, 0, details_handle);
                        })
                        .map_err(move |err| {
                            o_cb(user_data.0, ffi_error_code!(err), ptr::null_mut())
                        })
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e), ptr::null_mut());
                    None
                }
            }
        })));

        0
    })
}

/// Modify name and/or metadata of a directory.
#[no_mangle]
pub unsafe extern "C" fn nfs_modify_dir(session: *const Session,
                                        app_handle: AppHandle,
                                        dir_path: *const u8,
                                        dir_path_len: usize,
                                        is_shared: bool,
                                        new_name: *const u8,
                                        new_name_len: usize,
                                        new_user_metadata: *const u8,
                                        new_user_metadata_len: usize,
                                        user_data: *mut c_void,
                                        o_cb: extern "C" fn(*mut c_void, int32_t))
                                        -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI modify directory, given the path.");
        let dir_path = ffi_try!(helper::c_utf8_to_str(dir_path, dir_path_len));
        let new_name = ffi_try!(helper::c_utf8_to_opt_string(new_name, new_name_len));
        let new_user_metadata = helper::u8_ptr_to_opt_vec(new_user_metadata, new_user_metadata_len);

        let obj_cache = (*session).object_cache();
        let user_data = OpaqueCtx(user_data);

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());
            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = modify_dir(&client,
                                         &app,
                                         dir_path,
                                         is_shared,
                                         new_name,
                                         new_user_metadata)
                        .then(move |result| Ok(o_cb(user_data.0, ffi_result_code!(result))))
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e));
                    None
                }
            }
        })));

        0
    })
}

/// Move or copy a directory.
#[no_mangle]
pub unsafe extern "C" fn nfs_move_dir(session: *const Session,
                                      app_handle: AppHandle,
                                      src_path: *const u8,
                                      src_path_len: usize,
                                      is_src_path_shared: bool,
                                      dst_path: *const u8,
                                      dst_path_len: usize,
                                      is_dst_path_shared: bool,
                                      retain_src: bool,
                                      user_data: *mut c_void,
                                      o_cb: unsafe extern "C" fn(*mut c_void, int32_t))
                                      -> int32_t {
    helper::catch_unwind_i32(|| {
        trace!("FFI move directory, from {:?} to {:?}.", src_path, dst_path);

        let src_path = ffi_try!(helper::c_utf8_to_str(src_path, src_path_len));
        let dst_path = ffi_try!(helper::c_utf8_to_str(dst_path, dst_path_len));
        let user_data = OpaqueCtx(user_data);
        let obj_cache = (*session).object_cache();

        ffi_try!((*session).send(CoreMsg::new(move |client| {
            let mut obj_cache = unwrap!(obj_cache.lock());

            match obj_cache.get_app(app_handle) {
                Ok(app) => {
                    let fut = move_dir(&client,
                                       app,
                                       src_path,
                                       is_src_path_shared,
                                       dst_path,
                                       is_dst_path_shared,
                                       retain_src)
                        .then(move |result| Ok(o_cb(user_data.0, ffi_result_code!(result))))
                        .into_box();
                    Some(fut)
                }
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e));
                    None
                }
            }
        })));
        0
    })
}


fn create_dir(client: &Client,
              app: &App,
              dir_path: &str,
              user_metadata: &[u8],
              _is_private: bool,
              is_shared: bool)
              -> Box<FfiFuture<()>> {
    let mut tokens = helper::tokenise_path(dir_path, false);
    let dir_to_create = fry!(tokens.pop().ok_or(FfiError::InvalidPath));
    let user_metadata = user_metadata.to_owned();

    let c2 = client.clone();
    let c3 = client.clone();

    app.root_dir(client.clone(), is_shared)
        .map_err(FfiError::from)
        .and_then(move |start_dir_id| helper::final_sub_dir(&c2, &tokens, Some(&start_dir_id)))
        .and_then(move |(parent, metadata)| {
            // let access_level = if is_private {
            //     AccessLevel::Private
            // } else {
            //     AccessLevel::Public
            // };
            dir_helper::create_sub_dir(c3,
                                       dir_to_create,
                                       None,
                                       user_metadata,
                                       &parent,
                                       &metadata.id())
                .map_err(FfiError::from)
        })
        .map(move |_| ())
        .into_box()
}

fn delete_dir(client: &Client, app: &App, dir_path: &str, is_shared: bool) -> Box<FfiFuture<()>> {
    let mut tokens = helper::tokenise_path(dir_path, false);
    let dir_to_delete = fry!(tokens.pop().ok_or(FfiError::InvalidPath));

    let c2 = client.clone();
    let c3 = client.clone();
    let c4 = client.clone();

    app.root_dir(client.clone(), is_shared)
        .map_err(FfiError::from)
        .and_then(move |root_dir_id| {
            dir_helper::get(c2, &root_dir_id)
                .map(move |dir| (dir, root_dir_id))
                .map_err(FfiError::from)
        })
        .and_then(move |(root_dir, dir_id)| {
            if tokens.is_empty() {
                ok!((root_dir, dir_id))
            } else {
                helper::final_sub_dir(&c3, &tokens, Some(&dir_id))
                    .map(|(dir, dir_meta)| (dir, dir_meta.id()))
                    .into_box()
            }
        })
        .and_then(move |(mut parent, parent_id)| {
            dir_helper::delete(c4, &mut parent, &parent_id, &dir_to_delete).map_err(FfiError::from)
        })
        .map(|_| ())
        .into_box()
}

fn get_dir(client: &Client,
           app: &App,
           dir_path: &str,
           is_shared: bool)
           -> Box<FfiFuture<DirDetails>> {
    helper::dir(client, app, dir_path, is_shared)
        .and_then(move |(dir, dir_metadata)| DirDetails::from_dir(dir, dir_metadata))
        .into_box()
}

fn modify_dir(client: &Client,
              app: &App,
              dir_path: &str,
              is_shared: bool,
              new_name: Option<String>,
              new_metadata: Option<Vec<u8>>)
              -> Box<FfiFuture<()>> {
    let mut tokens = helper::tokenise_path(dir_path, false);
    let dir_to_modify = fry!(tokens.pop().ok_or(FfiError::InvalidPath));

    if new_name.is_none() && new_metadata.is_none() {
        return err!(FfiError::from("Optional parameters could not be parsed"));
    }

    let c2 = client.clone();
    let c3 = client.clone();

    app.root_dir(client.clone(), is_shared)
        .map_err(FfiError::from)
        .and_then(move |start_dir_id| helper::final_sub_dir(&c2, &tokens, Some(&start_dir_id)))
        .and_then(move |(mut parent, parent_meta)| {
            let mut dir_meta =
                fry!(parent.find_sub_dir(&dir_to_modify).ok_or(FfiError::InvalidPath)).clone();

            if let Some(name) = new_name {
                dir_meta.set_name(name);
            }
            if let Some(metadata) = new_metadata {
                dir_meta.set_user_metadata(metadata);
            }
            dir_meta.set_modified_time(time::now_utc());

            parent.upsert_sub_dir(dir_meta);

            dir_helper::update(c3, &parent_meta.id(), &parent)
                .map_err(FfiError::from)
                .into_box()
        })
        .into_box()
}

fn move_dir(client: &Client,
            app: &App,
            src_path: &str,
            is_src_path_shared: bool,
            dst_path: &str,
            is_dst_path_shared: bool,
            retain_src: bool)
            -> Box<FfiFuture<()>> {
    let c2 = client.clone();
    let c3 = client.clone();
    let c4 = c2.clone();

    let dst_path = dst_path.to_string();

    helper::dir_and_file(client, app, src_path, is_src_path_shared)
        .join(helper::dir(client, app, &dst_path, is_dst_path_shared))
        .and_then(move |((mut src_parent_dir, src_parent_meta, dir_to_move),
                         (mut dst_dir, dst_meta))| {
            if retain_src {
                // Copy
                let src_meta = fry!(src_parent_dir.find_sub_dir(&dir_to_move)
                    .cloned()
                    .ok_or(FfiError::PathNotFound));

                dir_helper::get(c2, &src_meta.id())
                    .map_err(FfiError::from)
                    .and_then(move |src_dir| {
                        dir_helper::deep_copy(c3, &src_dir, &src_meta).map_err(FfiError::from)
                    })
                    .and_then(move |(_copy, mut copy_meta)| {
                        copy_meta.set_name(dst_path);
                        dst_dir.upsert_sub_dir(copy_meta);

                        dir_helper::update(c4, &dst_meta.id(), &dst_dir).map_err(FfiError::from)
                    })
                    .into_box()
            } else {
                // Move
                let moved_meta = fry!(src_parent_dir.remove_sub_dir(&dir_to_move)
                    .map_err(FfiError::from));

                dst_dir.upsert_sub_dir(moved_meta);

                dir_helper::update(c2, &dst_meta.id(), &dst_dir)
                    .map_err(FfiError::from)
                    .and_then(move |_| {
                        dir_helper::update(c3, &src_parent_meta.id(), &src_parent_dir)
                            .map_err(FfiError::from)
                    })
                    .into_box()
            }
        })
        .into_box()
}

#[cfg(test)]
mod tests {
    use core::{Client, CoreMsg};
    use core::futures::FutureExt;
    use ffi::{App, FfiError, FfiFuture, test_utils};
    use futures::Future;
    // use nfs::AccessLevel;
    use nfs::helper::dir_helper;
    use std::slice;
    use std::sync::mpsc;
    use std::time::Duration;

    fn create_test_dir(client: Client, app: &App, name: &str) -> Box<FfiFuture<()>> {
        let app_dir_id = unwrap!(app.app_dir());
        let name = name.to_owned();

        dir_helper::get(client.clone(), &app_dir_id)
            .and_then(move |app_root_dir| {
                dir_helper::create_sub_dir(client,
                                           name,
                                           None,
                                           Vec::new(),
                                           &app_root_dir,
                                           &app_dir_id)
                    .map(|_| ())
            })
            .map_err(FfiError::from)
            .into_box()
    }

    #[test]
    fn create_dir() {
        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();
            let c5 = client.clone();
            let c6 = client.clone();
            let c7 = client.clone();

            let app2 = app.clone();
            let app3 = app.clone();
            let app4 = app.clone();
            let app5 = app.clone();
            let app6 = app.clone();

            let user_metadata = "user metadata".as_bytes().to_vec();

            let fut = super::create_dir(client, &app, "/", &user_metadata, true, false)
                .then(move |result| {
                    assert!(result.is_err(), "creating / should fail");
                    super::create_dir(&c2,
                                      &app2,
                                      "/test_dir/secondlevel",
                                      &"user metadata".as_bytes().to_vec(),
                                      true,
                                      false)
                })
                .then(move |result| {
                    assert!(result.is_err(),
                            "creating /test_dir/secondlevel should fail");
                    let user_metadata = "user metadata".as_bytes().to_vec();
                    super::create_dir(&c3, &app3, "/test_dir", &user_metadata, true, false)
                })
                .then(move |result| {
                    if let Err(e) = result {
                        panic!("failed creating /test_dir: {:?}", e);
                    }
                    let user_metadata = "user metadata".as_bytes().to_vec();
                    super::create_dir(&c4, &app4, "/test_dir2", &user_metadata, true, false)
                })
                .then(move |result| {
                    if let Err(e) = result {
                        panic!("failed creating /test_dir2: {:?}", e);
                    }
                    super::create_dir(&c5,
                                      &app5,
                                      "/test_dir/secondlevel",
                                      &user_metadata,
                                      true,
                                      false)
                })
                .then(move |result| {
                    if let Err(e) = result {
                        panic!("failed creating /test_dir/second_level: {:?}", e);
                    }
                    dir_helper::get(c6, &unwrap!(app6.app_dir()))
                })
                .then(move |result| {
                    let app_dir = unwrap!(result, "failed getting app6.app_dir");
                    assert!(app_dir.find_sub_dir("test_dir").is_some());
                    assert!(app_dir.find_sub_dir("test_dir2").is_some());
                    assert_eq!(app_dir.sub_dirs().len(), 2);

                    let test_dir_meta = unwrap!(app_dir.find_sub_dir("test_dir"));
                    dir_helper::get(c7, &test_dir_meta.id())
                })
                .then(move |result| {
                    let test_dir = unwrap!(result, "failed getting test_dir");

                    assert!(test_dir.find_sub_dir("secondlevel").is_some());
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
    fn delete_dir() {
        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();
            let c5 = client.clone();
            let c6 = client.clone();

            let app2 = app.clone();
            let app3 = app.clone();

            let app_dir_id = unwrap!(app.app_dir());
            let app_dir_id2 = app_dir_id.clone();

            let fut = create_test_dir(client.clone(), &app, "test_dir")
                .then(move |result| {
                    let _ = unwrap!(result);
                    super::delete_dir(&c2, &app, "/test_dir2", false)
                })
                .then(move |delete_result| {
                    assert!(delete_result.is_err());
                    dir_helper::get(c3, &app_dir_id)
                })
                .then(move |result| {
                    let app_root_dir = unwrap!(result);
                    assert_eq!(app_root_dir.sub_dirs().len(), 1);
                    assert!(app_root_dir.find_sub_dir("test_dir").is_some());

                    super::delete_dir(&c4, &app2, "/test_dir", false)
                })
                .then(move |result| {
                    let _ = unwrap!(result);
                    dir_helper::get(c5, &app_dir_id2)
                })
                .then(move |result| {
                    let app_root_dir = unwrap!(result);
                    assert_eq!(app_root_dir.sub_dirs().len(),
                               0,
                               "directory /test_dir hasn't been deleted");

                    super::delete_dir(&c6, &app3, "/test_dir", false)
                })
                .then(move |result| {
                    assert!(result.is_err());
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
    fn get_dir() {
        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();

            let app2 = app.clone();
            let app3 = app.clone();

            let fut = create_test_dir(client.clone(), &app, "test_dir")
                .then(move |result| {
                    let _ = unwrap!(result);

                    super::get_dir(&c2, &app2, "/test_dir", false)
                })
                .then(move |result| {
                    let details = unwrap!(result);
                    unsafe {
                        let name = slice::from_raw_parts(details.metadata.name,
                                                         details.metadata.name_len);
                        let name = unwrap!(String::from_utf8(name.to_owned()));
                        assert_eq!(name, "test_dir");
                    }
                    assert_eq!(details.files.len(), 0);
                    assert_eq!(details.sub_dirs.len(), 0);

                    super::get_dir(&c3, &app3, "/does_not_exist", false)
                })
                .then(move |result| {
                    assert!(result.is_err());

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
    fn rename_dir() {
        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();

            let app2 = app.clone();
            let app_dir_id = unwrap!(app.app_dir());
            let app_dir_id2 = app_dir_id.clone();

            let fut = create_test_dir(client.clone(), &app, "test_dir")
                .then(move |result| {
                    let _ = unwrap!(result);
                    dir_helper::get(c2, &app_dir_id)
                })
                .then(move |result| {
                    let app_root_dir = unwrap!(result);
                    assert_eq!(app_root_dir.sub_dirs().len(), 1);
                    assert!(app_root_dir.find_sub_dir("test_dir").is_some());

                    super::modify_dir(&c3,
                                      &app2,
                                      "/test_dir",
                                      false,
                                      Some("new_test_dir".to_string()),
                                      None)
                })
                .then(move |result| {
                    let _ = unwrap!(result);
                    dir_helper::get(c4, &app_dir_id2)
                })
                .then(move |result| {
                    let app_root_dir = unwrap!(result);
                    assert_eq!(app_root_dir.sub_dirs().len(), 1);
                    assert!(app_root_dir.find_sub_dir("test_dir").is_none());
                    assert!(app_root_dir.find_sub_dir("new_test_dir").is_some());

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
    fn dir_update_user_metadata() {
        const METADATA: &'static str = "user metadata";

        let sess = test_utils::create_session();
        let app = test_utils::create_app(&sess, false);

        let (tx, rx) = mpsc::channel::<()>();
        let tx2 = tx.clone();

        unwrap!(sess.send(CoreMsg::new(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();

            let app_dir_id = unwrap!(app.app_dir());
            let app_dir_id2 = app_dir_id.clone();

            let fut = create_test_dir(client.clone(), &app, "test_dir")
                .then(move |result| {
                    let _ = unwrap!(result);
                    dir_helper::get(c2, &app_dir_id)
                })
                .then(move |result| {
                    let app_root = unwrap!(result);
                    let dir_meta = unwrap!(app_root.find_sub_dir("test_dir"));
                    assert_eq!(dir_meta.user_metadata().len(), 0);

                    super::modify_dir(&c3,
                                      &app,
                                      "/test_dir",
                                      false,
                                      None,
                                      Some(METADATA.as_bytes().to_vec()))
                })
                .then(move |result| {
                    let _ = unwrap!(result);
                    dir_helper::get(c4, &app_dir_id2)
                })
                .then(move |result| {
                    let root_dir = unwrap!(result);
                    let dir_to_modify = unwrap!(root_dir.find_sub_dir("test_dir"));
                    assert!(dir_to_modify.user_metadata().len() > 0);
                    assert_eq!(dir_to_modify.user_metadata(), METADATA.as_bytes());

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
