// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod common;

#[macro_use]
extern crate duct;

use assert_cmd::prelude::*;
use common::{
    create_preload_and_get_keys, create_wallet_with_balance, get_bin_location,
    get_random_nrs_string, CLI, SAFE_PROTOCOL,
};
use predicates::prelude::*;
use std::process::Command;

const PRETTY_KEYS_CREATION_RESPONSE: &str = "New SafeKey created at:";

#[test]
fn calling_safe_keys_create_pretty() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["keys", "create", "--test-coins", "--preload", "123"])
        .assert()
        .stdout(predicate::str::contains(PRETTY_KEYS_CREATION_RESPONSE))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).from_utf8())
        .success();
}

#[test]
fn calling_safe_keys_create() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "keys",
        "create",
        "--test-coins",
        "--preload",
        "123",
        "--json",
    ])
    .assert()
    .stdout(predicate::str::contains(PRETTY_KEYS_CREATION_RESPONSE).count(0))
    .stdout(predicate::str::contains(SAFE_PROTOCOL).from_utf8())
    .success();
}

#[test]
fn calling_safe_keypair() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["keypair"])
        .assert()
        .stdout(predicate::str::contains("Secret Key = "))
        .stdout(predicate::str::contains("Public Key = "))
        .success();
}

#[test]
fn calling_safe_keypair_pretty() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["keypair"])
        .assert()
        .stdout(predicate::str::contains("Key pair generated:"))
        .stdout(predicate::str::contains("Secret Key = "))
        .stdout(predicate::str::contains("Public Key = "))
        .success();
}

#[test]
fn calling_safe_keys_balance() {
    let (pk_xor, sk) = create_preload_and_get_keys("123");
    assert!(pk_xor.contains("safe://"));

    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "keys", "balance", "--keyurl", &pk_xor, "--sk", &sk, "--json",
    ])
    .assert()
    .stdout("123.000000000\n")
    .success();
}

#[test]
fn calling_safe_keys_balance_with_nrs_for_keyurl() {
    let (pk_xor, sk) = create_preload_and_get_keys("3006.77");

    let nrsurl = format!("safe://{}", get_random_nrs_string());
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["nrs", "create", &nrsurl, "-l", &pk_xor])
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "keys", "balance", "--keyurl", &nrsurl, "--sk", &sk, "--json",
    ])
    .assert()
    .stdout("3006.770000000\n")
    .success();
}

#[test]
fn calling_safe_keys_transfer() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    let (_safekey1_xorurl, sk1) = create_preload_and_get_keys("160.0");
    let (safekey2_xorurl, sk2) = create_preload_and_get_keys("5.0");

    cmd.args(&vec![
        "keys",
        "transfer",
        "100",
        "--from",
        &sk1,
        "--to",
        &safekey2_xorurl,
    ])
    .assert()
    .stdout(predicate::str::contains("Success"))
    .stdout(predicate::str::contains("TX_ID"))
    .success();

    // To got coins?
    let to_has = cmd!(
        get_bin_location(),
        "keys",
        "balance",
        "--sk",
        &sk2,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(to_has, "105.000000000");

    // from lost coins?
    let from_has = cmd!(
        get_bin_location(),
        "keys",
        "balance",
        "--sk",
        &sk1,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_has, "60.000000000")
}

#[test]
fn calling_safe_keys_transfer_to_wallet_xorurl() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (to_wallet, _pk, _sk) = create_wallet_with_balance("0.000000001", None); // we need 1 nano to pay for the costs of creation
    let (_safekey_xorurl, safekey_sk) = create_preload_and_get_keys("35.65");

    cmd.args(&vec![
        "keys",
        "transfer",
        "18.23",
        "--from",
        &safekey_sk,
        "--to",
        &to_wallet,
    ])
    .assert()
    .stdout(predicate::str::contains("Success"))
    .stdout(predicate::str::contains("TX_ID"))
    .success();

    // deducted coins from sending SafeKey?
    let safekey_has = cmd!(
        get_bin_location(),
        "keys",
        "balance",
        "--sk",
        &safekey_sk,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(safekey_has, "17.420000000" /* 35.65 - 18.23 */);

    // Wallet got coins?
    let to_has = cmd!(
        get_bin_location(),
        "wallet",
        "balance",
        &to_wallet,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(to_has, "18.230000000")
}

#[test]
fn calling_safe_keys_transfer_to_key_nrsurl() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (_from_safekey_xorurl, from_safekey_sk) = create_preload_and_get_keys("1535.65");
    let (to_safekey_xorurl, to_safekey_sk) = create_preload_and_get_keys("0.0");

    let to_safekey_nrsurl = format!("safe://{}", get_random_nrs_string());
    let _ = cmd!(
        get_bin_location(),
        "nrs",
        "create",
        &to_safekey_nrsurl,
        "-l",
        &to_safekey_xorurl,
    )
    .read()
    .unwrap();

    cmd.args(&vec![
        "keys",
        "transfer",
        "118.23",
        "--from",
        &from_safekey_sk,
        "--to",
        &to_safekey_nrsurl,
    ])
    .assert()
    .stdout(predicate::str::contains("Success"))
    .stdout(predicate::str::contains("TX_ID"))
    .success();

    // SafeKey at NRS got coins?
    let key_has = cmd!(
        get_bin_location(),
        "keys",
        "balance",
        "--sk",
        &to_safekey_sk,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(key_has, "118.230000000");

    // deducted coins from sending SafeKey?
    let from_has = cmd!(
        get_bin_location(),
        "keys",
        "balance",
        "--sk",
        &from_safekey_sk,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_has, "1417.420000000" /* 1535.65 - 118.23 */)
}
