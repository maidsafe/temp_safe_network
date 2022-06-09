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
use sn_api::SafeUrl;
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
fn keys_create_with_json_should_output_keys_and_url() -> Result<()> {
    let pk_cmd_result = safe_cmd_stdout(["keys", "create", "--json"], Some(0))?;
    let (_, (pk_hex, sk_hex)): (SafeUrl, (String, String)) =
        parse_keys_create_output(&pk_cmd_result)?;
    // You could write these using `assert!(result.is_ok())` but the problem is you don't get the
    // failure message, so we do it in this more verbose fashion. We don't actually care what the
    // parsed values are, just that the outputs from the command get parsed correctly.
    match bls::PublicKey::from_hex(&pk_hex.trim()) {
        Ok(_) => {}
        Err(e) => {
            return Err(eyre!(e));
        }
    }
    match bls::SecretKey::from_hex(&sk_hex.trim()) {
        Ok(_) => {}
        Err(e) => {
            return Err(eyre!(e));
        }
    }
    Ok(())
}

#[test]
fn keys_show_should_output_url_and_public_key() -> Result<()> {
    let output = safe_cmd_stdout(["keys", "show", "--json"], Some(0))?;
    let lines: Vec<&str> = output.split("\n").collect();
    let (_, url) = lines[1]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a URL"))?;
    let (_, pk_hex) = lines[2]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a public key in hex"))?;

    match SafeUrl::from_url(url.trim()) {
        Ok(_) => {}
        Err(e) => {
            return Err(eyre!(e));
        }
    }
    match bls::PublicKey::from_hex(&pk_hex.trim()) {
        Ok(_) => {}
        Err(e) => {
            return Err(eyre!(e));
        }
    }
    Ok(())
}

#[test]
fn keys_show_should_output_url_public_key_and_secret_key() -> Result<()> {
    let output = safe_cmd_stdout(["keys", "show", "--show-sk", "--json"], Some(0))?;
    let lines: Vec<&str> = output.split("\n").collect();
    let (_, url) = lines[1]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a URL"))?;
    let (_, pk_hex) = lines[2]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a public key in hex"))?;
    let (_, sk_hex) = lines[3]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a secret key in hex"))?;

    match SafeUrl::from_url(url.trim()) {
        Ok(_) => {}
        Err(e) => {
            return Err(eyre!(e));
        }
    }
    match bls::PublicKey::from_hex(&pk_hex.trim()) {
        Ok(_) => {}
        Err(e) => {
            return Err(eyre!(e));
        }
    }
    match bls::SecretKey::from_hex(&sk_hex.trim()) {
        Ok(_) => {}
        Err(e) => {
            return Err(eyre!(e));
        }
    }
    Ok(())
}
