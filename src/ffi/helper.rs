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

use core::Client;
use core::futures::FutureExt;
use ffi::{App, FfiError, FfiFuture};
use ffi::config::SAFE_DRIVE_DIR_NAME;
use futures::{Future, stream};
use futures::stream::Stream;
use libc::{int32_t, int64_t};
use nfs::{Dir, DirId, DirMetadata};
use nfs::helper::dir_helper;
use std::{self, mem, panic, ptr, slice};
use std::error::Error;

pub unsafe fn c_utf8_to_string(ptr: *const u8, len: usize) -> Result<String, FfiError> {
    c_utf8_to_str(ptr, len).map(|v| v.to_owned())
}

pub unsafe fn c_utf8_to_str(ptr: *const u8, len: usize) -> Result<&'static str, FfiError> {
    std::str::from_utf8(slice::from_raw_parts(ptr, len))
        .map_err(|error| FfiError::from(error.description()))
}

pub unsafe fn c_utf8_to_opt_string(ptr: *const u8, len: usize) -> Result<Option<String>, FfiError> {
    if ptr.is_null() {
        Ok(None)
    } else {
        String::from_utf8(slice::from_raw_parts(ptr, len).to_owned())
            .map(|v| Some(v))
            .map_err(|error| FfiError::from(error.description()))
    }
}

// TODO: add c_utf8_to_opt_str (return Option<&str> instead of Option<String>)

/// Returns a heap-allocated raw string, usable by C/FFI-boundary. The tuple
/// means (pointer, length_in_bytes, capacity). Use `misc_u8_ptr_free` to free
/// the memory.
pub fn string_to_c_utf8(s: String) -> (*mut u8, usize, usize) {
    u8_vec_to_ptr(s.into_bytes())
}

pub unsafe fn u8_ptr_to_vec(ptr: *const u8, len: usize) -> Vec<u8> {
    slice::from_raw_parts(ptr, len).to_owned()
}

pub unsafe fn u8_ptr_to_opt_vec(ptr: *const u8, len: usize) -> Option<Vec<u8>> {
    if ptr.is_null() {
        None
    } else {
        Some(u8_ptr_to_vec(ptr, len))
    }
}

pub fn u8_vec_to_ptr(mut v: Vec<u8>) -> (*mut u8, usize, usize) {
    v.shrink_to_fit();
    let ptr = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    mem::forget(v);
    (ptr, len, cap)
}

pub fn catch_unwind_i32<F: FnOnce() -> int32_t>(f: F) -> int32_t {
    let errno: i32 = FfiError::Unexpected(String::new()).into();
    panic::catch_unwind(panic::AssertUnwindSafe(f)).unwrap_or(errno)
}

pub fn catch_unwind_i64<F: FnOnce() -> int64_t>(f: F) -> int64_t {
    let errno: i32 = FfiError::Unexpected(String::new()).into();
    panic::catch_unwind(panic::AssertUnwindSafe(f)).unwrap_or(errno as i64)
}

pub fn catch_unwind_ptr<T, F: FnOnce() -> *const T>(f: F) -> *const T {
    panic::catch_unwind(panic::AssertUnwindSafe(f)).unwrap_or(ptr::null())
}

pub fn tokenise_path(path: &str, keep_empty_splits: bool) -> Vec<String> {
    path.split(|element| element == '/')
        .filter(|token| keep_empty_splits || !token.is_empty())
        .map(|token| token.to_string())
        .collect()
}

pub fn safe_drive_metadata(client: Client) -> Box<FfiFuture<DirMetadata>> {
    trace!("Obtain directory metadata for SAFEDrive - This can be cached for efficiency. So if \
            this is seen many times, check for missed optimisation opportunity.");

    let safe_drive_dir_name = SAFE_DRIVE_DIR_NAME.to_string();

    let c2 = client.clone();

    dir_helper::user_root_dir(client)
        .map_err(FfiError::from)
        .and_then(move |(mut root_dir, _dir_id)| {
            match root_dir.find_sub_dir(&safe_drive_dir_name).cloned() {
                Some(metadata) => ok!(metadata),
                None => {
                    trace!("SAFEDrive does not exist - creating one.");

                    dir_helper::create_sub_dir(c2.clone(),
                                               safe_drive_dir_name,
                                               None,
                                               Vec::new(),
                                               &mut root_dir,
                                               &unwrap!(c2.user_root_dir_id(),
                                                        "Logic error: user root dir should exist \
                                                         at this point"))
                        .map_err(FfiError::from)
                        .map(move |(_, _, metadata)| metadata)
                        .into_box()
                }
            }
        })
        .into_box()
}

pub fn final_sub_dir(client: &Client,
                     tokens: &[String],
                     starting_directory: Option<&DirId>)
                     -> Box<FfiFuture<(Dir, DirMetadata)>> {
    trace!("Traverse directory tree to get the final subdirectory.");

    let dir_fut = match starting_directory {
        Some(dir_id) => {
            trace!("Traversal begins at given starting directory.");
            let dir_id = dir_id.clone();
            dir_helper::get(client.clone(), &dir_id).map(move |dir| (dir, dir_id)).into_box()
        }
        None => {
            trace!("Traversal begins at user-root-directory.");
            dir_helper::user_root_dir(client.clone())
        }
    };

    let tokens_iter = tokens.to_owned().into_iter().map(Ok);
    let c2 = client.clone();

    dir_fut.map_err(FfiError::from)
        .and_then(move |(current_dir, start_dir_id)| {
            let (dir_id, key) = start_dir_id;
            let meta = DirMetadata::new(*dir_id.name(), "root", Vec::new(), key);

            stream::iter(tokens_iter).fold((current_dir, meta), move |(dir, _metadata), token| {
                trace!("Traversing to dir with name: {}", token);

                let metadata = fry!(dir.find_sub_dir(&token)
                        .ok_or(FfiError::PathNotFound))
                    .clone();

                dir_helper::get(c2.clone(), &metadata.id())
                    .map(move |dir| (dir, metadata))
                    .map_err(FfiError::from)
                    .into_box()
            })
        })
        .into_box()
}

// Return a Dir corresponding to the path.
pub fn dir(client: &Client,
           app: &App,
           path: &str,
           is_shared: bool)
           -> Box<FfiFuture<(Dir, DirMetadata)>> {
    let c2 = client.clone();
    let tokens = tokenise_path(path, false);

    app.root_dir(client.clone(), is_shared)
        .and_then(move |start_dir| final_sub_dir(&c2, &tokens, Some(&start_dir)))
        .into_box()
}

pub fn dir_and_file(client: &Client,
                    app: &App,
                    path: &str,
                    is_shared: bool)
                    -> Box<FfiFuture<(Dir, DirMetadata, String)>> {
    let mut tokens = tokenise_path(path, false);
    let file_name = fry!(tokens.pop().ok_or(FfiError::PathNotFound));
    let c2 = client.clone();

    app.root_dir(client.clone(), is_shared)
        .and_then(move |start_dir| final_sub_dir(&c2, &tokens, Some(&start_dir)))
        .map(move |(dir_listing, dir_meta)| (dir_listing, dir_meta, file_name))
        .into_box()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_conversion() {
        let (ptr, size, cap) = string_to_c_utf8(String::new());
        assert_eq!(size, 0);
        unsafe {
            let _ = Vec::from_raw_parts(ptr, size, cap);
        }

        let (ptr, size, cap) = string_to_c_utf8("hello world".to_owned());
        assert!(ptr != 0 as *mut u8);
        assert_eq!(size, 11);
        assert!(cap >= 11);
        unsafe {
            let _ = Vec::from_raw_parts(ptr, size, cap);
        }
    }
}
