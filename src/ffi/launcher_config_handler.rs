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


use core::Client;
use core::futures::FutureExt;
use ffi::FfiFuture;
use ffi::config::{LAUNCHER_GLOBAL_CONFIG_FILE_NAME, LAUNCHER_GLOBAL_DIRECTORY_NAME};
use ffi::errors::FfiError;
use futures::Future;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use nfs::{AccessLevel, Dir, DirMetadata};
use nfs::helper::{dir_helper, file_helper};
use nfs::helper::writer::Mode::Overwrite;
use routing::XorName;
use rust_sodium::crypto::{box_, secretbox};
use rust_sodium::crypto::hash::sha256;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct LauncherConfiguration {
    pub app_id: XorName,
    pub app_info: AppInfo,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct AppInfo {
    pub app_root_dir: Dir,
    pub asym_keys: (box_::PublicKey, box_::SecretKey),
    pub sym_key: secretbox::Key,
}

pub fn app_info(client: Client,
                app_name: String,
                app_key: String,
                vendor: String)
                -> Box<FfiFuture<AppInfo>> {
    let app_id = app_id(&app_key, &vendor);
    let c2 = client.clone();

    launcher_global_config_and_dir(&client)
        .and_then(move |(configs, _, metadata)| {
            match configs.into_iter()
                .find(|config| config.app_id == app_id)
                .map(|config| config) {
                Some(config) => ok!(config.app_info),
                None => {
                    trace!("App's exclusive directory is not mapped inside Launcher's config. \
                            This must imply it's not present inside user-root-dir also - \
                            creating one.");

                    let client = c2.clone();
                    let c2 = c2.clone();

                    dir_helper::user_root_dir(c2.clone())
                        .and_then(move |root_dir| {
                            let app_dir_name = app_dir_name(&app_name, &root_dir);

                            dir_helper::create_sub_dir(client.clone(),
                                                       app_dir_name,
                                                       None,
                                                       Vec::new(),
                                                       &root_dir,
                                                       &metadata.id())
                        })
                        .map_err(FfiError::from)
                        .and_then(move |(_, dir, metadata)| {
                            let app_info = AppInfo {
                                app_root_dir: dir,
                                asym_keys: box_::gen_keypair(),
                                sym_key: secretbox::gen_key(),
                            };

                            let app_config = LauncherConfiguration {
                                app_id: app_id,
                                app_info: app_info.clone(),
                            };

                            upsert_to_launcher_global_config(&c2, app_config).map(move |_| {
                                app_info
                            })
                        })
                        .into_box()
                }
            }
        })
        .into_box()
}

fn app_id(app_key: &str, vendor: &str) -> XorName {
    let mut id_str = String::new();
    id_str.push_str(app_key);
    id_str.push_str(vendor);
    XorName(sha256::hash(id_str.as_bytes()).0)
}

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

fn upsert_to_launcher_global_config(client: &Client,
                                    config: LauncherConfiguration)
                                    -> Box<FfiFuture<()>> {
    trace!("Update (by overwriting) Launcher's config file by appending a new config.");

    let client_clone = client.clone();

    launcher_global_config_and_dir(&client)
        .map_err(FfiError::from)
        .and_then(move |(mut global_configs, dir, metadata)| {
            // (Spandan)
            // Unable to use `if let Some() .. else` logic to upsert to a vector due to
            // a language bug. Once the bug is resolved
            // - https://github.com/rust-lang/rust/issues/28449
            // then modify the following to use it.
            if let Some(pos) = global_configs.iter()
                .position(|existing_config| existing_config.app_id == config.app_id) {
                let existing_config = unwrap!(global_configs.get_mut(pos));
                *existing_config = config;
            } else {
                global_configs.push(config);
            }

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
}

fn launcher_global_config_and_dir
    (client: &Client)
     -> Box<FfiFuture<(Vec<LauncherConfiguration>, Dir, DirMetadata)>> {
    trace!("Get Launcher's config directory.");

    let c2 = client.clone();

    dir_helper::configuration_dir(client.clone(), LAUNCHER_GLOBAL_DIRECTORY_NAME.to_string())
        .map_err(FfiError::from)
        .and_then(move |(dir, metadata)| {
            let file_fut = match dir.find_file(LAUNCHER_GLOBAL_CONFIG_FILE_NAME).cloned() {
                Some(file) => ok!(file),
                None => {
                    trace!("Launcher's config file does not exist inside its config dir - \
                            creating one.");

                    file_helper::create(c2.clone(),
                                        LAUNCHER_GLOBAL_CONFIG_FILE_NAME.to_string(),
                                        Vec::new(),
                                        metadata.id(),
                                        dir.clone())
                        .and_then(move |writer| writer.close())
                        .map_err(FfiError::from)
                        .map(move |updated_dir| {
                            unwrap!(updated_dir.find_file(LAUNCHER_GLOBAL_CONFIG_FILE_NAME)).clone()
                        })
                        .into_box()
                }
            };

            file_fut.and_then(move |file| {
                    let reader = fry!(file_helper::read(c2, &file).map_err(FfiError::from));
                    let size = reader.size();
                    if size == 0 {
                        ok!(Vec::new())
                    } else {
                        reader.read(0, size)
                            .map_err(FfiError::from)
                            .and_then(move |data| deserialise(&data).map_err(FfiError::from))
                            .into_box()
                    }
                })
                .map(move |global_configs| (global_configs, dir, metadata))
                .map_err(FfiError::from)
                .into_box()
        })
        .into_box()
}
