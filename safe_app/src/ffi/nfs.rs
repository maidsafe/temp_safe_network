// Copyright 2016 MaidSafe.net limited.
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

use {App, AppContext};
use errors::AppError;
use ffi::helper::send_with_mdata_info;
use ffi_utils::{FFI_RESULT_OK, FfiResult, OpaqueCtx, ReprC, catch_unwind_cb, from_c_str};
use futures::Future;
use object_cache::{FileContextHandle, MDataInfoHandle};
use safe_core::FutureExt;
use safe_core::nfs::{Mode, Reader, Writer, file_helper};
use safe_core::nfs::File as NativeFile;
use safe_core::nfs::ffi::File;
use std::{ptr, slice};
use std::os::raw::{c_char, c_void};

/// Holds context for file operations, depending on the mode.
pub struct FileContext {
    reader: Option<Reader<AppContext>>,
    writer: Option<Writer<AppContext>>,
}

/// Replaces the entire content of the file when writing data.
pub static OPEN_MODE_OVERWRITE: u64 = 1;
/// Appends to existing data in the file.
pub static OPEN_MODE_APPEND: u64 = 2;
/// Open file to read.
pub static OPEN_MODE_READ: u64 = 4;
/// Read entire contents of a file.
pub static FILE_READ_TO_END: u64 = 0;

/// Retrieve file with the given name, and its version, from the directory.
#[no_mangle]
pub unsafe extern "C" fn dir_fetch_file(app: *const App,
                                        parent_h: MDataInfoHandle,
                                        file_name: *const c_char,
                                        user_data: *mut c_void,
                                        o_cb: extern "C" fn(*mut c_void,
                                                            FfiResult,
                                                            *const File,
                                                            u64)) {
    catch_unwind_cb(user_data, o_cb, || {
        let file_name = from_c_str(file_name)?;
        let user_data = OpaqueCtx(user_data);

        (*app).send(move |client, context| {
            let parent = try_cb!(context.object_cache().get_mdata_info(parent_h),
                                 user_data.0,
                                 o_cb);

            file_helper::fetch(client.clone(), parent.clone(), file_name)
                .map(move |(version, file)| {
                         let ffi_file = file.into_repr_c();
                         o_cb(user_data.0, FFI_RESULT_OK, &ffi_file, version)
                     })
                .map_err(AppError::from)
                .map_err(move |err| {
                             call_result_cb!(Err::<(), _>(err), user_data, o_cb);
                         })
                .into_box()
                .into()
        })
    })
}

/// Insert the file into the parent directory.
#[no_mangle]
pub unsafe extern "C" fn dir_insert_file(app: *const App,
                                         parent_h: MDataInfoHandle,
                                         file_name: *const c_char,
                                         file: *const File,
                                         user_data: *mut c_void,
                                         o_cb: extern "C" fn(*mut c_void, FfiResult)) {
    catch_unwind_cb(user_data, o_cb, || {
        let file = NativeFile::clone_from_repr_c(file)?;
        let file_name = from_c_str(file_name)?;

        send_with_mdata_info(app, parent_h, user_data, o_cb, move |client, _, parent| {
            file_helper::insert(client.clone(), parent.clone(), file_name, &file)
        })
    })
}

/// Replace the file in the parent directory.
/// If `version` is 0, the correct version is obtained automatically.
#[no_mangle]
pub unsafe extern "C" fn dir_update_file(app: *const App,
                                         parent_h: MDataInfoHandle,
                                         file_name: *const c_char,
                                         file: *const File,
                                         version: u64,
                                         user_data: *mut c_void,
                                         o_cb: extern "C" fn(*mut c_void, FfiResult)) {
    catch_unwind_cb(user_data, o_cb, || {
        let file = NativeFile::clone_from_repr_c(file)?;
        let file_name = from_c_str(file_name)?;

        send_with_mdata_info(app, parent_h, user_data, o_cb, move |client, _, parent| {
            file_helper::update(client.clone(), parent.clone(), file_name, &file, version)
        })
    })
}

/// Delete the file in the parent directory.
#[no_mangle]
pub unsafe extern "C" fn dir_delete_file(app: *const App,
                                         parent_h: MDataInfoHandle,
                                         file_name: *const c_char,
                                         version: u64,
                                         user_data: *mut c_void,
                                         o_cb: extern "C" fn(*mut c_void, FfiResult)) {
    catch_unwind_cb(user_data, o_cb, || {
        let file_name = from_c_str(file_name)?;
        send_with_mdata_info(app, parent_h, user_data, o_cb, move |client, _, parent| {
            file_helper::delete(client, parent, file_name, version)
        })
    })
}

/// Open the file to read of write its contents
#[no_mangle]
pub unsafe extern "C" fn file_open(app: *const App,
                                   file: *const File,
                                   open_mode: u64,
                                   user_data: *mut c_void,
                                   o_cb: extern "C" fn(*mut c_void, FfiResult, FileContextHandle)) {
    catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);
        let file = NativeFile::clone_from_repr_c(file)?;

        (*app).send(move |client, context| {
            let context = context.clone();

            // Initialise the reader if OPEN_MODE_READ is requested
            let reader = if open_mode & OPEN_MODE_READ != 0 {
                file_helper::read(client.clone(), &file)
                    .map(Some)
                    .into_box()
            } else {
                ok!(None)
            };

            // Initialise the writer if one of write modes is requested
            let writer = if open_mode & (OPEN_MODE_OVERWRITE | OPEN_MODE_APPEND) != 0 {
                let writer_mode = if open_mode & OPEN_MODE_APPEND != 0 {
                    Mode::Append
                } else {
                    Mode::Overwrite
                };
                file_helper::write(client.clone(), file, writer_mode)
                    .map(Some)
                    .into_box()
            } else {
                ok!(None)
            };

            reader
                .join(writer)
                .map(move |(reader, writer)| {
                         let file_ctx = FileContext { reader, writer };
                         let file_h = context.object_cache().insert_file(file_ctx);
                         o_cb(user_data.0, FFI_RESULT_OK, file_h);
                     })
                .map_err(move |err| {
                             call_result_cb!(Err::<(), _>(AppError::from(err)), user_data, o_cb);
                         })
                .into_box()
                .into()
        })
    })
}

/// Get a size of file opened for read.
#[no_mangle]
pub unsafe extern "C" fn file_size(app: *const App,
                                   file_h: FileContextHandle,
                                   user_data: *mut c_void,
                                   o_cb: extern "C" fn(*mut c_void, FfiResult, u64)) {
    catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);

        (*app).send(move |_client, context| {
            let file_ctx = try_cb!(context.object_cache().get_file(file_h), user_data, o_cb);

            if let Some(ref reader) = file_ctx.reader {
                o_cb(user_data.0, FFI_RESULT_OK, reader.size());
            } else {
                call_result_cb!(Err::<(), _>(AppError::InvalidFileMode), user_data, o_cb);
            }
            None
        })
    })
}

/// Read data from file.
#[no_mangle]
pub unsafe extern "C" fn file_read(app: *const App,
                                   file_h: FileContextHandle,
                                   position: u64,
                                   len: u64,
                                   user_data: *mut c_void,
                                   o_cb: extern "C" fn(*mut c_void, FfiResult, *const u8, usize)) {
    catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);

        (*app).send(move |_client, context| {
            let file_ctx = try_cb!(context.object_cache().get_file(file_h), user_data, o_cb);

            if let Some(ref reader) = file_ctx.reader {
                reader
                    .read(position,
                          if len == FILE_READ_TO_END {
                              reader.size() - position
                          } else {
                              len
                          })
                    .map(move |data| {
                             o_cb(user_data.0, FFI_RESULT_OK, data.as_ptr(), data.len());
                         })
                    .map_err(move |err| {
                                 call_result_cb!(Err::<(), _>(AppError::from(err)),
                                                 user_data,
                                                 o_cb);
                             })
                    .into_box()
                    .into()
            } else {
                call_result_cb!(Err::<(), _>(AppError::InvalidFileMode), user_data, o_cb);
                None
            }
        })
    })
}

/// Write data to file in smaller chunks.
#[no_mangle]
pub unsafe extern "C" fn file_write(app: *const App,
                                    file_h: FileContextHandle,
                                    data: *const u8,
                                    size: usize,
                                    user_data: *mut c_void,
                                    o_cb: extern "C" fn(*mut c_void, FfiResult)) {
    catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);
        let data = slice::from_raw_parts(data, size);

        (*app).send(move |_client, context| {
            let file_ctx = try_cb!(context.object_cache().get_file(file_h), user_data, o_cb);

            if let Some(ref writer) = file_ctx.writer {
                writer
                    .write(data)
                    .then(move |res| {
                              call_result_cb!(res.map_err(AppError::from), user_data, o_cb);
                              Ok(())
                          })
                    .into_box()
                    .into()
            } else {
                call_result_cb!(Err::<(), _>(AppError::InvalidFileMode), user_data, o_cb);
                None
            }
        })
    })
}

/// Close is invoked only after all the data is completely written. The
/// file is saved only when `close` is invoked.
#[no_mangle]
pub unsafe extern "C" fn file_close(app: *const App,
                                    file_h: FileContextHandle,
                                    user_data: *mut c_void,
                                    o_cb: extern "C" fn(*mut c_void, FfiResult, *const File)) {
    catch_unwind_cb(user_data, o_cb, || {
        let user_data = OpaqueCtx(user_data);

        (*app).send(move |_client, context| {
            let file_ctx = try_cb!(context.object_cache().remove_file(file_h), user_data, o_cb);

            if let Some(writer) = file_ctx.writer {
                writer
                    .close()
                    .map(move |file| { o_cb(user_data.0, FFI_RESULT_OK, &file.into_repr_c()); })
                    .map_err(move |err| {
                                 call_result_cb!(Err::<(), _>(AppError::from(err)),
                                                 user_data,
                                                 o_cb);
                             })
                    .into_box()
                    .into()
            } else {
                // The reader will be dropped automatically
                o_cb(user_data.0, FFI_RESULT_OK, ptr::null());
                None
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use errors::AppError;
    use ffi_utils::ErrorCode;
    use ffi_utils::test_utils::{call_0, call_1, call_2, call_vec_u8};
    use futures::Future;
    use safe_core::ipc::Permission;
    use safe_core::nfs::File as NativeFile;
    use safe_core::nfs::NfsError;
    use std::collections::HashMap;
    use std::ffi::CString;
    use test_utils::{create_app_with_access, run};

    #[test]
    fn basics() {
        let mut container_permissions = HashMap::new();
        let _ = container_permissions.insert("_videos".to_string(),
                                             btree_set![Permission::Read,
                                                        Permission::Insert,
                                                        Permission::Update,
                                                        Permission::Delete]);

        let app = create_app_with_access(container_permissions);

        let container_info_h = run(&app, move |client, context| {
            let context = context.clone();

            context
                .get_container_mdata_info(client, "_videos")
                .map(move |info| context.object_cache().insert_mdata_info(info))
        });

        let file_name0 = "file0.txt";
        let ffi_file_name0 = unwrap!(CString::new(file_name0));

        // fetching non-existing file fails.
        let res: Result<(NativeFile, u64), i32> = unsafe {
            call_2(|ud, cb| dir_fetch_file(&app, container_info_h, ffi_file_name0.as_ptr(), ud, cb))
        };

        match res {
            Err(code) if code == AppError::from(NfsError::FileNotFound).error_code() => (),
            Err(x) => panic!("Unexpected: {:?}", x),
            Ok(_) => panic!("Unexpected success"),
        }

        // Create empty file.
        let user_metadata = b"metadata".to_vec();
        let file = NativeFile::new(user_metadata.clone());
        let ffi_file = file.into_repr_c();

        unsafe {
            unwrap!(call_0(|ud, cb| {
                               dir_insert_file(&app,
                                               container_info_h,
                                               ffi_file_name0.as_ptr(),
                                               &ffi_file,
                                               ud,
                                               cb)
                           }))
        }

        // Fetch it back.
        let (retrieved_file, retrieved_version): (NativeFile, u64) =
            unsafe {
                unwrap!(call_2(|ud, cb| {
                                   dir_fetch_file(&app,
                                                  container_info_h,
                                                  ffi_file_name0.as_ptr(),
                                                  ud,
                                                  cb)
                               }))
            };
        assert_eq!(retrieved_file.user_metadata(), &user_metadata[..]);
        assert_eq!(retrieved_file.size(), 0);
        assert_eq!(retrieved_version, 0);

        // Delete file.
        unsafe {
            unwrap!(call_0(|ud, cb| {
                               dir_delete_file(&app,
                                               container_info_h,
                                               ffi_file_name0.as_ptr(),
                                               1,
                                               ud,
                                               cb)
                           }))
        }
    }

    // Tests NFS functions for writing and updating file contents.
    // 1. Create an empty file, open it for writing, write contents.
    // 2. Insert file into a container.
    // 3. Fetch the file from a container, check that it has a correct version.
    // 4. Open the file again, now in a combined append + read mode.
    // 5. Read the file contents; it should be the same as we have written it.
    // 6. Append a string to a file contents (by using `OPEN_MODE_APPEND`, _not_
    // by rewriting the existing data with an appended string).
    // 7. Update the file in the directory.
    // 8. Fetch the updated file version again and ensure that it contains
    // the expected string.
    #[test]
    fn open_file() {
        let mut container_permissions = HashMap::new();
        let _ = container_permissions.insert("_videos".to_string(),
                                             btree_set![Permission::Read,
                                                        Permission::Insert,
                                                        Permission::Update,
                                                        Permission::Delete]);

        let app = create_app_with_access(container_permissions);

        let container_info_h = run(&app, move |client, context| {
            let context = context.clone();

            context
                .get_container_mdata_info(client, "_videos")
                .map(move |info| context.object_cache().insert_mdata_info(info))
        });

        // Create non-empty file.
        let file = NativeFile::new(Vec::new());
        let ffi_file = file.into_repr_c();

        let file_name1 = "file1.txt";
        let ffi_file_name1 = unwrap!(CString::new(file_name1));

        let content = b"hello world";

        let write_h = unsafe {
            unwrap!(call_1(|ud, cb| file_open(&app, &ffi_file, OPEN_MODE_OVERWRITE, ud, cb)))
        };

        let written_file: NativeFile = unsafe {
            unwrap!(call_0(|ud, cb| {
                               file_write(&app, write_h, content.as_ptr(), content.len(), ud, cb)
                           }));
            unwrap!(call_1(|ud, cb| file_close(&app, write_h, ud, cb)))
        };

        // Insert file into container.
        unsafe {
            unwrap!(call_0(|ud, cb| {
                               dir_insert_file(&app,
                                               container_info_h,
                                               ffi_file_name1.as_ptr(),
                                               &written_file.into_repr_c(),
                                               ud,
                                               cb)
                           }))
        }

        // Fetch it back.
        let (file, version): (NativeFile, u64) = {
            unsafe {
                unwrap!(call_2(|ud, cb| {
                                   dir_fetch_file(&app,
                                                  container_info_h,
                                                  ffi_file_name1.as_ptr(),
                                                  ud,
                                                  cb)
                               }))
            }
        };
        assert_eq!(version, 0);

        // Read the content and append data
        let read_write_h = unsafe {
            unwrap!(call_1(|ud, cb| {
                               file_open(&app,
                                         &file.into_repr_c(),
                                         OPEN_MODE_READ | OPEN_MODE_APPEND,
                                         ud,
                                         cb)
                           }))
        };

        let retrieved_content = unsafe {
            unwrap!(call_vec_u8(|ud, cb| {
                                    file_read(&app, read_write_h, 0, FILE_READ_TO_END, ud, cb)
                                }))
        };
        assert_eq!(retrieved_content, content);

        let append_content = b" appended";

        let written_file: NativeFile = unsafe {
            unwrap!(call_0(|ud, cb| {
                               file_write(&app,
                                          read_write_h,
                                          append_content.as_ptr(),
                                          append_content.len(),
                                          ud,
                                          cb)
                           }));
            unwrap!(call_1(|ud, cb| file_close(&app, read_write_h, ud, cb)))
        };

        // Update it in the dir
        unsafe {
            unwrap!(call_0(|ud, cb| {
                dir_update_file(&app,
                                container_info_h,
                                ffi_file_name1.as_ptr(),
                                &written_file.into_repr_c(),
                                1,
                                ud,
                                cb)
            }))
        }

        // Read the updated content
        let (file, version): (NativeFile, u64) = {
            unsafe {
                unwrap!(call_2(|ud, cb| {
                                   dir_fetch_file(&app,
                                                  container_info_h,
                                                  ffi_file_name1.as_ptr(),
                                                  ud,
                                                  cb)
                               }))
            }
        };
        assert_eq!(version, 1);

        let read_h = unsafe {
            unwrap!(call_1(|ud, cb| file_open(&app, &file.into_repr_c(), OPEN_MODE_READ, ud, cb)))
        };

        let retrieved_content = unsafe {
            unwrap!(call_vec_u8(|ud, cb| file_read(&app, read_h, 0, FILE_READ_TO_END, ud, cb)))
        };
        assert_eq!(retrieved_content, b"hello world appended");

        let f: *const File = unsafe { unwrap!(call_1(|ud, cb| file_close(&app, read_h, ud, cb))) };
        assert!(f.is_null());
    }

    // Tests that NFS functions still work after deleting and updating file contents.
    // 1. Create an empty file, open it for writing, write original contents.
    // 2. Insert file into the container.
    // 3. Delete file in the container.
    // 4. Create non-empty file with new contents.
    // 5. Update the file in the container with new contents and version.
    // 6. Fetch the file from the container, check that it has the updated version.
    // 7. Read the file contents and ensure that they correspond to the data from step 4.
    #[test]
    fn delete_then_open_file() {
        let mut container_permissions = HashMap::new();
        let _ = container_permissions.insert("_videos".to_string(),
                                             btree_set![Permission::Read,
                                                        Permission::Insert,
                                                        Permission::Update,
                                                        Permission::Delete]);

        let app = create_app_with_access(container_permissions);

        let container_info_h = run(&app, move |client, context| {
            let context = context.clone();

            context
                .get_container_mdata_info(client, "_videos")
                .map(move |info| context.object_cache().insert_mdata_info(info))
        });

        // Create non-empty file.
        let file = NativeFile::new(Vec::new());
        let ffi_file = file.into_repr_c();

        let file_name2 = "file2.txt";
        let ffi_file_name2 = unwrap!(CString::new(file_name2));

        let content_original = b"hello world";

        let write_h = unsafe {
            unwrap!(call_1(|ud, cb| file_open(&app, &ffi_file, OPEN_MODE_OVERWRITE, ud, cb)))
        };

        let written_file: NativeFile = unsafe {
            unwrap!(call_0(|ud, cb| {
                               file_write(&app,
                                          write_h,
                                          content_original.as_ptr(),
                                          content_original.len(),
                                          ud,
                                          cb)
                           }));
            unwrap!(call_1(|ud, cb| file_close(&app, write_h, ud, cb)))
        };

        // Insert file into container.
        unsafe {
            unwrap!(call_0(|ud, cb| {
                               dir_insert_file(&app,
                                               container_info_h,
                                               ffi_file_name2.as_ptr(),
                                               &written_file.into_repr_c(),
                                               ud,
                                               cb)
                           }))
        }

        // Delete file.
        unsafe {
            unwrap!(call_0(|ud, cb| {
                               dir_delete_file(&app,
                                               container_info_h,
                                               ffi_file_name2.as_ptr(),
                                               1,
                                               ud,
                                               cb)
                           }))
        }

        // Create new non-empty file.
        let file = NativeFile::new(Vec::new());
        let ffi_file = file.into_repr_c();

        let content_new = b"hello goodbye";

        let write_h = unsafe {
            unwrap!(call_1(|ud, cb| file_open(&app, &ffi_file, OPEN_MODE_OVERWRITE, ud, cb)))
        };

        let new_file: NativeFile = unsafe {
            unwrap!(call_0(|ud, cb| {
                               file_write(&app,
                                          write_h,
                                          content_new.as_ptr(),
                                          content_new.len(),
                                          ud,
                                          cb)
                           }));
            unwrap!(call_1(|ud, cb| file_close(&app, write_h, ud, cb)))
        };

        // Update file in container.
        unsafe {
            unwrap!(call_0(|ud, cb| {
                dir_update_file(&app,
                                container_info_h,
                                ffi_file_name2.as_ptr(),
                                &new_file.into_repr_c(),
                                2,
                                ud,
                                cb)
            }))
        }

        // Fetch the file.
        let (file, version): (NativeFile, u64) = {
            unsafe {
                unwrap!(call_2(|ud, cb| {
                                   dir_fetch_file(&app,
                                                  container_info_h,
                                                  ffi_file_name2.as_ptr(),
                                                  ud,
                                                  cb)
                               }))
            }
        };
        assert_eq!(version, 2);

        // Read the content.
        let read_write_h = unsafe {
            unwrap!(call_1(|ud, cb| {
                               file_open(&app,
                                         &file.into_repr_c(),
                                         OPEN_MODE_READ | OPEN_MODE_APPEND,
                                         ud,
                                         cb)
                           }))
        };

        let retrieved_content = unsafe {
            unwrap!(call_vec_u8(|ud, cb| {
                                    file_read(&app, read_write_h, 0, FILE_READ_TO_END, ud, cb)
                                }))
        };
        assert_eq!(retrieved_content, content_new);
    }
}
