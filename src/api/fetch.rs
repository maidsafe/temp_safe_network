// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::files::FilesMap;
use super::xorurl::{xorurl_to_xorname, SafeContentType};
use super::{Safe, XorUrlEncoder};

pub enum SafeData {
    FilesContainer(FilesMap),
    ImmutableData(Vec<u8>),
}

impl Safe {
    pub fn fetch(&self, xorurl: &str) -> Result<SafeData, String> {
        let xorurl_encoder = XorUrlEncoder::from_url(xorurl)?;
        match xorurl_encoder.content_type() {
            SafeContentType::FilesContainer => {
                let files_map = self.files_container_get_latest(&xorurl)?;
                Ok(SafeData::FilesContainer({ files_map }))
            }
            SafeContentType::ImmutableData => {
                let data = self.files_get_published_immutable(&xorurl)?;
                Ok(SafeData::ImmutableData(data))
            }
            other => Err("Content tpe not supported yet by fetch".to_string()),
        }
    }
}

// Unit Tests

#[test]
fn test_fetch() {}
