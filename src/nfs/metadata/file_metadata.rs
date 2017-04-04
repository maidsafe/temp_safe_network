// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use chrono::prelude::{DateTime, UTC};

/// `FileMetadata` about a File or a Directory
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct FileMetadata {
    name: String,
    size: u64,
    created_time: DateTime<UTC>,
    modified_time: DateTime<UTC>,
    user_metadata: Vec<u8>,
    version: u32,
}

impl FileMetadata {
    /// Create a new instance of FileMetadata
    pub fn new(name: String, user_metadata: Vec<u8>) -> FileMetadata {
        FileMetadata {
            name: name,
            size: 0,
            // Version 0 is considered as invalid - do not change to version 0. This is used as
            // default vaule in comparisons.
            version: 1,
            created_time: UTC::now(),
            modified_time: UTC::now(),
            user_metadata: user_metadata,
        }
    }

    /// Get version
    pub fn get_version(&self) -> u32 {
        self.version
    }

    /// Get time of creation
    pub fn get_created_time(&self) -> &DateTime<UTC> {
        &self.created_time
    }

    /// Get time of modification
    pub fn get_modified_time(&self) -> &DateTime<UTC> {
        &self.modified_time
    }

    /// Get name associated with the structure (file or directory) that this metadata is a part
    /// of
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get size information
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Get user setteble custom metadata
    pub fn get_user_metadata(&self) -> &[u8] {
        &self.user_metadata
    }


    /// Set name associated with the structure (file or directory)
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Increment the file version
    pub fn increment_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    /// Set the size of file
    pub fn set_size(&mut self, size: u64) {
        self.size = size;
    }

    /// Set time of modification
    pub fn set_modified_time(&mut self, modified_time: DateTime<UTC>) {
        self.modified_time = modified_time
    }

    /// User setteble metadata for custom metadata
    pub fn set_user_metadata(&mut self, user_metadata: Vec<u8>) {
        self.user_metadata = user_metadata;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maidsafe_utilities::serialisation::{deserialise, serialise};

    #[test]
    fn serialise_and_deserialise_file_metadata() {
        let obj_before = FileMetadata::new("hello.txt".to_string(),
                                           "{mime: \"application/json\"}".to_string().into_bytes());
        let serialised_data = unwrap!(serialise(&obj_before));
        let obj_after = unwrap!(deserialise(&serialised_data));
        assert_eq!(obj_before, obj_after);
    }
}
