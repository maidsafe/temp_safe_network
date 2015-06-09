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

extern crate maidsafe_client;

use std::error::Error;
use maidsafe_client::nfs;

fn validate_pin_is_4_digits(mut pin: u32) -> bool {
    for _ in 0..3 {
        pin /= 10;
        if pin == 0 {
            return false;
        }
    }

    pin /= 10;

    if pin != 0 {
        false
    } else {
        true
    }
}

#[allow(unused_must_use)]
fn create_account() -> Result<maidsafe_client::client::Client, String> {
    let mut keyword = String::new();
    let mut password = String::new();
    let mut pin_str = String::new();
    let mut pin: u32;

    println!("\n\tAccount Creation");
    println!("\t================");

    println!("\n------------ Enter Keyword ---------------");
    std::io::stdin().read_line(&mut keyword);

    println!("\n\n------------ Enter Password --------------");
    std::io::stdin().read_line(&mut password);

    loop {
        println!("\n\n--------- Enter PIN (4 Digits) -----------");
        std::io::stdin().read_line(&mut pin_str);
        let result = pin_str.trim().parse::<u32>();
        if result.is_ok() {
            pin = result.ok().unwrap();
            if validate_pin_is_4_digits(pin) {
                break;
            }
        }
        println!("ERROR: PIN is not 4 Digits !!");
        pin_str.clear();
    }

    //TODO Not to be used if not using non_networking_test_framework.
    let data_store = maidsafe_client::client::non_networking_test_framework::get_new_data_store();

    // Account Creation
    println!("\nTrying to create an account ...");

    match maidsafe_client::client::Client::create_account(&keyword, pin, &password.as_bytes(), data_store.clone()) {
        Ok(_) => {
            println!("Account Created Successfully !!");
        },
        Err(io_error) => panic!("Account Created failed"),
    }

    println!("\n\n\tAuto Account Login");
    println!("\t==================");

    // Log into the created account
    println!("\nTrying to log into the created account using supplied credentials ...");
    match maidsafe_client::client::Client::log_in(&keyword, pin, &password.as_bytes(), data_store.clone()) {
        Ok(client) => {
            println!("Account Login Successful !!");
            Ok(client)
        },
        Err(io_error)  => Err("Account Login Failed !!".to_string()),
    }
}

fn get_container_name() -> String {
    let mut container_name = String::new();
    println!("------Enter Container name--------");
    std::io::stdin().read_line(&mut container_name);
    while container_name.is_empty() {
        println!("Name can not be empty");
        println!("------Enter Container name--------");
        std::io::stdin().read_line(&mut container_name);
    }
    container_name
}

fn container_operation(option: u32, container: &mut nfs::rest::Container) {
    match option {
        1 => {// Create container
            let name = get_container_name();
            container.create(name.clone(), None);
            println!("Container - {} has been created", name);
        },
        2 => { // List containers
            let containers = container.get_containers();
            if containers.is_empty() {
                println!("No containers found");
            } else {
                println!("List of containers");
                println!("\t Name \t Created On");
                for container_info in containers {
                    println!("\t {} \t {:?}", container_info.get_name(), container_info.get_created_time());
                }
            }
        }
        3 => { // List versions
            match container.get_versions() {
                Ok(versions) => {
                    if versions.is_empty() {
                        println!("No container versions found");
                    } else {
                        println!("List of container versions");
                        println!("\t No. \t Version Id");
                        println!("\t ___ \t __________");
                        for i in 0..versions.len() {
                            println!("\t {} \t {:?}..{:?}", i+1, &versions[i][0..4], &versions[i][60..64]);
                        }
                    }
                },
                Err(msg) => println!("Failed :: {}", msg)
            }
        },
        4 => { // Delete container
            let name = get_container_name();
            match container.delete_container(name.clone()) {
                Ok(_) => {
                    println!("Container deleted");
                },
                Err(msg) => println!("Failed :: {}", msg)
            };
        }
        _ => {}
    }
}

fn blob_operation(option: u32, container: &nfs::rest::Container) {
    match option {
        5 => {

        }
        _ => {}
    }
}

fn get_root_container(client: &::std::sync::Arc<::std::sync::Mutex<maidsafe_client::client::Client>>) -> nfs::rest::Container {
    let mut root_container;
    match nfs::rest::Container::authorise(client.clone(), [0u8;64]) {
        Ok(mut container) => root_container = container,
        Err(msg) => panic!(msg)
    };
    root_container
}
#[allow(unused_must_use)]
fn main() {
    let mut client;
    match create_account() {
        Ok(authorised_client) => client = ::std::sync::Arc::new(::std::sync::Mutex::new(authorised_client)),
        Err(msg) => panic!(msg)
    }
    let mut root_container = get_root_container(&client);
    loop {
        let mut option = String::new();

        {
            println!("\n----------Choose an Operation----------------");
            println!("1. Create Container");
            println!("2. List Containers");
            println!("3. Get Conatiner Versions");
            println!("4. Delete Conatiner");
            println!("5. List Blobs");
            println!("6. Create Blob");
            println!("7. Update Blob");
            println!("8. Get blob by version");
            println!("9. Delete blob");

            println!("------Enter number correspoding the operation--------------------");
            std::io::stdin().read_line(&mut option);
            println!("\n");
            match option.trim().parse::<u32>() {
                Ok(selection) => {
                    match selection {
                        1...4 => container_operation(selection, &mut root_container),
                        5...9 => blob_operation(selection, &mut root_container),
                        _ => println!("Invalid option"),
                    }
                },
                Err(_) => println!("Enter a valid number")
            }
        }
    }
}
