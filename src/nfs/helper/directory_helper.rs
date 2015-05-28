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
use nfs::types::DirectoryListing;
use maidsafe_types::{ImmutableData, StructuredData};
use rustc_serialize::{Decodable, Encodable};
use routing;
use routing::NameType;
use routing::sendable::Sendable;
use cbor;
use client::Client;
use WaitCondition;

/// DirectoryHelper provides helper functions to perform Operations on Directory
pub struct DirectoryHelper<'a> {
    client: &'a mut Client
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

impl <'a> DirectoryHelper<'a> {
    /// Create a new DirectoryHelper instance
    pub fn new(client: &'a mut Client) -> DirectoryHelper<'a> {
        DirectoryHelper {
            client: client
        }
    }

    /// Creates a Directory in the network.
    pub fn create(&mut self, owner: NameType, directory_name: String, user_metadata: Vec<u8>) -> Result<(), &str> {
        let directory = DirectoryListing::new(directory_name, user_metadata);
        let serialised_directory = serialise(directory.clone());
        let immutable_data = ImmutableData::new(serialised_directory);
        self.client.put(immutable_data.clone());
        let mut sdv: StructuredData = StructuredData::new(directory.get_id(), owner,
            vec![immutable_data.name()]);
        self.client.put(sdv);
        Ok(())
    }

    fn get_data(&mut self, wait_condition: WaitCondition) -> Result<Vec<u8>, routing::error::ResponseError>{
        let waiting_message_id = wait_condition.0.clone();
        let pair = wait_condition.1.clone();
        let &(ref lock, ref cvar) = &*pair;
        loop {
            let mut message_id = lock.lock().unwrap();
            message_id = cvar.wait(message_id).unwrap();
            if *message_id == waiting_message_id {
                return self.client.get_response(*message_id);
            }
        }
    }

    /// Updates an existing DirectoryListing in the network.
    pub fn update(&mut self, directory: DirectoryListing) -> Result<(), &str> {
        let get = self.client.get(directory.get_id());
        if get.is_err() {
            return Err("Network IO Error");
        }
        let data = self.get_data(get.unwrap());
        if data.is_err() {
            return Err("Routing Response Error");
        }
        let mut sdv: StructuredData = deserialise(data.unwrap());
        let serialised_directory = serialise(directory.clone());
        let immutable_data = ImmutableData::new(serialised_directory);
        self.client.put(immutable_data.clone());
        let mut versions = sdv.value();
        versions.push(immutable_data.name());
        sdv.set_value(versions);
        self.client.put(sdv);
        Ok(())
    }

    /// Return the versions of the directory
    pub fn get_versions(&mut self, directory_id: NameType) -> Result<Vec<NameType>, &str> {
        let get = self.client.get(directory_id);
        if get.is_err() {
            return Err("Network IO Error");
        }
        let data = self.get_data(get.unwrap());
        if data.is_err() {
            return Err("Routing Response Error");
        }
        let sdv: StructuredData = deserialise(data.unwrap());
        Ok(sdv.value())
    }

    /// Return the DirectoryListing for the specified version
    pub fn get_by_version(&mut self, directory_id: NameType, version: NameType) -> Result<DirectoryListing, &str> {
        let get = self.client.get(directory_id);
        if get.is_err() {
            return Err("Network IO Error");
        }
        let data = self.get_data(get.unwrap());
        if data.is_err() {
            return Err("Routing Response Error");
        }
        let sdv: StructuredData = deserialise(data.unwrap());
        if !sdv.value().contains(&version) {
            return Err("Version not found");
        };
        let get_data = self.client.get(version);
        if get_data.is_err() {
            return Err("Network IO Error");
        }
        let imm_data = self.get_data(get_data.unwrap());
        if imm_data.is_err() {
            return Err("Routing Response Error");
        }
        let imm: ImmutableData = deserialise(imm_data.unwrap());
        Ok(deserialise(imm.value().clone()))
    }

    /// Return the DirectoryListing for the latest version
    pub fn get(&mut self, directory_id: NameType) -> Result<DirectoryListing, &str> {
        let get = self.client.get(directory_id);
        if get.is_err() {
            return Err("Network IO Error");
        }
        let data = self.get_data(get.unwrap());
        if data.is_err() {
            return Err("Routing Response Error");
        }
        let sdv: StructuredData = deserialise(data.unwrap());
        let name = match sdv.value().last() {
            Some(data) => NameType(data.0),
            None => return Err("Could not find data")
        };
        let get_data = self.client.get(name);
        if get_data.is_err() {
            return Err("Network IO Error");
        }
        let imm_data = self.get_data(get_data.unwrap());
        if imm_data.is_err() {
            return Err("Routing Response Error");
        }
        let imm: ImmutableData = deserialise(imm_data.unwrap());
        Ok(deserialise(imm.value().clone()))
    }
}
