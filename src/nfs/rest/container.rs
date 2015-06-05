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
use routing;
use time;
use client;
use nfs::traits::FileWrapper;
use nfs::traits::DirectoryListingWrapper;
use nfs::traits::DirectoryInfoWrapper;

pub struct Container {
    client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>,
    directory_listing: nfs::directory_listing::DirectoryListing
}

impl Container {
    /// Authorizes the root directory access and return the Container
    /// Entry point for the Rest API
    pub fn authorise(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>, dir_id: [u8;64], parent_dir_id: [u8;64]) -> Result<Container, String> {
        let mut directory_helper = nfs::helper::DirectoryHelper::new(client.clone());
        let result = directory_helper.get(::routing::NameType(dir_id), ::routing::NameType(parent_dir_id));
        if result.is_err() {
            return Err(result.err().unwrap().to_string());
        }
        Ok(Container {
            client: client,
            directory_listing: result.unwrap()
        })
    }

    pub fn get_id(&self) -> [u8;64] {
        self.directory_listing.get_id().0
    }

    pub fn get_metadata(&self) -> Option<String> {
        let metadata = self.directory_listing.get_metadata().get_user_metadata();
        match metadata {
            Some(data) => Some(String::from_utf8(data).unwrap()),
            None => None
        }
    }

    pub fn get_name(&self) -> String {
        self.directory_listing.get_metadata().get_name()
    }

    pub fn get_created_time(&self) -> time::Tm {
        self.directory_listing.get_metadata().get_created_time()
    }

    pub fn get_modified_time(&self) -> time::Tm {
        self.directory_listing.get_metadata().get_modified_time()
    }

    pub fn get_blobs(&self) -> Vec<nfs::rest::Blob> {
        self.directory_listing.get_files().iter().map(|x| nfs::rest::Blob::convert_from_file(self.client.clone(), x.clone())).collect()
    }

    pub fn get_blob(&self, name: String, version: Option<[u8;64]>) -> Result<nfs::rest::Blob, String> {
        let mut directory_listing;
        if version.is_some() {
            let dir_id = self.directory_listing.get_id();
            let parent_id = self.directory_listing.get_parent_dir_id();
            let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
            let result = directory_helper.get_by_version(dir_id, parent_id, routing::NameType(version.unwrap()));
            if result.is_err() {
                return Err(result.unwrap_err());
            }
            directory_listing = result.unwrap();
        } else {
            directory_listing = self.directory_listing.clone();
        }
        match directory_listing.get_files().iter().find(|file| file.get_name() == name) {
            Some(file) => Ok(nfs::rest::Blob::convert_from_file(self.client.clone(), file.clone())),
            None => Err("File not found".to_string())
        }
    }
    

    pub fn create(&mut self, name: String, metadata: Option<String>) -> Result<(), String> {
        match self.validate_metadata(metadata) {
            Ok(user_metadata) => {
                let parent_dir_id = self.directory_listing.get_parent_dir_id();
                let mut dir_id;

                // Create directory
                {
                    let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
                    let result = directory_helper.create(parent_dir_id.clone(), name.clone(), user_metadata);

                    if result.is_err() {
                        return Err(result.unwrap_err());
                    }

                    dir_id = result.unwrap();
                }

                let mut created_directory;

                // Retrieve Created directory & add to the sub-folder list
                {
                    let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
                    let result = directory_helper.get(dir_id, parent_dir_id);

                    if result.is_err() {
                        return Err(result.unwrap_err());
                    }

                    let mut sub_dirs = self.directory_listing.get_sub_directories();
                    created_directory = result.unwrap();
                    sub_dirs.push(created_directory.get_info());
                    self.directory_listing.set_sub_directories(sub_dirs);
                }

                // Update the Container
                {
                    let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
                    let result = directory_helper.update(created_directory);
                    if result.is_err() {
                        return Err("Failed to create Conatiner".to_string());
                    }
                }

                Ok(())
            },
            Err(err) => Err(err),
        }
    }

    pub fn get_containers(&self) -> Vec<nfs::rest::ContainerInfo> {
        self.directory_listing.get_sub_directories().iter().map(|info| {
                nfs::rest::ContainerInfo::convert_from_directory_info(info.clone())
            }).collect()
    }

    pub fn update_metadata(&mut self, metadata: Option<String>) -> Result<(), String>{
        match self.validate_metadata(metadata) {
            Ok(user_metadata) => {
                self.directory_listing.set_user_metadata(user_metadata);
                let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
                match directory_helper.update(self.directory_listing.clone()) {
                    Ok(_) => Ok(()),
                    Err(msg) => Err(msg),
                }
            },
            Err(err) => Err(err),
        }
    }

    pub fn get_container(&mut self, name: String, version: Option<[u8; 64]>) -> Result<Container, String> {
        let sub_dirs = self.directory_listing.get_sub_directories();
        let dir_info = sub_dirs.iter().find(|&entry| entry.get_name() == name);
        if dir_info.is_none() {
            return Err("Container not found".to_string());
        }
        let dir_id = dir_info.unwrap().get_id();
        let parent_id = self.directory_listing.get_id();

        let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
        let mut result;
        if version.is_some() {
            let version_id = version.unwrap();
            result = directory_helper.get_by_version(dir_id, parent_id, routing::NameType(version_id))
        } else{
            result = directory_helper.get(dir_id, parent_id)
        }
        if result.is_err() {
            return Err(result.unwrap_err());
        }
        Ok(Container::convert_from_directory_listing(self.client.clone(), result.unwrap()))
    }

    pub fn get_versions(&mut self) -> Result<Vec<[u8;64]>, String> {
        let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
        match directory_helper.get_versions(self.directory_listing.get_id()) {
            Ok(versions) => {
                Ok(versions.iter().map(|v| v.0).collect())
            },
            Err(msg) => Err(msg)
        }
    }

    pub fn delete_container(&mut self, name: String) -> Result<(), String> {
        let mut sub_dirs = self.directory_listing.get_sub_directories();
        let find_result = sub_dirs.binary_search_by(|info| info.get_name().cmp(&name));
        if find_result.is_err() {
            return Err("Container not found".to_string());
        }
        sub_dirs.remove(find_result.unwrap());
        self.directory_listing.set_sub_directories(sub_dirs);
        let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
        match directory_helper.update(self.directory_listing.clone()) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg)
        }
    }

    pub fn create_blob(&mut self, name: String, metadata: Option<String>, size: u64) -> Result<nfs::io::Writer, String> {
        match self.validate_metadata(metadata) {
            Ok(user_metadata) => {
                let mut file_helper = nfs::helper::FileHelper::new(self.client.clone());
                file_helper.create(name, size, user_metadata, self.directory_listing.clone())
            },
            Err(err) => Err(err),
        }
    }

    pub fn delete_blob(&mut self, name: String) -> Result<(), String> {
        let mut sub_dirs = self.directory_listing.get_sub_directories();
        match sub_dirs.iter().position(|file| file.get_name() == name) {
            Some(pos) => {
                sub_dirs.remove(pos);
                self.directory_listing.set_sub_directories(sub_dirs);
                let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
                match directory_helper.update(self.directory_listing.clone()) {
                    Ok(_) => Ok(()),
                    Err(msg) => Err(msg)
                }
            },
            None => Err("File not found".to_string())
        }
    }

    fn validate_metadata(&self, metadata: Option<String>) -> Result<Vec<u8>, String> {
        match metadata {
            Some(data) => {
                if data.len() == 0 {
                    Err("Metadata cannot be empty".to_string())
                } else {
                    Ok(data.into_bytes())
                }
            },
            None => Ok(Vec::new()),
        }
    }
}

impl nfs::traits::DirectoryListingWrapper for Container {

    fn convert_to_directory_listing(&self) -> nfs::directory_listing::DirectoryListing {
        self.directory_listing.clone()
    }

    fn convert_from_directory_listing(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>, directory_listing: nfs::directory_listing::DirectoryListing) -> Container {
        Container {
            client: client,
            directory_listing: directory_listing
        }
    }
}
