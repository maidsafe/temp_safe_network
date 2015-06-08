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

pub struct ContainerInfo {
    info: nfs::directory_info::DirectoryInfo,
}

impl ContainerInfo {

    pub fn get_name(&self) -> &String {
        self.info.get_metadata().get_name()
    }

    pub fn get_metadata(&self) -> Option<String> {
        let metadata = self.info.get_metadata().get_user_metadata();
        match metadata {
            Some(data) => Some(String::from_utf8(data.clone()).unwrap()),
            None => None
        }
    }

    pub fn get_created_time(&self) -> time::Tm {
        self.info.get_metadata().get_created_time()
    }

    fn convert_to_directory_info(&self) -> nfs::directory_info::DirectoryInfo {
        self.info.clone()
    }

    pub fn convert_from_directory_info(info: nfs::directory_info::DirectoryInfo) -> ContainerInfo {
        ContainerInfo {
            info: info
        }
    }

}
