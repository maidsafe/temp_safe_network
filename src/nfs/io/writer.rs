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
#[allow(dead_code)]
use nfs;
use std::sync;
use super::network_storage::NetworkStorage;
use self_encryption;
use client;

#[allow(dead_code)]
pub struct Writer {
    file: nfs::file::File,
    directory: nfs::directory_listing::DirectoryListing,
    self_encryptor: self_encryption::SelfEncryptor<NetworkStorage>,
    client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>
}

#[allow(dead_code)]
impl Writer {

    pub fn new(directory: nfs::directory_listing::DirectoryListing, file: nfs::file::File,
        client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>) -> Writer {
        let storage = sync::Arc::new(NetworkStorage::new(client.clone()));
        Writer {
            file: file.clone(),
            directory: directory,
            self_encryptor: self_encryption::SelfEncryptor::new(storage.clone(), file.get_datamap()),
            client: client
        }
    }

    pub fn write(&mut self, data: &[u8], position: u64) {
        // let se = self.self_encryptor.clone().lock().unwrap();
        self.self_encryptor.write(data, position);
    }

    pub fn close(mut self) -> Result<(), String> {
        let mut directory = self.directory.clone();
        let ref mut file = self.file;
        file.set_datamap(self.self_encryptor.close());
        match directory.get_files().iter().find(|file| file.get_name() == file.get_name()) {
            Some(old_file) => {
                let pos = directory.get_files().binary_search_by(|p| p.get_name().cmp(&old_file.get_name())).unwrap();
                let mut files = directory.get_files();
                files.remove(pos);
                files.insert(pos, file.clone());
                directory.set_files(files);
            },
            None => {
                directory.add_file(file.clone());
            }
        }
        let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
        if directory_helper.update(directory).is_err() {
            return Err("Failed to save".to_string());
        }
        Ok(())
    }

}
