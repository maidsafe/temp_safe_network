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
use sn_api::{PublicKey, SafeUrl};
use sn_cmd_test_utilities::util::{parse_keys_create_output, safe_cmd_stdout, CLI, SAFE_PROTOCOL};
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
    let pk_cmd_result = safe_cmd_stdout(["keys", "create", "--json"], Some(0))?;
    let (_, (pk_hex, _)): (SafeUrl, (String, String)) = parse_keys_create_output(&pk_cmd_result)?;
    let result = PublicKey::bls_from_hex(&pk_hex);
    // At the moment there is a problem parsing a BLS SecretKey from hex, so for now we'll just
    // parse the public one.
    assert!(result.is_ok());
    Ok(())
}
