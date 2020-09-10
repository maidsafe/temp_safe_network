// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

extern crate sn_cmd_test_utilities;

#[macro_use]
extern crate duct;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use sn_cmd_test_utilities::{
    create_preload_and_get_keys, create_wallet_with_balance, get_random_nrs_string, CLI,
    SAFE_PROTOCOL,
};
use std::process::Command;

const PRETTY_WALLET_CREATION_RESPONSE: &str = "Wallet created at";
const UNMATCHED_SK_XORURL: &str =
    "The URL doesn't correspond to the public key derived from the provided secret key";

#[test]
fn calling_safe_wallet_transfer() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_from, _pk, _sk) = create_wallet_with_balance("160.000000001", None); // we need 1 nano to pay for the costs of creation
    assert!(wallet_from.contains(SAFE_PROTOCOL));
    let (wallet_to, _pk, _sk) = create_wallet_with_balance("5.000000001", None); // we need 1 nano to pay for the costs of creation
    assert!(wallet_to.contains(SAFE_PROTOCOL));

    // To got coins?
    let to_starts_with = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_to,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(to_starts_with, "5.000000000");

    // From got coins?
    let from_starts_with = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_from,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_starts_with, "160.000000000");

    cmd.args(&vec![
        "wallet",
        "transfer",
        "100",
        "--from",
        &wallet_from,
        "--to",
        &wallet_to,
    ])
    .assert()
    .stdout(predicate::str::contains("Success"))
    .stdout(predicate::str::contains("TX_ID"))
    .success();

    // To got coins?
    let to_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
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
        env!("CARGO_BIN_EXE_safe"),
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
fn calling_safe_wallet_transfer_spendable_balance_urls() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_from, _pk, _sk) =
        create_wallet_with_balance("96.000000001", Some("one-spendable-balance")); // we need 1 nano to pay for the costs of creation
    assert!(wallet_from.contains(SAFE_PROTOCOL));
    let (wallet_to, _pk, _sk) = create_wallet_with_balance("5.100000001", Some("my-savings")); // we need 1 nano to pay for the costs of creation
    assert!(wallet_to.contains(SAFE_PROTOCOL));

    // To got coins?
    let to_spendable_balance = format!("{}/my-savings", wallet_to);
    let to_starts_with = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &to_spendable_balance,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(to_starts_with, "5.100000000");

    // From got coins?
    let from_spendable_balance = format!("{}/one-spendable-balance", wallet_from);
    let from_starts_with = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &from_spendable_balance,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_starts_with, "96.000000000");

    cmd.args(&vec![
        "wallet",
        "transfer",
        "50",
        "--from",
        &from_spendable_balance,
        "--to",
        &to_spendable_balance,
    ])
    .assert()
    .stdout(predicate::str::contains("Success"))
    .stdout(predicate::str::contains("TX_ID"))
    .success();

    // To got coins?
    let to_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_to,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(to_has, "55.100000000");

    // from lost coins?
    let from_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_from,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_has, "46.000000000")
}

#[test]
fn calling_safe_wallet_unexisting_spendable_balances() {
    let (wallet_from, _pk, _sk) =
        create_wallet_with_balance("96.000000001", Some("one-spendable-balance")); // we need 1 nano to pay for the costs of creation
    assert!(wallet_from.contains(SAFE_PROTOCOL));
    let (wallet_to, _pk, _sk) = create_wallet_with_balance("5.100000001", Some("my-savings")); // we need 1 nano to pay for the costs of creation
    assert!(wallet_to.contains(SAFE_PROTOCOL));

    // Test failure when transferring from an unexisting source spendable balance
    let from_unexisting_balance = format!("{}/from-unexisting-spendable-balance", wallet_from);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "wallet",
        "transfer",
        "50",
        "--from",
        &from_unexisting_balance,
        "--to",
        &format!("{}/my-savings", wallet_to),
    ])
    .assert()
    .stderr(predicate::str::contains(&format!(
        "No spendable balance named 'from-unexisting-spendable-balance' found in Wallet: '{}'",
        from_unexisting_balance
    )))
    .failure();

    // Test failure when transferring to an unexisting destination spendable balance
    let to_unexisting_balance = format!("{}/to-unexisting-spendable-balance", wallet_to);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "wallet",
        "transfer",
        "50",
        "--from",
        &format!("{}/one-spendable-balance", wallet_from),
        "--to",
        &to_unexisting_balance,
    ])
    .assert()
    .stderr(predicate::str::contains(&format!(
        "No spendable balance named 'to-unexisting-spendable-balance' found in Wallet: '{}'",
        to_unexisting_balance
    )))
    .failure();

    // Test failure when checking balance of an unexisting spendable balance
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["wallet", "balance", &from_unexisting_balance])
        .assert()
        .stderr(predicate::str::contains(&format!(
            "No spendable balance named 'from-unexisting-spendable-balance' found in Wallet: '{}'",
            from_unexisting_balance
        )))
        .failure();
}

#[test]
fn calling_safe_wallet_balance() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("10.000000001", None); // we need 1 nano to pay for the costs of creation

    cmd.args(&vec!["wallet", "balance", &wallet_xor, "--json"])
        .assert()
        .stdout("10.000000000\n")
        .success();
}

#[test]
fn calling_safe_wallet_balance_with_spendable_balance_url() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_xor, _pk, _sk) =
        create_wallet_with_balance("453.006000001", Some("my-first-balance")); // we need 1 nano to pay for the costs of creation

    let spendable_balace_xorurl = format!("{}/my-first-balance", wallet_xor);
    cmd.args(&vec![
        "wallet",
        "balance",
        &spendable_balace_xorurl,
        "--json",
    ])
    .assert()
    .stdout("453.006000000\n")
    .success();
}

#[test]
fn calling_safe_wallet_insert() {
    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("50", None);
    let (key_pk_xor, sk) = create_preload_and_get_keys("300.000000001"); // we need 1 nano to pay for the costs of creation

    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let _wallet_insert_result = cmd!(
        env!("CARGO_BIN_EXE_safe"),
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

    cmd.args(&vec!["wallet", "create"]).assert().success();
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
    let (wallet_xor, _pk, _sk) = create_wallet_with_balance("55.000000001", None); // we need 1 nano to pay for the costs of creation

    let balance = cmd!(
        env!("CARGO_BIN_EXE_safe"),
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
        env!("CARGO_BIN_EXE_safe"),
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

    let (wallet_xorurl, _): (String, String) = serde_json::from_str(&wallet_create_result)
        .expect("Failed to parse output of `safe wallet create`");

    let balance = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_xorurl,
        "--json"
    )
    .read()
    .unwrap();
    assert_eq!("300.000000000", balance);
}

#[test]
fn calling_safe_wallet_create_w_sk_only() {
    let (_xorurl, sk) = create_preload_and_get_keys("333");

    let wallet_create_result = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "create",
        "--sk",
        sk,
        "--json"
    )
    .read()
    .unwrap();

    let (wallet_xorurl, _): (String, String) = serde_json::from_str(&wallet_create_result)
        .expect("Failed to parse output of `safe wallet create`");

    let balance = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_xorurl,
        "--json"
    )
    .read()
    .unwrap();
    assert_eq!("333.000000000", balance);
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
fn calling_safe_wallet_create_w_bad_location() {
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
        "Content not found at safe://nononooOOooo",
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

#[test]
fn calling_safe_wallet_transfer_to_key_xorurl() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_from, _pk, _sk) = create_wallet_with_balance("35.650000001", None); // we need 1 nano to pay for the costs of creation
    let (key_xorurl, key_sk) = create_preload_and_get_keys("0.0");

    cmd.args(&vec![
        "wallet",
        "transfer",
        "18.23",
        "--from",
        &wallet_from,
        "--to",
        &key_xorurl,
    ])
    .assert()
    .stdout(predicate::str::contains("Success"))
    .stdout(predicate::str::contains("TX_ID"))
    .success();

    // SafeKey got coins?
    let key_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "keys",
        "balance",
        "--sk",
        &key_sk,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(key_has, "18.230000000");

    // deducted coins from sending Wallet?
    let from_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_from,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_has, "17.420000000" /* 35.65 - 18.23 */)
}

#[test]
fn calling_safe_wallet_transfer_to_key_nrsurl() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();

    let (wallet_from, _pk, _sk) = create_wallet_with_balance("1535.650000001", None); // we need 1 nano to pay for the costs of creation
    let (key_xorurl, key_sk) = create_preload_and_get_keys("0.0");

    let key_nrsurl = format!("safe://{}", get_random_nrs_string());
    let _ = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "nrs",
        "create",
        &key_nrsurl,
        "-l",
        &key_xorurl,
    )
    .read()
    .unwrap();

    cmd.args(&vec![
        "wallet",
        "transfer",
        "118.23",
        "--from",
        &wallet_from,
        "--to",
        &key_nrsurl,
    ])
    .assert()
    .stdout(predicate::str::contains("Success"))
    .stdout(predicate::str::contains("TX_ID"))
    .success();

    // SafeKey got coins?
    let key_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "keys",
        "balance",
        "--sk",
        &key_sk,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(key_has, "118.230000000");

    // deducted coins from sending Wallet?
    let from_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_from,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(from_has, "1417.420000000" /* 1535.65 - 118.23 */)
}

#[test]
fn calling_safe_wallet_balance_with_nrsurl() {
    let (wallet_xorurl, _pk, _sk) =
        create_wallet_with_balance("1.120000001", Some("for-night-outs")); // we need 1 nano to pay for the costs of creation
    let wallet_nrsurl = format!("safe://{}", get_random_nrs_string());
    let _ = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "nrs",
        "create",
        &wallet_nrsurl,
        "-l",
        &wallet_xorurl,
    )
    .read()
    .unwrap();

    // check wallet balance with NRS url
    let wallet_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_nrsurl,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(wallet_has, "1.120000000");

    // check wallet's spendable balance with NRS url with
    let wallet_spendable_balance = format!("{}/for-night-outs", wallet_nrsurl);
    let spendable_balance_has = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "wallet",
        "balance",
        &wallet_spendable_balance,
        "--json"
    )
    .read()
    .unwrap();

    assert_eq!(spendable_balance_has, "1.120000000");
}
