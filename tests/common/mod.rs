// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub const CLI: &str = "safe";
#[allow(dead_code)]
pub const SAFE_PROTOCOL: &str = "safe://";

#[allow(dead_code)]
pub fn get_bin_location() -> &'static str {
    let mut location = "./target/release/safe";
    if cfg!(debug_assertions) {
        location = "./target/debug/safe";
    }
    location
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
        &pk,
        "--secret-key",
        &sk,
        "--json",
    )
    .read()
    .unwrap();

    let mut lines = wallet_create_result.lines().rev();
    let wallet_xor = lines.next().unwrap();

    (wallet_xor.to_string(), pk, sk)
}
