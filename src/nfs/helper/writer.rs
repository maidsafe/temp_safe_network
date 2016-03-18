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

use std::sync::{Arc, Mutex};

use nfs::errors::NfsError;
use nfs::directory_listing::DirectoryListing;
use nfs::helper::directory_helper::DirectoryHelper;
use nfs::file::File;
use core::client::Client;
use core::SelfEncryptionStorage;
use self_encryption::{DataMap, SelfEncryptor};

/// Mode of the writter
pub enum Mode {
    /// Will create new data
    Overwrite,
    /// Will modify the existing data
    Modify,
}

/// Writer is used to write contents to a File and especially in chunks if the file happens to be
/// too large
pub struct Writer {
    client: Arc<Mutex<Client>>,
    file: File,
    parent_directory: DirectoryListing,
    self_encryptor: SelfEncryptor<SelfEncryptionStorage>,
}

impl Writer {
    /// Create new instance of Writer
    pub fn new(client: Arc<Mutex<Client>>, mode: Mode, parent_directory: DirectoryListing, file: File) -> Writer {
        let datamap = match mode {
            Mode::Modify => file.get_datamap().clone(),
            Mode::Overwrite => DataMap::None,
        };

        Writer {
            client: client.clone(),
            file: file,
            parent_directory: parent_directory,
            self_encryptor: SelfEncryptor::new(SelfEncryptionStorage::new(client.clone()), datamap),
        }
    }

    /// Data of a file/blob can be written in smaller chunks
    pub fn write(&mut self, data: &[u8], position: u64) {
        debug!("Writing file data at position {:?} ...", position);
        self.self_encryptor.write(data, position);
    }

    /// close is invoked only after all the data is completely written
    /// The file/blob is saved only when the close is invoked.
    /// Returns the update DirectoryListing which owns the file and also the updated
    /// DirectoryListing of the file's parent
    /// Returns (files's parent_directory, Option<file's parent_directory's parent>)
    pub fn close(self) -> Result<(DirectoryListing, Option<DirectoryListing>), NfsError> {
        let mut file = self.file;
        let mut directory = self.parent_directory;
        let size = self.self_encryptor.len();

        file.set_datamap(self.self_encryptor.close());

        file.get_mut_metadata().set_modified_time(::time::now_utc());
        file.get_mut_metadata().set_size(size);
        file.get_mut_metadata().increment_version();

        directory.upsert_file(file.clone());

        let directory_helper = DirectoryHelper::new(self.client.clone());
        if let Some(updated_grand_parent) = try!(directory_helper.update(&directory)) {
            Ok((directory, Some(updated_grand_parent)))
        } else {
            Ok((directory, None))
        }
    }
}
