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
use self_encryption;

/// File provides helper functions to perform Operations on Files
pub struct FileHelper {
    routing: ::std::sync::Arc<::std::sync::Mutex<routing::routing_client::RoutingClient<client::callback_interface::CallbackInterface>>>,
    callback_interface: ::std::sync::Arc<::std::sync::Mutex<client::callback_interface::CallbackInterface>>,
    response_notifier: ::ResponseNotifier
}

impl FileHelper {
    /// Create a new FileHelper instance
    pub fn new(routing: ::std::sync::Arc<::std::sync::Mutex<routing::routing_client::RoutingClient<client::callback_interface::CallbackInterface>>>,
        callback_interface: ::std::sync::Arc<::std::sync::Mutex<client::callback_interface::CallbackInterface>>,
        response_notifier: ::ResponseNotifier) -> FileHelper {
        FileHelper {
            routing: routing,
            callback_interface: callback_interface,
            response_notifier: response_notifier
        }
    }

    pub fn create(&mut self, name: String, user_metatdata: Vec<u8>,
            directory: nfs::types::DirectoryListing) -> Result<nfs::io::Writer, &str> {
        if self.file_exists(directory.clone(), name.clone()) {
            return Err("File already exists");
        }
        let file = nfs::types::File::new(nfs::types::Metadata::new(name, user_metatdata), self_encryption::datamap::DataMap::None);
        Ok(nfs::io::Writer::new(directory, file, self.routing.clone(), self.callback_interface.clone(), self.response_notifier.clone()))
    }

    pub fn file_exists(&self, directory: nfs::types::DirectoryListing, file_name: String) -> bool {
        directory.get_files().iter().find(|file| {
                file.get_name() == file_name
            }).is_some()
    }

}
