// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::debug;
use std::io::{self, stdin, stdout, Write};

// Read the target location from the STDIN if is not an arg provided
pub fn get_target_location(target_arg: Option<String>) -> Result<String, String> {
    match target_arg {
        Some(t) => Ok(t),
        None => {
            // try reading target from stdin then
            println!("...awaiting target XOR-URL from STDIN stream...");
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(n) => {
                    debug!(
                        "Read ({} bytes) from STDIN for target location: {}",
                        n, input
                    );
                    input.truncate(input.len() - 1);
                    Ok(input)
                }
                Err(_) => Err("There is no `--target` specified and no STDIN stream".to_string()),
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
