// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

use nfs::File as NativeFile;
use routing::{XOR_NAME_LEN, XorName};
use std::slice;
use time::Tm;
use util::ffi::u8_vec_to_ptr;

/// FFI-wrapper for `File`.
#[repr(C)]
pub struct File {
    /// File size in bytes.
    pub size: u64,
    /// Creation time.
    pub created: Tm,
    /// Modification time.
    pub modified: Tm,
    /// Pointer to the user metadata.
    pub user_metadata_ptr: *mut u8,
    /// Size of the user metadata.
    pub user_metadata_len: usize,
    /// Capacity of the user metadata (internal field).
    pub user_metadata_cap: usize,
    /// Name of the `ImmutableData` containing the content of this file.
    pub data_map_name: [u8; XOR_NAME_LEN],
}

impl File {
    /// Construct FFI wrapper for the native rust `File`, consuming the file.
    pub fn from_native(file: NativeFile) -> Self {
        // TODO: move the metadata, not clone.
        let user_metadata = file.user_metadata().to_vec();
        let (user_metadata_ptr, user_metadata_len, user_metadata_cap) =
            u8_vec_to_ptr(user_metadata);

        File {
            size: file.size(),
            created: *file.created_time(),
            modified: *file.modified_time(),
            user_metadata_ptr: user_metadata_ptr,
            user_metadata_len: user_metadata_len,
            user_metadata_cap: user_metadata_cap,
            data_map_name: file.data_map_name().0,
        }
    }

    /// Convert to the native rust equivalent, consuming self.
    pub unsafe fn into_native(self) -> NativeFile {
        let user_metadata = Vec::from_raw_parts(self.user_metadata_ptr,
                                                self.user_metadata_len,
                                                self.user_metadata_cap);

        let mut file = NativeFile::new(user_metadata);
        file.set_size(self.size);
        file.set_created_time(self.created);
        file.set_modified_time(self.modified);
        file.set_data_map_name(XorName(self.data_map_name));
        file
    }

    /// Convert to the native rust equivalent by cloning the internal data, preserving self.
    pub unsafe fn to_native(&self) -> NativeFile {
        let user_metadata = slice::from_raw_parts(self.user_metadata_ptr, self.user_metadata_len)
            .to_vec();

        let mut file = NativeFile::new(user_metadata);
        file.set_size(self.size);
        file.set_created_time(self.created);
        file.set_modified_time(self.modified);
        file.set_data_map_name(XorName(self.data_map_name));
        file
    }
}

/// Free the file from memory.
#[no_mangle]
pub unsafe extern "C" fn file_free(file: File) {
    let _ = file.into_native();
}
