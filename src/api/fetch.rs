// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::files::FilesMap;
use super::xorurl::xorurl_to_xorname;
use super::Safe;

pub enum SafeData {
    FilesContainer(FilesMap),
    ImmutableData(Vec<u8>),
}

impl Safe {
    pub fn fetch(&self, xorurl: &str) -> Result<SafeData, String> {
        //let XorUrl = XorUrlEncoder::from_url(xorurl)?;
        let files_map = self.files_container_get_latest(&xorurl)?;
        //let data = safe.files_get_published_immutable(&xorurl)?;

        Ok(SafeData::FilesContainer({ files_map }))
    }
}

// Unit Tests

#[test]
fn test_fetch() {}
