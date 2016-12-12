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

use app::App;
use app::errors::AppError;
use app::object_cache::{CipherOptHandle, SelfEncryptorReaderHandle, SelfEncryptorWriterHandle,
                        XorNameHandle};
use core::{FutureExt, SelfEncryptionStorage, immutable_data};
use futures::Future;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::ImmutableData;
use self_encryption::{DataMap, SelfEncryptor, SequentialEncryptor};
use std::{mem, ptr, slice};
use std::os::raw::c_void;
use super::cipher_opt::CipherOpt;
use util::ffi::{OpaqueCtx, catch_unwind_cb};

type SEWriterHandle = SelfEncryptorWriterHandle;
type SEReaderHandle = SelfEncryptorReaderHandle;

/// Get a Self Encryptor
#[no_mangle]
pub unsafe extern "C" fn idata_new_self_encryptor(app: *const App,
                                                  user_data: *mut c_void,
                                                  o_cb: extern "C" fn(*mut c_void,
                                                                      i32,
                                                                      SEWriterHandle)) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data, o_cb, || {
        (*app).send(move |client, context| {
            let se_storage = SelfEncryptionStorage::new(client.clone());
            let context = context.clone();

            let fut = SequentialEncryptor::new(se_storage, None)
                .map_err(AppError::from)
                .map(move |se| {
                    let handle = context.object_cache().insert_se_writer(se);
                    o_cb(user_data.0, 0, handle);
                })
                .map_err(move |e| {
                    o_cb(user_data.0, ffi_error_code!(e), 0);
                })
                .into_box();

            Some(fut)
        })
    });
}

/// Write to Self Encryptor
#[no_mangle]
pub unsafe extern "C" fn idata_write_to_self_encryptor(app: *const App,
                                                       se_h: SEWriterHandle,
                                                       data: *const u8,
                                                       size: usize,
                                                       user_data: *mut c_void,
                                                       o_cb: extern "C" fn(*mut c_void, i32)) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data, o_cb, || {
        let data_slice = slice::from_raw_parts(data, size);

        (*app).send(move |_, context| {
            let fut = {
                match context.object_cache().get_se_writer(se_h) {
                    Ok(writer) => writer.write(data_slice),
                    Err(e) => {
                        o_cb(user_data.0, ffi_error_code!(e));
                        return None;
                    }
                }
            };
            let fut = fut.map_err(AppError::from)
                .then(move |res| {
                    o_cb(user_data.0, ffi_result_code!(res));
                    Ok(())
                })
                .into_box();
            Some(fut)
        })
    });
}

/// Close Self Encryptor
#[no_mangle]
pub unsafe extern "C" fn idata_close_self_encryptor(app: *const App,
                                                    se_h: SEWriterHandle,
                                                    cipher_opt_h: CipherOptHandle,
                                                    user_data: *mut c_void,
                                                    o_cb: extern "C" fn(*mut c_void,
                                                                        i32,
                                                                        XorNameHandle)) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data, o_cb, || {
        (*app).send(move |client, context| {
            let fut = {
                match context.object_cache().remove_se_writer(se_h) {
                    Ok(se_wrapper) => se_wrapper.close(),
                    Err(e) => {
                        o_cb(user_data.0, ffi_error_code!(e), 0);
                        return None;
                    }
                }
            };

            let client2 = client.clone();
            let client3 = client.clone();

            let context2 = context.clone();
            let context3 = context.clone();

            let fut = fut.map_err(AppError::from)
                .and_then(move |(data_map, _)| {
                    let ser_data_map = fry!(serialise(&data_map));
                    immutable_data::create(&client2, ser_data_map, None)
                        .map_err(AppError::from)
                        .into_box()
                })
                .and_then(move |final_immut_data| {
                    let ser_final_immut_data = serialise(&final_immut_data)?;

                    let raw_data = {
                        let cipher_opt = context2.object_cache().get_cipher_opt(cipher_opt_h)?;
                        let sym_key = context2.sym_enc_key()?;
                        cipher_opt.encrypt(&ser_final_immut_data, sym_key)?
                    };

                    let raw_immut_data = ImmutableData::new(raw_data);
                    Ok((*raw_immut_data.name(), raw_immut_data))
                })
                .and_then(move |(name, data)| {
                    client3.put_idata(data)
                        .map_err(AppError::from)
                        .map(move |_| context3.object_cache().insert_xor_name(name))
                })
                .then(move |result| {
                    match result {
                        Ok(handle) => o_cb(user_data.0, 0, handle),
                        Err(e) => o_cb(user_data.0, ffi_error_code!(e), 0),
                    }
                    Ok(())
                })
                .into_box();

            Some(fut)
        })
    });
}

/// Fetch Self Encryptor
#[no_mangle]
pub unsafe extern "C" fn idata_fetch_self_encryptor(app: *const App,
                                                    name_h: XorNameHandle,
                                                    user_data: *mut c_void,
                                                    o_cb: extern "C" fn(*mut c_void,
                                                                        i32,
                                                                        SEReaderHandle)) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data, o_cb, || {
        (*app).send(move |client, context| {
            let client2 = client.clone();
            let client3 = client.clone();
            let client4 = client.clone();

            let context2 = context.clone();
            let context3 = context.clone();

            let fut = {
                match context.object_cache().get_xor_name(name_h) {
                    Ok(data_id) => client.get_idata(*data_id),
                    Err(e) => {
                        o_cb(user_data.0, ffi_error_code!(e), 0);
                        return None;
                    }
                }
            };

            fut.map_err(AppError::from)
                .and_then(move |raw_immut_data| {
                    let sym_key = context2.sym_enc_key()?;
                    let (asym_pk, asym_sk) = client2.encryption_keypair()?;
                    let ser_final_immut_data =
                        CipherOpt::decrypt(raw_immut_data.value(), sym_key, &asym_pk, &asym_sk)?;

                    Ok(deserialise::<ImmutableData>(&ser_final_immut_data)?)
                })
                .and_then(move |final_immut_data| {
                    immutable_data::extract_value(&client3, final_immut_data, None)
                        .map_err(AppError::from)
                })
                .and_then(move |ser_data_map| {
                    let data_map = deserialise::<DataMap>(&ser_data_map)?;
                    let se_storage = SelfEncryptionStorage::new(client4);

                    SelfEncryptor::new(se_storage, data_map).map_err(AppError::from)
                })
                .map(move |se_wrapper| {
                    let handle = context3.object_cache().insert_se_reader(se_wrapper);
                    o_cb(user_data.0, 0, handle);
                })
                .map_err(move |e| {
                    o_cb(user_data.0, ffi_error_code!(e), 0);
                })
                .into_box()
                .into()
        })
    });
}

/// Get data size from Self Encryptor
#[no_mangle]
pub unsafe extern "C" fn idata_size(app: *const App,
                                    se_h: SEReaderHandle,
                                    user_data: *mut c_void,
                                    o_cb: extern "C" fn(*mut c_void, i32, u64)) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data, o_cb, || {
        (*app).send(move |_, context| {
            match context.object_cache().get_se_reader(se_h) {
                Ok(se) => {
                    o_cb(user_data.0, 0, se.len());
                }
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e), 0);
                }
            };
            None
        })
    });
}

/// Read from Self Encryptor
/// Callback parameters are: user data, error code, data, size, capacity
#[no_mangle]
pub unsafe extern "C" fn idata_read_from_self_encryptor(app: *const App,
                                                        se_h: SEReaderHandle,
                                                        from_pos: u64,
                                                        len: u64,
                                                        user_data: *mut c_void,
                                                        o_cb: extern "C" fn(*mut c_void,
                                                                            i32,
                                                                            *mut u8,
                                                                            usize,
                                                                            usize)) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data, o_cb, || {
        (*app).send(move |_, context| {
            let se = match context.object_cache().get_se_reader(se_h) {
                Ok(r) => r,
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e), ptr::null_mut(), 0, 0);
                    return None;
                }
            };

            if from_pos + len > se.len() {
                o_cb(user_data.0,
                     ffi_error_code!(AppError::InvalidSelfEncryptorReadOffsets),
                     ptr::null_mut(),
                     0,
                     0);
                return None;
            }

            let fut = se.read(from_pos, len)
                .map(move |mut data| {
                    let size = data.len();
                    let capacity = data.capacity();
                    o_cb(user_data.0, 0, data.as_mut_ptr(), size, capacity);
                    mem::forget(data);
                })
                .map_err(AppError::from)
                .map_err(move |e| {
                    o_cb(user_data.0, ffi_error_code!(e), ptr::null_mut(), 0, 0);
                })
                .into_box();

            Some(fut)
        })
    });
}

/// Free Self Encryptor Writer handle
#[no_mangle]
pub unsafe extern "C" fn idata_self_encryptor_writer_free(app: *const App,
                                                          handle: SEWriterHandle,
                                                          user_data: *mut c_void,
                                                          o_cb: extern "C" fn(*mut c_void, i32)) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data, o_cb, || {
        (*app).send(move |_, context| {
            let res = context.object_cache().remove_se_writer(handle);
            o_cb(user_data.0, ffi_result_code!(res));
            None
        })
    });
}

/// Free Self Encryptor Reader handle
#[no_mangle]
pub unsafe extern "C" fn idata_self_encryptor_reader_free(app: *const App,
                                                          handle: SEReaderHandle,
                                                          user_data: *mut c_void,
                                                          o_cb: extern "C" fn(*mut c_void, i32)) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data, o_cb, || {
        (*app).send(move |_, context| {
            let res = context.object_cache().remove_se_reader(handle);
            o_cb(user_data.0, ffi_result_code!(res));
            None
        })
    })
}

#[cfg(test)]
mod tests {
    use app::errors::AppError;
    use app::ffi::cipher_opt::*;
    use app::ffi::xor_name::*;
    use app::test_util::create_app;
    use core::utility;
    use super::*;
    use util::ffi::test_util::{call_0, call_1, call_3};

    #[test]
    fn immut_data_operations() {
        // TODO: uncomment and fix these tests. We need to create account
        // and authorize the app to be able to perform mutations.

        let app = create_app();

        let plain_text = unwrap!(utility::generate_random_vector::<u8>(10));

        unsafe {
            let cipher_opt_h = unwrap!(call_1(|ud, cb| cipher_opt_new_symmetric(&app, ud, cb)));
            let se_writer_h = unwrap!(call_1(|ud, cb| idata_new_self_encryptor(&app, ud, cb)));

            let res = call_0(|ud, cb| {
                idata_write_to_self_encryptor(&app,
                                              0,
                                              plain_text.as_ptr(),
                                              plain_text.len(),
                                              ud,
                                              cb)
            });
            assert_eq!(res, Err(AppError::InvalidSelfEncryptorHandle.into()));

            unwrap!(call_0(|ud, cb| {
                idata_write_to_self_encryptor(&app,
                                              se_writer_h,
                                              plain_text.as_ptr(),
                                              plain_text.len(),
                                              ud,
                                              cb)
            }));

            let name_h = unwrap!(call_1(|ud, cb| {
                idata_close_self_encryptor(&app, se_writer_h, cipher_opt_h, ud, cb)
            }));

            // It should've been closed by immut_data_close_self_encryptor
            {
                let res =
                    call_0(|ud, cb| idata_self_encryptor_writer_free(&app, se_writer_h, ud, cb));
                assert_eq!(res, Err(AppError::InvalidSelfEncryptorHandle.into()));
            }

            // Invalid Self encryptor reader.
            let res = call_1(|ud, cb| idata_size(&app, 0, ud, cb));
            assert_eq!(res, Err(AppError::InvalidSelfEncryptorHandle.into()));

            // Invalid Self encryptor reader.
            let res = call_1(|ud, cb| idata_size(&app, se_writer_h, ud, cb));
            assert_eq!(res, Err(AppError::InvalidSelfEncryptorHandle.into()));

            let se_reader_h =
                unwrap!(call_1(|ud, cb| idata_fetch_self_encryptor(&app, name_h, ud, cb)));

            let size = unwrap!(call_1(|ud, cb| idata_size(&app, se_reader_h, ud, cb)));
            assert_eq!(size, plain_text.len() as u64);

            let res =
                call_3(|ud, cb| idata_read_from_self_encryptor(&app, se_reader_h, 1, size, ud, cb));
            assert_eq!(res, Err(AppError::InvalidSelfEncryptorReadOffsets.into()));

            let (data_ptr, data_size, data_cap) = unwrap!(call_3(|ud, cb| {
                idata_read_from_self_encryptor(&app, se_reader_h, 0, size, ud, cb)
            }));
            let received_plain_text = Vec::from_raw_parts(data_ptr, data_size, data_cap);
            assert_eq!(plain_text, received_plain_text);

            unwrap!(call_0(|ud, cb| idata_self_encryptor_reader_free(&app, se_reader_h, ud, cb)));

            let res = call_0(|ud, cb| idata_self_encryptor_reader_free(&app, se_reader_h, ud, cb));
            assert_eq!(res, Err(AppError::InvalidSelfEncryptorHandle.into()));

            unwrap!(call_0(|ud, cb| cipher_opt_free(&app, cipher_opt_h, ud, cb)));

            unwrap!(call_0(|ud, cb| xor_name_free(&app, name_h, ud, cb)));
        }
    }
}
