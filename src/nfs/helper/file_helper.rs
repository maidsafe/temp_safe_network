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

/// Provides helper functions to perform Operations on Files

use core::SelfEncryptionStorage;
use core::Client;
use nfs::{File, Dir, NfsFuture};
use nfs::errors::NfsError;
use nfs::helper::dir_helper;
use nfs::helper::reader::Reader;
use nfs::helper::writer::{Mode, Writer};
use nfs::metadata::{DirMetadata, FileMetadata};
use routing::DataIdentifier;
use rust_sodium::crypto::secretbox;
use self_encryption::DataMap;

/// Helper function to create a file in a directory listing
/// A writer object is returned, through which the data for the file
/// can be written to the network
/// The file is actually saved in the directory listing only after
/// `writer.close()` is invoked
pub fn create(client: Client,
                   name: String,
                   user_metatdata: Vec<u8>,
                   parent_dir: Dir,
                   parent_metadata: DirMetadata)
                   -> Box<NfsFuture<Writer>> {
    trace!("Creating file with name: {}", name);

    if parent_dir.find_file(&name).is_some() {
        return err!(NfsError::FileAlreadyExistsWithSameName);
    }

    let file = File::Unversioned(FileMetadata::new(name, user_metatdata, DataMap::None));

    Writer::new(client.clone(),
                SelfEncryptionStorage::new(client),
                Mode::Overwrite,
                parent_dir,
                parent_metadata,
                file)
}

/// Delete a file from the Directory
/// Returns Option<parent_directory's parent>
pub fn delete(client: Client,
                   file_name: String,
                   parent_id: &(DataIdentifier, Option<secretbox::Key>),
                   parent_dir: &mut Dir)
                   -> Box<NfsFuture<()>> {
    trace!("Deleting file with name {}.", file_name);
    let _ = fry!(parent_dir.remove_file(&file_name));
    dir_helper::update(client, parent_id, parent_dir)
}

/// Updates the file metadata.
pub fn update_metadata(client: Client,
                       prev_name: &str,
                       file: File,
                       parent_id: &(DataIdentifier, Option<secretbox::Key>),
                       parent_dir: &mut Dir)
                       -> Box<NfsFuture<()>> {
    trace!("Updating metadata for file.");

    {
        let _ = fry!(parent_dir.find_file(prev_name).ok_or(NfsError::FileNotFound));

        if prev_name != file.name() &&
           parent_dir.find_file(file.name()).is_some() {
            return err!(NfsError::FileAlreadyExistsWithSameName);
        }
    }
    parent_dir.update_file(prev_name, file);
    dir_helper::update(client.clone(), parent_id, parent_dir)
}

/// Helper function to Update content of a file in a directory listing
/// A writer object is returned, through which the data for the file
/// can be written to the network
/// The file is actually saved in the directory listing only after
/// `writer.close()` is invoked
pub fn update_content(client: Client,
                      file: File,
                      mode: Mode,
                      parent_metadata: DirMetadata,
                      parent_dir: Dir)
                      -> Box<NfsFuture<Writer>> {
    trace!("Updating content in file with name {}", file.name());

    {
        let existing_file = fry!(parent_dir.find_file(file.name())
                                                 .ok_or(NfsError::FileNotFound));

        if *existing_file != file {
            return err!(NfsError::FileDoesNotMatch);
        }
    }

    Writer::new(client.clone(),
                SelfEncryptionStorage::new(client),
                mode,
                parent_dir,
                parent_metadata,
                file)
}


/// Returns a reader for reading the file contents
pub fn read(client: Client, file: &File) -> Result<Reader, NfsError> {
    trace!("Reading file with name: {}", file.name());
    Reader::new(client.clone(), SelfEncryptionStorage::new(client), file)
}

#[cfg(test)]
mod tests {
    // use core::utility::test_utils;
    // use nfs::AccessLevel;
    // use nfs::helper::{dir_helper, file_helper};
    // use nfs::helper::writer::Mode;

    #[test]
    fn file_crud() {
        // test_utils::register_and_run(|client| {
        //     dir_helper::create("DirName".to_string(),
        //                        Vec::new(),
        //                        true,
        //                        AccessLevel::Private,
        //                        None)
        //         .and_then(move |(mut directory, _)| {
        //             let file_name = "hello.txt".to_string();

        //             const ORIG_SIZE: usize = 100;
        //             {
        //                 // create
        //                 let mut writer = unwrap!(file_helper::create(client.clone(),
        //                                                      file_name.clone(),
        //                                                      Vec::new(),
        //                                                      directory));
        //                 unwrap!(writer.write(&[0u8; ORIG_SIZE]), "");
        //                 let (updated_directory, _) = unwrap!(writer.close());
        //                 directory = updated_directory;
        //                 assert!(directory.find_file(&file_name).is_some());
        //             }
        //             {
        //                 // read
        //                 let file = unwrap!(directory.find_file(&file_name), "File not found");
        //                 let mut reader = unwrap!(file_helper::read(file), "");
        //                 let size = reader.size();
        //                 assert_eq!(unwrap!(reader.read(0, size)), vec![0u8; 100]);
        //             }

        //             const NEW_SIZE: usize = 50;
        //             {
        //                 // update - full rewrite
        //                 let file = unwrap!(directory.find_file(&file_name).cloned(), "File not found");
        //                 {
        //                     let mut writer = unwrap!(file_helper::update_content(file, Mode::Overwrite, directory));
        //                     unwrap!(writer.write(&[1u8; NEW_SIZE]));
        //                     let (updated_directory, _) = unwrap!(writer.close());
        //                     directory = updated_directory;
        //                 }
        //                 let file = unwrap!(directory.find_file(&file_name), "File not found");
        //                 let mut reader = unwrap!(file_helper::read(file), "");
        //                 let size = reader.size();
        //                 assert_eq!(unwrap!(reader.read(0, size)), vec![1u8; 50]);
        //             }

        //             const APPEND_SIZE: usize = 10;
        //             {
        //                 // update - should append (after S.E behaviour changed)
        //                 let file = unwrap!(directory.find_file(&file_name).cloned(), "File not found");
        //                 {
        //                     let mut writer = unwrap!(file_helper::update_content(file, Mode::Modify, directory));
        //                     unwrap!(writer.write(&[2u8; APPEND_SIZE]));
        //                     let (updated_directory, _) = unwrap!(writer.close());
        //                     directory = updated_directory;
        //                 }
        //                 let file = unwrap!(directory.find_file(&file_name), "File not found");
        //                 let mut reader = unwrap!(file_helper::read(file), "");
        //                 let size = reader.size();
        //                 let data = unwrap!(reader.read(0, size));

        //                 assert_eq!(size, (NEW_SIZE + APPEND_SIZE) as u64);
        //                 assert_eq!(data[0..NEW_SIZE].to_owned(), vec![1u8; NEW_SIZE]);
        //                 assert_eq!(&data[NEW_SIZE..], [2u8; APPEND_SIZE]);
        //             }
        //             {
        //                 // versions
        //                 let file = unwrap!(directory.find_file(&file_name).cloned(), "File not found");
        //                 let versions = unwrap!(file_helper::get_versions(&file, &directory));
        //                 assert_eq!(versions.len(), 3);
        //             }
        //             {
        //                 // Update Metadata
        //                 let mut file = unwrap!(directory.find_file(&file_name).cloned(),
        //                                        "File not found");
        //                 file.get_mut_metadata().set_user_metadata(vec![12u8; 10]);
        //                 let _ = unwrap!(file_helper::update_metadata(file, &mut directory));
        //                 let file = unwrap!(directory.find_file(&file_name).cloned(), "File not found");
        //                 assert_eq!(*file.metadata().user_metadata(), [12u8; 10][..]);
        //             }
        //             {
        //                 // Delete
        //                 let _ = unwrap!(file_helper::delete(file_name.clone(), &mut directory));
        //                 assert!(directory.find_file(&file_name).is_none());
        //             }
        //         })
        // });
    }
}
