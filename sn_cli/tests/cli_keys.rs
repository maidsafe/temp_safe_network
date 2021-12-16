// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use assert_cmd::prelude::*;
use color_eyre::{eyre::eyre, Result};
use predicates::prelude::*;
use sn_cmd_test_utilities::util::{CLI, SAFE_PROTOCOL};
use std::process::Command;

const PRETTY_KEYS_CREATION_RESPONSE: &str = "New SafeKey created:";

#[test]
fn calling_safe_keys_create_pretty() -> Result<()> {
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec!["keys", "create"])
        .assert()
        .stdout(predicate::str::contains(PRETTY_KEYS_CREATION_RESPONSE))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).from_utf8())
        .success();
    Ok(())
}

#[test]
fn calling_safe_keys_create() -> Result<()> {
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec!["keys", "create", "--json"])
        .assert()
        .stdout(predicate::str::contains(PRETTY_KEYS_CREATION_RESPONSE).count(0))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).from_utf8())
        .success();
    Ok(())
}
