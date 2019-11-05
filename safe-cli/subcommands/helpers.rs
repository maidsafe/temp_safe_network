// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::debug;
use safe_api::XorName;
use std::io::{self, stdin, stdout, Write};

// Warn the user about a dry-run being performed
pub fn notice_dry_run() {
    println!("NOTE the operation is being performed in dry-run mode, therefore no changes will be committed to the network.");
}

// Converts the XOR name bytes into a hex encoded string
pub fn xorname_to_hex(xorname: &XorName) -> String {
    xorname.0.iter().map(|b| format!("{:02x}", b)).collect()
}

// Read the target location from the STDIN if is not an arg provided
pub fn get_from_arg_or_stdin(
    target_arg: Option<String>,
    message: Option<&str>,
) -> Result<String, String> {
    let the_message = message.unwrap_or_else(|| "...awaiting data from STDIN stream...");
    match target_arg {
        Some(t) => Ok(t),
        None => {
            println!("{}", &the_message);
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(n) => {
                    debug!("Read ({} bytes) from STDIN: {}", n, input);
                    input.truncate(input.len() - 1);
                    Ok(input)
                }
                Err(_) => Err("Failed to read from STDIN stream".to_string()),
            }
        }
    }
}

// Prompt the user with the message provided
pub fn prompt_user(prompt_msg: &str, error_msg: &str) -> Result<String, String> {
    let mut user_input = String::new();
    print!("{}", prompt_msg);
    let _ = stdout().flush();
    stdin().read_line(&mut user_input).map_err(|_| error_msg)?;
    if let Some('\n') = user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r') = user_input.chars().next_back() {
        user_input.pop();
    }

    if user_input.is_empty() {
        Err(error_msg.to_string())
    } else {
        Ok(user_input)
    }
}

// Unwrap secret key string provided, otherwise prompt user to provide it
pub fn get_secret_key(key_xorurl: &str, sk: Option<String>, msg: &str) -> Result<String, String> {
    let mut sk = sk.unwrap_or_else(|| String::from(""));

    if sk.is_empty() {
        let msg = if key_xorurl.is_empty() {
            format!("Enter secret key corresponding to {}: ", msg)
        } else {
            format!(
                "Enter secret key corresponding to public key at \"{}\": ",
                key_xorurl
            )
        };
        sk = prompt_user(&msg, "Invalid input")?;
    }

    Ok(sk)
}

pub fn parse_tx_id(src: &str) -> Result<u64, String> {
    src.parse::<u64>()
        .map_err(|err| format!("{}. A valid TX Id is a number between 0 and 2^64", err))
}
