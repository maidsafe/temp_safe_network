// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::env;

pub const CLI: &str = "safe";
#[allow(dead_code)]
pub const SAFE_PROTOCOL: &str = "safe://";
use std::collections::BTreeMap;
const TEST_FOLDER: &str = "./tests/testfolder/";

#[allow(dead_code)]
pub fn get_bin_location() -> String {
    let target_dir = match env::var("CARGO_TARGET_DIR") {
        Ok(target_dir) => target_dir,
        Err(_) => "./target".to_string(),
    };
    if cfg!(debug_assertions) {
        return format!("{}{}", target_dir, "/debug/safe");
    }
    format!("{}{}", target_dir, "/release/safe")
}

#[allow(dead_code)]
pub fn create_preload_and_get_keys(preload: &str) -> (String, String) {
    // KEY_FROM
    let pk_command_result = cmd!(
        get_bin_location(),
        "keys",
        "create",
        "--test-coins",
        "---preload",
        preload,
        "--json",
    )
    .read()
    .unwrap();

    let mut lines = pk_command_result.lines();
    let pk_xor_line = lines.next().unwrap();
    let pk_xor = &pk_xor_line["pk-xorurl=".len()..];
    let _pk = lines.next().unwrap();
    let sk_line = lines.next().unwrap();
    let sk = &sk_line["sk=".len()..];

    (pk_xor.to_string(), sk.to_string())
}

#[allow(dead_code)]
pub fn create_wallet_with_balance(preload: &str) -> (String, String, String) {
    let (pk, sk) = create_preload_and_get_keys(&preload);
    let wallet_create_result = cmd!(
        get_bin_location(),
        "wallet",
        "create",
        &pk,
        "--keyurl",
        &pk,
        "--sk",
        &sk,
        "--json",
    )
    .read()
    .unwrap();

    let mut lines = wallet_create_result.lines().rev();
    let wallet_xor = lines.next().unwrap();

    (wallet_xor.to_string(), pk, sk)
}

#[allow(dead_code)]
pub fn upload_test_folder() -> (String, BTreeMap<String, (String, String)>) {
    let files_container = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive",
        "--json"
    )
    .read()
    .unwrap();

    let (container_xorurl, file_map): (String, BTreeMap<String, (String, String)>) =
        match serde_json::from_str(&files_container) {
            Ok(s) => s,
            Err(err) => panic!(format!(
                "Failed to parse output of `safe nrs create`: {}",
                err
            )),
        };

    (container_xorurl, file_map)
}
