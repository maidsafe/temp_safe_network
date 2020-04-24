// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use safe_api::{files::ProcessedFiles, wallet::WalletSpendableBalances, BlsKeyPair};
use safe_nd::Coins;
use std::collections::BTreeMap;
use std::{env, str::FromStr};
use unwrap::unwrap;

#[allow(dead_code)]
pub const CLI: &str = "safe";
#[allow(dead_code)]
pub const SAFE_PROTOCOL: &str = "safe://";

pub const TEST_FOLDER: &str = "../testdata/";
pub const TEST_FOLDER_NO_TRAILING_SLASH: &str = "../testdata";

#[allow(dead_code)]
pub fn read_cmd(e: duct::Expression) -> Result<String, String> {
    e.read().map_err(|e| format!("{:#?}", e))
}

#[allow(dead_code)]
pub fn create_preload_and_get_keys(preload: &str) -> (String, String) {
    let pk_command_result = cmd!(
        env!("CARGO_BIN_EXE_safe"),
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
pub fn create_wallet_with_balance(
    preload: &str,
    balance_name: Option<&str>,
) -> (String, String, String) {
    let (_pk, sk) = create_preload_and_get_keys(&preload);
    // we spent 1 nano for creating the SafeKey, so we now preload it
    // with 1 nano less than amount request provided
    let preload_nanos = Coins::from_str(preload).unwrap().as_nano();
    let preload_minus_costs = Coins::from_nano(preload_nanos - 1).to_string();

    let wallet_create_result = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "create",
        "--pay-with",
        &sk,
        "--preload",
        preload_minus_costs,
        "--name",
        balance_name.unwrap_or_else(|| "default-balance"),
        "--json",
    )
    .read()
    .unwrap();

    let (wallet_xor, _key_xorurl, key_pair) = parse_wallet_create_output(&wallet_create_result);
    let unwrapped_key_pair = unwrap!(key_pair);
    (wallet_xor, unwrapped_key_pair.pk, unwrapped_key_pair.sk)
}

#[allow(dead_code)]
pub fn upload_test_folder_with_result(
    trailing_slash: bool,
) -> Result<(String, ProcessedFiles), String> {
    let path = if trailing_slash {
        TEST_FOLDER
    } else {
        TEST_FOLDER_NO_TRAILING_SLASH
    };
    let files_container = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "files",
        "put",
        path,
        "--recursive",
        "--json"
    )
    .read()
    .map_err(|e| format!("{:#?}", e))?;

    let (container_xorurl, file_map) = parse_files_put_or_sync_output(&files_container);

    Ok((container_xorurl, file_map))
}

#[allow(dead_code)]
pub fn upload_testfolder_trailing_slash() -> Result<(String, ProcessedFiles), String> {
    upload_test_folder_with_result(true)
}

#[allow(dead_code)]
pub fn upload_testfolder_no_trailing_slash() -> Result<(String, ProcessedFiles), String> {
    upload_test_folder_with_result(false)
}

// keeping for compat with older tests
#[allow(dead_code)]
pub fn upload_test_folder() -> (String, ProcessedFiles) {
    upload_test_folder_with_result(true).unwrap()
}

#[allow(dead_code)]
pub fn get_random_nrs_string() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(15).collect()
}

#[allow(dead_code)]
pub fn parse_files_container_output(
    output: &str,
) -> (String, BTreeMap<String, BTreeMap<String, String>>) {
    serde_json::from_str(output).expect("Failed to parse output of `safe cat` on FilesContainer")
}

#[allow(dead_code)]
pub fn parse_files_tree_output(output: &str) -> serde_json::Value {
    serde_json::from_str(output).expect("Failed to parse output of `safe tree`")
}

#[allow(dead_code)]
pub fn parse_files_put_or_sync_output(output: &str) -> (String, ProcessedFiles) {
    serde_json::from_str(output).expect("Failed to parse output of `safe files sync`")
}

#[allow(dead_code)]
pub fn parse_nrs_create_output(output: &str) -> (String, ProcessedFiles) {
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

#[allow(dead_code)]
pub fn parse_xorurl_output(output: &str) -> Vec<(String, String)> {
    serde_json::from_str(output).expect("Failed to parse output of `safe xorurl`")
}
