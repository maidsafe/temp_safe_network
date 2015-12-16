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

extern crate safe_core;
#[macro_use] extern crate maidsafe_utilities;

use safe_core::client::Client;

fn main() {
    maidsafe_utilities::log::init(true);

    let mut keyword = String::new();
    let mut password = String::new();
    let mut pin = String::new();

    println!("\nDo you already have an account created (enter Y for yes) ?");

    let mut user_option = String::new();
    let _ = std::io::stdin().read_line(&mut user_option);
    user_option = user_option.trim().to_string();

    if user_option != "Y" && user_option != "y" {
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
        {
            println!("\nTrying to create an account ...");

            let _ = unwrap_result!(Client::create_account(keyword.clone(), pin.clone(), password.clone()));
            println!("Account Created Successfully !!");
        }

        println!("\n\n\tAuto Account Login");
        println!("\t==================");

        // Log into the created account
        {
            println!("\nTrying to log into the created account using supplied credentials ...");

            let _ = unwrap_result!(Client::log_in(keyword, pin, password));
            println!("Account Login Successful !!");
        }
    }

    println!("\n\n\tManual Account Login");
    println!("\t====================");

    loop {
        keyword = String::new();
        password = String::new();

        println!("\n------------ Enter Keyword ---------------");
        let _ = std::io::stdin().read_line(&mut keyword);

        println!("\n\n------------ Enter Password --------------");
        let _ = std::io::stdin().read_line(&mut password);

        loop {
            pin = String::new();
            println!("\n\n--------- Enter PIN (4 Digits) -----------");
            let _ = std::io::stdin().read_line(&mut pin);
            pin = pin.trim().to_string();
            if pin.parse::<u16>().is_ok() && pin.len() == 4 {
                break;
            }
            println!("ERROR: PIN is not 4 Digits !!");
        }

        // Log into the created account
        {
            println!("\nTrying to log in ...");
            match Client::log_in(keyword, pin, password) {
                Ok(_) => {
                    println!("Account Login Successful !!");
                    break;
                }
                Err(error)  => println!("Account Login Failed !! Reason: {:?}\n\n", error),
            }
        }
    }
}
