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

use std::fmt;
use std::fs;
use std::fs::{File};
use std::io::prelude::*;
use std::path::Path;
use std::string::String;
use maidsafe_types;
use WaitCondition;
use ResponseNotifier;
use client;
use routing;
use rustc_serialize::{Decodable, Encodable};
use cbor;
use nfs;
use routing::sendable::Sendable;

// TODO update tag values for SDV and Immutable data
pub struct NetworkStorage {
    routing: ::std::sync::Arc<::std::sync::Mutex<routing::routing_client::RoutingClient<client::callback_interface::CallbackInterface>>>,
    callback_interface: ::std::sync::Arc<::std::sync::Mutex<client::callback_interface::CallbackInterface>>,
    response_notifier: ResponseNotifier
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
    pub fn new(routing: ::std::sync::Arc<::std::sync::Mutex<routing::routing_client::RoutingClient<client::callback_interface::CallbackInterface>>>,
        callback_interface: ::std::sync::Arc<::std::sync::Mutex<client::callback_interface::CallbackInterface>>,
        response_notifier: ResponseNotifier) -> NetworkStorage {
        NetworkStorage {
            routing: routing,
            callback_interface: callback_interface,
            response_notifier: response_notifier
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
                let interface = self.callback_interface.clone();
                return interface.lock().unwrap().get_response(*message_id);
            }
        }
    }

    fn network_get(&self, tag: u64, name: routing::NameType) -> Result<::WaitCondition, ::IoError>{
        let lock = self.routing.clone();
        let mut routing = lock.lock().unwrap();
        match routing.get(tag, name) {
            Ok(id)      => Ok((id, self.response_notifier.clone())),
            Err(io_err) => Err(io_err),
        }
    }

    pub fn save_directory(&self, directory: nfs::types::DirectoryListing) -> Result<(), &str> {
        let get = self.network_get(101u64, directory.get_id());
        if get.is_err() {
            return Err("Network IO Error");
        }
        let data = self.blocked_read(get.unwrap());
        if data.is_err() {
            return Err("Routing Response Error");
        }
        let mut sdv: maidsafe_types::StructuredData = deserialise(data.unwrap());
        let serialised_directory = serialise(directory.clone());
        let immutable_data = maidsafe_types::ImmutableData::new(serialised_directory);
        let lock = self.routing.clone();
        let mut routing = lock.lock().unwrap();
        routing.put(immutable_data.clone());
        let mut versions = sdv.value();
        versions.push(immutable_data.name());
        sdv.set_value(versions);
        routing.put(sdv);
        Ok(())
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
        let get_result = self.network_get(100u64, routing::NameType(name_id));
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
        let mut routing = self.routing.clone();
        routing.lock().unwrap().put(sendable);
    }

}
