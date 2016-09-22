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
use routing::XorName;

/// DirectoryKey represnts the meta information about a directory
/// A directory can be feteched with the DirectoryKey
#[derive(Debug, RustcEncodable, RustcDecodable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DirectoryKey {
    id: XorName,
    versioned: bool,
    access_level: AccessLevel,
}

impl DirectoryKey {
    /// Creates a new instance of DirectoryKey
    pub fn new(directory_id: XorName,
               versioned: bool,
               access_level: AccessLevel)
               -> DirectoryKey {
        DirectoryKey {
            id: directory_id,
            versioned: versioned,
            access_level: access_level,
        }
    }

    /// Returns the id
    pub fn get_id(&self) -> &XorName {
        &self.id
    }
    /// Returns the type_tag
    pub fn get_type_tag(&self) -> u64 {
        if self.versioned {
            ::nfs::VERSIONED_DIRECTORY_LISTING_TAG
        } else {
            ::nfs::UNVERSIONED_DIRECTORY_LISTING_TAG
        }
    }
    /// Returns true if the directory represented by the key is versioned, else returns false
    pub fn is_versioned(&self) -> bool {
        self.versioned
    }
    /// Returns the accesslevel of the directory represented by the key
    pub fn get_access_level(&self) -> &AccessLevel {
        &self.access_level
    }
}

#[cfg(test)]
mod test {
    use maidsafe_utilities::serialisation::{deserialise, serialise};
    use nfs::AccessLevel;
    use rand;
    use routing::XorName;
    use super::*;

    /// Should be able to serialise & deserialise the DirectoryKey
    #[test]
    fn serailise_and_deserialise_directory_key() {
        let id: XorName = rand::random();
        let versioned = false;
        let access_level = AccessLevel::Private;

        let directory_key = DirectoryKey::new(id,
                                              versioned,
                                              access_level.clone());

        let serialised = unwrap!(serialise(&directory_key));
        let deserilaised_key: DirectoryKey = unwrap!(deserialise(&serialised));
        assert_eq!(*deserilaised_key.get_id(), id);
        assert_eq!(*deserilaised_key.get_access_level(), access_level);
        assert_eq!(deserilaised_key.is_versioned(), versioned);
        assert_eq!(deserilaised_key.get_type_tag(),
                   ::nfs::UNVERSIONED_DIRECTORY_LISTING_TAG);

        let directory_key_versioned = DirectoryKey::new(id,
                                                        true,
                                                        access_level.clone());
        assert_eq!(directory_key_versioned.get_type_tag(),
                   ::nfs::VERSIONED_DIRECTORY_LISTING_TAG);
    }
}
