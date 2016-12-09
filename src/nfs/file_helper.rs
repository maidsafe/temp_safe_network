// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

use core::{Client, CoreFuture, Dir, FutureExt, SelfEncryptionStorage};
use futures::Future;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use nfs::{File, Mode, NfsError, Reader, Writer};
use routing::EntryActions;
use self_encryption::DataMap;

/// Gets a file from the directory
pub fn fetch<S: AsRef<str>>(client: Client, parent: Dir, name: S) -> Box<CoreFuture<(u64, File)>> {
    let key = fry!(parent.enc_entry_key(name.as_ref().as_bytes()));

    client.get_mdata_value(parent.name, parent.type_tag, key)
        .and_then(move |val| {
            let plaintext = parent.decrypt(&val.content)?;
            let file = deserialise::<File>(&plaintext)?;
            Ok((val.entry_version, file))
        })
        .into_box()
}

/// Returns a reader for reading the file contents
pub fn read(client: Client, file: &File) -> Result<Reader, NfsError> {
    trace!("Reading file {:?}", file);
    Reader::new(client.clone(), SelfEncryptionStorage::new(client), file)
}

/// Delete a file from the Directory
pub fn delete<S: AsRef<str>>(client: Client,
                             parent: Dir,
                             name: S,
                             version: u64)
                             -> Box<CoreFuture<()>> {
    let name = name.as_ref();
    trace!("Deleting file with name {}.", name);

    let key = fry!(parent.enc_entry_key(name.as_bytes()));

    client.mutate_mdata_entries(parent.name,
                                parent.type_tag,
                                EntryActions::new()
                                    .del(key, version)
                                    .into())
}

/// Updates the file.
pub fn update<S: AsRef<str>>(client: Client,
                             parent: Dir,
                             name: S,
                             file: File,
                             version: u64)
                             -> Box<CoreFuture<()>> {
    let name = name.as_ref();
    trace!("Updating file with name '{}'", name);

    let key = fry!(parent.enc_entry_key(name.as_bytes()));
    let plaintext = fry!(serialise(&file));
    let ciphertext = fry!(parent.enc_entry_value(&plaintext));

    client.mutate_mdata_entries(parent.name,
                                parent.type_tag,
                                EntryActions::new()
                                    .update(key, ciphertext, version)
                                    .into())
}

/// Helper function to Update content of a file in a directory. A writer
/// object is returned, through which the data for the file can be written to
/// the network. The file is actually saved in the directory listing only after
/// `writer.close()` is invoked
pub fn update_content<S: Into<String>>(client: Client,
                                       parent: Dir,
                                       name: S,
                                       file: File,
                                       version: u64,
                                       mode: Mode)
                                       -> Box<CoreFuture<Writer>> {
    let name = name.into();
    trace!("Updating content in file with name {}", name);

    Writer::new(client.clone(),
                SelfEncryptionStorage::new(client),
                mode,
                parent,
                file,
                name,
                Some(version))
}

/// Helper function to create a file in a given directory.
/// A writer object is returned, through which the data for the
/// file can be written to the network.
/// The file is actually saved in the directory listing only after
/// `writer.close()` is invoked
pub fn create<S: Into<String>>(client: Client,
                               parent: Dir,
                               name: S,
                               user_metadata: Vec<u8>)
                               -> Box<CoreFuture<Writer>> {
    let name = name.into();
    trace!("Creating file with name {}.", name);

    Writer::new(client.clone(),
                SelfEncryptionStorage::new(client),
                Mode::Overwrite,
                parent,
                File::new(user_metadata, DataMap::None),
                name,
                None)
}

#[cfg(test)]
mod tests {
    use core::{Client, CoreError, Dir, FutureExt};
    use core::utility::test_utils::random_client;
    use futures::Future;
    use nfs::{File, Mode, NfsFuture, file_helper};

    const APPEND_SIZE: usize = 10;
    const ORIG_SIZE: usize = 100;
    const NEW_SIZE: usize = 50;

    fn create_test_file(client: Client) -> Box<NfsFuture<(Dir, File)>> {
        let user_root = unwrap!(client.user_root_dir());

        file_helper::create(client.clone(), user_root.clone(), "hello.txt", Vec::new())
            .then(move |res| {
                let writer = unwrap!(res);

                writer.write(&[0u8; ORIG_SIZE])
                    .and_then(move |_| writer.close())
            })
            .map(move |file| (user_root, file))
            .into_box()
    }

    #[test]
    fn file_read() {
        random_client(|client| {
            let c2 = client.clone();

            create_test_file(client.clone())
                .then(move |res| {
                    let (_dir, file) = unwrap!(res);
                    let reader = unwrap!(file_helper::read(c2, &file));
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
    fn file_update_rewrite() {
        random_client(|client| {
            let c2 = client.clone();
            let c3 = client.clone();

            create_test_file(client.clone())
                .then(move |res| {
                    // Updating file - full rewrite
                    let (dir, file) = unwrap!(res);
                    file_helper::update_content(c2, dir, "hello.txt", file, 1, Mode::Overwrite)
                })
                .then(move |res| {
                    let writer = unwrap!(res);
                    writer.write(&[1u8; NEW_SIZE])
                        .and_then(move |_| writer.close())
                })
                .then(move |res| {
                    let file = unwrap!(res);

                    let reader = unwrap!(file_helper::read(c3, &file));
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
        random_client(|client| {
            let c2 = client.clone();
            let c3 = client.clone();

            create_test_file(client.clone())
                .then(move |res| {
                    let (dir, file) = unwrap!(res);
                    // Update - should append (after S.E behaviour changed)
                    file_helper::update_content(c2, dir, "hello.txt", file, 1, Mode::Modify)
                })
                .then(move |res| {
                    let writer = unwrap!(res);
                    writer.write(&[2u8; APPEND_SIZE])
                        .and_then(move |_| writer.close())
                })
                .then(move |res| {
                    let file = unwrap!(res);

                    let reader = unwrap!(file_helper::read(c3, &file));
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
        random_client(|client| {
            let c2 = client.clone();
            let c3 = client.clone();

            create_test_file(client.clone())
                .then(move |res| {
                    let (dir, mut file) = unwrap!(res);
                    file.set_user_metadata(vec![12u8; 10]);
                    file_helper::update(c2, dir, "hello.txt", file, 1)
                })
                .then(move |res| {
                    assert!(res.is_ok());
                    file_helper::fetch(c3.clone(), unwrap!(c3.user_root_dir()), "hello.txt")
                })
                .map(move |(_version, file)| {
                    assert_eq!(*file.user_metadata(), [12u8; 10][..]);
                })
        });
    }

    #[test]
    fn file_delete() {
        random_client(|client| {
            let c2 = client.clone();
            let c3 = client.clone();

            create_test_file(client.clone())
                .then(move |res| {
                    let (dir, _file) = unwrap!(res);
                    file_helper::delete(c2, dir, "hello.txt", 1)
                })
                .then(move |res| {
                    assert!(res.is_ok());
                    file_helper::fetch(c3.clone(), unwrap!(c3.user_root_dir()), "hello.txt")
                })
                .then(move |res| -> Result<_, CoreError> {
                    match res {
                        Ok(_) => {
                            // We expect an error in this case
                            panic!("Fetched non-existing file succesfully")
                        }
                        Err(_) => Ok(()),
                    }
                })
        });
    }
}
