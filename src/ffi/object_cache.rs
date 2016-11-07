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

use core::SelfEncryptionStorage;
use ffi::{App, AppHandle, AppendableDataHandle, CipherOptHandle, DataIdHandle, EncryptKeyHandle,
          ObjectHandle, SelfEncryptorReaderHandle, SelfEncryptorWriterHandle, SignKeyHandle,
          StructDataHandle};
use ffi::errors::FfiError;
use ffi::low_level_api::appendable_data::AppendableData;
use ffi::low_level_api::cipher_opt::CipherOpt;
use lru_cache::LruCache;
use routing::{DataIdentifier, StructuredData};
use rust_sodium::crypto::{box_, sign};
use self_encryption::{SelfEncryptor, SequentialEncryptor};
use std::cell::{Cell, RefCell, RefMut};
use std::rc::Rc;
use std::u64;

const DEFAULT_CAPACITY: usize = 100;

/// Contains session object cache
#[derive(Clone)]
pub struct ObjectCache {
    inner: Rc<Inner>,
}

struct Inner {
    handle_gen: HandleGenerator,
    ad: Store<AppendableData>,
    app: Store<App>,
    cipher_opt: Store<CipherOpt>,
    data_id: Store<DataIdentifier>,
    encrypt_key: Store<box_::PublicKey>,
    sd: Store<StructuredData>,
    se_reader: Store<SelfEncryptor<SelfEncryptionStorage>>,
    se_writer: Store<SequentialEncryptor<SelfEncryptionStorage>>,
    sign_key: Store<sign::PublicKey>,
}

impl ObjectCache {
    pub fn new() -> Self {
        ObjectCache {
            inner: Rc::new(Inner {
                handle_gen: HandleGenerator::new(),
                ad: Store::new(),
                app: Store::new(),
                cipher_opt: Store::new(),
                data_id: Store::new(),
                encrypt_key: Store::new(),
                sd: Store::new(),
                se_reader: Store::new(),
                se_writer: Store::new(),
                sign_key: Store::new(),
            }),
        }
    }

    pub fn reset(&self) {
        self.inner.handle_gen.reset();
        self.inner.ad.clear();
        self.inner.app.clear();
        self.inner.cipher_opt.clear();
        self.inner.data_id.clear();
        self.inner.encrypt_key.clear();
        self.inner.sd.clear();
        self.inner.se_reader.clear();
        self.inner.se_writer.clear();
        self.inner.sign_key.clear();
    }

    pub fn insert_sd_at(&self, handle: StructDataHandle, data: StructuredData) {
        let _ = self.inner.sd.insert(handle, data);
    }
}

macro_rules! impl_cache {
    ($name:ident,
     $ty:ty,
     $handle:ty,
     $error:ident,
     $get:ident,
     $insert:ident,
     $remove:ident) => {
        impl ObjectCache {
            pub fn $insert(&self, value: $ty) -> $handle {
                let handle = self.inner.handle_gen.gen();
                self.inner.$name.insert(handle, value);
                handle
            }

            pub fn $get(&self, handle: $handle) -> Result<RefMut<$ty>, FfiError> {
                self.inner.$name.get(handle).ok_or(FfiError::$error)
            }

            pub fn $remove(&self, handle: $handle) -> Result<$ty, FfiError> {
                self.inner.$name.remove(handle).ok_or(FfiError::$error)
            }
        }
    }
}

impl_cache!(app,
            App,
            AppHandle,
            InvalidAppHandle,
            get_app,
            insert_app,
            remove_app);
impl_cache!(ad,
            AppendableData,
            AppendableDataHandle,
            InvalidAppendableDataHandle,
            get_ad,
            insert_ad,
            remove_ad);
impl_cache!(cipher_opt,
            CipherOpt,
            CipherOptHandle,
            InvalidCipherOptHandle,
            get_cipher_opt,
            insert_cipher_opt,
            remove_cipher_opt);
impl_cache!(data_id,
            DataIdentifier,
            DataIdHandle,
            InvalidDataIdHandle,
            get_data_id,
            insert_data_id,
            remove_data_id);
impl_cache!(encrypt_key,
            box_::PublicKey,
            EncryptKeyHandle,
            InvalidEncryptKeyHandle,
            get_encrypt_key,
            insert_encrypt_key,
            remove_encrypt_key);
impl_cache!(se_reader,
            SelfEncryptor<SelfEncryptionStorage>,
            SelfEncryptorReaderHandle,
            InvalidSelfEncryptorHandle,
            get_se_reader,
            insert_se_reader,
            remove_se_reader);
impl_cache!(se_writer,
            SequentialEncryptor<SelfEncryptionStorage>,
            SelfEncryptorWriterHandle,
            InvalidSelfEncryptorHandle,
            get_se_writer,
            insert_se_writer,
            remove_se_writer);
impl_cache!(sign_key,
            sign::PublicKey,
            SignKeyHandle,
            InvalidSignKeyHandle,
            get_sign_key,
            insert_sign_key,
            remove_sign_key);
impl_cache!(sd,
            StructuredData,
            StructDataHandle,
            InvalidStructDataHandle,
            get_sd,
            insert_sd,
            remove_sd);

impl Default for ObjectCache {
    fn default() -> Self {
        Self::new()
    }
}

// Generator of unique object handles.
struct HandleGenerator(Cell<ObjectHandle>);

impl HandleGenerator {
    fn new() -> Self {
        HandleGenerator(Cell::new(u64::MAX))
    }

    fn gen(&self) -> ObjectHandle {
        let value = self.0.get().wrapping_add(1);
        self.0.set(value);
        value
    }

    fn reset(&self) {
        self.0.set(u64::MAX)
    }
}

struct Store<V> {
    inner: RefCell<LruCache<ObjectHandle, V>>,
}

impl<V> Store<V> {
    fn new() -> Self {
        Store { inner: RefCell::new(LruCache::new(DEFAULT_CAPACITY)) }
    }

    fn get(&self, handle: ObjectHandle) -> Option<RefMut<V>> {
        // TODO: find a way to avoid double lookup here.
        let mut inner = self.inner.borrow_mut();
        if inner.get_mut(&handle).is_some() {
            Some(RefMut::map(inner, |i| i.get_mut(&handle).unwrap()))
        } else {
            None
        }
    }

    fn insert(&self, handle: ObjectHandle, value: V) {
        let _ = self.inner.borrow_mut().insert(handle, value);
    }

    fn remove(&self, handle: ObjectHandle) -> Option<V> {
        self.inner.borrow_mut().remove(&handle)
    }

    fn clear(&self) {
        self.inner.borrow_mut().clear()
    }
}

#[cfg(test)]
mod tests {
    use rand;
    use routing::DataIdentifier;
    use super::*;

    #[test]
    fn reset() {
        let object_cache = ObjectCache::new();

        let handle = object_cache.insert_data_id(DataIdentifier::Immutable(rand::random()));
        assert!(object_cache.get_data_id(handle).is_ok());

        object_cache.reset();
        assert!(object_cache.get_data_id(handle).is_err());
    }
}
