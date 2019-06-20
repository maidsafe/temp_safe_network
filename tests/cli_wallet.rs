// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod common;
extern crate serde_json;
#[macro_use]
extern crate duct;

use assert_cmd::prelude::*;
use common::{create_preload_and_get_keys, create_wallet_with_balance, get_bin_location};
use predicates::prelude::*;

use std::process::Command;

static CLI: &str = "safe_cli";
static PRETTY_WALLET_CREATION_RESPONSE: &str = "Wallet created at";
static SAFE_PROTOCOL: &str = "safe://";
static NO_SOURCE: &str = "Missing the 'source' argument";

#[test]
fn calling_safe_wallet_transfer() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_from, _pk, _sk) = create_wallet_with_balance("160");
    assert!(wallet_from.contains(SAFE_PROTOCOL));
    let (wallet_to, _pk, _sk) = create_wallet_with_balance("5");
    assert!(wallet_to.contains(SAFE_PROTOCOL));

    // To got coins?
    let to_starts_with = cmd!(get_bin_location(), "wallet", "balance", &wallet_to)
        .read()
        .unwrap();

    assert_eq!(to_starts_with, "5");

    // To got coins?
    let from_starts_with = cmd!(get_bin_location(), "wallet", "balance", &wallet_from)
        .read()
        .unwrap();

    assert_eq!(from_starts_with, "160");

    cmd.args(&vec![
        "wallet",
        "transfer",
        "100",
        &wallet_to,
        &wallet_from,
        "--pretty",
    ])
    .assert()
    .stdout(predicate::str::contains("Success"))
    .stdout(predicate::str::contains("TX_ID"))
    .success();

    // To got coins?
    let to_has = cmd!(get_bin_location(), "wallet", "balance", &wallet_to)
        .read()
        .unwrap();

    assert_eq!(to_has, "105");

    // from lost coins?
    let from_has = cmd!(get_bin_location(), "wallet", "balance", &wallet_from)
        .read()
        .unwrap();

    assert_eq!(from_has, "60")
}

// TODO: this test should check for lack of SK when querying a balance not owned by user.
// And should fail.
// Blocked: until SCL queries of random balances
//
// #[test]
// fn calling_safe_wallet_balance_pretty_no_sk() {
//     let mut cmd = Command::cargo_bin(CLI).unwrap();
//
//     let wallet = cmd!(get_bin_location(), "wallet", "create", "--preload", "300", "--test-coins").read().unwrap();
//     assert!(wallet.contains(SAFE_PROTOCOL));
//
//     cmd.args(&vec!["wallet", "balance", &wallet])
//         .assert()
//         .stdout("300\n")
//         .success();
// }

#[test]
fn calling_safe_wallet_balance() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("10");

    cmd.args(&vec![
        "wallet",
        "balance",
        &wallet_xor,
        // "--pretty",
    ])
    .assert()
    .stdout("10\n")
    .success();
}

#[test]
fn calling_safe_wallet_insert() {
    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("50");

    let (pk_xor, sk) = create_preload_and_get_keys("300");

    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let wallet_insert_result = cmd!(
        get_bin_location(),
        "wallet",
        "insert",
        &pk_xor,
        &wallet_xor,
        &wallet_xor,
        "--secret-key",
        &sk
    )
    .read()
    .unwrap();

    cmd.args(&vec!["wallet", "balance", &wallet_xor])
        .assert()
        .stdout("350\n")
        .success();
}

#[test]
fn calling_safe_wallet_create_no_source() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    cmd.args(&vec!["wallet", "create"])
        .assert()
        .stderr(predicate::str::contains(NO_SOURCE))
        .failure();
}

#[test]
fn calling_safe_wallet_no_balance() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    cmd.args(&vec!["wallet", "create", "--no-balance", "--pretty"])
        .assert()
        .stdout(predicate::str::contains(PRETTY_WALLET_CREATION_RESPONSE))
        .success();
}

#[test]
fn calling_safe_wallet_create_w_preload_has_balance() {
    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("55");

    let balance = cmd!(get_bin_location(), "wallet", "balance", &wallet_xor)
        .read()
        .unwrap();
    assert_eq!("55", balance);
}

#[test]
fn calling_safe_wallet_create_w_premade_keys_has_balance() {
    let (pk_pay_xor, pay_sk) = create_preload_and_get_keys("300");

    let wallet_create_result = cmd!(
        get_bin_location(),
        "wallet",
        "create",
        &pk_pay_xor,
        &pk_pay_xor,
        "--secret-key",
        pay_sk
    )
    .read()
    .unwrap();

    let balance = cmd!(
        get_bin_location(),
        "wallet",
        "balance",
        &wallet_create_result
    )
    .read()
    .unwrap();
    assert_eq!("300", balance);
}
