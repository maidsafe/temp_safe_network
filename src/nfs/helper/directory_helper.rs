// Copyright 2015 MaidSafe.net limited.
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
use nfs;
use maidsafe_types;
use rustc_serialize::{Decodable, Encodable};
use routing;
use routing::sendable::Sendable;
use cbor;
use client;
use WaitCondition;
use std::error::Error;
use maidsafe_types::TypeTag;
use self_encryption;

// TODO: Remove the tag_id values and get from maidsafe_types
const SDV_TAG: u64 = 100u64;

/// DirectoryHelper provides helper functions to perform Operations on Directory
pub struct DirectoryHelper {
    client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>
}

fn serialise<T>(data: T) -> Vec<u8> where T : Encodable {
    let mut e = cbor::Encoder::from_memory();
    e.encode(&[&data]);
    e.into_bytes()
}

fn deserialise<T>(data: Vec<u8>) -> T where T : Decodable {
    let mut d = cbor::Decoder::from_bytes(data);
    d.decode().next().unwrap().unwrap()
}


impl DirectoryHelper {
    /// Create a new DirectoryHelper instance
    pub fn new(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>) -> DirectoryHelper {
        DirectoryHelper {
            client: client
        }
    }

    /// Creates a Directory in the network.
    pub fn create(&mut self, parent_dir_id: routing::NameType, directory_name: String, user_metadata: Vec<u8>) -> Result<(), &str> {
        let mutex_client = self.client.clone();
        let client = mutex_client.lock().unwrap();
        let directory = nfs::directory_listing::DirectoryListing::new(parent_dir_id, directory_name, user_metadata);
        let mut se = self_encryption::SelfEncryptor::new(::std::sync::Arc::new(nfs::io::NetworkStorage::new(self.client.clone())), self_encryption::datamap::DataMap::None);
        se.write(&serialise(directory.clone())[..], 0);
        let datamap = se.close();

        let encrypt_result = client.hybrid_encrypt(&serialise(datamap)[..], self.get_nonce(directory.get_id().clone(), directory.get_parent_dir_id().clone()));
        if encrypt_result.is_err() {
            return Err("Encryption failed");
        }

        let immutable_data = maidsafe_types::ImmutableData::new(encrypt_result.unwrap());
        let save_res = self.network_put(self.client.clone(), immutable_data.clone());
        if save_res.is_err() {
            return Err("Save Failed");
        }
        let mut sdv: maidsafe_types::StructuredData = maidsafe_types::StructuredData::new(directory.get_id(), client.get_owner(),
            vec![immutable_data.name()]);
        let save_sdv_res = self.network_put(self.client.clone(), sdv);
        if save_res.is_err() {
            return Err("Failed to create directory");
        }
        Ok(())
    }

    /// Updates an existing DirectoryListing in the network.
    pub fn update(&mut self, directory: nfs::directory_listing::DirectoryListing) -> Result<(), &str> {
        let mutex_client = self.client.clone();
        let client = mutex_client.lock().unwrap();
        let result = self.network_get(self.client.clone(), SDV_TAG, directory.get_id());
        if result.is_err() {
            return Err("Network IO Error");
        }
        let mut sdv: maidsafe_types::StructuredData = deserialise(result.unwrap());

        let mut se = self_encryption::SelfEncryptor::new(::std::sync::Arc::new(nfs::io::NetworkStorage::new(self.client.clone())), self_encryption::datamap::DataMap::None);
        se.write(&serialise(directory.clone())[..], 0);
        let datamap = se.close();

        let encrypt_result = client.hybrid_encrypt(&serialise(datamap)[..], self.get_nonce(directory.get_id().clone(), directory.get_parent_dir_id().clone()));
        if encrypt_result.is_err() {
            return Err("Encryption failed");
        }

        let immutable_data = maidsafe_types::ImmutableData::new(encrypt_result.unwrap());
        let immutable_data_put_result = self.network_put(self.client.clone(), immutable_data.clone());
        if immutable_data_put_result.is_err() {
            return Err("Failed to save directory");
        };
        let mut versions = sdv.value();
        versions.push(immutable_data.name());
        sdv.set_value(versions);
        let sdv_put_result = self.network_put(self.client.clone(), sdv);
        if sdv_put_result.is_err() {
            return Err("Failed to update directory version");
        };
        Ok(())
    }

    /// Return the versions of the directory
    pub fn get_versions(&mut self, directory_id: routing::NameType) -> Result<Vec<routing::NameType>, &str> {
        let result = self.network_get(self.client.clone(), SDV_TAG, directory_id);
        if result.is_err() {
            return Err("Network IO Error");
        }
        let sdv: maidsafe_types::StructuredData = deserialise(result.unwrap());
        Ok(sdv.value())
    }

    /// Return the DirectoryListing for the specified version
    pub fn get_by_version(&mut self, directory_id: routing::NameType, parent_directory_id: routing::NameType, version: routing::NameType) -> Result<nfs::directory_listing::DirectoryListing, &str> {
        let data_res = self.network_get(self.client.clone(), SDV_TAG, directory_id.clone());
        if data_res.is_err() {
            return Err("Network IO Error");
        }
        let sdv: maidsafe_types::StructuredData = deserialise(data_res.unwrap());
        if !sdv.value().contains(&version) {
            return Err("Version not found");
        };
        let immutable_data_type_id: maidsafe_types::data::ImmutableDataTypeTag = unsafe { ::std::mem::uninitialized() };
        let get_data = self.network_get(self.client.clone(), immutable_data_type_id.type_tag(), version);
        if get_data.is_err() {
            return Err("Network IO Error");
        }
        let imm: maidsafe_types::ImmutableData = deserialise(get_data.unwrap());

        let client_mutex = self.client.clone();
        let client = client_mutex.lock().unwrap();
        let decrypt_result = client.hybrid_decrypt(&imm.value()[..], self.get_nonce(directory_id.clone(), parent_directory_id.clone()));
        if decrypt_result.is_none() {
            return Err("Failed to decrypt");
        }
        let datamap = deserialise(decrypt_result.unwrap());

        let mut se = self_encryption::SelfEncryptor::new(::std::sync::Arc::new(nfs::io::NetworkStorage::new(self.client.clone())), datamap);
        let size = se.len();
        Ok(deserialise(se.read(0, size)))
    }

    /// Return the DirectoryListing for the latest version
    pub fn get(&mut self, directory_id: routing::NameType, parent_directory_id: routing::NameType) -> Result<nfs::directory_listing::DirectoryListing, &str> {
        let sdv_res = self.network_get(self.client.clone(), SDV_TAG, directory_id.clone());
        if sdv_res.is_err() {
            return Err("Network IO Error");
        }
        let sdv: maidsafe_types::StructuredData = deserialise(sdv_res.unwrap());
        let name = match sdv.value().last() {
            Some(data) => routing::NameType(data.0),
            None => return Err("Could not find data")
        };
        let immutable_data_type_id: maidsafe_types::data::ImmutableDataTypeTag = unsafe { ::std::mem::uninitialized() };
        let imm_data_res = self.network_get(self.client.clone(), immutable_data_type_id.type_tag(), name);
        if imm_data_res.is_err() {
            return Err("Network IO Error");
        }
        let imm: maidsafe_types::ImmutableData = deserialise(imm_data_res.unwrap());

        let client_mutex = self.client.clone();
        let client = client_mutex.lock().unwrap();
        let decrypt_result = client.hybrid_decrypt(&imm.value()[..], self.get_nonce(directory_id.clone(), parent_directory_id.clone()));
        if decrypt_result.is_none() {
            return Err("Failed to decrypt");
        }
        let datamap = deserialise(decrypt_result.unwrap());

        let mut se = self_encryption::SelfEncryptor::new(::std::sync::Arc::new(nfs::io::NetworkStorage::new(self.client.clone())), datamap);
        let size = se.len();
        Ok(deserialise(se.read(0, size)))
    }

    fn get_response(&self, client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>, wait_condition: WaitCondition) -> Result<Vec<u8>, &str> {
        let waiting_message_id = wait_condition.0.clone();
        let pair = wait_condition.1.clone();
        let &(ref lock, ref cvar) = &*pair;
        loop {
            let mut message_id = lock.lock().unwrap();
            message_id = cvar.wait(message_id).unwrap();
            if *message_id == waiting_message_id {
                let client_mutex = client.clone();
                let mut client = client_mutex.lock().unwrap();
                 return match client.get_response(*message_id) {
                     Ok(data) => Ok(data),
                     Err(_) => Err("IO Error")
                 }
            }
        }
    }

    fn network_get(&self, client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>, tag_id: u64,
        name: routing::NameType) -> Result<Vec<u8>, &str> {
        let client_mutex = client.clone();
        let mut safe = client_mutex.lock().unwrap();
        let get_result = safe.get(tag_id, name);
        if get_result.is_err() {
            return Err("Network IO Error");
        }
        self.get_response(client, get_result.unwrap())
    }

    fn network_put<T>(&self, client_arc: ::std::sync::Arc<::std::sync::Mutex<client::Client>>, sendable: T) -> Result<Vec<u8>, &str> where T: Sendable {
        let client_mutex = client_arc.clone();
        let mut client = client_mutex.lock().unwrap();
        let get_result = client.put(sendable);
        if get_result.is_err() {
            return Err("Network IO Error");
        }
        self.get_response(client_arc, get_result.unwrap())
    }

    fn get_nonce(&self, id: routing::NameType, parent_id: routing::NameType) -> Option<::sodiumoxide::crypto::asymmetricbox::Nonce> {
        let mut nonce = [0u8;24];
        for i in 0..24 {
            if i % 2 == 0 {
                nonce[i] = id.0[i];
                nonce[i+1] = parent_id.0[i];
            }
        }
        Some(::sodiumoxide::crypto::asymmetricbox::Nonce(nonce))
    }

}
