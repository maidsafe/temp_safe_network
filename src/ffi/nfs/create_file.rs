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

use std::mem;

use core::SelfEncryptionStorage;
use ffi::{ParameterPacket, helper};
use ffi::errors::FfiError;
use ffi::nfs::FfiWriterHandle;
use nfs::errors::NfsError;
use nfs::file::File;
use nfs::helper::writer::{Mode, Writer};
use nfs::metadata::file_metadata::FileMetadata;
use self_encryption::DataMap;

#[derive(RustcDecodable, Debug)]
pub struct CreateFile {
    file_path: String,
    user_metadata: String,
    is_path_shared: bool,
}

impl CreateFile {
    #[allow(unsafe_code)]
    pub fn create(&mut self, params: ParameterPacket) -> Result<FfiWriterHandle, FfiError> {
        use rustc_serialize::base64::FromBase64;

        trace!("JSON create file, given the path.");

        if self.is_path_shared && !params.safe_drive_access {
            return Err(FfiError::PermissionDenied);
        };

        let start_dir_key = if self.is_path_shared {
            try!(params.safe_drive_dir_key
                .ok_or(FfiError::from("Safe Drive directory key is not present")))
        } else {
            try!(params.app_root_dir_key
                .ok_or(FfiError::from("Application directory key is not present")))
        };

        let mut tokens = helper::tokenise_path(&self.file_path, false);
        let file_name = try!(tokens.pop().ok_or(FfiError::InvalidPath));

        let file_directory = try!(helper::get_final_subdirectory(params.client.clone(),
                                                                 &tokens,
                                                                 Some(&start_dir_key)));

        let bin_metadata = try!(parse_result!(self.user_metadata.from_base64(),
                                              "Failed Converting from Base64."));

        let mut storage = Box::new(SelfEncryptionStorage::new(params.client.clone()));
        let writer: Writer<'static> = {
            let writer = match file_directory.find_file(&file_name) {
                Some(_) => try!(Err(NfsError::FileAlreadyExistsWithSameName)),
                None => {
                    let file = try!(File::new(FileMetadata::new(file_name, bin_metadata),
                                              DataMap::None));
                    try!(Writer::new(params.client,
                                     &mut *storage,
                                     Mode::Overwrite,
                                     file_directory,
                                     file))
                }
            };
            unsafe { mem::transmute(writer) }
        };

        Ok(FfiWriterHandle {
            writer: writer,
            _storage: storage,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ffi::test_utils;
    use nfs::helper::directory_helper::DirectoryHelper;

    #[test]
    fn create_file() {
        let parameter_packet = unwrap!(test_utils::get_parameter_packet(false));

        let mut request = CreateFile {
            file_path: "/test.txt".to_string(),
            user_metadata: "InNhbXBsZSBtZXRhZGF0YSI=".to_string(),
            is_path_shared: false,
        };
        let writer = unwrap!(request.create(parameter_packet.clone()));
        let _ = unwrap!(writer.close());

        let dir_helper = DirectoryHelper::new(parameter_packet.client);
        let app_dir = unwrap!(dir_helper.get(&unwrap!(parameter_packet.app_root_dir_key)));
        let _ = unwrap!(app_dir.find_file(&"test.txt".to_string()));
    }
}
