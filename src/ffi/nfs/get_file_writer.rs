// Copyright 2016 MaidSafe.net limited.
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
use ffi::errors::FfiError;
use ffi::nfs::FfiWriterHandle;
use ffi::{ParameterPacket, helper};
use nfs::helper::writer::{Mode, Writer};

#[derive(RustcDecodable, Debug)]
pub struct GetFileWriter {
    file_path: String,
    is_path_shared: bool,
}

impl GetFileWriter {
    #[allow(unsafe_code)]
    pub fn get(&mut self, params: ParameterPacket) -> Result<FfiWriterHandle, FfiError> {
        if self.is_path_shared && !params.safe_drive_access {
            return Err(FfiError::PermissionDenied);
        }

        let start_dir_key = if self.is_path_shared {
            try!(params.safe_drive_dir_key
                .ok_or(FfiError::from("Safe Drive directory key is not present")))
        } else {
            try!(params.app_root_dir_key
                .ok_or(FfiError::from("Application directory key is not present")))
        };
        let mut tokens = helper::tokenise_path(&self.file_path, false);
        let file_name = try!(tokens.pop().ok_or(FfiError::InvalidPath));
        let dir_of_file = try!(helper::get_final_subdirectory(params.client.clone(),
                                                              &tokens,
                                                              Some(&start_dir_key)));

        let file = try!(dir_of_file.find_file(&file_name)
            .cloned()
            .ok_or(FfiError::InvalidPath));
        let mut storage = Box::new(SelfEncryptionStorage::new(params.client.clone()));
        let writer: Writer<'static> = unsafe {
            mem::transmute(try!(Writer::new(params.client,
                                            &mut *storage,
                                            Mode::Modify,
                                            dir_of_file,
                                            file)))
        };

        Ok(FfiWriterHandle {
            writer: writer,
            _storage: storage,
        })
    }
}
