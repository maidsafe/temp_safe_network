// Copyright 2020 MaidSafe.net limited.
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
use sn_api::resolver::{SafeData, SafeUrl};
use sn_cmd_test_utilities::util::{
    create_and_get_keys, get_random_nrs_string, parse_files_put_or_sync_output, safe_cmd,
    safe_cmd_stdout, upload_path,
};
use std::path::PathBuf;

const TEST_FILE: &str = "../resources/testdata/test.md";

#[test]
fn calling_safe_dog_files_container_nrsurl() -> Result<()> {
    let content = safe_cmd_stdout(["files", "put", TEST_FILE, "--json"], Some(0))?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let nrsurl = get_random_nrs_string();
    safe_cmd(
        ["nrs", "register", &nrsurl, "-l", &container_xorurl],
        Some(0),
    )?;

    let dog_output = safe_cmd_stdout(["dog", &nrsurl, "--json"], Some(0))?;
    let (url, mut content): (String, Vec<SafeData>) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog` on file");
    assert_eq!(url, format!("safe://{}", nrsurl));

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {:?}", content);
    }
}

#[test]
fn calling_safe_dog_files_container_nrsurl_with_safe_prefix() -> Result<()> {
    let content = safe_cmd_stdout(["files", "put", TEST_FILE, "--json"], Some(0))?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let site_name = get_random_nrs_string();
    let nrsurl = format!("safe://{}", site_name);
    safe_cmd(
        ["nrs", "register", &site_name, "-l", &container_xorurl],
        Some(0),
    )?;

    let dog_output = safe_cmd_stdout(["dog", &nrsurl, "--json"], Some(0))?;
    let (url, mut content): (String, Vec<SafeData>) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog` on file");
    assert_eq!(url, nrsurl);

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {:?}", content);
    }
}

#[test]
fn calling_safe_dog_files_container_nrsurl_jsoncompact() -> Result<()> {
    let content = safe_cmd_stdout(["files", "put", TEST_FILE, "--output=jsoncompact"], Some(0))?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let nrsurl = get_random_nrs_string();
    safe_cmd(
        ["nrs", "register", &nrsurl, "-l", &container_xorurl],
        Some(0),
    )?;

    let dog_output = safe_cmd_stdout(["dog", &nrsurl, "--output=jsoncompact"], Some(0))?;
    let (url, mut content): (String, Vec<SafeData>) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog`");
    assert_eq!(url, format!("safe://{}", nrsurl));

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {:?}", content);
    }
}

#[test]
fn calling_safe_dog_files_container_nrsurl_yaml() -> Result<()> {
    let content = safe_cmd_stdout(["files", "put", TEST_FILE, "--json"], Some(0))?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let nrsurl = get_random_nrs_string();
    let _ = safe_cmd_stdout(
        ["nrs", "register", &nrsurl, "-l", &container_xorurl],
        Some(0),
    )?;
    let dog_output = safe_cmd_stdout(["dog", &nrsurl, "--output=yaml"], Some(0))?;
    let (url, mut content): (String, Vec<SafeData>) =
        serde_yaml::from_str(&dog_output).expect("Failed to parse output of `safe dog`");
    assert_eq!(url, format!("safe://{}", nrsurl));

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {:?}", content);
    }
}

#[test]
fn calling_safe_dog_safekey_nrsurl() -> Result<()> {
    let (safekey_xorurl, _sk) = create_and_get_keys()?;

    let nrsurl = get_random_nrs_string();
    safe_cmd(["nrs", "register", &nrsurl, "-l", &safekey_xorurl], Some(0))?;
    let dog_output = safe_cmd_stdout(["dog", &nrsurl, "--json"], Some(0))?;
    let (url, mut content): (String, Vec<SafeData>) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog` on file");
    assert_eq!(url, format!("safe://{}", nrsurl));

    if let Some(SafeData::SafeKey { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, safekey_xorurl);
        Ok(())
    } else {
        Err(eyre!("Content retrieved was unexpected: {:?}", content))
    }
}

#[test]
fn calling_safe_dog_with_nrs_map_container_link() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

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

    let site_name = get_random_nrs_string();
    let container_xorurl = SafeUrl::from_url(&format!("safe://{}", site_name))?.to_xorurl_string();
    safe_cmd(
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
        [
            "nrs",
            "add",
            &format!("test.{site_name}"),
            "--link",
            &test_file_link,
        ],
        Some(0),
    )?;
    safe_cmd(
        [
            "nrs",
            "add",
            &format!("another.{site_name}"),
            "--link",
            &another_file_link,
        ],
        Some(0),
    )?;

    safe_cmd(["dog", &container_xorurl], Some(0))?
        .assert()
        .stdout(predicate::str::contains(&format!(
            "{site_name}: {}",
            files_container_xor
        )))
        .stdout(predicate::str::contains(&format!(
            "test.{site_name}: {}",
            test_file_link
        )))
        .stdout(predicate::str::contains(&format!(
            "another.{site_name}: {}",
            another_file_link
        )));
    Ok(())
}
