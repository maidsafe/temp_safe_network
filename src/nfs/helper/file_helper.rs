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
use time;
use nfs;
use nfs::helper::directory_helper::DirectoryHelper;
use routing;
use client;
use self_encryption;

/// File provides helper functions to perform Operations on Files
#[allow(dead_code)]
pub struct FileHelper {
    client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>
}

#[allow(dead_code)]
impl FileHelper {
    /// Create a new FileHelper instance
    pub fn new(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>) -> FileHelper {
        FileHelper {
            client: client
        }
    }

    pub fn create(&mut self, name: String, size: u64, user_metatdata: Vec<u8>,
            directory: nfs::directory_listing::DirectoryListing) -> Result<nfs::io::Writer, String> {
        if self.file_exists(directory.clone(), name.clone()) {
            return Err("File already exists".to_string());
        }
        let mut metadata = nfs::metadata::Metadata::new(name, user_metatdata);
        metadata.set_size(size);
        let file = nfs::file::File::new(metadata, self_encryption::datamap::DataMap::None);
        Ok(nfs::io::Writer::new(directory, file, self.client.clone()))
    }

    pub fn update(&mut self, file: nfs::file::File, directory: nfs::directory_listing::DirectoryListing) -> Result<nfs::io::Writer, String> {
        if !self.file_exists(directory.clone(), file.get_name()) {
            return Err("File not present in the directory".to_string());
        }
        Ok(nfs::io::Writer::new(directory, file, self.client.clone()))
    }

    /// Updates the file metadata. Returns the updated DirectoryListing
    pub fn update_metadata(&mut self, file: nfs::file::File, directory: &mut nfs::directory_listing::DirectoryListing, user_metadata: Vec<u8>) -> Result<(), String> {
        if !self.file_exists(directory.clone(), file.get_name()) {
            return Err("File not present in the directory".to_string());
        }
        file.get_metadata().set_user_metadata(user_metadata);
        let pos = directory.get_files().binary_search_by(|p| p.get_name().cmp(&file.get_name())).unwrap();
        let mut files = directory.get_files();
        files.remove(pos);
        files.insert(pos, file);
        directory.set_files(files);
        let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
        if directory_helper.update(directory.clone()).is_err() {
            return Err("Failed to update".to_string());
        }
        Ok(())
    }

    /// Return the versions of a directory containing modified versions of a file
    pub fn get_versions(&mut self, directory_id: routing::NameType, parent_id: routing::NameType, file: nfs::file::File)
                -> Result<Vec<routing::NameType>, String> {
        let mut versions = Vec::<routing::NameType>::new();
        let mut directory_helper = DirectoryHelper::new(self.client.clone());

        match directory_helper.get_versions(directory_id.clone()) {
            Ok(sdv_versions) => {
                let mut modified_time = time::empty_tm();
                for version_id in sdv_versions {
                    match directory_helper.get_by_version(directory_id.clone(), parent_id.clone(), version_id.clone()) {
                        Ok(directory_listing) => {
                            match directory_listing.get_files().iter().find(|&entry| entry.get_name() == file.get_name()) {
                                Some(file) => {
                                   if file.get_metadata().get_modified_time() != modified_time {
                                        modified_time = file.get_metadata().get_modified_time();
                                        versions.push(version_id);
                                    }
                                },
                                None => ()
                            }
                        }
                        Err(_) => { () }
                    }
                }
            },
            Err(_) => { () }
        }

        Ok(versions)
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

#[cfg(test)]
mod test {
    use super::*;

    fn get_dummy_client() -> ::client::Client {
        let keyword = "Spandan".to_string();
        let password = "Sharma".as_bytes();
        let pin = 1234u32;

        ::client::Client::create_account(&keyword,
                                         pin,
                                         &password,
                                         ::std::sync::Arc::new(::std::sync::Mutex::new(::std::collections::BTreeMap::new()))).ok().unwrap()
    }


    #[test]
    fn create_read_update() {
        let client = ::std::sync::Arc::new(::std::sync::Mutex::new(get_dummy_client()));
        let mut dir_helper = ::nfs::helper::directory_helper::DirectoryHelper::new(client.clone());

        let parent_id = ::routing::NameType::new([8u8; 64]);
        let created_dir_id: _;
        {
            let put_result = dir_helper.create(parent_id.clone(),
                                               "DirName".to_string(),
                                               vec![7u8; 100]);

            assert!(put_result.is_ok());
            created_dir_id = put_result.ok().unwrap();
        }

        let mut dir_listing: _;
        {
            let get_result = dir_helper.get(created_dir_id.clone(), parent_id.clone());
            assert!(get_result.is_ok());
            dir_listing = get_result.ok().unwrap();
        }

        let mut file_helper = FileHelper::new(client);
        let mut writer: _;
        {
            let result = file_helper.create("Name".to_string(), 0, vec![98u8; 100], dir_listing);
            assert!(result.is_ok());

            writer = result.ok().unwrap();
        }

        let data = vec![12u8; 20];
        writer.write(&data[..], 0);
        writer.close();

        {
            let get_result = dir_helper.get(created_dir_id.clone(), parent_id.clone());
            assert!(get_result.is_ok());
            dir_listing = get_result.ok().unwrap();
        }

        {
            let result = dir_listing.get_files();
            assert_eq!(result.len(), 1);

            let file = result[0].clone();

            let mut reader = file_helper.read(file);
            let rxd_data = reader.read(0, data.len() as u64).ok().unwrap();

            assert_eq!(rxd_data, data);

            {
                let mut writer: _;
                {
                    let result = file_helper.update(result[0].clone(), dir_listing);
                    assert!(result.is_ok());

                    writer = result.ok().unwrap();
                }

                let data = vec![11u8; 90];
                writer.write(&[11u8; 90], 0);
                writer.close();

                let get_result = dir_helper.get(created_dir_id.clone(), parent_id.clone());
                assert!(get_result.is_ok());
                dir_listing = get_result.ok().unwrap();

                let result = dir_listing.get_files();
                assert_eq!(result.len(), 1);

                let file = result[0].clone();

                let mut reader = file_helper.read(file.clone());
                let rxd_data = reader.read(0, data.len() as u64).ok().unwrap();

                assert_eq!(rxd_data, data);

                {
                    let versions = file_helper.get_versions(created_dir_id.clone(), parent_id.clone(), file);
                    assert_eq!(versions.unwrap().len(), 2);
                }
            }
        }
    }
}
