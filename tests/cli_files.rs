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
use common::{
    get_bin_location, get_random_nrs_string, parse_cat_files_container_output,
    parse_files_put_or_sync_output, CLI, SAFE_PROTOCOL,
};
use predicates::prelude::*;
use safe_api::XorUrlEncoder;
use std::fs;
use std::fs::OpenOptions;
use std::io::{prelude::*, Seek, SeekFrom};
use std::process::Command;
use unwrap::unwrap;

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

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_path("/aha/test.md");
    let file_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(file_cat, "hello tests!");

    xorurl_encoder.set_path("/aha/subfolder/subexists.md");
    let subfile_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(subfile_cat, "hello from a subfolder!");
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

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_path("/test.md");
    let file_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(file_cat, "hello tests!");

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_path("/subfolder/subexists.md");
    let subfile_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(subfile_cat, "hello from a subfolder!");
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

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_path("/testfolder/test.md");
    let file_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(file_cat, "hello tests!");

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_path("/testfolder/subfolder/subexists.md");
    let subfile_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(subfile_cat, "hello from a subfolder!");
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

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_path("/subexists.md");
    xorurl_encoder.set_content_version(Some(1));
    let synced_file_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(synced_file_cat, "hello from a subfolder!");
}

#[test]
fn calling_safe_files_removed_sync() {
    let files_container_output = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive",
        "--json"
    )
    .read()
    .unwrap();

    let (files_container_xor, processed_files) =
        parse_files_put_or_sync_output(&files_container_output);
    assert_eq!(processed_files.len(), 5);

    // let's first try with --dry-run and they should not be removed
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_content_version(None);
    let files_container_no_version = unwrap!(xorurl_encoder.to_string());
    let sync_cmd_output_dry_run = cmd!(
        get_bin_location(),
        "files",
        "sync",
        TEST_EMPTY_FOLDER, // rather than removing the files we pass an empty folder path
        &files_container_no_version,
        "--recursive",
        "--delete",
        "--dry-run",
        "--json",
    )
    .read()
    .unwrap();

    xorurl_encoder.set_content_version(Some(1));
    let files_container_v1 = unwrap!(xorurl_encoder.to_string());
    let (target, processed_files) = parse_files_put_or_sync_output(&sync_cmd_output_dry_run);
    assert_eq!(target, files_container_v1);
    assert_eq!(processed_files.len(), 5);

    let synced_file_cat = cmd!(get_bin_location(), "cat", &files_container_xor, "--json")
        .read()
        .unwrap();
    let (xorurl, files_map) = parse_cat_files_container_output(&synced_file_cat);
    assert_eq!(xorurl, files_container_xor);
    assert_eq!(files_map.len(), 5);

    // Now, let's try without --dry-run and they should be effectively removed
    let sync_cmd_output = cmd!(
        get_bin_location(),
        "files",
        "sync",
        TEST_EMPTY_FOLDER, // rather than removing the files we pass an empty folder path
        &files_container_no_version,
        "--recursive",
        "--delete",
        "--json",
    )
    .read()
    .unwrap();

    let (target, processed_files) = parse_files_put_or_sync_output(&sync_cmd_output);
    assert_eq!(target, files_container_v1);
    assert_eq!(processed_files.len(), 5);

    // now all files should be gone
    xorurl_encoder.set_content_version(None);
    let synced_file_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string()),
        "--json"
    )
    .read()
    .unwrap();
    let (xorurl, files_map) = parse_cat_files_container_output(&synced_file_cat);
    assert_eq!(xorurl, unwrap!(xorurl_encoder.to_string()));
    assert_eq!(files_map.len(), 0);
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

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_path("/subexists.md");
    xorurl_encoder.set_content_version(None);
    let file_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();

    // remove modified lines
    let mut replace_test_md = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file_to_modify)
        .unwrap();

    replace_test_md.seek(SeekFrom::Start(0)).unwrap();
    replace_test_md
        .write_all(b"hello from a subfolder!")
        .unwrap();

    // readd the removed missing file
    let mut readd_missing_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file_to_delete)
        .unwrap();

    readd_missing_file.seek(SeekFrom::Start(0)).unwrap();
    readd_missing_file.write_all(b"sub2").unwrap();

    // and now the tests...
    assert_eq!(file_cat, "hello from a subfolder! with more text!");
    assert!(files_sync_result.contains('*'));
    assert!(!files_sync_result.contains('+'));
}

#[test]
fn files_sync_and_fetch_with_version() {
    let files_container_output = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive",
        "--json"
    )
    .read()
    .unwrap();

    let (files_container_xor, processed_files) =
        parse_files_put_or_sync_output(&files_container_output);
    assert_eq!(processed_files.len(), 5);

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_content_version(None);
    let files_container_no_version = unwrap!(xorurl_encoder.to_string());
    let sync_cmd_output = cmd!(
        get_bin_location(),
        "files",
        "sync",
        TEST_EMPTY_FOLDER, // rather than removing the files we pass an empty folder path
        &files_container_no_version,
        "--recursive",
        "--delete",
        "--json",
    )
    .read()
    .unwrap();

    xorurl_encoder.set_content_version(Some(1));
    let files_container_v1 = unwrap!(xorurl_encoder.to_string());
    let (target, processed_files) = parse_files_put_or_sync_output(&sync_cmd_output);
    assert_eq!(target, files_container_v1);
    assert_eq!(processed_files.len(), 5);

    // now all files should be gone in version 1 of the FilesContainer
    let cat_container_v1 = cmd!(get_bin_location(), "cat", &files_container_v1, "--json")
        .read()
        .unwrap();
    let (xorurl, files_map) = parse_cat_files_container_output(&cat_container_v1);
    assert_eq!(xorurl, files_container_v1);
    assert_eq!(files_map.len(), 0);

    // but in version 0 of the FilesContainer all files should still be there
    xorurl_encoder.set_content_version(Some(0));
    let files_container_v0 = unwrap!(xorurl_encoder.to_string());
    let cat_container_v0 = cmd!(get_bin_location(), "cat", &files_container_v0, "--json")
        .read()
        .unwrap();
    let (xorurl, files_map) = parse_cat_files_container_output(&cat_container_v0);
    assert_eq!(xorurl, files_container_v0);
    assert_eq!(files_map.len(), 5);
}

#[test]
fn files_sync_and_fetch_with_nrsurl_and_nrs_update() {
    let files_container_output = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive",
        "--json"
    )
    .read()
    .unwrap();

    let (files_container_xor, processed_files) =
        parse_files_put_or_sync_output(&files_container_output);
    assert_eq!(processed_files.len(), 5);

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_content_version(Some(0));
    let files_container_v0 = &unwrap!(xorurl_encoder.to_string());
    let nrsurl = format!("safe://{}", get_random_nrs_string());

    let _ = cmd!(
        get_bin_location(),
        "nrs",
        "create",
        &nrsurl,
        "-l",
        &files_container_v0,
    )
    .read()
    .unwrap();

    let sync_cmd_output = cmd!(
        get_bin_location(),
        "files",
        "sync",
        TEST_EMPTY_FOLDER, // rather than removing the files we pass an empty folder path
        &nrsurl,
        "--recursive",
        "--delete",
        "--json",
        "--update-nrs"
    )
    .read()
    .unwrap();

    let (target, processed_files) = parse_files_put_or_sync_output(&sync_cmd_output);
    assert_eq!(target, nrsurl);
    assert_eq!(processed_files.len(), 5);

    // now all files should be gone in version 1 since NRS name was updated to link version 1 of the FilesContainer
    let cat_nrsurl_v1 = cmd!(get_bin_location(), "cat", &nrsurl, "--json")
        .read()
        .unwrap();
    let (xorurl, files_map) = parse_cat_files_container_output(&cat_nrsurl_v1);
    assert_eq!(xorurl, nrsurl);
    assert_eq!(files_map.len(), 0);

    // but in version 0 of the NRS name it should still link to version 0 of the FilesContainer
    // where all files should still be there
    let nrsurl_v0 = format!("{}?v=0", nrsurl);
    let cat_nrsurl_v0 = cmd!(get_bin_location(), "cat", &nrsurl_v0, "--json")
        .read()
        .unwrap();
    let (xorurl, files_map) = parse_cat_files_container_output(&cat_nrsurl_v0);
    assert_eq!(xorurl, nrsurl_v0);
    assert_eq!(files_map.len(), 5);
}

#[test]
fn files_sync_and_fetch_without_nrs_update() {
    let files_container_output = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive",
        "--json"
    )
    .read()
    .unwrap();

    let (files_container_xor, processed_files) =
        parse_files_put_or_sync_output(&files_container_output);
    assert_eq!(processed_files.len(), 5);
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_content_version(Some(0));
    let files_container_v0 = unwrap!(xorurl_encoder.to_string());
    let nrsurl = format!("safe://{}", get_random_nrs_string());

    let _ = cmd!(
        get_bin_location(),
        "nrs",
        "create",
        &nrsurl,
        "-l",
        &files_container_v0,
    )
    .read()
    .unwrap();

    let sync_cmd_output = cmd!(
        get_bin_location(),
        "files",
        "sync",
        TEST_EMPTY_FOLDER, // rather than removing the files we pass an empty folder path
        &nrsurl,
        "--recursive",
        "--delete",
        "--json",
    )
    .read()
    .unwrap();

    let (target, processed_files) = parse_files_put_or_sync_output(&sync_cmd_output);
    assert_eq!(target, nrsurl);
    assert_eq!(processed_files.len(), 5);

    // now all files should be gone in version 1 of the FilesContainer
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_content_version(Some(1));
    let files_container_v1 = unwrap!(xorurl_encoder.to_string());
    let cat_container_v1 = cmd!(get_bin_location(), "cat", &files_container_v1, "--json")
        .read()
        .unwrap();
    let (xorurl, files_map) = parse_cat_files_container_output(&cat_container_v1);
    assert_eq!(xorurl, files_container_v1);
    assert_eq!(files_map.len(), 0);

    // but the NRS name should still link to version 0 of the FilesContainer
    // where all files should still be there
    let cat_nrsurl = cmd!(get_bin_location(), "cat", &nrsurl, "--json")
        .read()
        .unwrap();
    let (xorurl, files_map) = parse_cat_files_container_output(&cat_nrsurl);
    assert_eq!(xorurl, nrsurl);
    assert_eq!(files_map.len(), 5);
}

#[test]
fn calling_safe_files_add() {
    let files_container_output = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive",
        "--json",
    )
    .read()
    .unwrap();

    let (files_container_xor, _processed_files) =
        parse_files_put_or_sync_output(&files_container_output);

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_content_version(None);
    let _ = cmd!(
        get_bin_location(),
        "files",
        "add",
        TEST_FILE,
        &format!("{}/new_test.md", xorurl_encoder),
    )
    .read()
    .unwrap();

    xorurl_encoder.set_path("/new_test.md");
    let synced_file_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(synced_file_cat, "hello tests!");
}

#[test]
fn calling_safe_files_add_a_url() {
    let files_container_output = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FOLDER,
        "--recursive",
        "--json"
    )
    .read()
    .unwrap();

    let (files_container_xor, processed_files) =
        parse_files_put_or_sync_output(&files_container_output);

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&files_container_xor));
    xorurl_encoder.set_content_version(None);
    xorurl_encoder.set_path("/new_test.md");
    let _ = cmd!(
        get_bin_location(),
        "files",
        "add",
        &processed_files[TEST_FILE].1,
        &unwrap!(xorurl_encoder.to_string()),
        "--json"
    )
    .read()
    .unwrap();

    let synced_file_cat = cmd!(
        get_bin_location(),
        "cat",
        &unwrap!(xorurl_encoder.to_string())
    )
    .read()
    .unwrap();
    assert_eq!(synced_file_cat, "hello tests!");
}
