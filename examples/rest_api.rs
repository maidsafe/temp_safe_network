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

//! REST API example.

// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(bad_style, exceeding_bitshifts, mutable_transmutes, no_mangle_const_items,
          unknown_crate_types, warnings)]
#![deny(deprecated, drop_with_repr_extern, improper_ctypes, missing_docs,
        non_shorthand_field_patterns, overflowing_literals, plugin_as_library,
        private_no_mangle_fns, private_no_mangle_statics, stable_features, unconditional_recursion,
        unknown_lints, unsafe_code, unused, unused_allocation, unused_attributes,
        unused_comparisons, unused_features, unused_parens, while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]
#![allow(box_pointers, fat_ptr_transmutes, missing_copy_implementations,
         missing_debug_implementations, variant_size_differences)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy, clippy_pedantic))]

#![allow(unused_extern_crates)]#[macro_use]
extern crate maidsafe_utilities;
extern crate time;
extern crate safe_core;

use std::sync::{Arc, Mutex};

use safe_core::core::client::Client;
use safe_core::nfs::AccessLevel;
use safe_core::nfs::errors::NfsError;
use safe_core::nfs::rest::{Blob, Container};

fn create_account() -> Result<Client, NfsError> {
    let mut pin = String::new();
    let mut keyword = String::new();
    let mut password = String::new();

    println!("\n\tAccount Creation");
    println!("\t================");

    println!("\n------------ Enter Keyword ---------------");
    let _ = std::io::stdin().read_line(&mut keyword);

    println!("\n\n------------ Enter Password --------------");
    let _ = std::io::stdin().read_line(&mut password);

    loop {
        println!("\n\n--------- Enter PIN (4 Digits) -----------");
        let _ = std::io::stdin().read_line(&mut pin);
        pin = pin.trim().to_string();
        if pin.parse::<u16>().is_ok() && pin.len() == 4 {
            break;
        }
        println!("ERROR: PIN is not 4 Digits !!");
        pin.clear();
    }

    // Account Creation
    println!("\nTrying to create an account ...");
    let _ = unwrap_result!(Client::create_account(keyword.clone(), pin.clone(), password.clone()));
    println!("Account Created Successfully !!");
    println!("\n\n\tAuto Account Login");
    println!("\t==================");

    // Log into the created account
    println!("\nTrying to log into the created account using supplied credentials ...");
    let client = try!(Client::log_in(keyword, pin, password));
    println!("Account Login Successful !!");
    Ok(client)
}

fn get_user_string(placeholder: &str) -> String {
    let mut txt = String::new();
    println!("------Enter {}--------", placeholder);
    let _ = std::io::stdin().read_line(&mut txt);
    while txt.is_empty() {
        println!("{} can not be empty", placeholder);
        println!("------Enter Container name--------");
        let _ = std::io::stdin().read_line(&mut txt);
    }
    txt
}

fn format_version_id(version_id: &[u8; 64]) -> String {
    let mut version = String::new();
    for j in 0..4 {
        version.push_str(&version_id[j].to_string()[..]);
    }
    version.push_str("..");
    for j in 60..64 {
        version.push_str(&version_id[j].to_string()[..]);
    }
    version
}

fn get_child_container(container: &mut Container) -> Result<Container, NfsError> {
    let sub_containers = container.get_containers();
    let ref container_name = get_user_string("Container name");
    let info = sub_containers.iter().find(|info| *info.get_name() == *container_name);
    let container_info = try!(info.ok_or(NfsError::DirectoryNotFound));
    container.get_container(container_info, None)
}

fn container_operation(option: u32, container: &mut Container) -> Result<(), NfsError> {
    match option {
        1 => {
            // Create container
            println!("----------Select the Container type-----------");
            println!("1. Versioned Private Container");
            println!("2. Versioned Public Container");
            println!("3. UnVersioned Private Container");
            println!("4. UnVersioned Public Container");
            match get_user_string("number corresponding to the type").trim().parse::<usize>() {
                Ok(index) => {
                    if index > 4 {
                        println!("Invalid input");
                        return Ok(());
                    }
                    let name = get_user_string("Container name");
                    let versioned = match index {
                        1 | 2 => true,
                        3 | 4 => false,
                        _ => true,
                    };
                    let access_level = match index {
                        1 | 3 => AccessLevel::Private,
                        2 | 4 => AccessLevel::Public,
                        _ => AccessLevel::Private,
                    };
                    let _ = try!(container.create(name.clone(), versioned, access_level, None));
                    println!("Created Container - {}", name);
                }
                Err(_) => println!("Invalid input"),
            };
        }
        2 => {
            // List containers
            let containers = container.get_containers();
            if containers.is_empty() {
                println!("No containers found");
            } else {
                println!("List of containers");
                println!("\t        Created On                  Name ");
                println!("\t =========================       ==========");
                for container_info in containers {
                    println!("\t {:?} \t {}",
                             time::strftime("%d-%m-%Y %H:%M UTC",
                                            &container_info.get_created_time())
                                 .unwrap(),
                             container_info.get_name());
                }
            }
        }
        3 => {
            // List versions
            let container = try!(get_child_container(container));
            let versions = try!(container.get_versions());
            if versions.is_empty() {
                println!("No container versions found");
            } else {
                println!("List of container versions");
                println!("\t No. \t Version Id");
                println!("\t === \t ==========");
                for i in 0..versions.len() {
                    println!("\t {} \t {:?}", i + 1, format_version_id(&versions[i]));
                }
            }
        }
        4 => {
            // Delete container
            let _ = try!(container.delete_container(&get_user_string("Container name")));
            println!("Container deleted");
        }
        _ => {}
    };
    Ok(())
}

fn blob_operation(option: u32, container: &mut Container) -> Result<(), NfsError> {
    match option {
        5 => {
            // List blobs
            let container = try!(get_child_container(container));
            let blobs: Vec<Blob> = container.get_blobs();
            if blobs.is_empty() {
                println!("No Blobs found in Container - {}", container.get_name());
            } else {
                println!("List of Blobs");
                println!("\t        Modified On                Name ");
                println!("\t =========================      ===========");
                for blob in blobs {
                    println!("\t {:?} \t {}",
                             time::strftime("%d-%m-%Y %H:%M UTC", &blob.get_modified_time())
                                 .unwrap(),
                             blob.get_name());
                }
            }
        }
        6 => {
            // Create blob
            let mut container = try!(get_child_container(container));
            let data = get_user_string("text to be saved as a file").into_bytes();
            let mut writer = try!(container.create_blob(get_user_string("Blob name"), None));
            writer.write(&data[..], 0);
            let _ = try!(writer.close());
            println!("Blob created");
        }
        7 => {
            // Update blob
            let mut container = try!(get_child_container(container));
            let blob = try!(container.get_blob(get_user_string("Blob name")));
            let data = get_user_string("text to be saved as a file").into_bytes();
            let _ = try!(container.update_blob_content(&blob, &data[..]));
            println!("Blob Updated");
        }
        8 => {
            // Read blob
            let container = try!(get_child_container(container));
            let blob = try!(container.get_blob(get_user_string("Blob name")));
            let mut reader = try!(container.get_blob_reader(&blob));
            let data_read = try!(reader.read(0, blob.get_size()));
            match String::from_utf8(data_read) {
                Ok(data) => {
                    println!("Content Read: ");
                    println!("{}\n", data);
                }
                Err(msg) => println!("Failed: {:?}", msg),
            }
        }
        9 => {
            // Read blob by version
            let container = try!(get_child_container(container));
            let blob_name = get_user_string("Blob name");
            let versions = try!(container.get_blob_versions(&blob_name.clone()));
            let ref blob_version;
            if versions.len() == 1 {
                blob_version = &versions[0];
            } else {
                println!("Available Versions::");
                for i in 0..versions.len() {
                    println!("\t{} Modified at {:?}",
                             i + 1,
                             time::strftime("%d-%m-%Y %H:%M UTC", &versions[i].get_modified_time())
                                 .unwrap())
                }
                match get_user_string("Number corresponding to the version")
                          .trim()
                          .parse::<usize>() {
                    Ok(index) => blob_version = &versions[index - 1],
                    Err(_) => {
                        println!("Invalid input : Fetching latest version");
                        blob_version = &versions[0];
                    }
                }
            }
            let mut reader = try!(container.get_blob_reader(&blob_version));
            let data_read = try!(reader.read(0, blob_version.get_size()));
            match String::from_utf8(data_read) {
                Ok(data) => {
                    println!("Content Read: ");
                    println!("{}\n", data);
                }
                Err(msg) => println!("Failed: {:?}", msg),
            }
        }
        10 => {
            // Delete blob
            let mut container = try!(get_child_container(container));
            try!(container.delete_blob(get_user_string("Blob name")));
            println!("Blob deleted");
        }
        11 => {
            // Copy blob
            let mut from_container = try!(get_child_container(container));
            let to_dir_name = get_user_string("Select the Container to copy blob to (Destination \
                                               Container)");
            let containers = container.get_containers();
            if containers.is_empty() || containers.len() == 1 {
                println!("No containers found");
                return Ok(());
            } else {
                match containers.iter().find(|dir| *dir.get_name() == to_dir_name) {
                    Some(info) => {
                        try!(from_container.copy_blob(&get_user_string("Blob name"), info));
                        println!("Blob copied");
                    }
                    None => println!("Destination Container not found"),
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn main() {
    let test_client = unwrap_result!(create_account());
    let client = Arc::new(Mutex::new(test_client));
    println!("\n\t-- Preparing storage ----\n");
    let mut root_container = unwrap_result!(Container::authorise(client.clone(), None));
    println!("\n\n------  (Tip) Start by creating a container and then store blob, modify blob \
              within the container --------------------");
    loop {
        let mut option = String::new();
        {
            println!("\n----------Choose an Operation----------------");
            println!("1. Create Container");
            println!("2. List Containers");
            println!("3. List Container Versions");
            println!("4. Delete Container");
            println!("5. List Blobs from container");
            println!("6. Create Blob");
            println!("7. Update Blob");
            println!("8. Get blob content");
            println!("9. Get blob content by version");
            println!("10. Delete blob");
            println!("11. Copy blob");
            println!("12. Exit");
            println!("------ Enter a number --------------------");
            let _ = std::io::stdin().read_line(&mut option);
            println!("\n");
            match option.trim().parse::<u32>() {
                Ok(selection) => {
                    match selection {
                        1...4 => {
                            match container_operation(selection, &mut root_container) {
                                Err(msg) => println!("Failed: {:?}", msg),
                                Ok(_) => (),
                            }
                        }
                        5...11 => {
                            match blob_operation(selection, &mut root_container) {
                                Err(msg) => println!("Failed: {:?}", msg),
                                Ok(_) => (),
                            }
                        }
                        12 => break,
                        _ => println!("Invalid option"),
                    }
                }
                Err(_) => println!("Enter a valid number"),
            }
        }
    }
}
