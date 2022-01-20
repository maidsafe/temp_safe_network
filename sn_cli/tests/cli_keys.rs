// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

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
