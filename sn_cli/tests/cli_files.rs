// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use color_eyre::{eyre::eyre, Report, Result};
use predicates::prelude::*;
use sn_api::{SafeUrl, VersionHash};
use sn_cmd_test_utilities::util::{
    get_directory_file_count, get_directory_len, get_file_len, get_random_nrs_string,
    mk_emptyfolder, parse_files_container_output, parse_files_put_or_sync_output,
    parse_files_tree_output, parse_nrs_register_output, safe_cmd, safe_cmd_stderr, safe_cmd_stdout,
    safeurl_from, test_symlinks_are_valid, upload_path, upload_test_symlinks_folder,
    upload_testfolder_trailing_slash, CLI, SAFE_PROTOCOL,
};
use std::{path::Path, process::Command, str::FromStr};

const PRETTY_FILES_CREATION_RESPONSE: &str = "FilesContainer created at: ";
const TEST_FILE: &str = "../resources/testdata/test.md";
const TEST_FILE_RANDOM_CONTENT: &str = "test_file_random_content.txt";
const TEST_FOLDER: &str = "../resources/testdata/";
const TEST_FOLDER_NO_TRAILING_SLASH: &str = "../resources/testdata";
const TEST_FOLDER_SUBFOLDER: &str = "../resources/testdata/subfolder/";

const EXPECT_TESTDATA_PUT_CNT: usize = 11; // 8 files, plus 3 directories

#[test]
fn calling_safe_files_put_pretty() -> Result<()> {
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec!["files", "put", TEST_FILE])
        .assert()
        .stdout(predicate::str::contains(PRETTY_FILES_CREATION_RESPONSE))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(2))
        .stdout(predicate::str::contains(TEST_FILE).count(1))
        .success();
    Ok(())
}

#[test]
fn calling_safe_files_put() -> Result<()> {
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec!["files", "put", TEST_FILE, "--json"])
        .assert()
        .stdout(predicate::str::contains(PRETTY_FILES_CREATION_RESPONSE).count(0))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(2))
        .stdout(predicate::str::contains(TEST_FILE).count(1))
        .success();
    Ok(())
}

#[test]
#[ignore = "dry_run"]
fn calling_safe_files_put_dry_run() -> Result<()> {
    let random_content: String = (0..10).map(|_| rand::random::<char>()).collect();
    std::fs::write(TEST_FILE_RANDOM_CONTENT, random_content).map_err(|e| eyre!(e.to_string()))?;

    let content = safe_cmd_stdout(
        [
            "files",
            "put",
            TEST_FILE_RANDOM_CONTENT,
            "--json",
            "--dry-run",
        ],
        Some(0),
    )?;

    let (_, processed_files) = parse_files_put_or_sync_output(&content)?;
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec![
        "cat",
        processed_files[Path::new(TEST_FILE_RANDOM_CONTENT)]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
    ])
    .assert()
    .failure();
    Ok(())
}

#[test]
fn calling_safe_files_put_recursive() -> Result<()> {
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec!["files", "put", TEST_FOLDER, "--recursive", "--json"])
        .assert()
        .stdout(predicate::str::contains(r#"Added"#).count(12))
        .stdout(predicate::str::contains("../resources/testdata/test.md").count(1))
        .stdout(predicate::str::contains("../resources/testdata/another.md").count(1))
        .stdout(predicate::str::contains("../resources/testdata/subfolder/subexists.md").count(1))
        .success();
    Ok(())
}

#[test]
fn calling_safe_files_put_recursive_and_set_dst_path() -> Result<()> {
    let files_container = safe_cmd_stdout(
        ["files", "put", TEST_FOLDER, "/aha", "--recursive"],
        Some(0),
    )?;

    let mut lines = files_container.lines();
    let files_container_xor_line = lines
        .next()
        .ok_or_else(|| eyre!("Could not fetch next line".to_string()))?;
    let files_container_xor =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace('"', "");

    let mut safeurl = safeurl_from(files_container_xor)?;
    safeurl.set_path("/aha/test.md");
    let file_cat = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    let contents = std::fs::read_to_string(format!("{}/test.md", TEST_FOLDER))?;
    assert_eq!(file_cat, contents);

    safeurl.set_path("/aha/subfolder/subexists.md");
    let subfile_cat = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    let contents = std::fs::read_to_string(format!("{}/subexists.md", TEST_FOLDER_SUBFOLDER))?;
    assert_eq!(subfile_cat, contents.trim());
    Ok(())
}

#[test]
fn calling_safe_files_put_recursive_subfolder() -> Result<()> {
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec![
        "files",
        "put",
        TEST_FOLDER_SUBFOLDER,
        "--recursive",
        "--json",
    ])
    .assert()
    .stdout(predicate::str::contains(SAFE_PROTOCOL).count(3))
    .stdout(predicate::str::contains("../resources/testdata/test.md").count(0))
    .stdout(predicate::str::contains("../resources/testdata/another.md").count(0))
    .stdout(predicate::str::contains("../resources/testdata/subfolder/subexists.md").count(1))
    .success();
    Ok(())
}

#[test]
fn calling_safe_files_put_emptyfolder() -> Result<()> {
    let emptyfolder_paths = mk_emptyfolder("emptyfolder").map_err(|e| eyre!(e.to_string()))?;

    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec![
        "files",
        "put",
        &emptyfolder_paths.1,
        "--recursive",
        "--json",
    ])
    .assert()
    .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
    .stdout(predicate::str::contains("./testdata/emptyfolder/").count(0))
    .success();

    // cleanup
    std::fs::remove_dir_all(&emptyfolder_paths.0).map_err(|e| eyre!(e.to_string()))?;
    Ok(())
}

#[test]
fn calling_safe_files_put_recursive_with_slash() -> Result<()> {
    let files_container = safe_cmd_stdout(["files", "put", TEST_FOLDER, "--recursive"], Some(0))?;

    let mut lines = files_container.lines();
    let files_container_xor_line = lines
        .next()
        .ok_or_else(|| eyre!("Could not fetch next line".to_string()))?;
    let files_container_xor =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace('"', "");

    let mut safeurl = safeurl_from(files_container_xor)?;
    safeurl.set_path("/test.md");
    let file_cat = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    let contents = std::fs::read_to_string(format!("{}/test.md", TEST_FOLDER))?;
    assert_eq!(file_cat, contents);

    let mut safeurl = safeurl_from(files_container_xor)?;
    safeurl.set_path("/subfolder/subexists.md");
    let subfile_cat = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    let contents = std::fs::read_to_string(format!("{}/subexists.md", TEST_FOLDER_SUBFOLDER))?;
    assert_eq!(subfile_cat, contents.trim());
    Ok(())
}

#[test]
fn calling_safe_files_put_recursive_without_slash() -> Result<()> {
    let files_container = safe_cmd_stdout(
        ["files", "put", TEST_FOLDER_NO_TRAILING_SLASH, "--recursive"],
        Some(0),
    )?;

    let mut lines = files_container.lines();
    let files_container_xor_line = lines
        .next()
        .ok_or_else(|| eyre!("Could not fetch next line".to_string()))?;
    let files_container_xor =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace('"', "");

    let mut safeurl = safeurl_from(files_container_xor)?;
    safeurl.set_path("/testdata/test.md");
    let file_cat = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    let contents = std::fs::read_to_string(format!("{}/test.md", TEST_FOLDER))?;
    assert_eq!(file_cat, contents);

    let mut safeurl = safeurl_from(files_container_xor)?;
    safeurl.set_path("/testdata/subfolder/subexists.md");
    let subfile_cat = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    let contents = std::fs::read_to_string(format!("{}/subexists.md", TEST_FOLDER_SUBFOLDER))?;
    assert_eq!(subfile_cat, contents.trim());
    Ok(())
}

#[test]
fn calling_safe_files_sync() -> Result<()> {
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata", &["**"])?;
    let output = safe_cmd_stdout(
        [
            "files",
            "put",
            &tmp_data_dir.path().display().to_string(),
            "--recursive",
        ],
        Some(0),
    )?;

    let mut lines = output.lines();
    let files_container_xor_line = lines
        .next()
        .ok_or_else(|| eyre!("Could not fetch next line".to_string()))?;
    let versioned_xorurl =
        &files_container_xor_line[PRETTY_FILES_CREATION_RESPONSE.len()..].replace('"', "");

    let mut url = SafeUrl::from_url(versioned_xorurl)?;
    url.set_content_version(None);
    let subfolder_dir = tmp_data_dir.child("subfolder");
    safe_cmd(
        [
            "files",
            "sync",
            &format!("{}/", subfolder_dir.path().display()),
            &url.to_string(),
            "--recursive",
        ],
        Some(0),
    )?;

    // The original content should be overwritten with the sync of the subfolder.
    let mut url = SafeUrl::from_url(versioned_xorurl)?;
    url.set_path("/subexists.md");
    url.set_content_version(None);
    let output = safe_cmd_stdout(["cat", &url.to_string()], Some(0))?;

    let subexists_file = tmp_data_dir.child("subfolder/subexists.md");
    let subexists_file_contents = std::fs::read_to_string(subexists_file.path())?;
    assert_eq!(output, subexists_file_contents.trim());
    Ok(())
}

#[test]
#[ignore = "dry-run issue"]
fn calling_safe_files_sync_dry_run() -> Result<()> {
    let content = safe_cmd_stdout(["files", "put", TEST_FOLDER, "--json"], Some(0))?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;
    let mut target = safeurl_from(&container_xorurl)?;
    target.set_content_version(None);

    let random_content: String = (0..10).map(|_| rand::random::<char>()).collect();
    std::fs::write(TEST_FILE_RANDOM_CONTENT, random_content).map_err(|e| eyre!(e.to_string()))?;
    let sync_content = safe_cmd_stdout(
        [
            "files",
            "sync",
            TEST_FILE_RANDOM_CONTENT,
            &target.to_string(),
            "--json",
            "--dry-run",
        ],
        Some(0),
    )?;
    let (_, processed_files) = parse_files_put_or_sync_output(&sync_content)?;
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    cmd.args(&vec![
        "cat",
        processed_files[Path::new(TEST_FILE_RANDOM_CONTENT)]
            .link()
            .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?,
    ])
    .assert()
    .failure();
    Ok(())
}

#[test]
#[ignore = "dry-run issue"]
fn calling_safe_files_removed_sync() -> Result<()> {
    let files_container_output = safe_cmd_stdout(
        ["files", "put", TEST_FOLDER, "--recursive", "--json"],
        Some(0),
    )?;
    let emptyfolder_paths = mk_emptyfolder("emptyfolder").map_err(|e| eyre!(e.to_string()))?;
    let (files_container_xor, processed_files) =
        parse_files_put_or_sync_output(&files_container_output)?;
    assert_eq!(processed_files.len(), EXPECT_TESTDATA_PUT_CNT);

    // let's first try with --dry-run and they should not be removed
    let mut safeurl = safeurl_from(&files_container_xor)?;
    safeurl.set_content_version(None);
    let files_container_no_version = safeurl.to_string();
    let sync_cmd_output_dry_run = safe_cmd_stdout(
        [
            "files",
            "sync",
            &emptyfolder_paths.1,
            &files_container_no_version,
            "--recursive",
            "--delete",
            "--dry-run",
            "--json",
        ],
        Some(0),
    )?;
    safeurl.set_content_version(Some(VersionHash::from_str(
        "uuxdihpgutnitqsniozdgfimosfmylfmnqsbvnuozwkgxiirwwyph",
    )?));
    let files_container_v1 = safeurl.to_string();
    let (target, processed_files) = parse_files_put_or_sync_output(&sync_cmd_output_dry_run)?;
    assert_eq!(target, files_container_v1);
    assert_eq!(processed_files.len(), EXPECT_TESTDATA_PUT_CNT);

    let synced_file_cat = safe_cmd_stdout(["cat", &files_container_xor, "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&synced_file_cat)?;
    assert_eq!(xorurl, files_container_xor);
    assert_eq!(files_map.len(), EXPECT_TESTDATA_PUT_CNT);

    // Now, let's try without --dry-run and they should be effectively removed
    let sync_cmd_output = safe_cmd_stdout(
        [
            "files",
            "sync",
            &emptyfolder_paths.1,
            &files_container_no_version,
            "--recursive",
            "--delete",
            "--json",
        ],
        Some(0),
    )?;

    // cleanup
    std::fs::remove_dir_all(&emptyfolder_paths.0).map_err(|e| eyre!(e.to_string()))?;

    let (target, processed_files) = parse_files_put_or_sync_output(&sync_cmd_output)?;
    assert_eq!(target, files_container_v1);
    assert_eq!(processed_files.len(), EXPECT_TESTDATA_PUT_CNT);

    // now all file items should be gone
    safeurl.set_content_version(None);
    let synced_file_cat = safe_cmd_stdout(["cat", &files_container_xor, "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&synced_file_cat)?;
    assert_eq!(xorurl, safeurl.to_string());
    assert_eq!(files_map.len(), 0);
    Ok(())
}

#[test]
#[ignore = "bad tests needs updated after tweaked table rendering"]
fn calling_safe_files_put_recursive_with_slash_then_sync_after_modifications() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata/subfolder", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_dir, with_trailing_slash)?;

    let sub2_file = tmp_data_dir.child("sub2.md");
    sub2_file.write_str("modify content for sub2 file")?;
    let subexists_file = tmp_data_dir.child("subexists.md");
    let subexists_file_content = std::fs::read_to_string(subexists_file.path())?;
    std::fs::remove_file(subexists_file.path()).map_err(|e| eyre!(e.to_string()))?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    let output = safe_cmd_stdout(
        [
            "files",
            "sync",
            &format!("{}/", tmp_data_dir.path().display()),
            &url.to_string(),
            "--recursive",
        ],
        Some(0),
    )?;

    // Due to the lack of --delete arg on the sync, the subexists file shouldn't have been removed.
    url.set_path("/subexists.md");
    let subexists_content = safe_cmd_stdout(["cat", &url.to_string()], Some(0))?;

    assert_eq!(subexists_content, subexists_file_content);
    assert!(output.contains('*'));
    assert!(!output.contains('+'));
    Ok(())
}

#[test]
fn calling_files_sync_and_fetch_with_version() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, processed_files, _) =
        upload_path(&tmp_data_dir, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    let version = url
        .content_version()
        .ok_or_else(|| eyre!("failed to get content version from xorurl"))?;
    let orig_directory_file_count = get_directory_file_count(&tmp_data_dir)?;
    assert_eq!(processed_files.len(), orig_directory_file_count);

    let empty_dir = tmp_data_dir.child("emptyfolder2");
    empty_dir.create_dir_all()?;
    url.set_content_version(None);
    let output = safe_cmd_stdout(
        [
            "files",
            "sync",
            &empty_dir.path().display().to_string(),
            &url.to_string(),
            "--recursive",
            "--delete",
            "--json",
        ],
        Some(0),
    )?;
    let (files_container_xor, processed_files) = parse_files_put_or_sync_output(&output)?;
    assert_eq!(
        processed_files.len(),
        get_directory_file_count(&tmp_data_dir)?
    );

    // Now the new version of the FilesContainer will only contain the empty folder.
    let output = safe_cmd_stdout(["cat", &files_container_xor, "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&output)?;
    assert_eq!(xorurl, files_container_xor);
    assert_eq!(files_map.len(), 1);

    // First version of the FilesContainer should still have the original files.
    url.set_content_version(Some(version));
    let output = safe_cmd_stdout(["cat", &url.to_string(), "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&output)?;
    assert_eq!(xorurl, url.to_string());
    assert_eq!(files_map.len(), orig_directory_file_count);
    Ok(())
}

#[test]
#[ignore = "relative url without a base"]
fn calling_files_sync_and_fetch_with_nrsurl_and_nrs_update() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, processed_files, _) =
        upload_path(&tmp_data_dir, with_trailing_slash)?;

    assert_eq!(
        processed_files.len(),
        get_directory_file_count(&tmp_data_dir)?
    );

    let nrsurl = get_random_nrs_string();
    let output = safe_cmd_stdout(
        [
            "nrs",
            "register",
            &nrsurl,
            "-l",
            &files_container_xor,
            "--json",
        ],
        Some(0),
    )?;
    let (_, nrs_xorurl, _) = parse_nrs_register_output(&output)?;
    let nrs_version = nrs_xorurl
        .content_version()
        .ok_or_else(|| eyre!("failed to get content version from xorurl"))?;

    let empty_dir = tmp_data_dir.child("emptyfolder2");
    empty_dir.create_dir_all()?;
    let output = safe_cmd_stdout(
        [
            "files",
            "sync",
            &empty_dir.path().display().to_string(),
            &nrsurl,
            "--recursive",
            "--delete",
            "--json",
            "--update-nrs",
        ],
        Some(0),
    )?;

    let (target, processed_files) = parse_files_put_or_sync_output(&output)?;
    assert_eq!(target, nrsurl);
    assert_eq!(
        processed_files.len(),
        get_directory_file_count(&tmp_data_dir)?
    );

    // With the use of --update-nrs, now there will only be the `emptyfolder2` entry.
    let output = safe_cmd_stdout(["cat", &nrsurl, "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&output)?;
    assert_eq!(xorurl, nrsurl);
    assert_eq!(files_map.len(), 1);

    //// but in version 0 of the NRS name it should still link to version 0 of the FilesContainer
    //// where all files should still be there
    let versioned_nrsurl = format!("{}?v={}", nrsurl, nrs_version);
    let output = safe_cmd_stdout(["cat", &versioned_nrsurl, "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&output)?;
    assert_eq!(xorurl, versioned_nrsurl);
    assert_eq!(files_map.len(), 12);
    Ok(())
}

#[test]
fn calling_files_sync_and_fetch_without_nrs_update() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, processed_files, _) =
        upload_path(&tmp_data_dir, with_trailing_slash)?;

    let orig_directory_file_count = get_directory_file_count(&tmp_data_dir)?;
    assert_eq!(processed_files.len(), orig_directory_file_count);

    let nrsurl = get_random_nrs_string();
    safe_cmd(
        [
            "nrs",
            "register",
            &nrsurl,
            "-l",
            &files_container_xor,
            "--json",
        ],
        Some(0),
    )?;

    let empty_dir = tmp_data_dir.child("emptyfolder2");
    empty_dir.create_dir_all()?;
    safe_cmd(
        [
            "files",
            "sync",
            &empty_dir.path().display().to_string(),
            &nrsurl,
            "--recursive",
            "--delete",
            "--json",
        ],
        Some(0),
    )?;

    // The current version should now only have the empty folder entry.
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    let output = safe_cmd_stdout(["cat", &url.to_string(), "--json"], Some(0))?;
    let (_, files_map) = parse_files_container_output(&output)?;
    assert_eq!(files_map.len(), 1);

    // but the NRS name should still link to version 0 of the FilesContainer
    // where all files should still be there
    let output = safe_cmd_stdout(["cat", &nrsurl, "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&output)?;
    assert_eq!(xorurl, format!("safe://{}", nrsurl));
    assert_eq!(files_map.len(), orig_directory_file_count);
    Ok(())
}

#[test]
fn calling_files_sync_and_fetch_without_nrs_url_with_safe_prefix() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, processed_files, _) =
        upload_path(&tmp_data_dir, with_trailing_slash)?;

    let orig_directory_file_count = get_directory_file_count(&tmp_data_dir)?;
    assert_eq!(processed_files.len(), orig_directory_file_count);

    let site_name = get_random_nrs_string();
    let nrsurl = format!("safe://{}", site_name);
    safe_cmd(
        [
            "nrs",
            "register",
            &site_name,
            "-l",
            &files_container_xor,
            "--json",
        ],
        Some(0),
    )?;

    let empty_dir = tmp_data_dir.child("emptyfolder2");
    empty_dir.create_dir_all()?;
    safe_cmd(
        [
            "files",
            "sync",
            &empty_dir.path().display().to_string(),
            &nrsurl,
            "--recursive",
            "--delete",
            "--json",
        ],
        Some(0),
    )?;

    // The current version should now only have the empty folder entry.
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    let output = safe_cmd_stdout(["cat", &url.to_string(), "--json"], Some(0))?;
    let (_, files_map) = parse_files_container_output(&output)?;
    assert_eq!(files_map.len(), 1);

    // but the NRS name should still link to version 0 of the FilesContainer
    // where all files should still be there
    let output = safe_cmd_stdout(["cat", &nrsurl, "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&output)?;
    assert_eq!(xorurl, nrsurl);
    assert_eq!(files_map.len(), orig_directory_file_count);
    Ok(())
}

#[test]
fn calling_safe_files_add() -> Result<()> {
    let files_container_output = safe_cmd_stdout(
        ["files", "put", TEST_FOLDER, "--recursive", "--json"],
        Some(0),
    )?;

    let (files_container_xor, _processed_files) =
        parse_files_put_or_sync_output(&files_container_output)?;

    let mut safeurl = safeurl_from(&files_container_xor)?;
    safeurl.set_content_version(None);
    safe_cmd(
        [
            "files",
            "add",
            TEST_FILE,
            &format!("{}/new_test.md", safeurl),
        ],
        Some(0),
    )?;

    safeurl.set_path("/new_test.md");
    let synced_file_cat = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    assert_eq!(synced_file_cat, "hello tests!");
    Ok(())
}

#[test]
#[ignore = "dry-run issue"]
fn calling_safe_files_add_dry_run() -> Result<(), Report> {
    let files_container_output = safe_cmd_stdout(
        ["files", "put", TEST_FOLDER, "--recursive", "--json"],
        Some(0),
    )?;
    let (files_container_xor, _) = parse_files_put_or_sync_output(&files_container_output)?;
    let mut safeurl = safeurl_from(&files_container_xor)?;
    safeurl.set_content_version(None);
    safe_cmd(
        [
            "files",
            "add",
            TEST_FILE,
            &format!("{}/new_test.md", safeurl),
            "--dry-run",
        ],
        Some(0),
    )?;

    safeurl.set_path("/new_test.md");
    let mut cmd = Command::cargo_bin(CLI)?;
    cmd.args(&vec!["cat", &safeurl.to_string()])
        .assert()
        .failure();
    Ok(())
}

#[test]
fn calling_safe_files_add_a_url() -> Result<()> {
    let files_container_output = safe_cmd_stdout(
        ["files", "put", TEST_FOLDER, "--recursive", "--json"],
        Some(0),
    )?;

    let (files_container_xor, processed_files) =
        parse_files_put_or_sync_output(&files_container_output)?;

    let mut safeurl = safeurl_from(&files_container_xor)?;
    safeurl.set_content_version(None);
    safeurl.set_path("/new_test.md");
    let link = processed_files[Path::new(TEST_FILE)]
        .link()
        .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?;
    safe_cmd(
        ["files", "add", link, &safeurl.to_string(), "--json"],
        Some(0),
    )?;

    let synced_file_cat = safe_cmd_stdout(["cat", &safeurl.to_string()], Some(0))?;
    assert_eq!(synced_file_cat, "hello tests!");
    Ok(())
}

#[test]
fn calling_files_ls() -> Result<()> {
    let files_container_output = safe_cmd_stdout(
        ["files", "put", TEST_FOLDER, "--recursive", "--json"],
        Some(0),
    )?;
    let (files_container_xor, processed_files) =
        parse_files_put_or_sync_output(&files_container_output)?;

    let mut safeurl = safeurl_from(&files_container_xor)?;
    safeurl.set_content_version(None);
    let container_xorurl_no_version = safeurl.to_string();

    let files_ls_output = safe_cmd_stdout(
        ["files", "ls", &container_xorurl_no_version, "--json"],
        Some(0),
    )?;

    // Sample output:
    //
    // Files of FilesContainer (version 0) at "safe://<xorurl>":
    // Files: 4   Size: 41   Total Files: 8   Total Size: 80
    // SIZE  CREATED               MODIFIED              NAME
    // 23    2020-05-20T19:55:26Z  2020-05-20T19:55:26Z  .hidden.txt
    // 12    2020-05-20T19:55:26Z  2020-05-20T19:55:26Z  .subhidden/
    // 6     2020-05-20T19:55:26Z  2020-05-20T19:55:26Z  another.md
    // 0     2020-05-20T19:55:26Z  2020-05-20T19:55:26Z  emptyfolder/
    // 0     2020-05-20T19:55:26Z  2020-05-20T19:55:26Z  noextension
    // 27    2020-05-20T19:55:26Z  2020-05-20T19:55:26Z  subfolder/
    // 12    2020-05-20T19:55:26Z  2020-05-20T19:55:26Z  test.md

    let (xorurl, files_map) = parse_files_container_output(&files_ls_output)?;
    assert_eq!(xorurl, container_xorurl_no_version);
    assert_eq!(files_map.len(), 8);
    assert_eq!(
        processed_files[Path::new(&format!("{}.hidden.txt", TEST_FOLDER))].link(),
        Some(&files_map[".hidden.txt"]["link"]),
    );
    assert_eq!(
        processed_files[Path::new(&format!("{}another.md", TEST_FOLDER))].link(),
        Some(&files_map["another.md"]["link"]),
    );
    assert_eq!(
        processed_files[Path::new(&format!("{}noextension", TEST_FOLDER))].link(),
        Some(&files_map["noextension"]["link"]),
    );
    assert_eq!(
        processed_files[Path::new(&format!("{}test.md", TEST_FOLDER))].link(),
        Some(&files_map["test.md"]["link"]),
    );

    let subfolder_len = get_directory_len(TEST_FOLDER_SUBFOLDER)?;
    assert_eq!(files_map["subfolder/"]["size"], subfolder_len.to_string());
    safeurl.set_path("subfolder");
    let subfolder_path = safeurl.to_string();
    assert_eq!(files_map["subfolder/"]["link"], subfolder_path);

    // now listing subfolder should show less files
    let files_ls_output = safe_cmd_stdout(["files", "ls", &subfolder_path, "--json"], Some(0))?;
    let (xorurl, files_map) = parse_files_container_output(&files_ls_output)?;
    assert_eq!(xorurl, subfolder_path);
    assert_eq!(files_map.len(), 2);
    assert_eq!(
        processed_files[Path::new(&format!("{}sub2.md", TEST_FOLDER_SUBFOLDER))].link(),
        Some(&files_map["sub2.md"]["link"]),
    );

    let sub2_len = get_file_len(&format!("{}/{}", TEST_FOLDER_SUBFOLDER, "sub2.md"))?;
    assert_eq!(files_map["sub2.md"]["size"], sub2_len.to_string());
    assert_eq!(
        processed_files[Path::new(&format!("{}subexists.md", TEST_FOLDER_SUBFOLDER))].link(),
        Some(&files_map["subexists.md"]["link"]),
    );

    let subexists_len = get_file_len(&format!("{}/{}", TEST_FOLDER_SUBFOLDER, "subexists.md"))?;
    assert_eq!(files_map["subexists.md"]["size"], subexists_len.to_string());
    Ok(())
}

// Test:  safe ls safe://<xorurl>/subfold
//
//    note: URL path is invalid.
//
//    expected result:
//       a. exit code = 1
//       b. stderr contains "No data found for path"
#[test]
fn calling_files_ls_with_invalid_path() -> Result<()> {
    let (files_container_xor, _processed_files) = upload_testfolder_trailing_slash()?;
    let mut safeurl = safeurl_from(&files_container_xor).map_err(|e| eyre!(e.to_string()))?;

    // set invalid path
    safeurl.set_path("subfold");
    let partial_path = safeurl.to_string();

    let stderr = safe_cmd_stderr(["files", "ls", &partial_path, "--json"], Some(1))
        .map_err(|e| eyre!(e.to_string()))?;

    assert!(stderr.contains("no data found for path: /subfold/"));

    Ok(())
}

// Test:  safe ls safe://<xorurl>/subfolder/subexists.md
//
//    expected result: We find the single file requested
#[test]
fn calling_files_ls_on_single_file() -> Result<()> {
    let (files_container_xor, _processed_files) = upload_testfolder_trailing_slash()?;

    let mut safeurl = safeurl_from(&files_container_xor).map_err(|e| eyre!(e.to_string()))?;
    safeurl.set_path("/subfolder/subexists.md");
    let single_file_url = safeurl.to_string();

    let files_ls_output = safe_cmd_stdout(["files", "ls", &single_file_url, "--json"], Some(0))?;

    let (_xorurl, files_map) = parse_files_container_output(&files_ls_output)?;
    let subexists_len = get_file_len(&format!("{}/subexists.md", TEST_FOLDER_SUBFOLDER))?;
    assert_eq!(files_map.len(), 1);
    assert_eq!(files_map["subexists.md"]["size"], subexists_len.to_string());

    Ok(())
}

// Test:  safe ls safe://<nrsname>/subfolder
//
//    safe://<nrsname> links to safe://<xorurl>/testdata
//
//    expected result: We find the 2 files beneath testdata/subfolder
#[test]
#[ignore = "investigate after sn_cli merge into workspace"]
fn calling_files_ls_on_nrs_with_path() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata", &["**"])?;
    let sub2_file = tmp_data_dir.child("subfolder/sub2.md");
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_dir, with_trailing_slash)?;

    let nrsurl = get_random_nrs_string();
    safe_cmd(
        [
            "nrs",
            "register",
            &nrsurl,
            "-l",
            &files_container_xor,
            "--json",
        ],
        Some(0),
    )?;

    let output = safe_cmd_stdout(
        ["files", "ls", &format!("{}/subfolder", nrsurl), "--json"],
        Some(0),
    )?;
    let (_xorurl, files_map) = parse_files_container_output(&output)?;
    let sub2_len: u64 = files_map["sub2.md"]["size"].parse()?;
    assert_eq!(files_map.len(), 2); // 2 files in the subfolder
    assert_eq!(sub2_len, sub2_file.metadata()?.len());

    Ok(())
}

// Test:  safe files ls <src> --json
//    src is symlinks_test dir, put with trailing slash.
//
//    expected result: result contains 9 FileItem and filenames match.
//                     those in ./test_symlinks
#[test]
fn calling_files_ls_with_symlinks() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid().map_err(|e| eyre!(e.to_string()))? {
        return Ok(());
    }

    let (files_container_xor, ..) =
        upload_test_symlinks_folder(true).map_err(|e| eyre!(e.to_string()))?;

    let args = ["files", "ls", &files_container_xor, "--json"];
    let files_ls_output = safe_cmd_stdout(args, Some(0)).map_err(|e| eyre!(e.to_string()))?;

    // Sample output:
    //
    // Files of FilesContainer (version 0) at "safe://hnyynyss1e1ihdzuspegnqft1y5tocd5o7qgfbmmcgjdizg49bdg68ysqgbnc":
    // Files: 11   Size: 520   Total Files: 20   Total Size: 564
    // SIZE  CREATED               MODIFIED              NAME
    // 391   2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  absolute_links.txt
    // 0     2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  broken_rel_link.txt -> non-existing-target
    // 0     2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  dir_link -> sub
    // 0     2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  dir_link_deep -> sub/deep
    // 0     2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  dir_link_link -> dir_link
    // 0     2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  dir_outside -> ../
    // 0     2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  file_link -> realfile.txt
    // 0     2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  file_link_link -> file_link
    // 0     2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  file_outside -> ../file_outside
    // 21    2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  realfile.txt
    // 34    2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  sub/
    // 10    2020-06-11T22:13:36Z  2020-06-11T22:13:36Z  sub2/

    let (xorurl, files_map) = parse_files_container_output(&files_ls_output)?;
    assert_eq!(xorurl, files_container_xor);
    assert_eq!(files_map.len(), 12);
    assert!(files_map.contains_key("absolute_links.txt"));
    assert!(files_map.contains_key("broken_rel_link.txt"));
    assert!(files_map.contains_key("file_link"));
    assert!(files_map.contains_key("file_link_link"));
    assert!(files_map.contains_key("dir_link"));
    assert!(files_map.contains_key("realfile.txt"));
    assert!(files_map.contains_key("sub/"));

    // todo:
    // 1. test ls'ing an individual symlink

    Ok(())
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn calling_files_tree() -> Result<()> {
    let (files_container_xor, _processed_files) =
        upload_testfolder_trailing_slash().map_err(|e| eyre!(e.to_string()))?;

    let mut safeurl = safeurl_from(&files_container_xor)?;
    safeurl.set_content_version(None);
    let container_xorurl_no_version = safeurl.to_string();

    let files_tree_output = safe_cmd_stdout(
        ["files", "tree", &container_xorurl_no_version, "--json"],
        Some(0),
    )?;

    let root = parse_files_tree_output(&files_tree_output)?;
    assert_eq!(root["name"], container_xorurl_no_version);
    assert_eq!(
        root["sub"]
            .as_array()
            .ok_or_else(|| eyre!("failed to read 'sub' array from files tree output"))?
            .len(),
        8
    );
    assert_eq!(root["sub"][0]["name"], ".hidden.txt");
    assert_eq!(root["sub"][1]["name"], ".subhidden");
    assert_eq!(root["sub"][1]["sub"][0]["name"], "test.md");
    assert_eq!(root["sub"][2]["name"], "another.md");
    assert_eq!(root["sub"][3]["name"], "emptyfolder");
    assert_eq!(root["sub"][3]["sub"][0]["name"], ".gitkeep");
    assert_eq!(root["sub"][4]["name"], "large_markdown_file.md");
    assert_eq!(root["sub"][5]["name"], "noextension");
    assert_eq!(root["sub"][6]["name"], "subfolder");
    assert_eq!(root["sub"][6]["sub"][0]["name"], "sub2.md");
    assert_eq!(root["sub"][6]["sub"][1]["name"], "subexists.md");
    assert_eq!(root["sub"][7]["name"], "test.md");

    let files_tree_output =
        safe_cmd_stdout(["files", "tree", &container_xorurl_no_version], Some(0))?;

    let should_match = format!(
        "{}\n{}",
        container_xorurl_no_version,
        "\
├── .hidden.txt
├── .subhidden
│   └── test.md
├── another.md
├── emptyfolder
│   └── .gitkeep
├── large_markdown_file.md
├── noextension
├── subfolder
│   ├── sub2.md
│   └── subexists.md
└── test.md

3 directories, 9 files"
    );
    assert_eq!(files_tree_output, should_match);

    let files_tree_output = safe_cmd_stdout(
        [
            "files",
            "tree",
            &container_xorurl_no_version,
            "--details",
            "--json",
        ],
        Some(0),
    )?;

    let root = parse_files_tree_output(&files_tree_output)?;
    assert_eq!(root["name"], container_xorurl_no_version);
    assert_eq!(
        root["sub"]
            .as_array()
            .ok_or_else(|| eyre!("failed to read 'sub' array from files tree output"))?
            .len(),
        8
    );
    assert_eq!(root["sub"][0]["name"], ".hidden.txt");
    assert_eq!(root["sub"][0]["details"]["type"], "text/plain");
    assert_eq!(root["sub"][1]["name"], ".subhidden");
    assert_eq!(root["sub"][1]["details"]["type"], "inode/directory");
    assert_eq!(root["sub"][1]["sub"][0]["name"], "test.md");

    // It seems to be possible for the sizes of files to vary based on the the OS they were
    // uploaded from, so we won't hard code the values.
    let another_file_len = get_file_len("../resources/testdata/another.md")?;
    assert_eq!(root["sub"][2]["name"], "another.md");
    assert_eq!(
        root["sub"][2]["details"]["size"],
        another_file_len.to_string()
    );
    assert_eq!(root["sub"][2]["details"]["type"], "text/markdown");

    assert_eq!(root["sub"][3]["name"], "emptyfolder");
    assert_eq!(root["sub"][3]["details"]["size"], "0");
    assert_eq!(root["sub"][3]["details"]["type"], "inode/directory");

    let markdown_file_len = get_file_len("../resources/testdata/large_markdown_file.md")?;
    assert_eq!(root["sub"][4]["name"], "large_markdown_file.md");
    assert_eq!(
        root["sub"][4]["details"]["size"],
        markdown_file_len.to_string()
    );
    assert_eq!(root["sub"][4]["details"]["type"], "text/markdown");

    let noextension_file_len = get_file_len("../resources/testdata/noextension")?;
    assert_eq!(root["sub"][5]["name"], "noextension");
    assert_eq!(
        root["sub"][5]["details"]["size"],
        noextension_file_len.to_string()
    );
    assert_eq!(root["sub"][5]["details"]["type"], "Raw");
    assert_eq!(root["sub"][6]["name"], "subfolder");
    assert_eq!(root["sub"][6]["sub"][0]["name"], "sub2.md");
    assert_eq!(root["sub"][6]["sub"][1]["name"], "subexists.md");
    assert_eq!(root["sub"][7]["name"], "test.md");
    Ok(())
}

// Test:  safe files tree <src>
//    src is symlinks_test dir, put with trailing slash.
//
//    expected result: output matches output of `tree ./test_symlinks`
#[test]
fn calling_files_tree_with_symlinks() -> Result<()> {
    // Bail if test_symlinks not valid. Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? {
        return Ok(());
    }

    let (files_container_xor, ..) = upload_test_symlinks_folder(true)?;

    let stdout = safe_cmd_stdout(["files", "tree", &files_container_xor], Some(0))?;

    // note: this is output from `tree` command on linux.
    // `files tree` output should match exactly.
    let should_match = format!(
        "{}\n{}",
        files_container_xor,
        "\
├── absolute_links.txt
├── broken_rel_link.txt -> non-existing-target
├── dir_link -> sub
├── dir_link_deep -> sub/deep
├── dir_link_link -> dir_link
├── dir_outside -> ../
├── file_link -> realfile.txt
├── file_link_link -> file_link
├── file_outside -> ../file_outside
├── realfile.txt
├── sub
│   ├── deep
│   │   └── a_file.txt
│   ├── infinite_loop -> infinite_loop
│   ├── parent_dir -> ..
│   ├── parent_dir_file_link.txt -> ../realfile.txt
│   ├── readme.md
│   ├── sibling_dir -> ../sub2
│   ├── sibling_dir_file.md -> ../sub2/hello.md
│   └── sibling_dir_trailing_slash -> ../sub2/
└── sub2
    ├── hello.md
    └── sub2 -> ../sub2

11 directories, 12 files"
    );
    assert_eq!(stdout, should_match);

    Ok(())
}
