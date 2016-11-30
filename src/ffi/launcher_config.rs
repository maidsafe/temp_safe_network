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

use core::Client;
// use core::futures::FutureExt;
use ffi::FfiFuture;
// use ffi::config::{LAUNCHER_GLOBAL_CONFIG_FILE_NAME, LAUNCHER_GLOBAL_DIRECTORY_NAME};
// use ffi::errors::FfiError;
// use futures::Future;
// use maidsafe_utilities::serialisation::{deserialise, serialise};
// use nfs::{Dir, DirMetadata};
// use nfs::helper::{dir_helper, file_helper};
// use nfs::helper::writer::Mode::Overwrite;
use routing::XorName;
use rust_sodium::crypto::{box_, secretbox};
use rust_sodium::crypto::hash::sha256;
// use std::collections::HashMap;
use super::App;

pub fn app(_client: &Client,
           _app_name: String,
           _app_key: String,
           _vendor: String,
           safe_drive_access: bool)
           -> Box<FfiFuture<App>> {

    // TODO: fix `launcher_global_config_and_dir` and replace the following code
    // with the commented out one below.

    ok!(App::Registered {
        asym_enc_keys: box_::gen_keypair(),
        sym_key: secretbox::gen_key(),
        safe_drive_access: safe_drive_access,
    })


    /*
    let app_id = app_id(&app_key, &vendor);
    let c2 = client.clone();

    launcher_global_config_and_dir(&client)
        .and_then(move |(configs, _, _metadata)| {
            match configs.get(&app_id) {
                Some(config) => ok!(config.clone()),
                None => {
                    trace!("App's exclusive directory is not mapped inside Launcher's config. \
                            This must imply it's not present inside user-root-dir also - \
                            creating one.");

                    let client = c2.clone();
                    let c2 = c2.clone();

                    dir_helper::user_root_dir(c2.clone())
                        .and_then(move |(root_dir, dir_id)| {
                            let app_dir_name = app_dir_name(&app_name, &root_dir);
                            let key = Some(secretbox::gen_key());

                            dir_helper::create_sub_dir(client.clone(),
                                                       app_dir_name,
                                                       key,
                                                       Vec::new(),
                                                       &root_dir,
                                                       &dir_id)
                        })
                        .map_err(FfiError::from)
                        .and_then(move |(_, _dir, metadata)| {
                            let app = App::Registered {
                                app_dir_id: metadata.id(),
                                asym_enc_keys: box_::gen_keypair(),
                                sym_key: secretbox::gen_key(),
                                safe_drive_access: safe_drive_access,
                            };

                            upsert_to_launcher_global_config(&c2, app_id, app.clone())
                                .map(move |_| app)
                        })
                        .into_box()
                }
            }
        })
        .into_box()
    */
}

#[allow(unused)] // <-- TODO: remove this
fn app_id(app_key: &str, vendor: &str) -> XorName {
    let mut id_str = String::new();
    id_str.push_str(app_key);
    id_str.push_str(vendor);
    XorName(sha256::hash(id_str.as_bytes()).0)
}

/*
fn app_dir_name(app_name: &str, directory: &Dir) -> String {
    let mut dir_name = format!("{}-Root-Dir", app_name);
    if directory.find_sub_dir(&dir_name).is_some() {
        let mut index = 1u8;
        loop {
            dir_name = format!("{}-{}-Root-Dir", app_name, index);
            if directory.find_sub_dir(&dir_name).is_some() {
                index += 1;
            } else {
                break;
            }
        }
    }
    dir_name
}
*/

#[allow(unused)] // <-- TODO: remove this
fn upsert_to_launcher_global_config(client: &Client,
                                    app_id: XorName,
                                    config: App)
                                    -> Box<FfiFuture<()>> {
    /*
    trace!("Update (by overwriting) Launcher's config file by appending a new config.");

    let client_clone = client.clone();

    launcher_global_config_and_dir(&client)
        .map_err(FfiError::from)
        .and_then(move |(mut global_configs, dir, metadata)| {
            let _ = global_configs.insert(app_id, config);

            let file = unwrap!(dir.find_file(LAUNCHER_GLOBAL_CONFIG_FILE_NAME),
                               "Logic Error - Launcher start-up should ensure the file must be \
                                present at this stage - Report bug.")
                .clone();

            file_helper::update_content(client_clone, file, Overwrite, metadata.id(), dir)
                .map_err(FfiError::from)
                .and_then(move |writer| {
                    let ser = fry!(serialise(&global_configs));
                    writer.write(&ser)
                        .and_then(move |_| writer.close())
                        .map_err(FfiError::from)
                        .into_box()
                })
        })
        .map(|_| ())
        .into_box()
    */
    unimplemented!()
}

/*
fn launcher_global_config_and_dir(client: &Client)
                                  -> Box<FfiFuture<(HashMap<XorName, App>, Dir, DirMetadata)>> {
    trace!("Get Launcher's config directory.");

    let c2 = client.clone();

    dir_helper::configuration_dir(client.clone(), LAUNCHER_GLOBAL_DIRECTORY_NAME.to_string())
        .map_err(FfiError::from)
        .and_then(move |(dir, metadata)| {
            let file_fut = match dir.find_file(LAUNCHER_GLOBAL_CONFIG_FILE_NAME).cloned() {
                Some(file) => ok!((dir, file)),
                None => {
                    trace!("Launcher's config file does not exist inside its config dir - \
                            creating one.");

                    file_helper::create(c2.clone(),
                                        LAUNCHER_GLOBAL_CONFIG_FILE_NAME.to_string(),
                                        Vec::new(),
                                        metadata.id(),
                                        dir.clone(),
                                        false)
                        .and_then(move |writer| writer.close())
                        .map_err(FfiError::from)
                        .map(move |updated_dir| {
                            let config_file =
                                unwrap!(updated_dir.find_file(LAUNCHER_GLOBAL_CONFIG_FILE_NAME))
                                    .clone();
                            (updated_dir, config_file)
                        })
                        .into_box()
                }
            };

            file_fut.and_then(move |(dir, file)| {
                    let reader = fry!(file_helper::read(c2, &file.metadata())
                        .map_err(FfiError::from));
                    let size = reader.size();
                    if size == 0 {
                        ok!((dir, HashMap::new()))
                    } else {
                        reader.read(0, size)
                            .map_err(FfiError::from)
                            .and_then(move |data| deserialise(&data).map_err(FfiError::from))
                            .map(|global_configs| (dir, global_configs))
                            .into_box()
                    }
                })
                .map(move |(dir, global_configs)| (global_configs, dir, metadata))
                .map_err(FfiError::from)
                .into_box()
        })
        .into_box()
}
*/
