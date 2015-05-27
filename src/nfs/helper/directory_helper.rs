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
use routing::NameType;
use routing::sendable::Sendable;
use cbor;
use Client;

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
    pub fn create(&mut self, directory: DirectoryListing) -> Result<(), &str> {
        let serialised_directory = serialise(directory.clone());
        let immutable_data = ImmutableData::new(serialised_directory);
        // FIXME: Krishna - pass owner from client.my-account once account creation is completed
        let owner: NameType = NameType([2u8;64]);
        self.client.put(immutable_data.clone());
        let mut sdv: StructuredData = StructuredData::new(directory.get_id(), owner,
            vec![vec![immutable_data.name()]]);
        self.client.put(sdv);
        Ok(())
    }

    /// Updates an existing DirectoryListing in the network.
    pub fn update(&mut self, directory: DirectoryListing) -> Result<(), &str> {
        let get_result = self.client.get(directory.get_id());
        if get_result.is_err() {
            return Err("Could not find data");
        }
        let mut sdv: StructuredData = deserialise(get_result.unwrap());
        let serialised_directory = serialise(directory.clone());
        let immutable_data = ImmutableData::new(serialised_directory);
        self.client.put(immutable_data.clone());
        let mut versions = sdv.get_value();
        versions[0].push(immutable_data.name());
        sdv.set_value(versions);
        self.client.put(sdv);
        Ok(())
    }

    /// Return the versions of the directory
    pub fn get_versions(&mut self, directory_id: NameType) -> Result<Vec<NameType>, &str> {
        let get_result = self.client.get(directory_id);
        if get_result.is_err() {
            return Err("Could not find Directory");
        }
        let sdv: StructuredData = deserialise(get_result.unwrap());
        Ok(sdv.get_value()[0].clone())
    }

    /// Return the DirectoryListing for the specified version
    pub fn get_by_version(&mut self, directory_id: NameType, version: NameType) -> Result<DirectoryListing, &str> {
        let get_result = self.client.get(directory_id);
        if get_result.is_err() {
            return Err("Could not find Directory");
        }
        let sdv: StructuredData = deserialise(get_result.unwrap());
        if !sdv.get_value()[0].contains(&version) {
            return Err("Version not found");
        };
        let data_result = self.client.get(version);
        if data_result.is_err() {
            return Err("Could not find data");
        }
        let imm: ImmutableData = deserialise(data_result.unwrap());
        Ok(deserialise(imm.get_value().clone()))
    }

    /// Return the DirectoryListing for the latest version
    pub fn get(&mut self, directory_id: NameType) -> Result<DirectoryListing, &str> {
        let get_result = self.client.get(directory_id);
        if get_result.is_err() {
            return Err("Could not find data");
        }
        let sdv: StructuredData = deserialise(get_result.unwrap());
        let name = match sdv.get_value()[0].last() {
            Some(data) => NameType(data.0),
            None => return Err("Could not find data")
        };
        let data_result = self.client.get(name);
        if data_result.is_err() {
            return Err("Could not find data");
        }
        let imm: ImmutableData = deserialise(data_result.unwrap());
        Ok(deserialise(imm.get_value().clone()))
    }
}
