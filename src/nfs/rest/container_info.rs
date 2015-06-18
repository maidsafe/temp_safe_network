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
use time;

/// Wrapper over DirectoryInfo to present Rest-friendly name to the Restful interface users
pub struct ContainerInfo {
    info: nfs::directory_info::DirectoryInfo,
}

impl ContainerInfo {
    /// Get Container ID. This is the directory ID which is unique for every directory and is the
    /// only way to retrieve that directory (DirectoryListing) from the network
    pub fn get_id(&self) -> [u8;64] {
        self.info.get_id().0
    }

    /// Get the name of the Container
    pub fn get_name(&self) -> &String {
        self.info.get_metadata().get_name()
    }

    // pub fn get_metadata(&self) -> Option<String> {
    //     let metadata = self.info.get_metadata().get_user_metadata();
    //     match metadata {
    //         Some(data) => Some(String::from_utf8(data.clone()).unwrap()),
    //         None => None
    //     }
    // }

    /// Get the creation time for this Container
    pub fn get_created_time(&self) -> time::Tm {
        self.info.get_metadata().get_created_time()
    }

    /// Convert the ContainerInfo to the format of DirectoryInfo that lower levels understand and
    /// operate on
    pub fn convert_to_directory_info(&self) -> nfs::directory_info::DirectoryInfo {
        self.info.clone()
    }

    /// Convert from the format of DirectoryInfo that the lower levels understand to the rest
    /// friendly ContainerInfo
    pub fn convert_from_directory_info(info: nfs::directory_info::DirectoryInfo) -> ContainerInfo {
        ContainerInfo {
            info: info
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nfs::metadata::Metadata;
    use nfs::directory_info::DirectoryInfo;

    #[test]
    fn create() {
        let name = ::utility::generate_random_string(10);
        let metadata = Metadata::new(name.clone(), Vec::new());
        let container_info = ContainerInfo{ info: DirectoryInfo::new(metadata) };

        assert_eq!(*container_info.get_name(), name);
    }

    #[test]
    fn convert_from() {
        let name = ::utility::generate_random_string(10);
        let metadata = Metadata::new(name.clone(), Vec::new());
        let directory_info = DirectoryInfo::new(metadata);

        assert_eq!(*directory_info.get_name(), name);

        let container_info = ContainerInfo::convert_from_directory_info(directory_info.clone());

        assert_eq!(container_info.get_name(), directory_info.get_name());
        assert_eq!(container_info.get_created_time(), directory_info.get_metadata().get_created_time());
    }

    #[test]
    fn convert_to() {
        let name = ::utility::generate_random_string(10);
        let metadata = Metadata::new(name.clone(), Vec::new());
        let container_info = ContainerInfo{ info: DirectoryInfo::new(metadata) };

        assert_eq!(*container_info.get_name(), name);

        let directory_info = container_info.convert_to_directory_info();

        assert_eq!(directory_info.get_name(), container_info.get_name());
        assert_eq!(directory_info.get_metadata().get_created_time(), container_info.get_created_time());
    }
}
