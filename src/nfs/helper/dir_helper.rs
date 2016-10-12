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
use core::structured_data_operations::unversioned;
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

    let client2 = client.clone();

    save(client.clone(), dir, &data_id, encryption_key)
        .and_then(move |structured_data| {
            let data_id = structured_data.identifier();
            client2.put_recover(Data::Structured(structured_data), None)
                .map_err(NfsError::from)
                .map(move |_| data_id)
        })
        .into_box()
}

/// Adds a sub directory to a parent.
/// Parent directory is updated and the sub directory metadata returned as a result.
pub fn add(client: Client,
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
    let (parent_locator, parent_key) = parent_id.clone();
    let client2 = client.clone();

    save(client.clone(),
         &parent,
         &parent_locator,
         parent_key.as_ref())
        .and_then(move |structured_data| {
            let client = client2.clone();
            client2.put_recover(Data::Structured(structured_data), None)
                .map_err(NfsError::from)
                .and_then(move |_| {
                    update(client, &(parent_locator, parent_key), &parent)
                        .map(move |_| metadata)
                        .into_box()
                })
        })
        .into_box()
}

/// Deletes a sub directory
pub fn delete(client: Client, parent: &mut Dir, dir_to_delete: &str) -> Box<NfsFuture<()>> {
    trace!("Deleting directory with name: {}", dir_to_delete);

    // TODO (Spandan) - Fetch and issue a DELETE on the removed directory.
    let _dir_meta = fry!(parent.remove_sub_dir(dir_to_delete));
    update(client.clone(),
           &_dir_meta.id(),
           parent)
}

/// Updates an existing Directory in the network.
pub fn update(client: Client,
              dir_id: &(DataIdentifier, Option<secretbox::Key>),
              directory: &Dir)
              -> Box<NfsFuture<()>> {
    trace!("Updating directory given the directory listing.");

    let client2 = client.clone();
    let dir_clone = directory.clone();
    let (locator, secret_key) = dir_id.clone();

    get_structured_data(client.clone(), &locator)
        .and_then(move |structured_data| {
            let client2 = client2.clone();

            let signing_key = fry!(client.secret_signing_key());
            let owner_key = fry!(client.public_signing_key());

            let serialised_data = fry!(serialise(&dir_clone));

            let updated_future = {
                trace!("Updating directory listing with a new one (will convert DL to an \
                        unversioned StructuredData).");

                if let DataIdentifier::Structured(id, type_tag) = locator {
                    unversioned::create(client2.clone(),
                                        type_tag,
                                        id,
                                        structured_data.get_version() + 1,
                                        serialised_data,
                                        vec![owner_key.clone()],
                                        Vec::new(),
                                        &signing_key,
                                        secret_key.as_ref())
                        .map_err(NfsError::from)
                        .into_box()
                } else {
                    err!(NfsError::ParameterIsNotValid)
                }
            };

            debug!("Posting updated structured data to the network ...");

            updated_future.and_then(move |updated_structured_data| {
                    client2.post(Data::Structured(updated_structured_data), None)
                        .map_err(NfsError::from)
                })
                .into_box()
        })
        .into_box()
}

/// Return the Directory
pub fn get(client: Client, dir_id: &(DataIdentifier, Option<secretbox::Key>)) -> Box<NfsFuture<Dir>> {
    trace!("Getting a directory.");

    let client2 = client.clone();
    let (id, sk) = dir_id.clone();

    get_structured_data(client.clone(), &id)
        .and_then(move |structured_data| {
            unversioned::get_data(client2, &structured_data, sk.as_ref()).map_err(NfsError::from)
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
            let mut client2 = client.clone();
            let key = None;
            let dir = Dir::new();

            create(client.clone(), &dir, key)
                .and_then(move |data_id| {
                    // ::nfs::ROOT_DIRECTORY_NAME.to_string(),
                    client2.set_user_root_dir_id((data_id, None))
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
pub fn configuration_dir(client: Client, dir_name: String) -> Box<NfsFuture<Dir>> {
    trace!("Getting a configuration directory (from withing configuration root dir) with name: \
            {}.",
           dir_name);

    let config_dir_id = client.config_root_dir_id();

    let fut = match config_dir_id {
        Some(dir_id) => get(client.clone(), &dir_id).map(|dir| (dir_id, dir)).into_box(),
        None => {
            debug!("Configuartion Root directory does not exist - creating one.");

            let mut client2 = client.clone();
            let dir = Dir::new();
            let key = None;

            create(client.clone(), &dir, key)
                .and_then(move |created_id| {
                    let config_dir_id = (created_id, None);
                    client2.set_config_root_dir_id(config_dir_id.clone())
                        .map_err(NfsError::from)
                        .map(|_| (config_dir_id, dir))
                })
                .into_box()
        }
    };

    fut.and_then(move |(config_dir_id, config_dir)| {
            match config_dir.sub_dirs()
                .iter()
                .position(|metadata| *metadata.name() == dir_name) {
                Some(index) => {
                    let ref metadata = config_dir.sub_dirs()[index];
                    get(client.clone(), &metadata.id())
                }
                None => {
                    debug!("Given configuration directory does not exist (inside the root \
                            configuration dir) - creating one.");

                    let dir = Dir::new();
                    let mut cdir = config_dir.clone();

                    create(client.clone(), &dir, None)
                        .and_then(move |dir_id| {
                            add(client.clone(),
                                dir_name,
                                &(dir_id, None),
                                Vec::new(),
                                &mut cdir,
                                &config_dir_id)
                                .map(move |_| cdir)
                        })
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
        unversioned::create(client.clone(),
                            type_tag,
                            id,
                            0,
                            encoded,
                            vec![owner_key.clone()],
                            Vec::new(),
                            &signing_key,
                            encryption_key)
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

    client.put_recover(Data::Immutable(immutable_data), None)
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
    use core::utility::test_utils::get_client;
    use core::futures::FutureExt;
    use core::{self, CoreMsg};
    use futures::Future;
    use nfs::Dir;
    use nfs::helper::dir_helper;
    use std::cell::RefCell;
    use std::rc::Rc;
    use tokio_core::channel;
    use tokio_core::reactor::Core;

    #[test]
    fn create_dir() {
        let core = unwrap!(Core::new());
        let (core_tx, core_rx) = unwrap!(channel::channel(&core.handle()));
        let client = unwrap!(get_client(core_tx.clone()));

        unwrap!(core_tx.send(CoreMsg::new(move |cptr| {
            // Create a Directory
            let dir = Dir::new();
            let cptr2 = cptr.clone();
            let cptr3 = cptr.clone();

            let f = dir_helper::create(cptr.clone(), &dir, None)
                .map(|_| panic!("test"))
                .and_then(move |dir_id| {
                    dir_helper::get(cptr2.clone(), &(dir_id, None))
                        .map(move |new_dir| (dir_id, new_dir))
                })
                .and_then(move |(dir_id, mut new_dir)| {
                    assert_eq!(new_dir, dir);

                    // Create a Child directory and update the parent_dir
                    let child_dir = Dir::new();

                    dir_helper::create(cptr3.clone(), &child_dir, None).and_then(move |child_id| {
                        dir_helper::add(cptr3.clone(),
                                        "Child".to_string(),
                                        &(child_id, None),
                                        Vec::new(),
                                        &mut new_dir,
                                        &(dir_id, None))
                    })
                })
                .map(|_| ())
                .map_err(|_| panic!("dir_helper::create returned error"))
                .into_box();

            Some(f)
        })));

        unwrap!(core_tx.send(CoreMsg::build_terminator()));
        core::run(core, Rc::new(RefCell::new(client)), core_rx);

        //let child_metadata = unwrap!(fut.wait());

        // // Assert whether parent is updated
        // let parent = unwrap!(dir_helper::get(client.clone(), &dir_id).wait());
        // assert!(parent.find_sub_dir(child_metadata.name()).is_some());

        // let child_id = child_metadata.id();
        // let grand_child = unwrap!(dir_helper::create(client.clone(), Dir::new(), None)
        //     .and_then(|child_id| {
        //         dir_helper::add(client.clone(),
        //                         "Grand Child".to_string(),
        //                         child_id,
        //                         Vec::new(),
        //                         &mut child_dir,
        //                         child_id)
        //     })
        //     .wait());

        // // We expect result to be an error if we try to create a dir with the same name
        // assert!(dir_helper::create(client.clone(), &Dir::new(), None)
        //     .and_then(|child_id| {
        //         dir_helper::add(client.clone(),
        //                         "Grand Child".to_string(),
        //                         child_id,
        //                         Vec::new(),
        //                         &mut child_dir,
        //                         child_id)
        //     })
        //     .wait()
        //     .is_err());

        // assert_eq!(*grand_parent.metadata().name(),
        //            *directory.metadata().name());
        // assert_eq!(*grand_parent.metadata().modified_time(),
        //            *grand_child_directory.metadata().modified_time());
    }

    // #[test]
    // fn create_public_dir() {
    //     let public_dir = Dir::new();
    //     let public_dir_id;
    //     {
    //         let core = unwrap!(Core::new());
    //         let (core_tx, _) = unwrap!(channel::channel(&core.handle()));
    //         let client = unwrap!(get_client(core_tx.clone()));
    //         let client = Rc::new(RefCell::new(client));

    //         let directory = unwrap!(dir_helper::create(client.clone(), &public_dir, None).wait());
    //         public_dir_id = directory;
    //     }
    //     {
    //         let core = unwrap!(Core::new());
    //         let (core_tx, _) = unwrap!(channel::channel(&core.handle()));
    //         let client = unwrap!(get_client(core_tx.clone()));
    //         let client = Rc::new(RefCell::new(client));

    //         let retrieved_public_directory =
    //             unwrap!(dir_helper::get(client.clone(), &(public_dir_id, None)).wait());

    //         assert_eq!(retrieved_public_directory, public_directory);
    //     }
    // }

    // #[test]
    // fn user_root_configuration() {
    //     let core = unwrap!(Core::new());
    //     let (core_tx, _) = unwrap!(channel::channel(&core.handle()));
    //     let client = unwrap!(get_client(core_tx.clone()));
    //     let client = Rc::new(RefCell::new(client));

    //     let mut root_dir = unwrap!(dir_helper::user_root_dir(client.clone()).wait());
    //     let (created_dir, _) = unwrap!(create_dir(client.clone(),
    //                                               "DirName".to_string(),
    //                                               Vec::new(),
    //                                               true,
    //                                               AccessLevel::Private,
    //                                               Some(&mut root_dir).wait())
    //         .wait());
    //     let root_dir = unwrap!(dir_helper::user_root_dir(client.clone()).wait());
    //     assert!(root_dir.find_sub_dir(created_dir.metadata().name()).is_some());
    // }

    // #[test]
    // fn configuration_directory() {
    //     let core = unwrap!(Core::new());
    //     let (core_tx, _) = unwrap!(channel::channel(&core.handle()));
    //     let client = unwrap!(get_client(core_tx.clone()));
    //     let client = Rc::new(RefCell::new(client));

    //     let config_dir = unwrap!(configuration_directory_listing(client.clone(),
    //                                                              "DNS".to_string()));
    //     assert_eq!(config_dir.metadata().name().clone(), "DNS".to_string());
    //     let id = config_dir.key().id();
    //     let config_dir = unwrap!(configuration_directory_listing(client.clone(),
    //                                                              "DNS".to_string()));
    //     assert_eq!(config_dir.key().id(), id);
    // }

    // #[test]
    // fn delete_directory() {
    //     let core = unwrap!(Core::new());
    //     let (core_tx, _) = unwrap!(channel::channel(&core.handle()));
    //     let client = unwrap!(get_client(core_tx.clone()));
    //     let client = Rc::new(RefCell::new(client));

    //     // Create a Directory
    //     let parent = Dir::new();
    //     let parent_id = unwrap!(dir_helper::create(client.clone(), &parent, None));

    //     assert_eq!(directory,
    //                unwrap!(dir_helper::get(client.clone(), directory.key())));

    //     // Create a Child directory and update the parent_dir
    //     let child = unwrap!(dir_helper::create_child(client.clone(),
    //                                                  "Child".to_string(),
    //                                                  Vec::new(),
    //                                                  true,
    //                                                  AccessLevel::Private,
    //                                                  Some(&mut directory)));
    //     // Assert whether parent is updated
    //     let parent = unwrap!(dir_helper::get(client.clone(), directory.key()));
    //     assert!(parent.find_sub_dir(child_directory.metadata().name()).is_some());

    //     let (grand_child_directory, grand_parent) =
    //         unwrap!(dir_helper::create("Grand Child".to_string(),
    //                                    Vec::new(),
    //                                    true,
    //                                    AccessLevel::Private,
    //                                    Some(&mut child_directory)));

    //     let _ = unwrap!(grand_parent, "Grand Parent Should be updated");

    //     let delete_result = unwrap!(dir_helper::delete(client.clone(),
    //                                                    &mut child_directory,
    //                                                    grand_child_directory.metadata()
    //                                                        .name())
    //         .wait());
    //     let updated_grand_parent = unwrap!(delete_result, "Parent directory should be returned");
    //     assert_eq!(*updated_grand_parent.metadata().id(),
    //                *directory.metadata().id());

    //     let delete_result = unwrap!(dir_helper::delete(client.clone(),
    //                                                    &mut directory,
    //                                                    child_directory.metadata()
    //                                                        .name())
    //         .wait());
    //     assert!(delete_result.is_none());
    // }
}
