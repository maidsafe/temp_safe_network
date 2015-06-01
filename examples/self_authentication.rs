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

fn main() {
    let mut keyword = "".to_string();
    let mut password = "".to_string();
    let mut pin_str = "".to_string();
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
    {
        println!("\nTrying to create an account ...");

        match maidsafe_client::client::Client::create_account(&keyword, pin, &password.as_bytes(), data_store.clone()) {
            Ok(_) => println!("Account Created Successfully !!"),
            Err(io_error)  => println!("Account Creation Failed !! Reason: {:?}", io_error.description()),
        }
    }

    println!("\n\n\tAuto Account Login");
    println!("\t==================");

    // Log into the created account
    {
        println!("\nTrying to log into the created account using supplied credentials ...");
        match maidsafe_client::client::Client::log_in(&keyword, pin, &password.as_bytes(), data_store.clone()) {
            Ok(_) => println!("Account Login Successful !!"),
            Err(io_error)  => println!("Account Login Failed !! Reason: {:?}", io_error.description()),
        }
    }

    println!("\n\n\tManual Account Login");
    println!("\t====================");

    password.clear();
    keyword.clear();

    println!("\n------------ Enter Keyword ---------------");
    std::io::stdin().read_line(&mut keyword);

    println!("\n\n------------ Enter Password --------------");
    std::io::stdin().read_line(&mut password);

    loop {
        pin_str.clear();
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
    }

    // Log into the created account
    {
        println!("\nTrying to log in ...");
        match maidsafe_client::client::Client::log_in(&keyword, pin, &password.as_bytes(), data_store.clone()) {
            Ok(_) => println!("Account Login Successful !!"),
            Err(io_error)  => println!("Account Login Failed !! Reason: {:?}", io_error.description()),
        }
    }
}
