// Copyright 2016 MaidSafe.net limited.
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

use core::Client;
use core::SelfEncryptionStorage;
use core::futures::FutureExt;
use futures::Future;
use nfs::{Dir, NfsFuture};
use nfs::file::File;
use nfs::helper::dir_helper;
use nfs::metadata::DirMetadata;
use self_encryption::SequentialEncryptor;

/// Mode of the writer
pub enum Mode {
    /// Will create new data
    Overwrite,
    /// Will modify the existing data
    Modify,
}

/// Writer is used to write contents to a File and especially in chunks if the file happens to be
/// too large
pub struct Writer {
    client: Client,
    file: File,
    parent_dir: Dir,
    parent_dir_metadata: DirMetadata,
    self_encryptor: SequentialEncryptor<SelfEncryptionStorage>,
}

impl Writer {
    /// Create new instance of Writer
    pub fn new(client: Client,
               storage: SelfEncryptionStorage,
               mode: Mode,
               parent_dir: Dir,
               parent_dir_metadata: DirMetadata,
               file: File)
               -> Box<NfsFuture<Writer>> {
        let data_map = match mode {
            Mode::Modify => Some(file.datamap().clone()),
            Mode::Overwrite => None,
        };

        let client = client.clone();
        let future = SequentialEncryptor::new(storage, data_map)
            .map(move |encryptor| {
                Writer {
                    client: client,
                    file: file,
                    parent_dir: parent_dir,
                    parent_dir_metadata: parent_dir_metadata,
                    self_encryptor: encryptor,
                }
            })
            .map_err(From::from);

        Box::new(future)
    }

    /// Data of a file/blob can be written in smaller chunks
    pub fn write(&self, data: &[u8]) -> Box<NfsFuture<()>> {
        trace!("Writer writing file data of size {} into self-encryptor.",
               data.len());
        Box::new(self.self_encryptor.write(data).map_err(From::from))
    }

    /// close is invoked only after all the data is completely written
    /// The file/blob is saved only when the close is invoked.
    /// Returns the update Directory which owns the file and also the updated
    /// Directory of the file's parent
    /// Returns (files's updated parent_dir)
    pub fn close(self) -> Box<NfsFuture<Dir>> {
        trace!("Writer induced self-encryptor close.");

        let mut file = self.file;
        let mut dir = self.parent_dir;
        let metadata = self.parent_dir_metadata;
        let size = self.self_encryptor.len();
        let client = self.client;

        self.self_encryptor
            .close()
            .map_err(From::from)
            .and_then(move |(data_map, _)| {
                file.set_datamap(data_map);
                file.metadata_mut().set_modified_time(::time::now_utc());
                file.metadata_mut().set_size(size);
                // file.metadata_mut().increment_version();

                dir.upsert_file(file.clone());

                dir_helper::update(client.clone(), &metadata.id(), &dir).map(move |_| dir)
            })
            .into_box()
    }
}
