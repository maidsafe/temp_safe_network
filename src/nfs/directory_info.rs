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
use nfs::metadata::Metadata;
use routing;
use std::fmt;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DirectoryInfo {
    metadata: Metadata,
    id: routing::NameType
}

impl DirectoryInfo {
    pub fn new(metadata: Metadata, id: routing::NameType) -> DirectoryInfo {
        DirectoryInfo {
            metadata: metadata,
            id: id
        }
    }

    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }

    pub fn get_metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    pub fn get_id(&self) -> routing::NameType {
        self.id.clone()
    }
}

impl fmt::Debug for DirectoryInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "metadata: {}, id: {}", self.get_metadata(), self.get_id())
    }
}

impl fmt::Display for DirectoryInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "metadata: {}, id: {}", self.get_metadata(), self.get_id())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use super::metadata::Metadata;
    use routing;
    use cbor;

    #[test]
    fn serialise() {
        let obj_before = DirectoryInfo::new(Metadata::new("hello.txt".to_string(), "{mime:\"application/json\"}".to_string().into_bytes()), routing::NameType([1u8;64]));

        let mut e = cbor::Encoder::from_memory();
        e.encode(&[&obj_before]).unwrap();

        let mut d = cbor::Decoder::from_bytes(e.as_bytes());
        let obj_after: DirectoryInfo = d.decode().next().unwrap().unwrap();

        assert_eq!(obj_before, obj_after);
    }
}
