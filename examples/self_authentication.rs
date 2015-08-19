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

#[macro_use] extern crate safe_client;

fn main() {
    let mut keyword = String::new();
    let mut password = String::new();
    let mut pin_str = String::new();
    let mut pin: u32;

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
            let _ = std::io::stdin().read_line(&mut pin_str);
            let result = pin_str.trim().parse::<u32>();
            if result.is_ok() && pin_str.trim().len() == 4 {
                pin = result.ok().unwrap();
                break;
            }
            println!("ERROR: PIN is not 4 Digits !!");
            pin_str.clear();
        }

        // Account Creation
        {
            println!("\nTrying to create an account ...");

            let _ = eval_result!(safe_client::client::Client::create_account(&keyword, pin, &password));
            println!("Account Created Successfully !!");
        }

        println!("\n\n\tAuto Account Login");
        println!("\t==================");

        // Log into the created account
        {
            println!("\nTrying to log into the created account using supplied credentials ...");

            let _ = eval_result!(safe_client::client::Client::log_in(&keyword, pin, &password));
            println!("Account Login Successful !!");
        }
    }

    println!("\n\n\tManual Account Login");
    println!("\t====================");

    loop {
        password.clear();
        keyword.clear();

        println!("\n------------ Enter Keyword ---------------");
        let _ = std::io::stdin().read_line(&mut keyword);

        println!("\n\n------------ Enter Password --------------");
        let _ = std::io::stdin().read_line(&mut password);

        loop {
            pin_str.clear();
            println!("\n\n--------- Enter PIN (4 Digits) -----------");
            let _ = std::io::stdin().read_line(&mut pin_str);
            let result = pin_str.trim().parse::<u32>();
            if result.is_ok() && pin_str.trim().len() == 4 {
                pin = result.ok().unwrap();
                break;
            }
            println!("ERROR: PIN is not 4 Digits !!");
        }

        // Log into the created account
        {
            println!("\nTrying to log in ...");
            match safe_client::client::Client::log_in(&keyword, pin, &password) {
                Ok(_) => {
                    println!("Account Login Successful !!");
                    break;
                }
                Err(error)  => println!("Account Login Failed !! Reason: {:?}\n\n", error),
            }
        }
    }
}
