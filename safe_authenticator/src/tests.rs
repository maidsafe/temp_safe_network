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

use Authenticator;
use access_container::access_container_key;
use errors::{AuthError, ERR_ALREADY_AUTHORISED};
use ffi_utils::{FfiString, base64_encode};
use ffi_utils::test_utils::{call_1, send_via_user_data, sender_as_user_data};
use futures::{Future, future};
use ipc::{encode_auth_resp, get_config};
use maidsafe_utilities::serialisation::deserialise;
use routing::{Action, PermissionSet, User};
use rust_sodium::crypto::hash::sha256;
use safe_core::{MDataInfo, mdata_info, utils};
use safe_core::ipc::{self, AppExchangeInfo, AuthReq, ContainersReq, IpcMsg, IpcReq, IpcResp,
                     Permission};
use safe_core::ipc::req::ffi::AuthReq as FfiAuthReq;
use safe_core::ipc::req::ffi::ContainersReq as FfiContainersReq;
use safe_core::nfs::{DEFAULT_PRIVATE_DIRS, DEFAULT_PUBLIC_DIRS};
use std::collections::{BTreeSet, HashMap};
use std::os::raw::c_void;
use std::sync::mpsc;
use test_utils::run;

// Test creation and content of std dirs after account creation.
#[test]
fn user_root_dir() {
    let authenticator = create_account_and_login();
    let std_dir_names: Vec<_> =
        DEFAULT_PRIVATE_DIRS.iter().chain(DEFAULT_PUBLIC_DIRS.iter()).collect();

    // Fetch the entries of the user root dir.
    let (dir, entries) = run(&authenticator, |client| {
        let dir = unwrap!(client.user_root_dir());
        client.list_mdata_entries(dir.name, dir.type_tag)
            .map(move |entries| (dir, entries))
            .map_err(AuthError::from)
    });

    let entries = unwrap!(mdata_info::decrypt_entries(&dir, &entries));

    // Verify that all the std dirs are there.
    for name in &std_dir_names {
        assert!(entries.contains_key(name.as_bytes()));
    }

    // Fetch all the dirs under user root dir and verify they are empty.
    let dirs: Vec<_> = entries.into_iter()
        .map(|(_, value)| unwrap!(deserialise::<MDataInfo>(&value.content)))
        .collect();

    let dirs = run(&authenticator, move |client| {
        let fs: Vec<_> = dirs.into_iter()
            .map(|dir| {
                let f1 = client.list_mdata_entries(dir.name, dir.type_tag);
                let f2 = client.list_mdata_permissions(dir.name, dir.type_tag);

                f1.join(f2).map_err(AuthError::from)
            })
            .collect();

        future::join_all(fs)
    });

    assert_eq!(dirs.len(), std_dir_names.len());

    for (entries, permissions) in dirs {
        assert!(entries.is_empty());
        assert!(permissions.is_empty());
    }
}

// Test creation and content of config dir after account creation.
#[test]
fn config_root_dir() {
    let authenticator = create_account_and_login();

    // Fetch the entries of the config root dir.
    let (dir, entries) = run(&authenticator, |client| {
        let dir = unwrap!(client.config_root_dir());
        client.list_mdata_entries(dir.name, dir.type_tag)
            .map(move |entries| (dir, entries))
            .map_err(AuthError::from)
    });

    let entries = unwrap!(mdata_info::decrypt_entries(&dir, &entries));

    // Verify it contains the required entries.
    let config = unwrap!(entries.get(&b"authenticator-config"[..]));
    assert!(config.content.is_empty());

    let ac = unwrap!(entries.get(&b"access-container"[..]));
    let ac: MDataInfo = unwrap!(deserialise(&ac.content));

    // Fetch access container and verify it's empty.
    let (entries, permissions) = run(&authenticator, move |client| {
        let f1 = client.list_mdata_entries(ac.name, ac.type_tag);
        let f2 = client.list_mdata_permissions(ac.name, ac.type_tag);

        f1.join(f2).map_err(AuthError::from)
    });

    assert!(entries.is_empty());
    assert!(permissions.is_empty());
}

// Test app authentication.
#[test]
fn app_authentication() {
    let authenticator = create_account_and_login();

    let req_id = ipc::gen_req_id();
    let app_id = unwrap!(utils::generate_random_string(10));
    let app_exchange_info = AppExchangeInfo {
        id: app_id.clone(),
        scope: None,
        name: unwrap!(utils::generate_random_string(10)),
        vendor: unwrap!(utils::generate_random_string(10)),
    };

    let auth_req = {
        let mut containers = HashMap::new();
        let _ = containers.insert("_documents".to_string(), btree_set![Permission::Insert]);
        let _ = containers.insert("_videos".to_string(),
                                  btree_set![Permission::Read,
                                             Permission::Insert,
                                             Permission::Update,
                                             Permission::Delete,
                                             Permission::ManagePermissions]);

        AuthReq {
            app: app_exchange_info.clone(),
            app_container: true,
            containers: containers,
        }
    };

    let msg = IpcMsg::Req {
        req_id: req_id,
        req: IpcReq::Auth(auth_req.clone()),
    };

    let encoded_msg = unwrap!(ipc::encode_msg(&msg, "safe-auth"));

    let (received_req_id, received_auth_req) = match unwrap!(decode_ipc_msg(&authenticator,
                                                                            &encoded_msg)) {
        IpcMsg::Req { req_id, req: IpcReq::Auth(req) } => (req_id, req),
        x => panic!("Unexpected {:?}", x),
    };

    assert_eq!(received_req_id, req_id);
    assert_eq!(received_auth_req, auth_req);

    let encoded_auth_resp = unsafe {
        let ffi_resp = unwrap!(call_1(|ud, cb| {
            encode_auth_resp(&authenticator,
                             auth_req.into_repr_c(),
                             req_id,
                             true, // is_granted
                             ud,
                             cb)
        }));

        let resp = unwrap!(ffi_resp.to_string());
        ffi_resp.deallocate();
        resp
    };

    let base64_app_id = base64_encode(app_id.as_bytes());
    assert!(encoded_auth_resp.starts_with(&format!("safe-{}", base64_app_id)));

    let auth_granted = match unwrap!(ipc::decode_msg(&encoded_auth_resp)) {
        IpcMsg::Resp { req_id: received_req_id, resp: IpcResp::Auth(Ok(auth_granted)) } => {
            assert_eq!(received_req_id, req_id);
            auth_granted
        }
        x => panic!("Unexpected {:?}", x),
    };

    let ac_info = auth_granted.access_container;
    let app_keys = auth_granted.app_keys;
    let app_sign_pk = app_keys.sign_pk;
    let app_enc_key = app_keys.enc_key.clone();

    // Fetch the access container entry for the app.
    let ac_app_entry_key = access_container_key(&app_id, &app_keys, &ac_info.nonce);
    let mut access_container = {
        let encrypted = run(&authenticator, move |client| {
            client.get_mdata_value(ac_info.id, ac_info.tag, ac_app_entry_key)
                .map(|value| value.content)
                .map_err(AuthError::from)
        });

        let encoded = unwrap!(utils::symmetric_decrypt(&encrypted, &app_enc_key));
        unwrap!(deserialise::<HashMap<String, (MDataInfo, BTreeSet<Permission>)>>(&encoded))
    };

    assert_eq!(access_container.len(), 3);
    let (documents_info, documents_permissions) = unwrap!(access_container.remove("_documents"));
    let (videos_info, videos_permissions) = unwrap!(access_container.remove("_videos"));
    let (app_dir_info, app_dir_permissions) = unwrap!(access_container.remove(&app_id));

    // Check the requested permissions in the access container.
    let all = btree_set![Permission::Read,
                         Permission::Insert,
                         Permission::Update,
                         Permission::Delete,
                         Permission::ManagePermissions];
    assert_eq!(documents_permissions, btree_set![Permission::Insert]);
    assert_eq!(videos_permissions, all);
    assert_eq!(app_dir_permissions, all);

    // Check the permission on the the mutable data for each of the above directories.
    let (documents_permissions, videos_permissions, app_dir_permissions) = {
        let app_dir_info = app_dir_info.clone();

        run(&authenticator, move |client| {
            let user = User::Key(app_sign_pk);
            let documents = client.list_mdata_user_permissions(documents_info.name,
                                                               documents_info.type_tag,
                                                               user.clone());
            let videos =
                client.list_mdata_user_permissions(videos_info.name,
                                                   videos_info.type_tag,
                                                   user.clone());
            let app_dir =
                client.list_mdata_user_permissions(app_dir_info.name, app_dir_info.type_tag, user);

            documents.join3(videos, app_dir).map_err(AuthError::from)
        })
    };

    let all = PermissionSet::new()
        .allow(Action::Insert)
        .allow(Action::Update)
        .allow(Action::Delete)
        .allow(Action::ManagePermissions);
    assert_eq!(documents_permissions,
               PermissionSet::new().allow(Action::Insert));
    assert_eq!(videos_permissions, all);
    assert_eq!(app_dir_permissions, all);

    // Check the app info is present in the config file.
    let config = run(&authenticator,
                     |client| get_config(client).map(|(_, config)| config));

    let app_config_key = sha256::hash(app_id.as_bytes());
    let app_info = unwrap!(config.get(&app_config_key));

    assert_eq!(app_info.info, app_exchange_info);
    assert_eq!(app_info.keys, app_keys);

    // Check there app dir is present in the user root.
    let received_app_dir_info = run(&authenticator, move |client| {
        let user_root_dir = unwrap!(client.user_root_dir());

        let app_dir_key = format!("apps/{}", app_id).into_bytes();
        let app_dir_key = unwrap!(user_root_dir.enc_entry_key(&app_dir_key));

        client.get_mdata_value(user_root_dir.name, user_root_dir.type_tag, app_dir_key)
            .and_then(move |value| {
                let encoded = user_root_dir.decrypt(&value.content)?;
                let decoded = deserialise::<MDataInfo>(&encoded)?;
                Ok(decoded)
            })
            .map_err(AuthError::from)
    });

    assert_eq!(received_app_dir_info, app_dir_info);

    // Check the app is authorised.
    let auth_keys = run(&authenticator, |client| {
        client.list_auth_keys_and_version().map(|(keys, _)| keys).map_err(AuthError::from)
    });

    assert!(auth_keys.contains(&app_sign_pk));
}

#[test]
fn authenticated_app_cannot_be_authenticated_again() {
    let authenticator = create_account_and_login();

    let app_id = unwrap!(utils::generate_random_string(10));
    let app_exchange_info = AppExchangeInfo {
        id: app_id.clone(),
        scope: None,
        name: unwrap!(utils::generate_random_string(10)),
        vendor: unwrap!(utils::generate_random_string(10)),
    };

    let auth_req = AuthReq {
        app: app_exchange_info.clone(),
        app_container: false,
        containers: Default::default(),
    };

    let req_id = ipc::gen_req_id();
    let msg = IpcMsg::Req {
        req_id: req_id,
        req: IpcReq::Auth(auth_req.clone()),
    };
    let encoded_msg = unwrap!(ipc::encode_msg(&msg, "safe-auth"));

    match unwrap!(decode_ipc_msg(&authenticator, &encoded_msg)) {
        IpcMsg::Req { req: IpcReq::Auth(_), .. } => (),
        x => panic!("Unexpected {:?}", x),
    };

    unsafe {
        let resp = unwrap!(call_1(|ud, cb| {
            encode_auth_resp(&authenticator,
                             auth_req.clone().into_repr_c(),
                             req_id,
                             true, // is_granted
                             ud,
                             cb)
        }));

        resp.deallocate();
    };

    // Second authentication fails.
    let req_id = ipc::gen_req_id();
    let msg = IpcMsg::Req {
        req_id: req_id,
        req: IpcReq::Auth(auth_req),
    };
    let encoded_msg = unwrap!(ipc::encode_msg(&msg, "safe-auth"));

    match decode_ipc_msg(&authenticator, &encoded_msg) {
        Err(code) if code == ERR_ALREADY_AUTHORISED => (),
        x => panic!("Unexpected {:?}", x),
    };
}

fn create_account_and_login() -> Authenticator {
    let locator = unwrap!(utils::generate_random_string(10));
    let password = unwrap!(utils::generate_random_string(10));

    let _ = unwrap!(Authenticator::create_acc(locator.clone(), password.clone(), |_| ()));
    unwrap!(Authenticator::login(locator, password, |_| ()))
}

// Helper to decode IpcMsg.
// TODO: there should be a public function with a signature like this, and the
//       FFI function `ipc::decode_ipc_msg` should be only wrapper over it.
fn decode_ipc_msg(authenticator: &Authenticator, msg: &str) -> Result<IpcMsg, i32> {
    let (tx, rx) = mpsc::channel::<Result<IpcMsg, i32>>();

    extern "C" fn auth_cb(user_data: *mut c_void, req_id: u32, req: FfiAuthReq) {
        unsafe {
            let req = unwrap!(AuthReq::from_repr_c(req));
            let msg = IpcMsg::Req {
                req_id: req_id,
                req: IpcReq::Auth(req),
            };

            send_via_user_data(user_data, Ok::<_, i32>(msg))
        }
    }

    extern "C" fn containers_cb(user_data: *mut c_void, req_id: u32, req: FfiContainersReq) {
        unsafe {
            let req = unwrap!(ContainersReq::from_repr_c(req));
            let msg = IpcMsg::Req {
                req_id: req_id,
                req: IpcReq::Containers(req),
            };

            send_via_user_data(user_data, Ok::<_, i32>(msg))
        }
    }

    extern "C" fn err_cb(user_data: *mut c_void, error_code: i32, _: FfiString) {
        unsafe { send_via_user_data(user_data, Err::<IpcMsg, _>(error_code)) }
    }

    let ffi_msg = FfiString::from_str(msg);

    unsafe {
        use ipc::decode_ipc_msg;
        decode_ipc_msg(authenticator,
                       ffi_msg,
                       sender_as_user_data(&tx),
                       auth_cb,
                       containers_cb,
                       err_cb);
    };

    unwrap!(rx.recv())
}
