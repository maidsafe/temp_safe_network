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
use common::{create_preload_and_get_keys, CLI, SAFE_PROTOCOL};
use predicates::prelude::*;
use std::process::Command;

static PRETTY_KEYS_CREATION_RESPONSE: &str = "New Key created at:";

#[test]
fn calling_safe_keys_create_pretty() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "keys",
        "create",
        "--test-coins",
        "--preload",
        "123",
        "--pretty",
    ])
    .assert()
    .stdout(predicate::str::contains(PRETTY_KEYS_CREATION_RESPONSE))
    .stdout(predicate::str::contains(SAFE_PROTOCOL).from_utf8())
    .success();
}

#[test]
fn calling_safe_keys_create() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["keys", "create", "--test-coins", "--preload", "123"])
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
        .stdout(predicate::str::contains("sk="))
        .stdout(predicate::str::contains("pk="))
        .success();
}

#[test]
fn calling_safe_keypair_pretty() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["keypair", "--pretty"])
        .assert()
        .stdout(predicate::str::contains("Key pair generated:"))
        .stdout(predicate::str::contains("sk="))
        .stdout(predicate::str::contains("pk="))
        .success();
}

#[test]
fn calling_safe_keys_balance() {
    let (pk_xor, _sk) = create_preload_and_get_keys("123");

    assert!(pk_xor.contains("safe://"));

    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["keys", "balance", &pk_xor])
        .assert()
        .stdout("123\n")
        .success();
}
