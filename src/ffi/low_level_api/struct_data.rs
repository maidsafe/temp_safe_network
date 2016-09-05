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

use core::{CLIENT_STRUCTURED_DATA_TAG, immut_data_operations};
use core::errors::CoreError;
use core::structured_data_operations::{unversioned, versioned};
use ffi::app::App;
use ffi::errors::FfiError;
use ffi::helper;
use ffi::low_level_api::{CipherOptHandle, DataIdHandle, StructDataHandle};
use ffi::low_level_api::cipher_opt::CipherOpt;
use ffi::low_level_api::object_cache::object_cache;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::{Data, DataIdentifier, ImmutableData, StructuredData, XOR_NAME_LEN, XorName};
use std::{mem, ptr, slice};

/// Create new StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_new(app: *const App,
                                         type_tag: u64,
                                         id: *const [u8; XOR_NAME_LEN],
                                         cipher_opt_h: CipherOptHandle,
                                         data: *const u8,
                                         size: u64,
                                         o_handle: *mut StructDataHandle)
                                         -> i32 {
    helper::catch_unwind_i32(|| {
        let app = &*app;
        let client = app.get_client();
        let xor_id = XorName(*id);
        let plain_text = slice::from_raw_parts(data, size as usize).to_owned();

        let (owner_keys, sign_key) = {
            let client_guard = unwrap!(client.lock());
            let owner_keys = vec![*ffi_try!(client_guard.get_public_signing_key())];
            let sign_key = ffi_try!(client_guard.get_secret_signing_key()).clone();
            (owner_keys, sign_key)
        };

        let sd = match type_tag {
            ::UNVERSIONED_STRUCT_DATA_TYPE_TAG => {
                let raw_data = ffi_try!(ffi_try!(unwrap!(object_cache().lock())
                        .cipher_opt
                        .get_mut(&cipher_opt_h)
                        .ok_or(FfiError::InvalidCipherOptHandle))
                    .encrypt(app, &plain_text));

                ffi_try!(unversioned::create(client,
                                             type_tag,
                                             xor_id,
                                             0,
                                             raw_data,
                                             owner_keys,
                                             Vec::new(),
                                             &sign_key,
                                             None))
            }
            ::VERSIONED_STRUCT_DATA_TYPE_TAG => {
                let immut_data =
                    ffi_try!(immut_data_operations::create(client.clone(), plain_text, None));
                let ser_immut_data = ffi_try!(serialise(&immut_data).map_err(FfiError::from));
                let raw_data = ffi_try!(ffi_try!(unwrap!(object_cache().lock())
                        .cipher_opt
                        .get_mut(&cipher_opt_h)
                        .ok_or(FfiError::InvalidCipherOptHandle))
                    .encrypt(app, &ser_immut_data));

                let immut_data_final = Data::Immutable(ImmutableData::new(raw_data));
                let immut_data_final_name = *immut_data_final.name();

                let resp_getter = ffi_try!(unwrap!(client.lock()).put(immut_data_final, None));
                ffi_try!(resp_getter.get());

                ffi_try!(versioned::create(client,
                                           immut_data_final_name,
                                           type_tag,
                                           xor_id,
                                           0,
                                           owner_keys,
                                           Vec::new(),
                                           &sign_key))
            }
            x if x >= CLIENT_STRUCTURED_DATA_TAG => {
                let raw_data = ffi_try!(ffi_try!(unwrap!(object_cache().lock())
                        .cipher_opt
                        .get_mut(&cipher_opt_h)
                        .ok_or(FfiError::InvalidCipherOptHandle))
                    .encrypt(app, &plain_text));

                ffi_try!(StructuredData::new(type_tag,
                                             xor_id,
                                             0,
                                             raw_data,
                                             owner_keys,
                                             Vec::new(),
                                             Some(&sign_key))
                    .map_err(CoreError::from))
            }
            _ => ffi_try!(Err(FfiError::InvalidStructuredDataTypeTag)),
        };


        let mut obj_cache = unwrap!(object_cache().lock());
        let handle = obj_cache.new_handle();
        if let Some(prev) = obj_cache.struct_data.insert(handle, sd) {
            debug!("Displaced StructuredData from ObjectCache: {:?}", prev);
        }
        ptr::write(o_handle, handle);

        0
    })
}

/// Fetch an existing StructuredData from Network.
#[no_mangle]
pub unsafe extern "C" fn struct_data_fetch(app: *const App,
                                           data_id_h: DataIdHandle,
                                           o_handle: *mut StructDataHandle)
                                           -> i32 {
    helper::catch_unwind_i32(|| {
        let client = (*app).get_client();
        let data_id = *ffi_try!(unwrap!(object_cache().lock())
            .data_id
            .get_mut(&data_id_h)
            .ok_or(FfiError::InvalidDataIdHandle));
        let resp_getter = ffi_try!(unwrap!(client.lock()).get(data_id, None));
        let sd = match ffi_try!(resp_getter.get()) {
            Data::Structured(sd) => sd,
            _ => ffi_try!(Err(CoreError::ReceivedUnexpectedData)),
        };

        let mut obj_cache = unwrap!(object_cache().lock());
        let handle = obj_cache.new_handle();
        if let Some(prev) = obj_cache.struct_data.insert(handle, sd) {
            debug!("Displaced StructuredData from ObjectCache: {:?}", prev);

        }
        ptr::write(o_handle, handle);

        0
    })
}

// TODO possibly move this to data_id module
/// Extract DataIdentifier from StructuredData.
#[no_mangle]
pub unsafe extern "C" fn struct_data_extract_data_id(sd_h: StructDataHandle,
                                                     o_handle: *mut DataIdHandle)
                                                     -> i32 {
    helper::catch_unwind_i32(|| {
        let mut obj_cache = unwrap!(object_cache().lock());
        let data_id =
            ffi_try!(obj_cache.struct_data.get_mut(&sd_h).ok_or(FfiError::InvalidStructDataHandle))
                .identifier();
        let handle = obj_cache.new_handle();
        if let Some(prev) = obj_cache.data_id.insert(handle, data_id) {
            debug!("Displaced DataIdentifier from ObjectCache: {:?}", prev);
        }
        ptr::write(o_handle, handle);

        0
    })
}

// TODO See if we can extract common functionality and merge with new() above
/// Put new data into StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_new_data(app: *const App,
                                              sd_h: StructDataHandle,
                                              cipher_opt_h: CipherOptHandle,
                                              data: *const u8,
                                              size: u64)
                                              -> i32 {
    helper::catch_unwind_i32(|| {
        let mut sd = ffi_try!(unwrap!(object_cache().lock())
            .struct_data
            .remove(&sd_h)
            .ok_or(FfiError::InvalidStructDataHandle));

        let app = &*app;
        let client = app.get_client();
        let plain_text = slice::from_raw_parts(data, size as usize).to_owned();

        let sign_key = ffi_try!(unwrap!(client.lock()).get_secret_signing_key()).clone();

        sd = match sd.get_type_tag() {
            ::UNVERSIONED_STRUCT_DATA_TYPE_TAG => {
                let raw_data = ffi_try!(ffi_try!(unwrap!(object_cache().lock())
                        .cipher_opt
                        .get_mut(&cipher_opt_h)
                        .ok_or(FfiError::InvalidCipherOptHandle))
                    .encrypt(app, &plain_text));

                ffi_try!(unversioned::create(client,
                                             sd.get_type_tag(),
                                             *sd.name(),
                                             sd.get_version() + 1,
                                             raw_data,
                                             // TODO I am discarding this SD. Why does routing make
                                             // me clone unnecessarily ? - check.
                                             sd.get_owner_keys().clone(),
                                             sd.get_previous_owner_keys().clone(),
                                             &sign_key,
                                             None))
            }
            ::VERSIONED_STRUCT_DATA_TYPE_TAG => {
                let immut_data =
                    ffi_try!(immut_data_operations::create(client.clone(), plain_text, None));
                let ser_immut_data = ffi_try!(serialise(&immut_data).map_err(FfiError::from));
                let raw_data = ffi_try!(ffi_try!(unwrap!(object_cache().lock())
                        .cipher_opt
                        .get_mut(&cipher_opt_h)
                        .ok_or(FfiError::InvalidCipherOptHandle))
                    .encrypt(app, &ser_immut_data));

                let immut_data_final = Data::Immutable(ImmutableData::new(raw_data));
                let immut_data_final_name = *immut_data_final.name();

                let resp_getter = ffi_try!(unwrap!(client.lock()).put(immut_data_final, None));
                ffi_try!(resp_getter.get());

                ffi_try!(versioned::append_version(client, sd, immut_data_final_name, &sign_key))
            }
            x if x >= CLIENT_STRUCTURED_DATA_TAG => {
                let raw_data = ffi_try!(ffi_try!(unwrap!(object_cache().lock())
                        .cipher_opt
                        .get_mut(&cipher_opt_h)
                        .ok_or(FfiError::InvalidCipherOptHandle))
                    .encrypt(app, &plain_text));

                ffi_try!(StructuredData::new(sd.get_type_tag(),
                                             *sd.name(),
                                             sd.get_version() + 1,
                                             raw_data,
                                             sd.get_owner_keys().clone(),
                                             sd.get_previous_owner_keys().clone(),
                                             Some(&sign_key))
                    .map_err(CoreError::from))
            }
            _ => ffi_try!(Err(FfiError::InvalidStructuredDataTypeTag)),
        };

        if let Some(prev) = unwrap!(object_cache().lock()).struct_data.insert(sd_h, sd) {
            debug!("Displaced StructuredData from ObjectCache: {:?}", prev);
        }

        0
    })
}

/// Extract data from StructuredData
#[no_mangle]
pub unsafe extern "C" fn struct_data_extract_data(app: *const App,
                                                  sd_h: StructDataHandle,
                                                  o_data: *mut *const u8,
                                                  o_size: *mut u64,
                                                  o_capacity: *mut u64)
                                                  -> i32 {
    helper::catch_unwind_i32(|| {
        let app = &*app;
        let client = app.get_client();

        let mut obj_cache = unwrap!(object_cache().lock());
        let sd =
            ffi_try!(obj_cache.struct_data.get_mut(&sd_h).ok_or(FfiError::InvalidDataIdHandle));

        let plain_text = match sd.get_type_tag() {
            ::UNVERSIONED_STRUCT_DATA_TYPE_TAG => {
                let raw_data = ffi_try!(unversioned::get_data(client, sd, None));
                ffi_try!(CipherOpt::decrypt(&app, &raw_data))
            }
            ::VERSIONED_STRUCT_DATA_TYPE_TAG => {
                let mut versions = ffi_try!(versioned::get_all_versions(client.clone(), sd));
                if let Some(immut_data_final_name) = versions.pop() {
                    let resp_getter = ffi_try!(unwrap!(client.lock())
                        .get(DataIdentifier::Immutable(immut_data_final_name), None));
                    let immut_data_final = match ffi_try!(resp_getter.get()) {
                        Data::Immutable(immut_data) => immut_data,
                        _ => ffi_try!(Err(CoreError::ReceivedUnexpectedData)),
                    };

                    let ser_immut_data = ffi_try!(CipherOpt::decrypt(&app,
                                                                     immut_data_final.value()));
                    let immut_data = ffi_try!(deserialise::<ImmutableData>(&ser_immut_data)
                        .map_err(FfiError::from));
                    ffi_try!(immut_data_operations::get_data(client, *immut_data.name(), None))
                } else {
                    Vec::new()
                }
            }
            x if x >= CLIENT_STRUCTURED_DATA_TAG => {
                ffi_try!(CipherOpt::decrypt(&app, sd.get_data()))
            }
            _ => ffi_try!(Err(FfiError::InvalidStructuredDataTypeTag)),
        };

        *o_data = plain_text.as_ptr();
        ptr::write(o_size, plain_text.len() as u64);
        ptr::write(o_capacity, plain_text.capacity() as u64);
        mem::forget(plain_text);

        0
    })
}
