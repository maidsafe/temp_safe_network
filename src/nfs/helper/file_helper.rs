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
use client::Client;
use WaitCondition;
use self_encryption;

/// File provides helper functions to perform Operations on Files
pub struct FileHelper<'a> {
    client: &'a mut Client
}

impl <'a> FileHelper<'a> {
    /// Create a new FileHelper instance
    pub fn new(client: &'a mut Client) -> FileHelper<'a> {
        FileHelper {
            client: client
        }
    }

    pub fn create(&mut self, name: String, user_metatdata: Vec<u8>,
            directory: nfs::types::DirectoryListing) -> Option<nfs::io::Writer> {
        if self.file_exists(directory.clone(), name.clone()) {
            return None;
        }
        let file = nfs::types::File::new(nfs::types::Metadata::new(name, user_metatdata), self_encryption::datamap::DataMap::None);
        Some(nfs::io::Writer::new(directory, file, self.client.get_routing_client(), self.client.get_network_response_callback(),
            self.client.get_response_notifier()))
    }

    pub fn file_exists(&self, directory: nfs::types::DirectoryListing, file_name: String) -> bool {
        directory.get_files().iter().find(|file| {
                file.get_name() == file_name
            }).is_some()
    }

}
