// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use color_eyre::{eyre::eyre, Result};
use predicates::prelude::*;
use sn_api::resolver::{ContentType, DataType, SafeUrl};
use sn_cmd_test_utilities::util::{
    create_and_get_keys, get_random_nrs_string, parse_files_container_output,
    parse_files_put_or_sync_output, parse_nrs_register_output, safe_cmd, safe_cmd_stderr,
    safe_cmd_stdout, safeurl_from, test_symlinks_are_valid, upload_path,
    upload_test_symlinks_folder, CLI,
};
use std::{path::Path, process::Command};

const TEST_DATA: &str = "../resources/testdata/";
const TEST_FILE: &str = "../resources/testdata/test.md";
const TEST_FILE_CONTENT: &str = "hello tests!";
const ID_RELATIVE_FILE_ERROR: &str = "Cannot get relative path of Immutable Data";
const TEST_FILE_HEXDUMP_CONTENT: &str = "Length: 12 (0xc) bytes\n0000:   68 65 6c 6c  6f 20 74 65  73 74 73 21                hello tests!\n";

/// A 'spot' file is one that's less than 3072 bytes in size.
/// These small files are rejected by self encryption and need another way to be stored.
#[test]
fn calling_safe_cat_using_spot_file() -> Result<()> {
    let content = safe_cmd_stdout(
        ["files", "put", "../resources/testdata/test.md", "--json"],
        Some(0),
    )?;

    let (_, processed_files) = parse_files_put_or_sync_output(&content)?;
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec![
        "cat",
        processed_files[Path::new("../resources/testdata/test.md")]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
    ])
    .assert()
    .stdout(predicate::str::contains(TEST_FILE_CONTENT))
    .success();

    let safeurl = safeurl_from(
        processed_files[Path::new("../resources/testdata/test.md")]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
    )?;
    assert_eq!(
        safeurl.content_type(),
        ContentType::MediaType("text/markdown".to_string())
    );
    assert_eq!(safeurl.data_type(), DataType::File);
    Ok(())
}

/// A 'large' file is one that's larger than 3072 bytes in size.
/// These use self encryption and are stored in a different way from 'small' files.
#[test]
fn calling_safe_cat_using_large_file() -> Result<()> {
    let output = safe_cmd_stdout(
        [
            "files",
            "put",
            "../resources/testdata/large_markdown_file.md",
            "--json",
        ],
        Some(0),
    )?;

    let content = std::fs::read_to_string("../resources/testdata/large_markdown_file.md")?;
    let (_, processed_files) = parse_files_put_or_sync_output(&output)?;
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec![
        "cat",
        processed_files[Path::new("../resources/testdata/large_markdown_file.md")]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
    ])
    .assert()
    .stdout(predicate::str::contains(content))
    .success();

    let safeurl = safeurl_from(
        processed_files[Path::new("../resources/testdata/large_markdown_file.md")]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
    )?;
    assert_eq!(
        safeurl.content_type(),
        ContentType::MediaType("text/markdown".to_string())
    );
    assert_eq!(safeurl.data_type(), DataType::File);
    Ok(())
}

#[test]
fn calling_safe_cat_subfolders() -> Result<()> {
    let content = safe_cmd_stdout(
        ["files", "put", TEST_DATA, "--json", "--recursive"],
        Some(0),
    )?;

    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let content = safe_cmd_stdout(["cat", &container_xorurl, "--json"], Some(0))?;
    let (_xorurl, filesmap) = parse_files_container_output(&content)?;

    assert_eq!(filesmap["/emptyfolder"]["type"], "inode/directory");
    assert_eq!(filesmap["/emptyfolder"]["size"], "0");
    assert_eq!(filesmap["/subfolder"]["type"], "inode/directory");
    assert_eq!(filesmap["/subfolder"]["size"], "0");
    Ok(())
}

#[test]
fn calling_safe_cat_on_relative_file_from_id_fails() -> Result<()> {
    let content = safe_cmd_stdout(["files", "put", TEST_FILE, "--json"], Some(0))?;

    let (_, processed_files) = parse_files_put_or_sync_output(&content)?;
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;

    let relative_url = format!(
        "{}/something_relative.wasm",
        &processed_files[Path::new(TEST_FILE)]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?
    );
    cmd.args(&vec!["cat", &relative_url])
        .assert()
        .stderr(predicate::str::contains(ID_RELATIVE_FILE_ERROR))
        .failure();
    Ok(())
}

#[test]
fn calling_safe_cat_hexdump() -> Result<()> {
    let content = safe_cmd_stdout(["files", "put", TEST_FILE, "--json"], Some(0))?;

    let (_, processed_files) = parse_files_put_or_sync_output(&content)?;
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec![
        "cat",
        "--hexdump",
        processed_files[Path::new(TEST_FILE)]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
    ])
    .assert()
    .stdout(predicate::str::contains(TEST_FILE_HEXDUMP_CONTENT))
    .success();

    let safeurl = safeurl_from(
        processed_files[Path::new(TEST_FILE)]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
    )?;
    assert_eq!(
        safeurl.content_type(),
        ContentType::MediaType("text/markdown".to_string())
    );
    assert_eq!(safeurl.data_type(), DataType::File);
    Ok(())
}

#[test]
fn calling_safe_cat_xorurl_with_version() -> Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let md_file = tmp_dir.child("test.md");
    md_file.write_str("hello tests!")?;

    let output = safe_cmd_stdout(
        [
            "files",
            "put",
            &md_file.path().display().to_string(),
            "--json",
        ],
        Some(0),
    )?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&output)?;
    let mut url = SafeUrl::from_url(&container_xorurl)?;
    url.set_path("test.md");

    safe_cmd(["cat", &url.to_string()], Some(0))?
        .assert()
        .stdout(predicate::str::contains("hello tests!"));

    Ok(())
}

#[test]
fn calling_safe_cat_nrsurl_with_version() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let public_name = format!("test.{}", get_random_nrs_string());
    let output = safe_cmd_stdout(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(), // to_string has the version on the url
            "--register-top-name",
            "--json",
        ],
        Some(0),
    )?;

    let (nrs_url, _) = parse_nrs_register_output(&output)?;
    let version = nrs_url.content_version();

    let mut nrs_url = SafeUrl::from_url(&format!("safe://{}", public_name))?;
    nrs_url.set_content_version(version);
    safe_cmd(["cat", &nrs_url.to_string()], Some(0))?
        .assert()
        .stdout(predicate::str::contains("hello tests!"));

    Ok(())
}

#[test]
fn calling_safe_cat_safekey() -> Result<()> {
    let (safekey_xorurl, _sk) = create_and_get_keys()?;
    let cat_output = safe_cmd_stdout(["cat", &safekey_xorurl], Some(0))?;
    assert_eq!(cat_output, "No content to show since the URL targets a SafeKey. Use the 'dog' command to obtain additional information about the targeted SafeKey.");
    Ok(())
}

// Test:  safe cat <src>/<path>
//    src is symlinks_test dir, put with trailing slash.
//    path references both directory and file relative symlinks
//         including parent dir and sibling dir link targets.
//         Final destination is the file sibling_dir_file.md
//         which is itself a symlink to hello.md.
//
//         realpath: /sub2/hello.md
//
//    expected result: cmd output matches contents of
//                     ./test_symlinks/sub2/hello.md
#[test]
fn calling_cat_symlinks_resolve_dir_and_file() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let (url, ..) = upload_test_symlinks_folder(true)?;
    let mut safeurl = safeurl_from(&url)?;
    safeurl.set_path("/dir_link_link/parent_dir/dir_link/sibling_dir_file.md");

    let output = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    assert_eq!(output, "= Hello =");
    Ok(())
}

// Test:  safe cat <src>/<path>
//    src is symlinks_test dir, put with trailing slash.
//    path references a symlink that links to itself.
//         (infinite loop)
//
//    expected result: error, too many links.
#[test]
fn calling_cat_symlinks_resolve_infinite_loop() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let (url, ..) = upload_test_symlinks_folder(true)?;
    let mut safeurl = safeurl_from(&url)?;

    safeurl.set_path("/sub/infinite_loop");
    let output = safe_cmd_stderr(["cat", &safeurl.to_string()], Some(1))?;
    assert!(output.contains("ContentNotFound: Too many levels of symbolic links"));

    Ok(())
}

// Test:  safe cat <src>/dir_link_deep/../readme.md
//    src is symlinks_test dir, put with trailing slash.
//    path should resolve as follows:
//         dir_link_deep --> sub/deep
//         ../           --> sub
//         readme.md     --> readme.md
//
//         realpath: /sub/readme.md
//
//    This test verifies that "../" is being resolved
//    correctly *after* dir_link_deep resolution, not before.
//
//    On unix, this behavior can be verified with:
//       $ cat ./test_symlinks/dir_link_deep/../readme.md
//       = This is a real markdown file. =
//
//    note: This test always failed when SafeUrl
//          used rust-url for parsing path because it
//          normalizes away the "../" with no option
//          to obtain the raw path.
//          filed issue: https://github.com/servo/rust-url/issues/602
//
//    expected result: cmd output matches contents of
//                     /sub/readme.md
#[test]
fn calling_cat_symlinks_resolve_parent_dir() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let (url, ..) = upload_test_symlinks_folder(true)?;
    let mut safeurl = safeurl_from(&url)?;

    safeurl.set_path("/dir_link_deep/../readme.md");
    let output = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    assert_eq!(output, "= This is a real markdown file. =");

    Ok(())
}

// Test:  safe cat <src>/dir_outside
//    src is symlinks_test dir, put with trailing slash.
//    path references a symlink with target outside the FileContainer
//
//    expected result: error, too many links.
#[test]
fn calling_cat_symlinks_resolve_dir_outside() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let (url, ..) = upload_test_symlinks_folder(true)?;
    let mut safeurl = safeurl_from(&url)?;

    safeurl.set_path("/dir_outside");
    let output = safe_cmd_stderr(["cat", &safeurl.to_string()], Some(1))?;
    assert!(output.contains("ContentNotFound: Cannot ascend beyond root directory"));

    Ok(())
}
