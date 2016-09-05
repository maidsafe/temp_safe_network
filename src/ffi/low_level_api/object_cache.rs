// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

// TODO Remove
#![allow(unused)]

use lru_cache::LruCache;
use routing::{DataIdentifier, StructuredData};
use rust_sodium::crypto::{box_, sign};
// use self_encryption::SequentialEncryptor;
use std::sync::{Mutex, ONCE_INIT, Once};
use std::u64;
use super::{AppendableDataHandle, CipherOptHandle, DataIdHandle, EncryptKeyHandle, ObjectHandle,
            SelfEncryptorHandle, SignKeyHandle, StructDataHandle};
// use super::appendable_data::AppendableData;
use super::cipher_opt::CipherOpt;

const DEFAULT_CAPACITY: usize = 100;

pub fn object_cache() -> &'static Mutex<ObjectCache> {
    static mut OBJECT_CACHE: *const Mutex<ObjectCache> = 0 as *const Mutex<ObjectCache>;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| OBJECT_CACHE = Box::into_raw(Box::new(Default::default())));
        &*OBJECT_CACHE
    }
}

// TODO Instead of this make each field a Mutex - that way operation on one handle does not block
// operations on others.
pub struct ObjectCache {
    new_handle: ObjectHandle,
    pub struct_data: LruCache<StructDataHandle, StructuredData>,
    pub data_id: LruCache<DataIdHandle, DataIdentifier>,
    // pub appendable_data: LruCache<AppendableDataHandle, AppendableData>,
    // pub se: LruCache<SelfEncryptorHandle, SequentialEncryptor>,
    pub cipher_opt: LruCache<CipherOptHandle, CipherOpt>,
    pub encrypt_key: LruCache<EncryptKeyHandle, box_::PublicKey>,
    pub sign_key: LruCache<SignKeyHandle, sign::PublicKey>,
}

impl ObjectCache {
    pub fn new_handle(&mut self) -> ObjectHandle {
        self.new_handle = self.new_handle.wrapping_add(1);
        self.new_handle
    }

    pub fn reset(&mut self) {
        self.new_handle = u64::MAX;

        self.struct_data.clear();
        self.data_id.clear();
        // self.appendable_data.clear();
        // self.se.clear();
        self.cipher_opt.clear();
        self.encrypt_key.clear();
        self.sign_key.clear();
    }
}

impl Default for ObjectCache {
    fn default() -> Self {
        ObjectCache {
            new_handle: u64::MAX,
            struct_data: LruCache::new(DEFAULT_CAPACITY),
            data_id: LruCache::new(DEFAULT_CAPACITY),
            //  appendable_data: LruCache::new(DEFAULT_CAPACITY),
            // se: LruCache::new(DEFAULT_CAPACITY),
            cipher_opt: LruCache::new(DEFAULT_CAPACITY),
            encrypt_key: LruCache::new(DEFAULT_CAPACITY),
            sign_key: LruCache::new(DEFAULT_CAPACITY),
        }
    }
}
