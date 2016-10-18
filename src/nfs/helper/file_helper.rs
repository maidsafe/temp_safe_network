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
use nfs::metadata::FileMetadata;
use routing::DataIdentifier;
use rust_sodium::crypto::secretbox;
use self_encryption::DataMap;

/// Helper function to create a file in a directory listing
/// A writer object is returned, through which the data for the file
/// can be written to the network
/// The file is actually saved in the directory listing only after
/// `writer.close()` is invoked
pub fn create<S>(client: Client,
                 name: S,
                 user_metadata: Vec<u8>,
                 parent_id: (DataIdentifier, Option<secretbox::Key>),
                 parent_dir: Dir)
                 -> Box<NfsFuture<Writer>>
    where S: Into<String> {
    let name = name.into();
    trace!("Creating file with name: {}", name);

    if parent_dir.find_file(&name).is_some() {
        return err!(NfsError::FileAlreadyExistsWithSameName);
    }

    let file = File::Unversioned(FileMetadata::new(name, user_metadata, DataMap::None));

    Writer::new(client.clone(),
                SelfEncryptionStorage::new(client),
                Mode::Overwrite,
                parent_id,
                parent_dir,
                file)
}

/// Delete a file from the Directory
/// Returns Option<parent_directory's parent>
pub fn delete(client: Client,
              file_name: &str,
              parent_id: &(DataIdentifier, Option<secretbox::Key>),
              parent_dir: &mut Dir)
              -> Box<NfsFuture<()>> {
    trace!("Deleting file with name {}.", file_name);
    let _ = fry!(parent_dir.remove_file(file_name));
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
                      parent_id: (DataIdentifier, Option<secretbox::Key>),
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
                parent_id,
                parent_dir,
                file)
}


/// Returns a reader for reading the file contents
pub fn read(client: Client, file: &File) -> Result<Reader, NfsError> {
    trace!("Reading file with name: {}", file.name());
    Reader::new(client.clone(), SelfEncryptionStorage::new(client), file)
}

#[cfg(test)]
mod tests {
    use core::Client;
    use core::futures::FutureExt;
    use core::utility::test_utils;
    use futures::Future;
    use nfs::{Dir, DirId, NfsFuture};
    use nfs::helper::{dir_helper, file_helper};
    use nfs::helper::writer::Mode;

    const APPEND_SIZE: usize = 10;
    const ORIG_SIZE: usize = 100;
    const NEW_SIZE: usize = 50;

    fn create_test_file(client: Client) -> Box<NfsFuture<(Dir, DirId)>> {
        let c2 = client.clone();
        let dir = Dir::new();

        dir_helper::create(client, &dir, None)
            .and_then(move |dir_id| {
                file_helper::create(c2, "hello.txt", Vec::new(), (dir_id, None), dir)
                    .map(move |writer| (writer, (dir_id, None)))
            })
            .and_then(move |(writer, dir_id)| {
                writer.write(&[0u8; ORIG_SIZE])
                    .and_then(move |_| writer.close())
                    .map(move |updated_dir| (updated_dir, dir_id))
            })
            .into_box()
    }

    #[test]
    fn file_read() {
        test_utils::register_and_run(|client| {
            let c2 = client.clone();

            create_test_file(client.clone())
                .and_then(move |(dir, _)| {
                    let file = unwrap!(dir.find_file("hello.txt"), "File not found");
                    let reader = unwrap!(file_helper::read(c2, file));
                    let size = reader.size();
                    println!("reading {} bytes", size);
                    reader.read(0, size)
                })
                .map(move |data| {
                    assert_eq!(data, vec![0u8; 100]);
                })
        });
    }

    #[test]
    fn file_update() {
        test_utils::register_and_run(|client| {
            let c2 = client.clone();
            let c3 = client.clone();

            create_test_file(client.clone())
                .and_then(move |(dir, metadata)| {
                    // Update - full rewrite
                    let file = unwrap!(dir.find_file("hello.txt").cloned(), "File not found");
                    file_helper::update_content(c2, file, Mode::Overwrite, metadata, dir)
                })
                .and_then(move |writer| {
                    writer.write(&[1u8; NEW_SIZE])
                        .and_then(move |_| writer.close())
                })
                .and_then(move |dir| {
                    let file = unwrap!(dir.find_file("hello.txt"), "File not found");

                    let reader = unwrap!(file_helper::read(c3, file));
                    let size = reader.size();
                    println!("reading {} bytes", size);
                    reader.read(0, size)
                })
                .map(move |data| {
                    assert_eq!(data, vec![1u8; 50]);
                })
        });
    }

    #[test]
    fn file_update_append() {
        test_utils::register_and_run(|client| {
            let c2 = client.clone();
            let c3 = client.clone();

            create_test_file(client.clone())
                .and_then(move |(dir, metadata)| {
                    // Update - should append (after S.E behaviour changed)
                    let file = unwrap!(dir.find_file("hello.txt").cloned(), "File not found");
                    file_helper::update_content(c2, file, Mode::Modify, metadata, dir)
                })
                .and_then(move |writer| {
                    writer.write(&[2u8; APPEND_SIZE])
                        .and_then(move |_| writer.close())
                })
                .and_then(move |dir| {
                    let file = unwrap!(dir.find_file("hello.txt"), "File not found");

                    let reader = unwrap!(file_helper::read(c3, file));
                    let size = reader.size();
                    reader.read(0, size)
                        .map(move |data| {
                            assert_eq!(size, (ORIG_SIZE + APPEND_SIZE) as u64);
                            assert_eq!(data[0..ORIG_SIZE].to_owned(), vec![0u8; ORIG_SIZE]);
                            assert_eq!(&data[ORIG_SIZE..], [2u8; APPEND_SIZE]);
                        })
                })
        });
    }

    #[test]
    fn file_update_metadata() {
        test_utils::register_and_run(|client| {
            let c2 = client.clone();

            create_test_file(client.clone())
                .and_then(move |(mut dir, dir_metadata)| {
                    // Update Metadata
                    let mut file = unwrap!(dir.find_file("hello.txt").cloned(),
                                           "File not found");
                    file.metadata_mut().set_name("hello.jpg");
                    file.metadata_mut().set_user_metadata(vec![12u8; 10]);
                    file_helper::update_metadata(c2, "hello.txt", file, &dir_metadata, &mut dir)
                        .map(|_| dir)
                })
                .map(|dir| {
                    let file = unwrap!(dir.find_file("hello.jpg").cloned(), "File not found");
                    assert_eq!(*file.metadata().user_metadata(), [12u8; 10][..]);
                })
        });
    }

    #[test]
    fn file_delete() {
        test_utils::register_and_run(|client| {
            let c2 = client.clone();

            create_test_file(client.clone())
                .and_then(move |(mut dir, metadata)| {
                    file_helper::delete(c2, "hello.txt", &metadata, &mut dir)
                        .map(move |_| {
                            assert!(dir.find_file("hello.txt").is_none());
                        })
                })
        })
    }
}
