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

use core::{CoreError, CoreMsg};
use ffi::{App, FfiError, OpaqueCtx, Session, helper};
use ffi::object_cache::{CipherOptHandle, EncryptKeyHandle};
use libc::{c_void, int32_t};
use maidsafe_utilities::serialisation::{deserialise, serialise};
use rust_sodium::crypto::{box_, sealedbox, secretbox};

/// Cipher Options
#[derive(Debug)]
pub enum CipherOpt {
    /// No encryption
    PlainText,
    /// Encrypt using symmetric keys (usually for private data)
    Symmetric,
    /// Encrypt using asymmetric encryption (encrypting for peer to read)
    Asymmetric {
        /// PublicKey of the peer to whom we want to encrypt
        peer_encrypt_key: box_::PublicKey,
    },
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
enum WireFormat {
    Plain(Vec<u8>),
    Symmetric {
        nonce: secretbox::Nonce,
        cipher_text: Vec<u8>,
    },
    Asymmetric(Vec<u8>),
}

impl CipherOpt {
    /// Encrypt plain text
    pub fn encrypt(&self, app: &App, plain_text: &[u8]) -> Result<Vec<u8>, FfiError> {
        match *self {
            CipherOpt::PlainText => Ok(try!(serialise(&WireFormat::Plain(plain_text.to_owned())))),
            CipherOpt::Symmetric => {
                let sym_key = try!(app.sym_key());
                let nonce = secretbox::gen_nonce();
                let cipher_text = secretbox::seal(plain_text, &nonce, &sym_key);
                let wire_format = WireFormat::Symmetric {
                    nonce: nonce,
                    cipher_text: cipher_text,
                };

                Ok(try!(serialise(&wire_format)))
            }
            CipherOpt::Asymmetric { ref peer_encrypt_key } => {
                let cipher_text = sealedbox::seal(plain_text, peer_encrypt_key);
                Ok(try!(serialise(&WireFormat::Asymmetric(cipher_text))))
            }
        }
    }

    /// Decrypt something encrypted by CipherOpt::encrypt()
    pub fn decrypt(app: &App, raw_data: &[u8]) -> Result<Vec<u8>, FfiError> {
        match try!(deserialise::<WireFormat>(raw_data)) {
            WireFormat::Plain(plain_text) => Ok(plain_text),
            WireFormat::Symmetric { nonce, cipher_text } => {
                let sym_key = try!(app.sym_key());
                Ok(try!(secretbox::open(&cipher_text, &nonce, &sym_key)
                    .map_err(|()| CoreError::SymmetricDecipherFailure)))
            }
            WireFormat::Asymmetric(cipher_text) => {
                let (pk, sk) = try!(app.asym_enc_keys());
                Ok(try!(sealedbox::open(&cipher_text, &pk, &sk)
                    .map_err(|()| CoreError::SymmetricDecipherFailure)))
            }
        }
    }
}

/// Construct CipherOpt::PlainText handle
#[no_mangle]
pub unsafe extern "C" fn cipher_opt_new_plaintext(session: *const Session,
                                                  user_data: *mut c_void,
                                                  o_cb: unsafe extern "C" fn(*mut c_void,
                                                                             int32_t,
                                                                             CipherOptHandle))
                                                  -> i32 {
    helper::catch_unwind_i32(|| {
        let user_data = OpaqueCtx(user_data);
        let obj_cache = (*session).object_cache();

        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let handle = unwrap!(obj_cache.lock()).insert_cipher_opt(CipherOpt::PlainText);
            o_cb(user_data.0, 0, handle);
            None
        })));

        0
    })
}

/// Construct CipherOpt::Symmetric handle
#[no_mangle]
pub unsafe extern "C" fn cipher_opt_new_symmetric(session: *const Session,
                                                  user_data: *mut c_void,
                                                  o_cb: unsafe extern "C" fn(*mut c_void,
                                                                             int32_t,
                                                                             CipherOptHandle))
                                                  -> i32 {
    helper::catch_unwind_i32(|| {
        let user_data = OpaqueCtx(user_data);
        let obj_cache = (*session).object_cache();

        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let handle = unwrap!(obj_cache.lock()).insert_cipher_opt(CipherOpt::Symmetric);
            o_cb(user_data.0, 0, handle);
            None
        })));

        0
    })
}

/// Construct CipherOpt::Asymmetric handle
#[no_mangle]
pub unsafe extern "C" fn cipher_opt_new_asymmetric(session: *const Session,
                                                   peer_encrypt_key_h: EncryptKeyHandle,
                                                   user_data: *mut c_void,
                                                   o_cb: unsafe extern "C" fn(*mut c_void,
                                                                              int32_t,
                                                                              CipherOptHandle))
                                                   -> i32 {
    helper::catch_unwind_i32(|| {
        let user_data = OpaqueCtx(user_data);
        let obj_cache = (*session).object_cache();

        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let pk = match unwrap!(obj_cache.lock()).get_encrypt_key(peer_encrypt_key_h) {
                Ok(pk) => *pk,
                Err(e) => {
                    o_cb(user_data.0, ffi_error_code!(e), 0);
                    return None;
                }
            };
            let handle = unwrap!(obj_cache.lock())
                .insert_cipher_opt(CipherOpt::Asymmetric { peer_encrypt_key: pk });
            o_cb(user_data.0, 0, handle);
            None
        })));
        0
    })
}

/// Free CipherOpt handle
#[no_mangle]
pub unsafe extern "C" fn cipher_opt_free(session: *const Session,
                                         handle: CipherOptHandle,
                                         user_data: *mut c_void,
                                         o_cb: unsafe extern "C" fn(*mut c_void, int32_t))
                                         -> i32 {
    helper::catch_unwind_i32(|| {
        let user_data = OpaqueCtx(user_data);
        let obj_cache = (*session).object_cache();

        ffi_try!((*session).send(CoreMsg::new(move |_| {
            let res = unwrap!(obj_cache.lock()).remove_cipher_opt(handle);
            o_cb(user_data.0, ffi_result_code!(res));
            None
        })));

        0
    })
}

#[cfg(test)]
mod tests {
    use core::utility;
    use ffi::{App, FfiError, Session};
    use ffi::object_cache::CipherOptHandle;
    use ffi::test_utils;
    use libc::c_void;
    use rust_sodium::crypto::box_;
    use std::sync::mpsc;
    use super::*;

    fn decrypt_and_check(app: &App, raw_data: &[u8], orig_plain_text: &[u8]) -> bool {
        let plain_text_rx = match CipherOpt::decrypt(&app, &raw_data) {
            Ok(pt) => pt,
            Err(_) => return false,
        };
        orig_plain_text == &plain_text_rx[..]
    }

    #[test]
    fn app_0_to_app_0_plain() {
        let sess = test_utils::create_session();
        let obj_cache = sess.object_cache();
        let app_0 = test_utils::create_app(&sess, false);
        let sess_ptr = Box::into_raw(Box::new(sess));

        let (tx, rx) = mpsc::channel::<Result<CipherOptHandle, i32>>();
        let tx = Box::into_raw(Box::new(tx.clone())) as *mut c_void;

        let plain_text = unwrap!(utility::generate_random_vector::<u8>(10));
        let cipher_opt_handle: CipherOptHandle;
        unsafe {
            assert_eq!(cipher_opt_new_plaintext(sess_ptr, tx, handle_cb), 0);
            cipher_opt_handle = unwrap!(unwrap!(rx.recv()));
        }
        let raw_data = {
            let mut obj_cache = unwrap!(obj_cache.lock());
            let cipher_opt = unwrap!(obj_cache.get_cipher_opt(cipher_opt_handle));
            unwrap!(cipher_opt.encrypt(&app_0, &plain_text))
        };
        assert_free(sess_ptr, cipher_opt_handle, 0);

        assert!(unwrap!(obj_cache.lock()).get_cipher_opt(cipher_opt_handle).is_err());
        assert!(raw_data != plain_text);

        assert!(decrypt_and_check(&app_0, &raw_data, &plain_text));
    }

    #[test]
    fn app_0_to_app_0_sym() {
        let sess = test_utils::create_session();
        let obj_cache = sess.object_cache();
        let app_0 = test_utils::create_app(&sess, false);
        let sess_ptr = Box::into_raw(Box::new(sess));

        let (tx, rx) = mpsc::channel::<Result<CipherOptHandle, i32>>();
        let tx = Box::into_raw(Box::new(tx.clone())) as *mut c_void;

        let plain_text = unwrap!(utility::generate_random_vector::<u8>(10));
        let cipher_opt_handle: CipherOptHandle;
        unsafe {
            assert_eq!(cipher_opt_new_symmetric(sess_ptr, tx, handle_cb), 0);
            cipher_opt_handle = unwrap!(unwrap!(rx.recv()));
        }
        let raw_data = {
            let mut obj_cache = unwrap!(obj_cache.lock());
            let cipher_opt = unwrap!(obj_cache.get_cipher_opt(cipher_opt_handle));
            unwrap!(cipher_opt.encrypt(&app_0, &plain_text))
        };
        assert_free(sess_ptr, cipher_opt_handle, 0);

        assert!(unwrap!(obj_cache.lock()).get_cipher_opt(cipher_opt_handle).is_err());
        assert!(raw_data != plain_text);

        assert!(decrypt_and_check(&app_0, &raw_data, &plain_text));
    }

    #[test]
    fn app_0_to_app_1_asym() {
        // Setup
        let sess = test_utils::create_session();
        let app_0 = test_utils::create_app(&sess, false);
        let app_1 = test_utils::create_app(&sess, false);

        let obj_cache = sess.object_cache();
        let sess_ptr = Box::into_raw(Box::new(sess));

        let app_1_encrypt_key_handle = {
            let app_1_pub_encrypt_key = unwrap!(app_1.asym_enc_keys()).0;
            unwrap!(obj_cache.lock()).insert_encrypt_key(app_1_pub_encrypt_key)
        };

        let (tx, rx) = mpsc::channel::<Result<CipherOptHandle, i32>>();
        let tx = Box::into_raw(Box::new(tx.clone())) as *mut c_void;

        // Test
        // At app-0 end
        let plain_text = unwrap!(utility::generate_random_vector::<u8>(10));
        let cipher_opt_handle: CipherOptHandle;
        unsafe {
            assert_eq!(cipher_opt_new_asymmetric(sess_ptr,
                                                 app_1_encrypt_key_handle,
                                                 tx,
                                                 handle_cb),
                       0);
            cipher_opt_handle = unwrap!(unwrap!(rx.recv()));
        }

        let raw_data = {
            let mut obj_cache = unwrap!(obj_cache.lock());
            let cipher_opt = unwrap!(obj_cache.get_cipher_opt(cipher_opt_handle));
            unwrap!(cipher_opt.encrypt(&app_0, &plain_text))
        };
        assert_free(sess_ptr, cipher_opt_handle, 0);

        assert!(unwrap!(obj_cache.lock()).get_cipher_opt(cipher_opt_handle).is_err());
        assert!(raw_data != plain_text);

        assert!(!decrypt_and_check(&app_0, &raw_data, &plain_text));
        assert!(decrypt_and_check(&app_1, &raw_data, &plain_text));
    }

    #[test]
    fn create_and_free() {
        let sess = test_utils::create_session();
        let obj_cache = sess.object_cache();
        let sess_ptr = Box::into_raw(Box::new(sess));

        let peer_encrypt_key_handle = {
            let (pk, _) = box_::gen_keypair();
            unwrap!(obj_cache.lock()).insert_encrypt_key(pk)
        };

        let (tx, rx) = mpsc::channel::<Result<CipherOptHandle, i32>>();
        let tx = Box::into_raw(Box::new(tx.clone())) as *mut c_void;

        let cipher_opt_handle_pt;
        let cipher_opt_handle_sym;
        let cipher_opt_handle_asym;

        unsafe {
            assert_eq!(cipher_opt_new_plaintext(sess_ptr, tx, handle_cb), 0);
            cipher_opt_handle_pt = unwrap!(unwrap!(rx.recv()));

            assert_eq!(cipher_opt_new_symmetric(sess_ptr, tx, handle_cb), 0);
            cipher_opt_handle_sym = unwrap!(unwrap!(rx.recv()));

            let err_code = FfiError::InvalidEncryptKeyHandle.into();
            assert_eq!(cipher_opt_new_asymmetric(sess_ptr, 29293290, tx, handle_cb),
                       0);
            let res = unwrap!(rx.recv());
            assert!(res.is_err());
            assert_eq!(unwrap!(res.err()), err_code);

            assert_eq!(cipher_opt_new_asymmetric(sess_ptr, peer_encrypt_key_handle, tx, handle_cb),
                       0);
            cipher_opt_handle_asym = unwrap!(unwrap!(rx.recv()));
        }

        {
            let mut obj_cache = unwrap!(obj_cache.lock());
            let _ = unwrap!(obj_cache.get_cipher_opt(cipher_opt_handle_pt));
            let _ = unwrap!(obj_cache.get_cipher_opt(cipher_opt_handle_sym));
            let _ = unwrap!(obj_cache.get_cipher_opt(cipher_opt_handle_asym));
        }

        assert_free(sess_ptr, cipher_opt_handle_pt, 0);
        assert_free(sess_ptr, cipher_opt_handle_sym, 0);
        assert_free(sess_ptr, cipher_opt_handle_asym, 0);

        let err_code = FfiError::InvalidCipherOptHandle.into();
        assert_free(sess_ptr, cipher_opt_handle_pt, err_code);
        assert_free(sess_ptr, cipher_opt_handle_sym, err_code);
        assert_free(sess_ptr, cipher_opt_handle_asym, err_code);

        {
            let mut obj_cache = unwrap!(obj_cache.lock());
            assert!(obj_cache.get_cipher_opt(cipher_opt_handle_pt).is_err());
            assert!(obj_cache.get_cipher_opt(cipher_opt_handle_sym).is_err());
            assert!(obj_cache.get_cipher_opt(cipher_opt_handle_asym).is_err());
        }
    }

    fn assert_free(sess_ptr: *const Session, cipher_opt_handle: CipherOptHandle, expected: i32) {
        let (tx, rx) = mpsc::channel::<i32>();
        unsafe {
            assert_eq!(cipher_opt_free(sess_ptr,
                                       cipher_opt_handle,
                                       Box::into_raw(Box::new(tx)) as *mut _,
                                       free_cb),
                       0);
        }
        let err_code = unwrap!(rx.recv());
        assert_eq!(err_code, expected);
    }

    unsafe extern "C" fn free_cb(tx: *mut c_void, error_code: i32) {
        let tx = tx as *mut mpsc::Sender<i32>;
        unwrap!((*tx).send(error_code));
    }

    unsafe extern "C" fn handle_cb(tx: *mut c_void, error_code: i32, handle: CipherOptHandle) {
        let tx = tx as *mut mpsc::Sender<Result<CipherOptHandle, i32>>;
        let res = if error_code == 0 {
            Ok(handle)
        } else {
            Err(error_code)
        };
        unwrap!((*tx).send(res));
    }
}
