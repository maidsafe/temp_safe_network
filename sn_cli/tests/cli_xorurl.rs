// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[macro_use]
extern crate duct;

use assert_cmd::prelude::*;
use color_eyre::{eyre::eyre, Result};
use predicates::prelude::*;
use sn_api::SafeUrl;
use sn_cmd_test_utilities::util::{parse_xorurl_output, CLI, SAFE_PROTOCOL};
use std::process::Command;

const TEST_FILE: &str = "../resources/testdata/test.md";
const TEST_FOLDER: &str = "../resources/testdata/";

#[test]
fn calling_safe_xorurl() -> Result<()> {
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec!["xorurl", TEST_FILE])
        .assert()
        .stdout(predicate::str::contains("1 file/s processed"))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
        .stdout(predicate::str::contains(TEST_FILE).count(1))
        .success();
    Ok(())
}

#[test]
fn calling_safe_xorurl_recursive() -> Result<()> {
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec!["xorurl", TEST_FOLDER, "--recursive"])
        .assert()
        .stdout(predicate::str::contains("12 file/s processed"))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(9))
        .stdout(predicate::str::contains(TEST_FOLDER).count(12))
        .success();
    Ok(())
}

#[test]
fn calling_safe_xorurl_decode() -> Result<()> {
    let content = cmd!(env!("CARGO_BIN_EXE_safe"), "xorurl", TEST_FILE, "--json",)
        .read()
        .map_err(|e| eyre!(e.to_string()))?;

    let xorurls = parse_xorurl_output(&content)?;
    let file_xorurl = &xorurls[0].1;
    let safeurl = SafeUrl::from_url(file_xorurl)?;

    let xorurl_decoded = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "xorurl",
        "decode",
        &file_xorurl,
        "--json",
    )
    .read()
    .map_err(|e| eyre!(e.to_string()))?;

    let decoded_obj: SafeUrl = serde_json::from_str(&xorurl_decoded)
        .expect("Failed to parse output of `safe xorurl decode`");

    assert_eq!(safeurl, decoded_obj);
    Ok(())
}
