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
use common::{get_bin_location, CLI, SAFE_PROTOCOL};
use predicates::prelude::*;
use std::process::Command;

use std::fs;
use std::fs::OpenOptions;
use std::io::{prelude::*, Seek, SeekFrom};

const PRETTY_FILES_CREATION_RESPONSE: &str = "FilesContainer created at: ";
const TEST_FILE: &str = "./tests/testfolder/test.md";
const TEST_FOLDER: &str = "./tests/testfolder/";
const TEST_FOLDER_NO_TRAILING_SLASH: &str = "./tests/testfolder";
const TEST_FOLDER_SUBFOLDER: &str = "./tests/testfolder/subfolder/";
const TEST_EMPTY_FOLDER: &str = "./tests/testfolder/emptyfolder/";

#[test]
fn calling_safe_files_put_pretty() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["files", "put", TEST_FILE])
        .assert()
        .stdout(predicate::str::contains(PRETTY_FILES_CREATION_RESPONSE))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(2))
        .stdout(predicate::str::contains(TEST_FILE).count(1))
        .success();
}

#[test]
fn calling_safe_files_put() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["files", "put", TEST_FILE, "--json"])
        .assert()
        .stdout(predicate::str::contains(PRETTY_FILES_CREATION_RESPONSE).count(0))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(2))
        .stdout(predicate::str::contains(TEST_FILE).count(1))
        .success();
}

#[test]
fn calling_safe_files_put_recursive() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["files", "put", TEST_FOLDER, "--recursive", "--json"])
        .assert()
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(6))
        .stdout(predicate::str::contains("./tests/testfolder/test.md").count(1))
        .stdout(predicate::str::contains("./tests/testfolder/another.md").count(1))
        .stdout(predicate::str::contains("./tests/testfolder/subfolder/subexists.md").count(1))
        .success();
}

#[test]
fn calling_safe_files_put_recursive_and_set_dest_path() {
    let files_container = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "/aha",
        "--recursive",
    )
    .read()
    .unwrap();

    let mut lines = files_container.lines();
    let files_container_xor_line = lines.next().unwrap();
    let files_container_xor =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace("\"", "");

    let file = format!("{}/aha/test.md", files_container_xor);
    let file_cat = cmd!(get_bin_location(), "cat", &file).read().unwrap();
    assert_eq!(file_cat, "hello tests!");

    let subfile = format!("{}/aha/subfolder/subexists.md", files_container_xor);
    let subfile_cat = cmd!(get_bin_location(), "cat", &subfile).read().unwrap();
    assert_eq!(subfile_cat, "the sub");
}

#[test]
fn calling_safe_files_put_recursive_subfolder() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "files",
        "put",
        TEST_FOLDER_SUBFOLDER,
        "--recursive",
        "--json",
    ])
    .assert()
    .stdout(predicate::str::contains(SAFE_PROTOCOL).count(3))
    .stdout(predicate::str::contains("./tests/testfolder/test.md").count(0))
    .stdout(predicate::str::contains("./tests/testfolder/another.md").count(0))
    .stdout(predicate::str::contains("./tests/testfolder/subfolder/subexists.md").count(1))
    .success();
}

#[test]
fn calling_safe_files_put_emptyfolder() {
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "files",
        "put",
        TEST_EMPTY_FOLDER,
        "--recursive",
        "--json",
    ])
    .assert()
    .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
    .stdout(predicate::str::contains("./tests/testfolder/emptyfolder/").count(0))
    .success();
}

#[test]
fn calling_safe_files_put_recursive_with_slash() {
    let files_container = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive"
    )
    .read()
    .unwrap();

    let mut lines = files_container.lines();
    let files_container_xor_line = lines.next().unwrap();
    let files_container_xor =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace("\"", "");

    let file = format!("{}/test.md", files_container_xor);
    let file_cat = cmd!(get_bin_location(), "cat", &file).read().unwrap();
    assert_eq!(file_cat, "hello tests!");

    let subfile = format!("{}/subfolder/subexists.md", files_container_xor);
    let subfile_cat = cmd!(get_bin_location(), "cat", &subfile).read().unwrap();
    assert_eq!(subfile_cat, "the sub");
}

#[test]
fn calling_safe_files_put_recursive_without_slash() {
    let files_container = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER_NO_TRAILING_SLASH,
        "--recursive"
    )
    .read()
    .unwrap();

    let mut lines = files_container.lines();
    let files_container_xor_line = lines.next().unwrap();
    let files_container_xor =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace("\"", "");

    let file = format!("{}/testfolder/test.md", files_container_xor);
    let file_cat = cmd!(get_bin_location(), "cat", &file).read().unwrap();
    assert_eq!(file_cat, "hello tests!");

    let subfile = format!("{}/testfolder/subfolder/subexists.md", files_container_xor);
    let subfile_cat = cmd!(get_bin_location(), "cat", &subfile).read().unwrap();
    assert_eq!(subfile_cat, "the sub");
}

#[test]
fn calling_safe_files_sync() {
    let files_container = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive"
    )
    .read()
    .unwrap();

    let mut lines = files_container.lines();
    let files_container_xor_line = lines.next().unwrap();
    let files_container_xor =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace("\"", "");

    let _ = cmd!(
        get_bin_location(),
        "files",
        "sync",
        TEST_FOLDER_SUBFOLDER,
        files_container_xor,
        "--recursive"
    )
    .read()
    .unwrap();

    let file = format!("{}/subexists.md", files_container_xor);
    let synced_file_cat = cmd!(get_bin_location(), "cat", &file).read().unwrap();
    assert_eq!(synced_file_cat, "the sub");
}

#[test]
fn calling_safe_files_put_recursive_with_slash_then_sync_after_modifications() {
    let files_container = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER_SUBFOLDER,
        "--recursive"
    )
    .read()
    .unwrap();

    let file_to_delete = format!("{}/sub2.md", TEST_FOLDER_SUBFOLDER);
    let file_to_modify = format!("{}/subexists.md", TEST_FOLDER_SUBFOLDER);

    let mut lines = files_container.lines();
    let files_container_xor_line = lines.next().unwrap();
    let files_container_xor =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace("\"", "");

    //modify file
    let file_to_modify_write = OpenOptions::new()
        .append(true)
        .open(&file_to_modify)
        .unwrap();

    if let Err(e) = writeln!(&file_to_modify_write, " with more text!") {
        eprintln!("Couldn't write to file: {}", e);
    }

    //remove another
    fs::remove_file(&file_to_delete).unwrap();

    // now sync
    let files_sync_result = cmd!(
        get_bin_location(),
        "files",
        "sync",
        TEST_FOLDER_SUBFOLDER,
        files_container_xor,
        "--recursive",
        // "--delete"
    )
    .read()
    .unwrap();

    let file = format!("{}/subexists.md", files_container_xor);
    let file_cat = cmd!(get_bin_location(), "cat", &file).read().unwrap();

    // remove modified lines
    let mut replace_test_md = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file_to_modify)
        .unwrap();

    replace_test_md.seek(SeekFrom::Start(0)).unwrap();
    replace_test_md.write_all(b"the sub").unwrap();

    // readd the removed missing file
    let mut readd_missing_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file_to_delete)
        .unwrap();

    readd_missing_file.seek(SeekFrom::Start(0)).unwrap();
    readd_missing_file.write_all(b"sub2").unwrap();

    // and now the tests...
    assert_eq!(file_cat, "the sub with more text!");
    assert!(files_sync_result.contains('*'));
    assert!(!files_sync_result.contains('+'));
}
