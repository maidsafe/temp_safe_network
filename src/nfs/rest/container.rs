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

use core::client::Client;
use xor_name::XorName;

use nfs::errors::NfsError;
use nfs::directory_listing::DirectoryListing;
use nfs::helper::directory_helper::DirectoryHelper;
use nfs::helper::file_helper::FileHelper;
use nfs::helper::reader::Reader;
use nfs::helper::writer::{Mode, Writer};
use nfs::AccessLevel;
use nfs::rest::{Blob, ContainerInfo};

/// Container Repersents a Directory.
/// Container can have its own metadata, sub-containers and files
pub struct Container {
    client: Arc<Mutex<Client>>,
    directory_listing: DirectoryListing,
}

impl Container {
    /// Authorises the directory access.
    /// This sevrves as the initial access point of the Rest API.
    /// Operations can only be performed on a Container object.
    /// If the ContainerInfo parameter is None, then the user's root directory is returned.
    /// Returns the Container, if authorisation is successful.
    pub fn authorise(client: Arc<Mutex<Client>>, container_info: Option<ContainerInfo>) -> Result<Container, NfsError> {
        let directory_helper = DirectoryHelper::new(client.clone());
        let directory = if let Some(container_info) = container_info {
            debug!("Authorising specific container ...");
            let metadata = container_info.into_directory_metadata();
            try!(directory_helper.get(metadata.get_key()))
        } else {
            debug!("Authorising root container ...");
            try!(directory_helper.get_user_root_directory_listing())
        };
        Ok(Container {
            client: client,
            directory_listing: directory,
        })
    }

    /// This functions is incoked to create a new container
    /// Say there are nested containers,
    ///     Home
    ///       -  Pictures
    /// In the above example, `Home` is top level container and
    /// `Pictures` container is a child of `Home`
    /// When a new Conatiner `Tour` is created within `Pictures`,
    /// the following updates are carried out.
    ///     1. A new container is created
    ///     2. The metadata of the new Container (`Tour`) is added to
    ///        the list of containers of `Pictures`.
    ///        Modified time of `Tour` Container is also updated.
    ///     3. Metadata of `Tour` is updated in `Home`.
    /// Thus when a Container is created, the function returns the created Container and also the
    /// parent of the Container in which the new Container is being returned.
    /// Based on the above example, when the Container `Tour` is created in `Pictures`, this
    /// function would return a tpule of (Tour, Home)
    /// In case if there is no parent for the Container then `None` is returned.
    /// Returns tuple of created_container & parent_container of the the current
    pub fn create(&mut self,
                  name: String,
                  versioned: bool,
                  access_level: AccessLevel,
                  metadata: Option<String>)
                  -> Result<(Container, Option<Container>), NfsError> {
        if name.is_empty() {
            return Err(NfsError::ParameterIsNotValid);
        }
        let user_metadata = try!(self.validate_metadata(metadata));
        let tag_type = if versioned {
            ::nfs::VERSIONED_DIRECTORY_LISTING_TAG
        } else {
            ::nfs::UNVERSIONED_DIRECTORY_LISTING_TAG
        };

        let directory_helper = DirectoryHelper::new(self.client.clone());
        let (created_directory, grand_parent) = try!(directory_helper.create(name,
                                                                             tag_type,
                                                                             user_metadata,
                                                                             versioned,
                                                                             access_level,
                                                                             Some(&mut self.directory_listing)));
        let created_container = Container {
            client: self.client.clone(),
            directory_listing: created_directory,
        };
        let parent = grand_parent.map(|parent_directory| {
            Container {
                client: self.client.clone(),
                directory_listing: parent_directory.clone(),
            }
        });
        Ok((created_container, parent))
    }

    /// Returns the created time of the container
    pub fn get_created_time(&self) -> &::time::Tm {
        self.directory_listing.get_metadata().get_created_time()
    }

    /// Returns the last modified time of the container
    pub fn get_modified_time(&self) -> &::time::Tm {
        self.directory_listing.get_metadata().get_modified_time()
    }

    /// Return the unique id of the container
    pub fn get_info(&self) -> ContainerInfo {
        ContainerInfo::from(self.directory_listing.get_metadata().clone())
    }

    /// Returns the user metadata saved as String.
    pub fn get_metadata(&self) -> String {
        match String::from_utf8(self.directory_listing.get_metadata().get_user_metadata().clone()) {
            Ok(data) => data,
            Err(_) => "".to_string(),
        }
    }

    /// Returns the name of the container
    pub fn get_name(&self) -> &String {
        self.directory_listing.get_metadata().get_name()
    }

    /// Returns the list of Blobs in the container
    pub fn get_blobs(&self) -> Vec<Blob> {
        self.directory_listing.get_files().iter().map(|x| Blob::from(x.clone())).collect()
    }

    /// Returns a Blob from the container
    pub fn get_blob(&self, name: String) -> Result<Blob, NfsError> {
        match self.directory_listing.find_file(&name) {
            Some(file) => Ok(Blob::from(file.clone())),
            None => Err(NfsError::FileNotFound),
        }
    }

    /// Returns the list of child containers
    pub fn get_containers(&self) -> Vec<ContainerInfo> {
        self.directory_listing
            .get_sub_directories()
            .iter()
            .map(|info| ContainerInfo::from(info.clone()))
            .collect()
    }

    /// Updates the metadata of the container
    pub fn update_metadata(&mut self, metadata: Option<String>) -> Result<Option<Container>, NfsError> {
        let user_metadata = try!(self.validate_metadata(metadata));
        self.directory_listing.get_mut_metadata().set_user_metadata(user_metadata);
        let directory_helper = DirectoryHelper::new(self.client.clone());
        let parent_directory = try!(directory_helper.update(&self.directory_listing));
        Ok(parent_directory.iter().next().map(|parent_directory| {
            Container {
                client: self.client.clone(),
                directory_listing: parent_directory.clone(),
            }
        }))
    }

    /// Retrieves Versions for the container
    pub fn get_versions(&self) -> Result<Vec<[u8; 64]>, NfsError> {
        self.list_container_versions(self.directory_listing.get_key().get_id(),
                                     self.directory_listing.get_key().get_type_tag())
    }

    /// Retrieves Versions for the container being referred by the container_id
    pub fn get_container_versions(&self, container_info: &ContainerInfo) -> Result<Vec<[u8; 64]>, NfsError> {
        let directory_metadata = container_info.into_directory_metadata();
        self.list_container_versions(directory_metadata.get_id(),
                                     directory_metadata.get_type_tag())
    }

    /// Fetches the latest version of the child container.
    /// Can fetch a specific version of the Container by passing the corresponding VersionId.
    pub fn get_container(&mut self,
                         container_info: &ContainerInfo,
                         version: Option<[u8; 64]>)
                         -> Result<Container, NfsError> {
        let directory_metadata = container_info.into_directory_metadata();
        let directory_helper = DirectoryHelper::new(self.client.clone());
        let dir_listing = match version {
            Some(version_id) => {
                debug!("Retrieving using version id ...");
                try!(directory_helper.get_by_version(directory_metadata.get_id(),
                                                     directory_metadata.get_access_level(),
                                                     XorName(version_id)))
            }
            None => {
                debug!("Retrieving the latest version ...");
                try!(directory_helper.get(directory_metadata.get_key()))
            }
        };
        Ok(Container {
            client: self.client.clone(),
            directory_listing: dir_listing,
        })
    }

    /// Deletes the child container
    pub fn delete_container(&mut self, name: &String) -> Result<Option<Container>, NfsError> {
        let directory_helper = DirectoryHelper::new(self.client.clone());
        let parent_directory = try!(directory_helper.delete(&mut self.directory_listing, name));
        Ok(parent_directory.iter().next().map(|parent_directory| {
            Container {
                client: self.client.clone(),
                directory_listing: parent_directory.clone(),
            }
        }))
    }

    /// Creates a Blob within the container
    /// Returns a Writter object
    /// The content of the blob is written using the writter.
    /// The blob is created only after the writter.close() is invoked
    pub fn create_blob(&mut self, name: String, metadata: Option<String>) -> Result<Writer, NfsError> {
        if name.is_empty() {
            return Err(NfsError::ParameterIsNotValid);
        }
        let user_metadata = try!(self.validate_metadata(metadata));
        let file_helper = FileHelper::new(self.client.clone());
        file_helper.create(name, user_metadata, self.directory_listing.clone())
    }

    /// Updates the blob content. Writes the complete data and updates the Blob
    pub fn update_blob_content(&mut self, blob: &Blob, data: &[u8]) -> Result<Option<Container>, NfsError> {
        let mut writer = try!(self.get_writer_for_blob(blob, Mode::Overwrite));
        debug!("Writing data to blob ...");
        writer.write(data, 0);
        let (parent_directory, grand_parent) = try!(writer.close());
        self.directory_listing = parent_directory.clone();
        Ok(grand_parent.iter().next().map(|parent_directory| {
            Container {
                client: self.client.clone(),
                directory_listing: parent_directory.clone(),
            }
        }))
    }

    /// Return a writter object for the Blob, through which the content of the blob can be updated
    /// This is useful while handling larger files, to enable writting content in parts
    pub fn get_blob_writer(&mut self, blob: &Blob) -> Result<Writer, NfsError> {
        self.get_writer_for_blob(blob, Mode::Modify)
    }

    /// Reads the content of the blob and returns the complete content
    pub fn get_blob_content(&self, blob: &Blob) -> Result<Vec<u8>, NfsError> {
        let mut reader = try!(self.get_reader_for_blob(blob));
        debug!("Reading contents of a blob ...");
        let size = reader.size();
        reader.read(0, size)
    }

    /// Returns a reader for the blob
    /// Using a Reader helps in handling large file contents and also fetch data in a specific range
    pub fn get_blob_reader<'a>(&self, blob: &'a Blob) -> Result<Reader<'a>, NfsError> {
        self.get_reader_for_blob(blob)
    }

    /// Returns the list of versions_id for the blob
    pub fn get_blob_versions(&self, name: &String) -> Result<Vec<Blob>, NfsError> {
        let file = try!(self.directory_listing.find_file(name).ok_or(NfsError::FileNotFound));
        let file_helper = FileHelper::new(self.client.clone());
        let versions = try!(file_helper.get_versions(&file, &self.directory_listing));
        Ok(versions.iter().map(|file| Blob::from(file.clone())).collect())
    }

    /// Update the metadata of the Blob in the container
    /// Returns Updated parent container, if the parent container exists.
    pub fn update_blob_metadata(&mut self,
                                mut blob: Blob,
                                metadata: Option<String>)
                                -> Result<Option<Container>, NfsError> {
        let user_metadata = try!(self.validate_metadata(metadata));
        let file_helper = FileHelper::new(self.client.clone());
        let mut file = blob.into_mut_file();
        file.get_mut_metadata().set_user_metadata(user_metadata);
        if let Some(parent_directory_listing) = try!(file_helper.update_metadata(file.clone(),
                                                                                 &mut self.directory_listing)) {
            Ok(Some(Container {
                client: self.client.clone(),
                directory_listing: parent_directory_listing,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete blob from the container
    pub fn delete_blob(&mut self, name: String) -> Result<(), NfsError> {
        let file_helper = FileHelper::new(self.client.clone());
        let _ = try!(file_helper.delete(name, &mut self.directory_listing));
        Ok(())
    }

    /// Copies the latest blob version from the container to the specified destination container
    pub fn copy_blob(&mut self, blob_name: &String, to_container: &ContainerInfo) -> Result<(), NfsError> {
        let to_dir = to_container.into_directory_metadata();
        if self.directory_listing.get_key() == to_dir.get_key() {
            return Err(NfsError::DestinationAndSourceAreSame);
        }
        let file = try!(self.directory_listing.find_file(blob_name).ok_or(NfsError::FileNotFound));
        let directory_helper = DirectoryHelper::new(self.client.clone());
        let mut destination = try!(directory_helper.get(to_dir.get_key()));
        if destination.find_file(blob_name).is_some() {
            return Err(NfsError::FileAlreadyExistsWithSameName);
        }
        debug!("Adding {:?} blob to destination files ...", blob_name);
        destination.get_mut_files().push(file.clone());
        let _ = try!(directory_helper.update(&destination));
        Ok(())
    }

    fn get_writer_for_blob(&self, blob: &Blob, mode: Mode) -> Result<Writer, NfsError> {
        let helper = FileHelper::new(self.client.clone());
        helper.update_content(blob.into_file().clone(),
                              mode,
                              self.directory_listing.clone())
    }

    fn get_reader_for_blob<'a>(&self, blob: &'a Blob) -> Result<Reader<'a>, NfsError> {
        match self.directory_listing.find_file(blob.get_name()) {
            Some(_) => Ok(Reader::new(self.client.clone(), blob.into_file())),
            None => Err(NfsError::FileNotFound),
        }
    }

    fn list_container_versions(&self, dir_id: &XorName, type_tag: u64) -> Result<Vec<[u8; 64]>, NfsError> {
        let directory_helper = DirectoryHelper::new(self.client.clone());
        let versions = try!(directory_helper.get_versions(dir_id, type_tag));
        Ok(versions.iter().map(|v| v.0).collect())
    }

    fn validate_metadata(&self, metadata: Option<String>) -> Result<Vec<u8>, NfsError> {
        match metadata {
            Some(data) => {
                if data.len() == 0 {
                    Err(NfsError::ParameterIsNotValid)
                } else {
                    Ok(data.into_bytes())
                }
            }
            None => Ok(Vec::new()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{Arc, Mutex};
    use core::client::Client;
    use core::utility::test_utils;
    use nfs::AccessLevel;

    fn get_client() -> Arc<Mutex<Client>> {
        Arc::new(Mutex::new(unwrap_result!(test_utils::get_client())))
    }

    #[test]
    fn authorise_container() {
        let client = get_client();
        let root_dir = unwrap_result!(Container::authorise(client.clone(), None));
        let root_dir_second = unwrap_result!(Container::authorise(client.clone(), None));
        assert_eq!(*root_dir.get_info().into_directory_metadata().get_key().get_id(),
                   *root_dir_second.get_info().into_directory_metadata().get_key().get_id());

        let root_dir_from_info = unwrap_result!(Container::authorise(client, Some(root_dir.get_info())));
        assert_eq!(*root_dir.get_info().into_directory_metadata().get_key().get_id(),
                   *root_dir_from_info.get_info().into_directory_metadata().get_key().get_id());
    }

    #[test]
    fn create_container() {
        let client = get_client();
        let mut container = unwrap_result!(Container::authorise(client.clone(), None));
        let _ = unwrap_result!(container.create("Home".to_string(), true, AccessLevel::Private, None));

        assert_eq!(container.get_containers().len(), 1);
        assert_eq!(container.get_containers()[0].get_name(), "Home");
        assert!(container.create("Home".to_string(), true, AccessLevel::Private, None).is_err());
    }


    #[test]
    fn delete_container() {
        let client = get_client();
        let dir_name = "Home".to_string();
        let mut container = unwrap_result!(Container::authorise(client, None));
        let _ = unwrap_result!(container.create(dir_name.clone(), true, AccessLevel::Private, None));

        assert_eq!(container.get_containers().len(), 1);
        assert_eq!(container.get_containers()[0].get_name(), "Home");

        let _ = unwrap_result!(container.delete_container(&dir_name));

        assert_eq!(container.get_containers().len(), 0);
    }

    #[test]
    fn create_update_delete_blob() {
        let client = get_client();
        let mut container = unwrap_result!(Container::authorise(client.clone(), None));
        let (mut home_container, _) = unwrap_result!(container.create("Home".to_string(),
                                                                      true,
                                                                      AccessLevel::Private,
                                                                      None));

        assert_eq!(container.get_containers().len(), 1);
        assert_eq!(container.get_containers()[0].get_name(), "Home");

        let mut writer = unwrap_result!(home_container.create_blob("sample.txt".to_string(), None));
        let data = "Hello World!".to_string().into_bytes();
        writer.write(&data[..], 0);
        let _ = unwrap_result!(writer.close());
        home_container = unwrap_result!(container.get_container(&home_container.get_info(), None));
        assert!(home_container.create_blob("sample.txt".to_string(), None).is_err());

        assert_eq!(unwrap_result!(home_container.get_blob_versions(&"sample.txt".to_string())).len(),
                   1);
        let blob = unwrap_result!(home_container.get_blob("sample.txt".to_string()));
        assert_eq!(unwrap_result!(home_container.get_blob_content(&blob)), data);

        let data_updated = "Hello World updated!".to_string().into_bytes();
        let _ = unwrap_result!(home_container.update_blob_content(&blob, &data_updated[..]));

        let blob = unwrap_result!(home_container.get_blob("sample.txt".to_string()));
        assert_eq!(unwrap_result!(home_container.get_blob_content(&blob)),
                   data_updated);

        // Assert versions
        let versions = unwrap_result!(home_container.get_blob_versions(&"sample.txt".to_string()));
        assert_eq!(versions.len(), 2);
        for i in 0..2 {
            if i == 0 {
                assert_eq!(unwrap_result!(home_container.get_blob_content(&versions[i])),
                           data);
            } else {
                assert_eq!(unwrap_result!(home_container.get_blob_content(&versions[i])),
                           data_updated);
            }
        }
        let metadata = "{\"purpose\": \"test\"}".to_string();
        let _ = unwrap_result!(home_container.update_blob_metadata(blob, Some(metadata.clone())));
        let blob = unwrap_result!(home_container.get_blob("sample.txt".to_string()));
        assert_eq!(blob.get_metadata(), metadata);

        let (mut docs_container, _) = unwrap_result!(container.create("Docs".to_string(),
                                                                      true,
                                                                      AccessLevel::Private,
                                                                      None));
        assert_eq!(docs_container.get_blobs().len(), 0);
        let _ = home_container.copy_blob(&"sample.txt".to_string(), &docs_container.get_info());
        docs_container = unwrap_result!(container.get_container(&docs_container.get_info(), None));
        assert_eq!(docs_container.get_blobs().len(), 1);

        let _ = home_container.delete_blob("sample.txt".to_string());
        assert_eq!(home_container.get_blobs().len(), 0);

        let (_, parent) = unwrap_result!(home_container.create("Pictures".to_string(),
                                                               true,
                                                               AccessLevel::Private,
                                                               None));
        assert!(parent.is_some());
        assert_eq!(*unwrap_option!(parent, "parent container should be present").get_name(),
                   *container.get_name());
    }
}
