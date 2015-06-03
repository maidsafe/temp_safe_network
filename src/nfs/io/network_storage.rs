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
use self_encryption;
use maidsafe_types;
use WaitCondition;
use ResponseNotifier;
use client;
use routing;
use rustc_serialize::{Decodable, Encodable};
use cbor;
use nfs;
use routing::sendable::Sendable;
use maidsafe_types::TypeTag;

const IMMUTABLE_TAG: u64 = 101u64;

// TODO update tag values for SDV and Immutable data
pub struct NetworkStorage {
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


impl NetworkStorage {
    pub fn new(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>) -> NetworkStorage {
        NetworkStorage {
            client: client
        }
    }

    fn blocked_read(&self, wait_condition: WaitCondition) -> Result<Vec<u8>, routing::error::ResponseError>{
        let waiting_message_id = wait_condition.0.clone();
        let pair = wait_condition.1.clone();
        let &(ref lock, ref cvar) = &*pair;
        loop {
            let mut message_id = lock.lock().unwrap();
            message_id = cvar.wait(message_id).unwrap();
            if *message_id == waiting_message_id {
                let mut client = self.client.clone();
                return client.lock().unwrap().get_response(*message_id);
            }
        }
    }

}

// FIXME There is no error handling mechanism in self_encryption::Storage?
impl self_encryption::Storage for NetworkStorage {

    fn get(&self, name: Vec<u8>) -> Vec<u8> {
        let mut name_id = [0u8;64];
        assert_eq!(name.len(), 64);
        for i in 0..64 {
            name_id[i] = *name.get(i).unwrap();
        }
        let client_mutex = self.client.clone();
        let mut client = client_mutex.lock().unwrap();
        let immutable_data_type_id: maidsafe_types::ImmutableDataTypeTag = unsafe { ::std::mem::uninitialized() };
        let get_result = client.get(immutable_data_type_id.type_tag(), routing::NameType(name_id));
        if get_result.is_err() {
            return Vec::new();
        }
        let data = self.blocked_read(get_result.unwrap());
        if data.is_err() {
            return Vec::new();
        }
        data.unwrap()
    }

    fn put(&self, name: Vec<u8>, data: Vec<u8>) {
        let sendable = maidsafe_types::ImmutableData::new(data);
        let client_mutex = self.client.clone();
        let mut client = client_mutex.lock().unwrap();
        client.put(sendable);
    }

}
