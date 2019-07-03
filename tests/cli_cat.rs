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
use common::{get_bin_location, CLI};
use predicates::prelude::*;
use serde_json;
use std::collections::BTreeMap;
use std::process::Command;

const OUR_DATA: &str = "hello tests!";
const TEST_FILE: &str = "./tests/testfolder/test.md";

#[test]
fn calling_safe_cat() {
    let content = cmd!(get_bin_location(), "files", "put", TEST_FILE, "--json",)
        .read()
        .unwrap();

    let (_container_xorurl, map): (String, BTreeMap<String, (String, String)>) =
        match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(err) => panic!(format!(
                "Failed to parse output of `safe file put`: {}",
                err
            )),
        };

    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &map[TEST_FILE].1])
        .assert()
        .stdout(predicate::str::contains(OUR_DATA))
        .success();
}
