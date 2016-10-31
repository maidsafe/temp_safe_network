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

use core::{CoreError, utility};
use core::CLIENT_STRUCTURED_DATA_TAG;
use ffi::{FfiError, Session, test_utils};
use ffi::low_level_api::cipher_opt::*;
use ffi::object_cache::{AppHandle, CipherOptHandle};
use rand;
use routing::XOR_NAME_LEN;
use super::*;

#[test]
fn unversioned_struct_data_crud() {
    let (session, app_h, cipher_opt_h) = setup_session();

    // Create SD
    let id: [u8; XOR_NAME_LEN] = rand::random();
    let content_0 = unwrap!(utility::generate_random_vector::<u8>(10));

    let sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_new(&session,
                            app_h,
                            ::UNVERSIONED_STRUCT_DATA_TYPE_TAG,
                            &id,
                            0,
                            cipher_opt_h,
                            content_0.as_ptr(),
                            content_0.len(),
                            user_data,
                            cb)
        }))
    };

    let sd_data_id_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_extract_data_id(&session, sd_h, user_data, cb)
        }))
    };

    // PUT
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_put(&session, sd_h, user_data, cb)))
    }

    test_utils::run_now(&session, move |_, object_cache| {
        let sd = unwrap!(object_cache.get_sd(sd_h));
        assert_eq!(sd.get_version(), 0);
    });

    // Remove SD from the object cache.
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_free(&session, sd_h, user_data, cb)))
    }

    test_utils::run_now(&session, move |_, object_cache| {
        assert!(object_cache.get_sd(sd_h).is_err());
    });

    // GET
    let sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&session, sd_data_id_h, user_data, cb)
        }))
    };

    test_utils::run_now(&session, move |_, object_cache| {
        let _ = unwrap!(object_cache.get_sd(sd_h));
    });

    // Extract Data
    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_extract_data(&session, app_h, sd_h, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_0);

    // Update data
    let content_1 = unwrap!(utility::generate_random_vector::<u8>(10));

    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| {
            struct_data_update(&session,
                               app_h,
                               sd_h,
                               cipher_opt_h,
                               content_1.as_ptr(),
                               content_1.len(),
                               user_data,
                               cb)
        }))
    }

    // POST
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_post(&session, sd_h, user_data, cb)))
    }

    test_utils::run_now(&session, move |_, object_cache| {
        let sd = unwrap!(object_cache.get_sd(sd_h));
        assert_eq!(sd.get_version(), 1);
    });

    // Remove SD from the object cache.
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_free(&session, sd_h, user_data, cb)))
    }

    test_utils::run_now(&session, move |_, object_cache| {
        assert!(object_cache.get_sd(sd_h).is_err());
    });

    // Fetch
    let sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&session, sd_data_id_h, user_data, cb)
        }))
    };

    test_utils::run_now(&session, move |_, object_cache| {
        let _ = unwrap!(object_cache.get_sd(sd_h));
    });

    // Extract Data
    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_extract_data(&session, app_h, sd_h, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_1);

    // Perform Invalid Operations - should error out
    let result = unsafe {
        test_utils::call_1(|user_data, cb| {
            struct_data_num_of_versions(&session, sd_h, user_data, cb)
        })
    };
    match result {
        Ok(_) => panic!("Unexpected success"),
        Err(error_code) => {
            assert_eq!(error_code,
                       FfiError::from(CoreError::InvalidStructuredDataTypeTag).into())
        }
    }

    let result = unsafe {
        test_utils::call_3(|user_data, cb| {
            struct_data_nth_version(&session, app_h, sd_h, 0, user_data, cb)
        })
    };
    match result {
        Ok(_) => panic!("Unexpected success"),
        Err(error_code) => {
            assert_eq!(error_code,
                       FfiError::from(CoreError::InvalidStructuredDataTypeTag).into())
        }
    }

    // Check SD owners
    let is_owned = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_is_owned(&session, sd_h, user_data, cb)
        }))
    };
    assert!(is_owned);

    let other_session = test_utils::create_session();

    let sd_data_id = test_utils::run_now(&session, move |_, object_cache| {
        *unwrap!(object_cache.get_data_id(sd_data_id_h))
    });

    let other_sd_data_id_h = test_utils::run_now(&other_session, move |_, object_cache| {
        object_cache.insert_data_id(sd_data_id)
    });

    let other_sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&other_session, other_sd_data_id_h, user_data, cb)
        }))
    };

    let is_owned = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_is_owned(&other_session, other_sd_h, user_data, cb)
        }))
    };
    assert!(!is_owned);

    // Clone the SD to simulate Re-deletion later
    let sd_clone_h = test_utils::run_now(&session, move |_, object_cache| {
        let sd_clone = unwrap!(object_cache.get_sd(sd_h)).clone();
        object_cache.insert_sd(sd_clone)
    });

    // DELETE
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_delete(&session, sd_h, user_data, cb)))
    }

    test_utils::run_now(&session, move |_, object_cache| {
        assert!(object_cache.get_sd(sd_h).is_err());
    });

    // Re-DELETE should fail
    let result = unsafe {
        test_utils::call_0(|user_data, cb| struct_data_delete(&session, sd_clone_h, user_data, cb))
    };
    match result {
        Ok(_) => panic!("Unexpected success"),
        Err(error_code) => {
            // -26 is InvalidOperation
            assert_eq!(error_code, -26);
        }
    }

    // Fetch deleted data is OK
    let sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&session, sd_data_id_h, user_data, cb)
        }))
    };

    test_utils::run_now(&session, move |_, object_cache| {
        let sd = unwrap!(object_cache.get_sd(sd_h));
        assert!(sd.is_deleted());
        assert_eq!(sd.get_version(), 2);
    });

    // Deleted data should be empty
    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_extract_data(&session, app_h, sd_h, user_data, cb)
        }))
    };
    assert!(retrieved_content.is_empty());

    // Re-claim via PUT
    let content_2 = unwrap!(utility::generate_random_vector(10));
    unsafe {
        let sd_h = unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_new(&session,
                            app_h,
                            ::UNVERSIONED_STRUCT_DATA_TYPE_TAG,
                            &id,
                            3,
                            cipher_opt_h,
                            content_2.as_ptr(),
                            content_2.len(),
                            user_data,
                            cb)
        }));

        unwrap!(test_utils::call_0(|user_data, cb| struct_data_put(&session, sd_h, user_data, cb)));
    };
}

#[test]
fn versioned_struct_data_crud() {
    let (session, app_h, cipher_opt_h) = setup_session();

    // Create SD
    let id = rand::random();
    let content_0 = unwrap!(utility::generate_random_vector(10));

    let sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_new(&session,
                            app_h,
                            ::VERSIONED_STRUCT_DATA_TYPE_TAG,
                            &id,
                            0,
                            cipher_opt_h,
                            content_0.as_ptr(),
                            content_0.len(),
                            user_data,
                            cb)
        }))
    };

    let sd_data_id_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_extract_data_id(&session, sd_h, user_data, cb)
        }))
    };

    // PUT and Fetch
    let sd_h = unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_put(&session, sd_h, user_data, cb)));

        unwrap!(test_utils::call_0(|user_data, cb| struct_data_free(&session, sd_h, user_data, cb)));

        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&session, sd_data_id_h, user_data, cb)
        }))
    };

    // Check content
    let num_versions = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_num_of_versions(&session, sd_h, user_data, cb)
        }))
    };
    assert_eq!(num_versions, 1);

    let version = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_version(&session, sd_h, user_data, cb)
        }))
    };
    assert_eq!(version, 0);

    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_nth_version(&session, app_h, sd_h, 0, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_0);

    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_extract_data(&session, app_h, sd_h, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_0);

    // Update the content
    let content_1 = unwrap!(utility::generate_random_vector(10));
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| {
            struct_data_update(&session,
                               app_h,
                               sd_h,
                               cipher_opt_h,
                               content_1.as_ptr(),
                               content_1.len(),
                               user_data,
                               cb)
        }))
    }

    // POST and Fetch
    let sd_h = unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_post(&session, sd_h, user_data, cb)));

        unwrap!(test_utils::call_0(|user_data, cb| struct_data_free(&session, sd_h, user_data, cb)));

        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&session, sd_data_id_h, user_data, cb)
        }))
    };

    // Check content
    let num_versions = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_num_of_versions(&session, sd_h, user_data, cb)
        }))
    };
    assert_eq!(num_versions, 2);

    let version = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_version(&session, sd_h, user_data, cb)
        }))
    };
    assert_eq!(version, 1);

    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_nth_version(&session, app_h, sd_h, 0, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_0);

    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_nth_version(&session, app_h, sd_h, 1, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_1);

    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_extract_data(&session, app_h, sd_h, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_1);

    // DELETE
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_delete(&session, sd_h, user_data, cb)))
    }

    // Fetch deleted data is OK
    let sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&session, sd_data_id_h, user_data, cb)
        }))
    };

    // Deleted data should be empty
    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_extract_data(&session, app_h, sd_h, user_data, cb)
        }))
    };
    assert!(retrieved_content.is_empty());
}

#[test]
fn client_struct_data_crud() {
    let (session, app_h, cipher_opt_h) = setup_session();

    let id = rand::random();
    let content_0 = unwrap!(utility::generate_random_vector(10));

    // Create with invalid client tag
    let result = unsafe {
        test_utils::call_1(|user_data, cb| {
            struct_data_new(&session,
                            app_h,
                            CLIENT_STRUCTURED_DATA_TAG - 1,
                            &id,
                            0,
                            cipher_opt_h,
                            content_0.as_ptr(),
                            content_0.len(),
                            user_data,
                            cb)
        })
    };
    match result {
        Ok(_) => panic!("Unexpected success"),
        Err(error_code) => {
            assert_eq!(error_code,
                       FfiError::from(CoreError::InvalidStructuredDataTypeTag).into())
        }
    }

    // Create
    let sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_new(&session,
                            app_h,
                            CLIENT_STRUCTURED_DATA_TAG + 1,
                            &id,
                            0,
                            cipher_opt_h,
                            content_0.as_ptr(),
                            content_0.len(),
                            user_data,
                            cb)
        }))
    };

    let sd_data_id_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_extract_data_id(&session, sd_h, user_data, cb)
        }))
    };

    // PUT and Fetch
    let sd_h = unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_put(&session, sd_h, user_data, cb)));

        unwrap!(test_utils::call_0(|user_data, cb| struct_data_free(&session, sd_h, user_data, cb)));

        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&session, sd_data_id_h, user_data, cb)
        }))
    };

    // Check content
    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_extract_data(&session, app_h, sd_h, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_0);

    // Update the content
    let content_1 = unwrap!(utility::generate_random_vector(10));
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| {
            struct_data_update(&session,
                               app_h,
                               sd_h,
                               cipher_opt_h,
                               content_1.as_ptr(),
                               content_1.len(),
                               user_data,
                               cb)
        }))
    }

    // POST and Fetch
    let sd_h = unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_post(&session, sd_h, user_data, cb)));

        unwrap!(test_utils::call_0(|user_data, cb| struct_data_free(&session, sd_h, user_data, cb)));

        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_fetch(&session, sd_data_id_h, user_data, cb)
        }))
    };

    // Check content
    let retrieved_content = unsafe {
        unwrap!(test_utils::call_vec_u8(|user_data, cb| {
            struct_data_extract_data(&session, app_h, sd_h, user_data, cb)
        }))
    };
    assert_eq!(retrieved_content, content_1);

    // Invalid operations
    let result = unsafe {
        test_utils::call_1(|user_data, cb| {
            struct_data_num_of_versions(&session, sd_h, user_data, cb)
        })
    };
    match result {
        Ok(_) => panic!("Unexpected success"),
        Err(error_code) => {
            assert_eq!(error_code,
                       FfiError::from(CoreError::InvalidStructuredDataTypeTag).into())
        }
    }

    // DELETE
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_delete(&session, sd_h, user_data, cb)))
    }
}

#[test]
fn reclaim() {
    let (session, app_h, cipher_opt_h) = setup_session();

    // Create SD
    let id: [u8; XOR_NAME_LEN] = rand::random();
    let content_0 = unwrap!(utility::generate_random_vector(10));

    let sd_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| {
            struct_data_new(&session,
                            app_h,
                            ::UNVERSIONED_STRUCT_DATA_TYPE_TAG,
                            &id,
                            0,
                            cipher_opt_h,
                            content_0.as_ptr(),
                            content_0.len(),
                            user_data,
                            cb)
        }))
    };

    let sd_clone_h = test_utils::run_now(&session, move |_, object_cache| {
        let sd_clone = unwrap!(object_cache.get_sd(sd_h)).clone();
        object_cache.insert_sd(sd_clone)
    });

    // PUT the original SD
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_put(&session, sd_h, user_data, cb)))
    }

    test_utils::run_now(&session, move |_, object_cache| {
        let sd = unwrap!(object_cache.get_sd(sd_h));
        assert_eq!(sd.get_version(), 0);
    });

    // DELETE it
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| struct_data_delete(&session, sd_h, user_data, cb)))
    }

    // Now PUT the cloned one. This should reclaim the deleted data and
    // update the version number properly.
    unsafe {
        unwrap!(test_utils::call_0(|user_data, cb| {
            struct_data_put(&session, sd_clone_h, user_data, cb)
        }))
    }

    test_utils::run_now(&session, move |_, object_cache| {
        let sd = unwrap!(object_cache.get_sd(sd_clone_h));
        assert_eq!(sd.get_version(), 2);
    });
}

fn setup_session() -> (Session, AppHandle, CipherOptHandle) {
    let session = test_utils::create_session();

    let app = test_utils::create_app(&session, false);
    let app_h = test_utils::run_now(&session,
                                    move |_, object_cache| object_cache.insert_app(app));

    // Create cipher opt.
    let cipher_opt_h = unsafe {
        unwrap!(test_utils::call_1(|user_data, cb| cipher_opt_new_symmetric(&session, user_data, cb)))
    };

    (session, app_h, cipher_opt_h)
}
