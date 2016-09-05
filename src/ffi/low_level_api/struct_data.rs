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

use ffi::low_level_api::cipher_opt::CipherOpt;
// use ffi::low_level_api::object_cache::object_cache;
use routing::{StructuredData, XOR_NAME_LEN};
use std::slice;

#[no_mangle]
pub unsafe extern "C" fn struct_data_create(app: *const App,
                                            type_tag: u64,
                                            id: *const [u8; XOR_NAME_LEN],
                                            cipher_opt: *const CipherOpt,
                                            data: *const u8,
                                            size: u64,
                                            o_sd: *mut *const StructuredData)
                                            -> i32 {
    helper::catch_unwind_i32(|| {
        let data_vec = {
            let data_owned = slice::from_raw_parts(data, size).to_owned();
        };

        0
    })
}
