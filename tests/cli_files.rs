// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

extern crate serde_json;
#[macro_use]
extern crate duct;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use std::process::Command;

static CLI: &str = "safe_cli";
static PRETTY_FILES_CREATION_RESPONSE: &str = "File uploaded to:";
static SAFE_PROTOCOL: &str = "safe://";
static TEST_FILE: &str = "./tests/testfolder/test.md";
static TEST_FOLDER: &str = "./tests/testfolder/";
static TEST_FOLDER_SUBFOLDER: &str = "./tests/testfolder/subfolder/";

fn get_bin_location() -> &'static str {
    let mut location = "./target/release/safe_cli";
    if cfg!(debug_assertions) {
        location = "./target/debug/safe_cli";
    }
    location
}

#[test]
fn calling_safe_files_put_pretty() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["files", "put", TEST_FILE, "--pretty"])
        .assert()
        .stdout(predicate::str::contains(PRETTY_FILES_CREATION_RESPONSE))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
        .stdout(predicate::str::contains(TEST_FILE).count(1))
        .success();
}

#[test]
fn calling_safe_files_put() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["files", "put", TEST_FILE])
        .assert()
        .stdout(predicate::str::contains(PRETTY_FILES_CREATION_RESPONSE).count(0))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
        .stdout(predicate::str::contains(TEST_FILE).count(1))
        .success();
}

#[test]
fn calling_safe_files_put_recursive() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["files", "put", TEST_FOLDER, "--recursive"])
        .assert()
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(3))
        .stdout(predicate::str::contains("./tests/testfolder/test.md").count(1))
        .stdout(predicate::str::contains("./tests/testfolder/another.md").count(1))
        .stdout(predicate::str::contains("./tests/testfolder/subfolder/subexists.md").count(1))
        .success();
}

#[test]
fn calling_safe_files_put_recursive_subfolder() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["files", "put", TEST_FOLDER_SUBFOLDER, "--recursive"])
        .assert()
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
        .stdout(predicate::str::contains("./tests/testfolder/test.md").count(0))
        .stdout(predicate::str::contains("./tests/testfolder/another.md").count(0))
        .stdout(predicate::str::contains("./tests/testfolder/subfolder/subexists.md").count(1))
        .success();
}
