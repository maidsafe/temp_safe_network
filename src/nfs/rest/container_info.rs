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

use nfs::AccessLevel;
use nfs::metadata::directory_metadata::DirectoryMetadata;

/// Wrapper over DirectoryInfo to present Rest-friendly name to the Restful interface users
pub struct ContainerInfo {
    metadata: DirectoryMetadata,
}

impl ContainerInfo {
    /// Get the name of the Container
    pub fn get_name(&self) -> &String {
        self.metadata.get_name()
    }

    /// Get the creation time for this Container
    pub fn get_created_time(&self) -> &::time::Tm {
        self.metadata.get_created_time()
    }

    /// Get the creation time for this Container
    pub fn get_modified_time(&self) -> &::time::Tm {
        self.metadata.get_modified_time()
    }

    /// Returns AccessLevel of the Container
    pub fn get_access_level(&self) -> &AccessLevel {
        self.metadata.get_access_level()
    }

    /// Returns type_tag of the Container
    pub fn get_type_tag(&self) -> u64 {
        self.metadata.get_type_tag()
    }

    /// Returns true if the Container is versioned, else false is returned
    pub fn is_versioned(&self) -> bool {
        self.metadata.is_versioned()
    }


    /// Convert the ContainerInfo to the format of DirectoryInfo that lower levels understand and
    /// operate on
    pub fn into_directory_metadata(&self) -> DirectoryMetadata {
        self.metadata.clone()
    }
}

impl From<DirectoryMetadata> for ContainerInfo {
    fn from(metadata: DirectoryMetadata) -> ContainerInfo {
        ContainerInfo { metadata: metadata }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use nfs::AccessLevel;
    use nfs::metadata::directory_metadata::DirectoryMetadata;
    use core::utility;

    #[test]
    fn create() {
        let name = unwrap_result!(utility::generate_random_string(10));
        let container_info = ContainerInfo {
            metadata: unwrap_result!(DirectoryMetadata::new(name.clone(),
                                                            10u64,
                                                            true,
                                                            AccessLevel::Public,
                                                            Vec::new(),
                                                            None)),
        };
        assert_eq!(*container_info.get_name(), name);
    }

    #[test]
    fn convert_from() {
        let name = unwrap_result!(utility::generate_random_string(10));
        let directory_metadata = unwrap_result!(DirectoryMetadata::new(name.clone(),
                                                                       10u64,
                                                                       true,
                                                                       AccessLevel::Public,
                                                                       Vec::new(),
                                                                       None));

        assert_eq!(*directory_metadata.get_name(), name);

        let container_info = ContainerInfo::from(directory_metadata.clone());

        assert_eq!(container_info.get_name(), directory_metadata.get_name());
        assert_eq!(container_info.get_created_time(),
                   directory_metadata.get_created_time());
    }

    #[test]
    fn convert_to() {
        let name = unwrap_result!(utility::generate_random_string(10));
        let container_info = ContainerInfo {
            metadata: unwrap_result!(DirectoryMetadata::new(name.clone(),
                                                            10u64,
                                                            true,
                                                            AccessLevel::Public,
                                                            Vec::new(),
                                                            None)),
        };

        assert_eq!(*container_info.get_name(), name.clone());

        let directory_metadata: DirectoryMetadata = container_info.into_directory_metadata();

        assert_eq!(*directory_metadata.get_name(), name);
        assert_eq!(directory_metadata.get_key().get_type_tag(), 10u64);
    }
}
