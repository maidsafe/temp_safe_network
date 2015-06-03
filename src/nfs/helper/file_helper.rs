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
use routing::sendable::Sendable;
use client;
use self_encryption;

/// File provides helper functions to perform Operations on Files
pub struct FileHelper {
    client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>
}

impl FileHelper {
    /// Create a new FileHelper instance
    pub fn new(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>) -> FileHelper {
        FileHelper {
            client: client
        }
    }

    pub fn create(&mut self, name: String, user_metatdata: Vec<u8>,
            directory: nfs::directory_listing::DirectoryListing) -> Result<nfs::io::Writer, &str> {
        if self.file_exists(directory.clone(), name.clone()) {
            return Err("File already exists");
        }
        let file = nfs::file::File::new(nfs::metadata::Metadata::new(name, user_metatdata), self_encryption::datamap::DataMap::None);
        Ok(nfs::io::Writer::new(directory, file, self.client.clone()))
    }

    pub fn update(&mut self, file: nfs::file::File, directory: nfs::directory_listing::DirectoryListing) -> Result<nfs::io::Writer, &str> {
        if !self.file_exists(directory.clone(), file.get_name()) {
            return Err("File not present in the directory");
        }
        Ok(nfs::io::Writer::new(directory, file, self.client.clone()))
    }

    /// Updates the file metadata. Returns the updated DirectoryListing
    pub fn update_metadata(&mut self, file: nfs::file::File, directory: nfs::directory_listing::DirectoryListing, user_metadata: Vec<u8>) -> Result<(), &str> {
        if !self.file_exists(directory.clone(), file.get_name()) {
            return Err("File not present in the directory");
        }
        file.get_metadata().set_user_metadata(user_metadata);
        let pos = directory.get_files().binary_search_by(|p| p.cmp(&file)).unwrap();
        directory.get_files().remove(pos);
        directory.get_files().insert(pos, file);
        let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
        if directory_helper.update(directory.clone()).is_err() {
            return Err("Failed to update");
        }
        Ok(())
    }

    pub fn read(&mut self, file: nfs::file::File) -> nfs::io::Reader {
        nfs::io::Reader::new(file, self.client.clone())
    }

    pub fn file_exists(&self, directory: nfs::directory_listing::DirectoryListing, file_name: String) -> bool {
        directory.get_files().iter().find(|file| {
                file.get_name() == file_name
            }).is_some()
    }

}
