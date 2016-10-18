// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences".to_string()).
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
use core::errors::CoreError;
use core::futures::FutureExt;
use core::structured_data::unversioned;
use futures::Future;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use nfs::{Dir, NfsFuture};
use nfs::errors::NfsError;
use nfs::metadata::DirMetadata;
use rand;
use routing::{Data, DataIdentifier, ImmutableData, StructuredData, XorName};
use rust_sodium::crypto::secretbox;

/// Creates a Dir in the network.
/// Returns DataIdentifier for the created dir.
pub fn create(client: Client,
              dir: &Dir,
              encryption_key: Option<&secretbox::Key>)
              -> Box<NfsFuture<DataIdentifier>> {
    trace!("Creating directory");

    let id = rand::random();
    let data_id = DataIdentifier::Structured(id, ::UNVERSIONED_STRUCT_DATA_TYPE_TAG);

    let c2 = client.clone();

    save(client.clone(), dir, &data_id, encryption_key)
        .and_then(move |structured_data| {
            let data_id = structured_data.identifier();

            c2.put_recover(Data::Structured(structured_data), None)
                .map_err(NfsError::from)
                .map(move |_| data_id)
        })
        .into_box()
}

/// Adds a sub directory to a parent.
/// Parent directory is updated and the sub directory metadata returned as a result.
pub fn add_sub_dir(client: Client,
                   name: String,
                   dir_id: &(DataIdentifier, Option<secretbox::Key>),
                   user_metadata: Vec<u8>,
                   parent: &mut Dir,
                   parent_id: &(DataIdentifier, Option<secretbox::Key>))
                   -> Box<NfsFuture<DirMetadata>> {
    trace!("Creating directory with name: {}", name);

    if parent.find_sub_dir(&name)
        .is_some() {
        return err!(NfsError::DirectoryAlreadyExistsWithSameName);
    }

    let (dir_id, secret_key) = dir_id.clone();
    let metadata = DirMetadata::new(dir_id.name().clone(), name, user_metadata, secret_key);
    parent.upsert_sub_dir(metadata.clone());

    let parent = parent.clone();

    update(client.clone(), parent_id, &parent)
        .map(move |_| metadata)
        .into_box()
}

/// Creates and adds a child directory
/// Returns (updated_parent_dir, created_dir, created_dir_metadata)
pub fn create_sub_dir(client: Client,
                      name: String,
                      encrypt_key: Option<secretbox::Key>,
                      user_metadata: Vec<u8>,
                      parent: &Dir,
                      parent_id: &(DataIdentifier, Option<secretbox::Key>))
                      -> Box<NfsFuture<(Dir, Dir, DirMetadata)>> {
    let dir = Dir::new();
    let c2 = client.clone();
    let mut parent = parent.clone();
    let parent_id = parent_id.clone();
    create(client.clone(), &dir, encrypt_key.as_ref())
        .and_then(move |child_id| {
            add_sub_dir(c2,
                        name,
                        &(child_id, encrypt_key),
                        user_metadata,
                        &mut parent,
                        &parent_id)
                .map(move |metadata| (parent, metadata))
        })
        .map(move |(parent, metadata)| (parent, dir, metadata))
        .into_box()
}

/// Deletes a sub directory
pub fn delete(client: Client, parent: &mut Dir, dir_to_delete: &str) -> Box<NfsFuture<()>> {
    trace!("Deleting directory with name: {}", dir_to_delete);

    // TODO (Spandan) - Fetch and issue a DELETE on the removed directory.
    let _dir_meta = fry!(parent.remove_sub_dir(dir_to_delete));
    update(client.clone(), &_dir_meta.id(), parent)
}

/// Updates an existing Directory in the network.
pub fn update(client: Client,
              dir_id: &(DataIdentifier, Option<secretbox::Key>),
              directory: &Dir)
              -> Box<NfsFuture<()>> {
    trace!("Updating directory given the directory listing.");

    let dir_clone = directory.clone();
    let (locator, secret_key) = dir_id.clone();
    let c2 = client.clone();
    let c3 = client.clone();

    let signing_key = fry!(client.secret_signing_key());
    let owner_key = fry!(client.public_signing_key());

    get_structured_data(client.clone(), &locator)
        .and_then(move |structured_data| {
            let serialised_data = fry!(serialise(&dir_clone));

            trace!("Updating directory listing with a new one (will convert DL to an \
                    unversioned StructuredData).");

            if let DataIdentifier::Structured(id, type_tag) = locator {
                unversioned::create(&c2,
                                    type_tag,
                                    id,
                                    structured_data.get_version() + 1,
                                    serialised_data,
                                    vec![owner_key.clone()],
                                    Vec::new(),
                                    signing_key,
                                    secret_key)
                    .map_err(NfsError::from)
                    .into_box()
            } else {
                err!(NfsError::ParameterIsNotValid)
            }
        })
        .and_then(move |updated_structured_data| {
            debug!("Posting updated structured data to the network ...");

            c3.post(Data::Structured(updated_structured_data), None)
                .map_err(NfsError::from)
        })
        .into_box()
}

/// Return the Directory
pub fn get(client: Client,
           dir_id: &(DataIdentifier, Option<secretbox::Key>))
           -> Box<NfsFuture<Dir>> {
    trace!("Getting a directory.");

    let c2 = client.clone();
    let (id, sk) = dir_id.clone();

    get_structured_data(client.clone(), &id)
        .and_then(move |structured_data| {
            unversioned::extract_value(&c2, &structured_data, sk).map_err(NfsError::from)
        })
        .and_then(move |encoded| Ok(try!(deserialise::<Dir>(&encoded))))
        .into_box()
}

/// Returns the Root Directory
pub fn user_root_dir(client: Client) -> Box<NfsFuture<Dir>> {
    trace!("Getting the user root directory listing.");

    let root_directory_id = client.user_root_dir_id();

    let fut = match root_directory_id {
        Some((id, key)) => get(client.clone(), &(id, key)).into_box(),
        None => {
            debug!("Root directory does not exist - creating one.");
            let c2 = client.clone();
            let key = None;
            let dir = Dir::new();

            create(client.clone(), &dir, key)
                .and_then(move |data_id| {
                    // ::nfs::ROOT_DIRECTORY_NAME.to_string(),
                    c2.set_user_root_dir_id((data_id, None))
                        .map_err(NfsError::from)
                        .map(move |_| dir)
                })
                .into_box()
        }
    };

    fut.into_box()
}

/// Returns the Configuration Directory from the configuration root folder
/// Creates the directory or the root or both if it doesn't find one.
pub fn configuration_dir(client: Client, dir_name: String) -> Box<NfsFuture<(Dir, DirMetadata)>> {
    trace!("Getting a configuration directory (from withing configuration root dir) with name: \
            {}.",
           dir_name);

    let config_dir_id = client.config_root_dir_id();

    let fut = match config_dir_id {
        Some(dir_id) => get(client.clone(), &dir_id).map(|dir| (dir_id, dir)).into_box(),
        None => {
            debug!("Configuartion Root directory does not exist - creating one.");

            let c2 = client.clone();
            let dir = Dir::new();
            let key = None;

            create(client.clone(), &dir, key)
                .and_then(move |created_id| {
                    let config_dir_id = (created_id, None);
                    c2.set_config_root_dir_id(config_dir_id.clone())
                        .map_err(NfsError::from)
                        .map(|_| (config_dir_id, dir))
                })
                .into_box()
        }
    };

    fut.and_then(move |(config_dir_id, config_dir)| {
            match config_dir.find_sub_dir(&dir_name) {
                Some(metadata) => {
                    let metadata = metadata.clone();
                    get(client.clone(), &metadata.id())
                        .map(move |cdir| (cdir, metadata))
                        .into_box()
                }
                None => {
                    debug!("Given configuration directory does not exist (inside the root \
                            configuration dir) - creating one.");

                    let cdir = config_dir.clone();

                    create_sub_dir(client.clone(),
                                   dir_name,
                                   None,
                                   Vec::new(),
                                   &cdir,
                                   &config_dir_id)
                        .map(move |(_, cdir, metadata)| (cdir, metadata))
                        .into_box()
                }
            }
        })
        .into_box()
}

/// Creates a StructuredData with the directory data in the Network
fn save(client: Client,
        dir: &Dir,
        locator: &DataIdentifier,
        encryption_key: Option<&secretbox::Key>)
        -> Box<NfsFuture<StructuredData>> {
    trace!("Converting directory to an unversioned StructuredData.");

    let signing_key = fry!(client.secret_signing_key());
    let owner_key = fry!(client.public_signing_key());

    let encoded = fry!(serialise(&dir));

    if let DataIdentifier::Structured(id, type_tag) = *locator {
        unversioned::create(&client,
                            type_tag,
                            id,
                            0,
                            encoded,
                            vec![owner_key.clone()],
                            Vec::new(),
                            signing_key,
                            encryption_key.cloned())
            .map_err(NfsError::from)
            .into_box()
    } else {
        err!(NfsError::ParameterIsNotValid)
    }
}

/// Saves the data as ImmutableData in the network and returns the name
#[allow(unused)]
fn save_as_immutable_data(client: Client, data: Vec<u8>) -> Box<NfsFuture<XorName>> {
    let immutable_data = ImmutableData::new(data);
    let name = *immutable_data.name();
    debug!("Posting PUT request to save immutable data to the network ...");

    client.put(Data::Immutable(immutable_data), None)
        .map(move |_| name)
        .map_err(NfsError::from)
        .into_box()
}

/// Get StructuredData from the Network
fn get_structured_data(client: Client, request: &DataIdentifier) -> Box<NfsFuture<StructuredData>> {
    debug!("Getting structured data from the network ...");

    let response_future = client.get(*request, None);

    let final_fut = response_future.and_then(move |result| {
        match result {
            Data::Structured(structured_data) => Ok(structured_data),
            _ => Err(CoreError::ReceivedUnexpectedData),
        }
    });

    final_fut.map_err(NfsError::from).into_box()
}

/// Get ImmutableData from the Network
#[allow(unused)]
fn get_immutable_data(client: Client, id: XorName) -> Box<NfsFuture<ImmutableData>> {
    let request = DataIdentifier::Immutable(id);
    debug!("Getting immutable data from the network ...");

    let response_future = client.get(request, None);

    let final_fut = response_future.and_then(move |result| {
        match result {
            Data::Immutable(immutable_data) => Ok(immutable_data),
            _ => Err(CoreError::ReceivedUnexpectedData),
        }
    });

    Box::new(final_fut.map_err(NfsError::from))
}

#[cfg(test)]
mod tests {
    use core::utility::test_utils;
    use futures::Future;
    use nfs::{Dir, DirMetadata};
    use nfs::helper::dir_helper;
    use rand;
    use routing::DataIdentifier;
    use std::sync::mpsc;

    #[test]
    fn create_dir() {
        test_utils::register_and_run(|client| {
            // Create a Directory
            let mut dir = Dir::new();
            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();
            let c5 = client.clone();
            let c6 = client.clone();

            let sub_dir = DirMetadata::new(rand::random(), "Fake", Vec::new(), None);
            dir.sub_dirs_mut().push(sub_dir);

            dir_helper::create(client.clone(), &dir, None)
                .and_then(move |dir_id| {
                    dir_helper::get(c2, &(dir_id, None)).map(move |new_dir| (dir_id, new_dir))
                })
                .and_then(move |(dir_id, new_dir)| {
                    assert_eq!(new_dir, dir);

                    // Create a Child directory and update the parent_dir
                    let child_dir = Dir::new();

                    dir_helper::create(c3, &child_dir, None)
                        .map(move |child_id| (dir_id, child_id, child_dir))
                })
                .and_then(move |(dir_id, child_id, mut new_dir)| {
                    dir_helper::add_sub_dir(c4,
                                            "Child".to_string(),
                                            &(child_id, None),
                                            "test".to_owned().into_bytes(),
                                            &mut new_dir,
                                            &(dir_id, None))
                        .map(move |dir_meta| (dir_meta, new_dir))
                })
                .and_then(move |(dir_meta, new_dir)| {
                    assert_eq!(dir_meta.name(), "Child");
                    assert_eq!(dir_meta.user_metadata(), b"test");
                    assert!(new_dir.find_sub_dir("Child").is_some());

                    dir_helper::create_sub_dir(c5,
                                               "Grand Child".to_string(),
                                               None,
                                               Vec::new(),
                                               &new_dir,
                                               &dir_meta.id())
                        .map(move |(parent_dir, _, grand_child_meta)| {
                            (dir_meta, parent_dir, grand_child_meta)
                        })
                })
                .and_then(move |(parent_dir_meta, parent_dir, grand_child_meta)| {
                    assert_eq!(grand_child_meta.name(), "Grand Child");

                    // We expect result to be an error if we try to create a dir with the same name
                    dir_helper::create_sub_dir(c6,
                                               "Grand Child".to_string(),
                                               None,
                                               Vec::new(),
                                               &parent_dir,
                                               &parent_dir_meta.id())
                        .then(|r| {
                            match r {
                                Ok(_) => panic!("Created dir with the same name"),
                                Err(_) => Ok(()),
                            }
                        })
                })
                .map_err(|e| panic!("dir_helper::create returned error: {:?}", e))
        });
    }

    #[test]
    fn create_public_dir() {
        let mut public_dir = Dir::new();
        let sub_dir = DirMetadata::new(rand::random(), "fake_sub_dir", Vec::new(), None);
        public_dir.sub_dirs_mut().push(sub_dir);

        let (tx, rx) = mpsc::channel::<DataIdentifier>();

        let pd = public_dir.clone();
        test_utils::register_and_run(move |client| {
            dir_helper::create(client.clone(), &pd, None).map(move |dir_id| tx.send(dir_id))
        });

        let public_dir_id = rx.recv().unwrap();

        test_utils::register_and_run(move |client| {
            dir_helper::get(client.clone(), &(public_dir_id, None))
                .map(move |retrieved_public_dir| {
                    assert_eq!(retrieved_public_dir, public_dir);
                })
        });
    }

    #[test]
    fn user_root_configuration() {
        test_utils::register_and_run(move |client| {
            let c2 = client.clone();
            let c3 = client.clone();

            dir_helper::user_root_dir(client.clone())
                .and_then(move |mut root_dir| {
                    dir_helper::create_sub_dir(c2.clone(),
                                               "DirName".to_string(),
                                               None,
                                               Vec::new(),
                                               &mut root_dir,
                                               &unwrap!(c2.user_root_dir_id()))
                })
                .and_then(move |(updated_parent, _, metadata)| {
                    dir_helper::user_root_dir(c3).map(move |root_dir| {
                        assert_eq!(updated_parent, root_dir);
                        assert!(root_dir.find_sub_dir(metadata.name()).is_some());
                    })
                })
        });
    }

    #[test]
    fn configuration_directory() {
        test_utils::register_and_run(move |client| {
            let c2 = client.clone();

            dir_helper::configuration_dir(client.clone(), "DNS".to_string())
                .and_then(move |(_, metadata)| {
                    assert_eq!(metadata.name().clone(), "DNS".to_string());

                    let id = metadata.id();

                    dir_helper::configuration_dir(c2, "DNS".to_string()).map(move |(_, metadata)| {
                        assert_eq!(metadata.id(), id);
                    })
                })
        });
    }

    #[test]
    fn delete_directory() {
        test_utils::register_and_run(move |client| {
            // Create a Directory
            let mut parent = Dir::new();
            parent.sub_dirs_mut()
                .push(DirMetadata::new(rand::random(), "fake_sub_dir", Vec::new(), None));

            let c2 = client.clone();
            let c3 = client.clone();
            let c4 = client.clone();
            let c5 = client.clone();

            dir_helper::create(client.clone(), &parent, None)
                .and_then(move |parent_id| {
                    dir_helper::get(c2, &(parent_id, None))
                        .map(move |get_result| (get_result, parent_id))
                })
                .and_then(move |(get_result, parent_id)| {
                    assert_eq!(parent, get_result);

                    // Create a Child directory and update the parent_dir
                    dir_helper::create_sub_dir(c3,
                                               "Child".to_string(),
                                               None,
                                               Vec::new(),
                                               &parent,
                                               &(parent_id, None))
                })
                .and_then(move |(parent_dir, _, metadata)| {
                    // Assert whether parent is updated
                    assert!(parent_dir.find_sub_dir(metadata.name()).is_some());

                    dir_helper::create_sub_dir(c4,
                                               "Grand Child".to_string(),
                                               None,
                                               Vec::new(),
                                               &parent_dir,
                                               &metadata.id())
                })
                .and_then(move |(mut parent_dir, _, metadata)| {
                    dir_helper::delete(c5, &mut parent_dir, metadata.name()).map(move |_| {
                        assert!(parent_dir.find_sub_dir("Grand Child").is_none());
                    })
                })
        });
    }
}
