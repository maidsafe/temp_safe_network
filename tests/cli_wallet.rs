// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod common;

#[macro_use]
extern crate duct;

use assert_cmd::prelude::*;
use common::{
    create_preload_and_get_keys, create_wallet_with_balance, get_bin_location, CLI, SAFE_PROTOCOL,
};
use predicates::prelude::*;
use std::process::Command;

const PRETTY_WALLET_CREATION_RESPONSE: &str = "Wallet created at";
const UNMATCHED_SK_XORURL: &str =
    "The XOR-URL doesn't correspond to the public key derived from the provided secret key";

#[test]
fn calling_safe_wallet_transfer() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_from, _pk, _sk) = create_wallet_with_balance("160");
    assert!(wallet_from.contains(SAFE_PROTOCOL));
    let (wallet_to, _pk, _sk) = create_wallet_with_balance("5");
    assert!(wallet_to.contains(SAFE_PROTOCOL));

    // To got coins?
    let to_starts_with = cmd!(
        get_bin_location(),
        "wallet",
        "balance",
        &wallet_to,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(to_starts_with, "5.000000000");

    // To got coins?
    let from_starts_with = cmd!(
        get_bin_location(),
        "wallet",
        "balance",
        &wallet_from,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_starts_with, "160.000000000");

    cmd.args(&vec!["wallet", "transfer", "100", &wallet_to, &wallet_from])
        .assert()
        .stdout(predicate::str::contains("Success"))
        .stdout(predicate::str::contains("TX_ID"))
        .success();

    // To got coins?
    let to_has = cmd!(
        get_bin_location(),
        "wallet",
        "balance",
        &wallet_to,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(to_has, "105.000000000");

    // from lost coins?
    let from_has = cmd!(
        get_bin_location(),
        "wallet",
        "balance",
        &wallet_from,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_has, "60.000000000")
}

#[test]
fn calling_safe_wallet_balance() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("10");

    cmd.args(&vec!["wallet", "balance", &wallet_xor, "--json"])
        .assert()
        .stdout("10.000000000\n")
        .success();
}

#[test]
fn calling_safe_wallet_insert() {
    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("50");
    let (key_pk_xor, sk) = create_preload_and_get_keys("300");

    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let _wallet_insert_result = cmd!(
        get_bin_location(),
        "wallet",
        "insert",
        &wallet_xor,
        "--keyurl",
        &key_pk_xor,
        "--sk",
        &sk,
        "--json"
    )
    .read()
    .unwrap();

    cmd.args(&vec!["wallet", "balance", &wallet_xor, "--json"])
        .assert()
        .stdout("350.000000000\n")
        .success();
}

#[test]
fn calling_safe_wallet_create_no_source() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    cmd.args(&vec!["wallet", "create"])
        .assert()
        .stderr(predicate::str::contains(
            "Failed to create a CoinBalance: \"NoSuchBalance\"",
        ))
        .failure();
}

#[test]
fn calling_safe_wallet_no_balance() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    cmd.args(&vec!["wallet", "create", "--no-balance"])
        .assert()
        .stdout(predicate::str::contains(PRETTY_WALLET_CREATION_RESPONSE))
        .success();
}

#[test]
fn calling_safe_wallet_create_w_preload_has_balance() {
    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("55");

    let balance = cmd!(
        get_bin_location(),
        "wallet",
        "balance",
        &wallet_xor,
        "--json"
    )
    .read()
    .unwrap();
    assert_eq!("55.000000000", balance);
}

#[test]
fn calling_safe_wallet_create_w_premade_keys_has_balance() {
    let (pk_pay_xor, pay_sk) = create_preload_and_get_keys("300");

    let wallet_create_result = cmd!(
        get_bin_location(),
        "wallet",
        "create",
        "--pay-with",
        &pay_sk,
        "--keyurl",
        &pk_pay_xor,
        "--sk",
        pay_sk,
        "--json"
    )
    .read()
    .unwrap();

    let balance = cmd!(
        get_bin_location(),
        "wallet",
        "balance",
        &wallet_create_result,
        "--json"
    )
    .read()
    .unwrap();
    assert_eq!("300.000000000", balance);
}

#[test]
fn calling_safe_wallet_create_w_bad_secret() {
    let (pk_pay_xor, pay_sk) = create_preload_and_get_keys("300");

    let mut cmd = Command::cargo_bin(CLI).unwrap();

    cmd.args(&vec![
        "wallet",
        "create",
        "--pay-with",
        &pay_sk,
        "--keyurl",
        &pk_pay_xor,
        "--sk",
        "badbadbad",
        "--json",
    ])
    .assert()
    .stderr(predicate::str::contains("Invalid secret key provided"))
    .failure();
}

#[test]
fn calling_safe_wallet_create_w_bad_pk() {
    let (_pk_pay_xor, pay_sk) = create_preload_and_get_keys("300");

    let mut cmd = Command::cargo_bin(CLI).unwrap();

    cmd.args(&vec![
        "wallet",
        "create",
        "--pay-with",
        &pay_sk,
        "--keyurl",
        "safe://nononooOOooo",
        "--sk",
        &pay_sk,
        "--json",
    ])
    .assert()
    .stderr(predicate::str::contains(
        "Failed to decode XOR-URL: UnkownBase",
    ))
    .failure();
}

#[test]
fn calling_safe_wallet_create_w_wrong_pk_for_sk() {
    let (pk_pay_xor, pay_sk) = create_preload_and_get_keys("300");
    let (_, key_sk) = create_preload_and_get_keys("300");

    let mut cmd = Command::cargo_bin(CLI).unwrap();

    cmd.args(&vec![
        "wallet",
        "create",
        "--pay-with",
        &pay_sk,
        "--keyurl",
        &pk_pay_xor,
        "--sk",
        &key_sk,
        "--json",
    ])
    .assert()
    .stderr(predicate::str::contains(UNMATCHED_SK_XORURL))
    .failure();
}
