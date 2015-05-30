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
use std::sync;
use super::network_storage::NetworkStorage;
use self_encryption;
use std::fs;
use rand;
use rand::Rng;
use routing;
use client;
use ResponseNotifier;

pub struct Writer {
    file: nfs::types::File,
    directory: nfs::types::DirectoryListing,
    self_encryptor: self_encryption::SelfEncryptor<NetworkStorage>,
}

impl Writer {

    pub fn new(directory: nfs::types::DirectoryListing, file: nfs::types::File,
        routing: sync::Arc<::std::sync::Mutex<routing::routing_client::RoutingClient<client::callback_interface::CallbackInterface>>>,
        interface: ::std::sync::Arc<::std::sync::Mutex<client::callback_interface::CallbackInterface>>,
        response_notifier: ResponseNotifier) -> Writer {
        let storage = sync::Arc::new(NetworkStorage::new(routing, interface, response_notifier));
        Writer {
            file: file.clone(),
            directory: directory,
            self_encryptor: self_encryption::SelfEncryptor::new(storage.clone(), file.get_datamap()),
        }
    }

    pub fn write(&mut self, data: &[u8], position: u64) {
        self.self_encryptor.write(data, position);
    }

    pub fn close(mut self) -> self_encryption::datamap::DataMap {
        let datamap = self.self_encryptor.close();
        // update file object with datamap
        // update directory
        datamap
    }

}
