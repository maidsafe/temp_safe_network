// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use safe_cli::Safe;
use structopt::StructOpt;

use crate::subcommands::helpers::get_target_location;
use std::fs;
use unwrap::unwrap;

pub fn cat_command(
    location: Option<String>,
    _version: Option<String>,
    _pretty: bool,
    safe: &mut Safe,
) -> Result<(), String> {
    // TODO: Get specific versions.
    // Handle mutable types
    // Pretty print tables for key-value data.
    let location_xor = get_target_location(location);
    let data = unwrap!(safe.get_published_immutable(unwrap!(location_xor)));

    let data_string = match String::from_utf8(data) {
        Ok(string) => string,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    // data always has \n at end?
    println!("{}", data_string);

    Ok(())
}
