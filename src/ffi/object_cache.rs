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

use ffi::App;
use ffi::errors::FfiError;
use ffi::low_level_api::appendable_data::AppendableData;
use ffi::low_level_api::cipher_opt::CipherOpt;
use ffi::low_level_api::immut_data::{SelfEncryptorReaderWrapper, SelfEncryptorWriterWrapper};
use lru_cache::LruCache;
use routing::DataIdentifier;
use rust_sodium::crypto::{box_, sign};
use std::u64;

/// Object handle associated with objects. In normal C API one would expect
/// rust code to pass pointers to opaque object to C. C code would then need to
/// pass these pointers back each time they needed rust code to execute
/// something on those objects. However our code base deals with communication
/// over Web framework (like WebServers for instance). Hence it is not possible
/// to pass pointers to remote apps interfacing with us. Pointers represent
/// handle to actual object.  Using similar concept, we instead pass
/// ObjectHandle type over Web interface and manage the objects ourselves. This
/// leads to extra type and memory safety and no chance of Undefined Behaviour.
/// Passing of pointer handles to C is replaced by passing of ObjectHandle to
/// remote apps which they will use to do RPC's.
pub type ObjectHandle = u64;

/// Disambiguating ObjectHandle
pub type AppHandle = ObjectHandle;
/// Disambiguating ObjectHandle
pub type AppendableDataHandle = ObjectHandle;
/// Disambiguating ObjectHandle
pub type StructDataHandle = ObjectHandle;
/// Disambiguating ObjectHandle
pub type DataIdHandle = ObjectHandle;
/// Disambiguating ObjectHandle
pub type SelfEncryptorReaderHandle = ObjectHandle;
/// Disambiguating ObjectHandle
pub type SelfEncryptorWriterHandle = ObjectHandle;
/// Disambiguating ObjectHandle
pub type CipherOptHandle = ObjectHandle;
/// Disambiguating ObjectHandle
pub type EncryptKeyHandle = ObjectHandle;
/// Disambiguating ObjectHandle
pub type SignKeyHandle = ObjectHandle;

const DEFAULT_CAPACITY: usize = 100;

/// Contains session object cache
pub struct ObjectCache {
    new_handle: ObjectHandle,
    app: LruCache<AppHandle, App>,
    // struct_data: LruCache<StructDataHandle, StructuredData>,
    data_id: LruCache<DataIdHandle, DataIdentifier>,
    appendable_data: LruCache<AppendableDataHandle, AppendableData>,
    se_reader: LruCache<SelfEncryptorReaderHandle, SelfEncryptorReaderWrapper>,
    se_writer: LruCache<SelfEncryptorWriterHandle, SelfEncryptorWriterWrapper>,
    cipher_opt: LruCache<CipherOptHandle, CipherOpt>,
    encrypt_key: LruCache<EncryptKeyHandle, box_::PublicKey>,
    sign_key: LruCache<SignKeyHandle, sign::PublicKey>,
}

impl ObjectCache {
    pub fn new_handle(&mut self) -> ObjectHandle {
        self.new_handle = self.new_handle.wrapping_add(1);
        self.new_handle
    }

    pub fn reset(&mut self) {
        self.new_handle = u64::MAX;

        self.app.clear();
        // self.struct_data.clear();
        self.data_id.clear();
        self.appendable_data.clear();
        self.se_reader.clear();
        self.se_writer.clear();
        self.cipher_opt.clear();
        self.encrypt_key.clear();
        self.sign_key.clear();
    }

    // ----------------------------------------------------------
    pub fn insert_app(&mut self, app: App) -> AppHandle {
        let handle = self.new_handle();
        if let Some(_) = self.app.insert(handle, app) {
            debug!("Displaced App from ObjectCache");
        }
        handle
    }

    pub fn get_app(&mut self, handle: AppHandle) -> Result<&mut App, FfiError> {
        self.app.get_mut(&handle).ok_or(FfiError::InvalidAppendableDataHandle)
    }

    pub fn remove_app(&mut self, handle: AppHandle) -> Result<App, FfiError> {
        self.app.remove(&handle).ok_or(FfiError::InvalidAppendableDataHandle)
    }

    // ----------------------------------------------------------
    pub fn insert_ad(&mut self, data: AppendableData) -> AppendableDataHandle {
        let handle = self.new_handle();
        if let Some(prev) = self.appendable_data.insert(handle, data) {
            debug!("Displaced AppendableData from ObjectCache: {:?}", prev);
        }

        handle
    }

    pub fn get_ad(&mut self,
                  handle: AppendableDataHandle)
                  -> Result<&mut AppendableData, FfiError> {
        self.appendable_data.get_mut(&handle).ok_or(FfiError::InvalidAppendableDataHandle)
    }

    pub fn remove_ad(&mut self, handle: AppendableDataHandle) -> Result<AppendableData, FfiError> {
        self.appendable_data.remove(&handle).ok_or(FfiError::InvalidAppendableDataHandle)
    }

    // ----------------------------------------------------------
    pub fn insert_cipher_opt(&mut self, cipher_opt: CipherOpt) -> CipherOptHandle {
        let handle = self.new_handle();
        if let Some(prev) = self.cipher_opt.insert(handle, cipher_opt) {
            debug!("Displaced CipherOpt from ObjectCache: {:?}", prev);
        }

        handle
    }

    pub fn get_cipher_opt(&mut self, handle: CipherOptHandle) -> Result<&mut CipherOpt, FfiError> {
        self.cipher_opt.get_mut(&handle).ok_or(FfiError::InvalidCipherOptHandle)
    }

    pub fn remove_cipher_opt(&mut self, handle: CipherOptHandle) -> Result<CipherOpt, FfiError> {
        self.cipher_opt.remove(&handle).ok_or(FfiError::InvalidCipherOptHandle)
    }

    // ----------------------------------------------------------
    pub fn insert_data_id(&mut self, data_id: DataIdentifier) -> DataIdHandle {
        let handle = self.new_handle();
        if let Some(prev) = self.data_id.insert(handle, data_id) {
            debug!("Displaced DataIdentifier from ObjectCache: {:?}", prev);
        }

        handle
    }

    pub fn get_data_id(&mut self, handle: DataIdHandle) -> Result<&mut DataIdentifier, FfiError> {
        self.data_id.get_mut(&handle).ok_or(FfiError::InvalidDataIdHandle)
    }

    pub fn remove_data_id(&mut self, handle: DataIdHandle) -> Result<DataIdentifier, FfiError> {
        self.data_id.remove(&handle).ok_or(FfiError::InvalidDataIdHandle)
    }

    // ----------------------------------------------------------
    pub fn insert_se_reader(&mut self,
                            se_reader: SelfEncryptorReaderWrapper)
                            -> SelfEncryptorReaderHandle {
        let handle = self.new_handle();
        if let Some(_) = self.se_reader.insert(handle, se_reader) {
            debug!("Displaced SelfEncryptorReaderWrapper from ObjectCache");
        }

        handle
    }

    pub fn get_se_reader(&mut self,
                         handle: SelfEncryptorReaderHandle)
                         -> Result<&mut SelfEncryptorReaderWrapper, FfiError> {
        self.se_reader.get_mut(&handle).ok_or(FfiError::InvalidSelfEncryptorHandle)
    }

    pub fn remove_se_reader(&mut self,
                            handle: SelfEncryptorReaderHandle)
                            -> Result<SelfEncryptorReaderWrapper, FfiError> {
        self.se_reader.remove(&handle).ok_or(FfiError::InvalidSelfEncryptorHandle)
    }

    // ----------------------------------------------------------
    pub fn insert_se_writer(&mut self,
                            se_reader: SelfEncryptorWriterWrapper)
                            -> SelfEncryptorWriterHandle {
        let handle = self.new_handle();
        if let Some(_) = self.se_writer.insert(handle, se_reader) {
            debug!("Displaced SelfEncryptorWriterWrapper from ObjectCache");
        }

        handle
    }

    pub fn get_se_writer(&mut self,
                         handle: SelfEncryptorWriterHandle)
                         -> Result<&mut SelfEncryptorWriterWrapper, FfiError> {
        self.se_writer.get_mut(&handle).ok_or(FfiError::InvalidSelfEncryptorHandle)
    }

    pub fn remove_se_writer(&mut self,
                            handle: SelfEncryptorWriterHandle)
                            -> Result<SelfEncryptorWriterWrapper, FfiError> {
        self.se_writer.remove(&handle).ok_or(FfiError::InvalidSelfEncryptorHandle)
    }

    // ----------------------------------------------------------
    pub fn insert_sign_key(&mut self, key: sign::PublicKey) -> SignKeyHandle {
        let handle = self.new_handle();
        if let Some(prev) = self.sign_key.insert(handle, key) {
            debug!("Displaced Sign Key from ObjectCache: {:?}", prev);
        }

        handle
    }

    pub fn get_sign_key(&mut self,
                        handle: SignKeyHandle)
                        -> Result<&mut sign::PublicKey, FfiError> {
        self.sign_key.get_mut(&handle).ok_or(FfiError::InvalidSignKeyHandle)
    }

    pub fn remove_sign_key(&mut self, handle: SignKeyHandle) -> Result<sign::PublicKey, FfiError> {
        self.sign_key.remove(&handle).ok_or(FfiError::InvalidSignKeyHandle)
    }

    // ----------------------------------------------------------
    pub fn insert_encrypt_key(&mut self, key: box_::PublicKey) -> EncryptKeyHandle {
        let handle = self.new_handle();
        if let Some(prev) = self.encrypt_key.insert(handle, key) {
            debug!("Displaced Encrypt Key from ObjectCache: {:?}", prev);
        }

        handle
    }

    pub fn get_encrypt_key(&mut self,
                           handle: EncryptKeyHandle)
                           -> Result<&mut box_::PublicKey, FfiError> {
        self.encrypt_key.get_mut(&handle).ok_or(FfiError::InvalidEncryptKeyHandle)
    }

    pub fn remove_encrypt_key(&mut self,
                              handle: EncryptKeyHandle)
                              -> Result<box_::PublicKey, FfiError> {
        self.encrypt_key.remove(&handle).ok_or(FfiError::InvalidEncryptKeyHandle)
    }

    // ----------------------------------------------------------
    // pub fn insert_sd(&mut self, data: StructuredData) -> StructDataHandle {
    //     let handle = self.new_handle();
    //     if let Some(prev) = self.struct_data.insert(handle, data) {
    //         debug!("Displaced StructuredData from ObjectCache: {:?}", prev);
    //     }

    //     handle
    // }

    // pub fn get_sd(&mut self, handle: StructDataHandle) -> Result<&mut
    // StructuredData, FfiError> {
    // self.struct_data.get_mut(&handle).ok_or(FfiError::
    // InvalidStructDataHandle)
    // }

    // pub fn remove_sd(&mut self, handle: StructDataHandle) ->
    // Result<StructuredData, FfiError> {
    //     self.struct_data.remove(&handle).ok_or(FfiError::InvalidStructDataHandle)
    // }
}

impl Default for ObjectCache {
    fn default() -> Self {
        ObjectCache {
            new_handle: u64::MAX,
            app: LruCache::new(DEFAULT_CAPACITY),
            // struct_data: LruCache::new(DEFAULT_CAPACITY),
            data_id: LruCache::new(DEFAULT_CAPACITY),
            appendable_data: LruCache::new(DEFAULT_CAPACITY),
            se_reader: LruCache::new(DEFAULT_CAPACITY),
            se_writer: LruCache::new(DEFAULT_CAPACITY),
            cipher_opt: LruCache::new(DEFAULT_CAPACITY),
            encrypt_key: LruCache::new(DEFAULT_CAPACITY),
            sign_key: LruCache::new(DEFAULT_CAPACITY),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand;
    use routing::DataIdentifier;
    use super::*;

    #[test]
    fn reset() {
        let mut object_cache = ObjectCache::default();

        let handle = object_cache.insert_data_id(DataIdentifier::Immutable(rand::random()));
        assert!(object_cache.get_data_id(handle).is_ok());

        object_cache.reset();
        assert!(object_cache.data_id.is_empty());
    }
}
