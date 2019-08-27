// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use safe_cli::{BlsKeyPair, WalletSpendableBalances};
use std::env;
use unwrap::unwrap;

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

    let (xorurl, pair): (String, BlsKeyPair) = serde_json::from_str(&pk_command_result)
        .expect("Failed to parse output of `safe files sync`");
    (xorurl, pair.sk)
}

#[allow(dead_code)]
pub fn create_wallet_with_balance(preload: &str) -> (String, String, String) {
    let (_pk, sk) = create_preload_and_get_keys(&preload);
    let wallet_create_result = cmd!(
        get_bin_location(),
        "wallet",
        "create",
        "--pay-with",
        &sk,
        "--preload",
        preload,
        "--json",
    )
    .read()
    .unwrap();

    let (wallet_xor, _key_xorurl, key_pair) = parse_wallet_create_output(&wallet_create_result);
    let unwrapped_key_pair = unwrap!(key_pair);
    (
        wallet_xor.to_string(),
        unwrapped_key_pair.pk,
        unwrapped_key_pair.sk,
    )
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

    let (container_xorurl, file_map) = parse_files_put_or_sync_output(&files_container);

    (container_xorurl, file_map)
}

#[allow(dead_code)]
pub fn get_random_nrs_string() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(15).collect()
}

#[allow(dead_code)]
pub fn parse_cat_files_container_output(
    output: &str,
) -> (String, BTreeMap<String, BTreeMap<String, String>>) {
    serde_json::from_str(output).expect("Failed to parse output of `safe cat` on FilesContainer")
}

#[allow(dead_code)]
pub fn parse_files_put_or_sync_output(
    output: &str,
) -> (String, BTreeMap<String, (String, String)>) {
    serde_json::from_str(output).expect("Failed to parse output of `safe files sync`")
}

#[allow(dead_code)]
pub fn parse_nrs_create_output(output: &str) -> (String, BTreeMap<String, (String, String)>) {
    serde_json::from_str(output).expect("Failed to parse output of `safe nrs create`")
}

#[allow(dead_code)]
pub fn parse_wallet_create_output(output: &str) -> (String, String, Option<BlsKeyPair>) {
    serde_json::from_str(&output).expect("Failed to parse output of `safe wallet create`")
}

#[allow(dead_code)]
pub fn parse_cat_wallet_output(output: &str) -> (String, WalletSpendableBalances) {
    serde_json::from_str(output).expect("Failed to parse output of `safe cat wallet`")
}
