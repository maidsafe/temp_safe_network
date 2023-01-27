// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use color_eyre::{eyre::eyre, Result};
use predicates::prelude::*;
use sn_api::resolver::SafeUrl;
use sn_cmd_test_utilities::util::{
    get_bearer_dbc_on_file, get_random_string, parse_files_container_output,
    parse_files_put_or_sync_output, parse_nrs_register_output, parse_wallet_create_output,
    safe_cmd, safe_cmd_stderr, safe_cmd_stdout, test_symlinks_are_valid, upload_path,
    upload_test_symlinks_folder, use_isolated_safe_config_dir, CLI,
};
use std::path::{Path, PathBuf};
use std::process::Command;

const TEST_DATA: &str = "../resources/testdata/";
const TEST_FILE: &str = "../resources/testdata/test.md";
const TEST_FILE_CONTENT: &str = "hello tests!";
const ID_RELATIVE_FILE_ERROR: &str = "Cannot get relative path of Immutable Data";
const TEST_FILE_HEXDUMP_CONTENT: &str = "Length: 12 (0xc) bytes\n0000:   68 65 6c 6c  6f 20 74 65  73 74 73 21                hello tests!\n";

/// A 'spot' file is one that's less than 3072 bytes in size.
/// These small files are rejected by self encryption and need another way to be stored.
#[test]
fn cat_should_display_file_contents_when_url_points_to_spot_file() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let content = safe_cmd_stdout(
        &config_dir,
        ["files", "put", "../resources/testdata/test.md", "--json"],
        Some(0),
    )?;

    let (_, processed_files) = parse_files_put_or_sync_output(&content)?;
    safe_cmd(
        &config_dir,
        [
            "cat",
            processed_files[Path::new("../resources/testdata/test.md")]
                .link()
                .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(TEST_FILE_CONTENT))
    .success();

    Ok(())
}

/// A 'large' file is one that's larger than 3072 bytes in size.
/// These use self encryption and are stored in a different way from 'small' files.
#[test]
fn cat_should_display_file_contents_when_url_points_to_large_file() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let output = safe_cmd_stdout(
        &config_dir,
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
    safe_cmd(
        &config_dir,
        [
            "cat",
            processed_files[Path::new("../resources/testdata/large_markdown_file.md")]
                .link()
                .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(content))
    .success();

    Ok(())
}

#[test]
fn cat_should_display_folder_contents() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let content = safe_cmd_stdout(
        &config_dir,
        ["files", "put", TEST_DATA, "--json", "--recursive"],
        Some(0),
    )?;

    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let content = safe_cmd_stdout(&config_dir, ["cat", &container_xorurl, "--json"], Some(0))?;
    let (_xorurl, filesmap) = parse_files_container_output(&content)?;

    assert_eq!(filesmap["/emptyfolder"]["type"], "inode/directory");
    assert_eq!(filesmap["/emptyfolder"]["size"], "0");
    assert_eq!(filesmap["/subfolder"]["type"], "inode/directory");
    assert_eq!(filesmap["/subfolder"]["size"], "0");

    Ok(())
}

#[test]
fn cat_should_fail_when_file_is_not_in_container() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let content = safe_cmd_stdout(&config_dir, ["files", "put", TEST_FILE, "--json"], Some(0))?;

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
fn cat_should_display_file_contents_as_hex_when_hex_arg_is_used() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let content = safe_cmd_stdout(&config_dir, ["files", "put", TEST_FILE, "--json"], Some(0))?;

    let (_, processed_files) = parse_files_put_or_sync_output(&content)?;
    safe_cmd(
        &config_dir,
        [
            "cat",
            "--hexdump",
            processed_files[Path::new(TEST_FILE)]
                .link()
                .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(TEST_FILE_HEXDUMP_CONTENT))
    .success();

    Ok(())
}

#[test]
fn cat_should_display_file_contents_when_xor_url_is_used() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let tmp_dir = assert_fs::TempDir::new()?;
    let md_file = tmp_dir.child("test.md");
    md_file.write_str("hello tests!")?;

    let output = safe_cmd_stdout(
        &config_dir,
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

    safe_cmd(&config_dir, ["cat", &url.to_string()], Some(0))?
        .assert()
        .stdout(predicate::str::contains("hello tests!"));

    Ok(())
}

#[test]
fn cat_should_display_file_contents_when_nrs_url_has_version() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&config_dir, &tmp_data_path, with_trailing_slash)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let public_name = format!("test.{}", get_random_string());
    let output = safe_cmd_stdout(
        &config_dir,
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

    let (_, nrs_url, _) = parse_nrs_register_output(&output)?;
    let version = nrs_url.content_version();

    let mut nrs_url = SafeUrl::from_url(&format!("safe://{public_name}"))?;
    nrs_url.set_content_version(version);
    safe_cmd(&config_dir, ["cat", &nrs_url.to_string()], Some(0))?
        .assert()
        .stdout(predicate::str::contains("hello tests!"));

    Ok(())
}

#[test]
fn cat_should_display_file_contents_when_nrs_url_has_no_safe_prefix() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&config_dir, &tmp_data_path, with_trailing_slash)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let public_name = format!("test.{}", get_random_string());
    safe_cmd(
        &config_dir,
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

    safe_cmd(&config_dir, ["cat", &public_name], Some(0))?
        .assert()
        .stdout(predicate::str::contains("hello tests!"));

    Ok(())
}

#[test]
fn cat_should_display_file_contents_when_nrs_url_points_to_file() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let markdown_file = tmp_data_path.child("another.md");
    let (_files_container_xor, processed_files, _) =
        upload_path(&config_dir, &tmp_data_path, with_trailing_slash)?;

    let change = processed_files
        .get(&markdown_file.path().to_path_buf())
        .ok_or_else(|| eyre!("Could not retrieve markdown file from processed files"))?;
    let file_url = change
        .link()
        .ok_or_else(|| eyre!("Could not retrieve URL from processed file change"))?;
    let url = SafeUrl::from_url(file_url)?;

    let public_name = format!("test.{}", get_random_string());
    safe_cmd_stdout(
        &config_dir,
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

    let nrs_url = SafeUrl::from_url(&format!("safe://{public_name}"))?;
    safe_cmd(&config_dir, ["cat", &nrs_url.to_string()], Some(0))?
        .assert()
        .stdout(predicate::str::contains("exists"));

    Ok(())
}

#[tokio::test]
async fn cat_should_display_wallet_balances() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let json_output = safe_cmd_stdout(&config_dir, ["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, dbc, balance) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    let dbc_hex_string = dbc.to_hex()?;
    dbc_file_path.write_str(&dbc_hex_string)?;

    safe_cmd(
        &config_dir,
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    // The CLI shows DBC hex string in a shortened way, keeping the
    // first and last 8 bytes of the DBC content.
    let shortened_dbc_str = format!(
        "{}...{}",
        &dbc_hex_string[..8],
        &dbc_hex_string[dbc_hex_string.len() - 8..]
    );

    safe_cmd(&config_dir, ["cat", &wallet_xorurl], Some(0))?
        .assert()
        .stdout(predicate::str::contains(format!(
            "Spendable balances of wallet at \"{wallet_xorurl}\":",
        )))
        .stdout(predicate::str::contains("my-first-dbc"))
        .stdout(predicate::str::contains(balance.to_string()))
        .stdout(predicate::str::contains(shortened_dbc_str));

    Ok(())
}

/// Test:  safe cat <src>/<path>
///    src is symlinks_test dir, put with trailing slash.
///    path references both directory and file relative symlinks
///         including parent dir and sibling dir link targets.
///         Final destination is the file sibling_dir_file.md
///         which is itself a symlink to hello.md.
///
///         realpath: /sub2/hello.md
///
///    expected result: cmd output matches contents of
///                     ./test_symlinks/sub2/hello.md
#[test]
fn cat_should_display_file_contents_when_path_contains_multiple_symlinked_targets() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let config_dir = use_isolated_safe_config_dir()?;

    let (url, ..) = upload_test_symlinks_folder(&config_dir, true)?;
    let mut safeurl = SafeUrl::from_url(&url)?;
    safeurl.set_path("/dir_link_link/parent_dir/dir_link/sibling_dir_file.md");

    let output = safe_cmd_stdout(&config_dir, ["cat", &safeurl.to_string()], Some(0))?;
    assert_eq!(output, "= Hello =");

    Ok(())
}

/// Test:  safe cat <src>/<path>
///    src is symlinks_test dir, put with trailing slash.
///    path references a symlink that links to itself.
///         (infinite loop)
///
///    expected result: error, too many links.
#[test]
fn cat_should_fail_when_symlink_has_an_infinite_loop() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let config_dir = use_isolated_safe_config_dir()?;

    let (url, ..) = upload_test_symlinks_folder(&config_dir, true)?;
    let mut safeurl = SafeUrl::from_url(&url)?;

    safeurl.set_path("/sub/infinite_loop");
    let output = safe_cmd_stderr(&config_dir, ["cat", &safeurl.to_string()], Some(1))?;
    assert!(output.contains("ContentNotFound"));
    assert!(output.contains("Too many levels of symbolic links"));

    Ok(())
}

/// Test:  safe cat <src>/dir_link_deep/../readme.md
///    src is symlinks_test dir, put with trailing slash.
///    path should resolve as follows:
///         dir_link_deep --> sub/deep
///         ../           --> sub
///         readme.md     --> readme.md
///
///         realpath: /sub/readme.md
///
///    This test verifies that "../" is being resolved
///    correctly *after* dir_link_deep resolution, not before.
///
///    On unix, this behavior can be verified with:
///       $ cat ./test_symlinks/dir_link_deep/../readme.md
///       = This is a real markdown file. =
///
///    note: This test always failed when SafeUrl
///          used rust-url for parsing path because it
///          normalizes away the "../" with no option
///          to obtain the raw path.
///          filed issue: https://github.com/servo/rust-url/issues/602
///
///    expected result: cmd output matches contents of
///                     /sub/readme.md
#[test]
fn cat_should_display_file_contents_when_url_has_relative_path() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let config_dir = use_isolated_safe_config_dir()?;

    let (url, ..) = upload_test_symlinks_folder(&config_dir, true)?;
    let mut safeurl = SafeUrl::from_url(&url)?;

    safeurl.set_path("/dir_link_deep/../readme.md");
    let output = safe_cmd_stdout(&config_dir, ["cat", &safeurl.to_string()], Some(0))?;
    assert_eq!(output, "= This is a real markdown file. =");

    Ok(())
}

/// Test:  safe cat <src>/dir_outside
///    src is symlinks_test dir, put with trailing slash.
///    path references a symlink with target outside the FileContainer
///
///    expected result: error, too many links.
#[test]
fn cat_should_fail_when_symlinked_target_is_outside_the_container() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let config_dir = use_isolated_safe_config_dir()?;

    let (url, ..) = upload_test_symlinks_folder(&config_dir, true)?;
    let mut safeurl = SafeUrl::from_url(&url)?;

    safeurl.set_path("/dir_outside");
    let output = safe_cmd_stderr(&config_dir, ["cat", &safeurl.to_string()], Some(1))?;
    assert!(output.contains("ContentNotFound"));
    assert!(output.contains("Cannot ascend beyond root directory"));

    Ok(())
}

#[test]
fn cat_should_display_nrs_map_container_contents() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, processed_files, _) =
        upload_path(&config_dir, &tmp_data_path, with_trailing_slash)?;

    let test_file_link = processed_files
        .get(&PathBuf::from(tmp_data_path.path()).join("test.md"))
        .ok_or_else(|| eyre!("test.md should be in files container"))?
        .link()
        .ok_or_else(|| eyre!("should have link"))?;
    let another_file_link = processed_files
        .get(&PathBuf::from(tmp_data_path.path()).join("another.md"))
        .ok_or_else(|| eyre!("another.md should be in files container"))?
        .link()
        .ok_or_else(|| eyre!("should have link"))?;

    let site_name = get_random_string();
    let container_xorurl = SafeUrl::from_url(&format!("safe://{site_name}"))?.to_xorurl_string();
    safe_cmd(
        &config_dir,
        [
            "nrs",
            "register",
            &site_name,
            "--link",
            &files_container_xor,
        ],
        Some(0),
    )?;
    safe_cmd(
        &config_dir,
        [
            "nrs",
            "add",
            &format!("test.{site_name}"),
            "--link",
            test_file_link,
        ],
        Some(0),
    )?;
    safe_cmd(
        &config_dir,
        [
            "nrs",
            "add",
            &format!("another.{site_name}"),
            "--link",
            another_file_link,
        ],
        Some(0),
    )?;

    safe_cmd(&config_dir, ["cat", &container_xorurl], Some(0))?
        .assert()
        .stdout(predicate::str::contains(format!(
            "{site_name}: {files_container_xor}",
        )))
        .stdout(predicate::str::contains(format!(
            "test.{site_name}: {test_file_link}",
        )))
        .stdout(predicate::str::contains(format!(
            "another.{site_name}: {another_file_link}",
        )));

    Ok(())
}
