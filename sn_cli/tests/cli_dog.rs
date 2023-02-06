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
use sn_api::{
    resolver::{SafeData, SafeUrl},
    XorName,
};
use sn_cmd_test_utilities::util::{
    get_random_string, parse_files_put_or_sync_output, parse_wallet_create_output, safe_cmd,
    safe_cmd_stdout, upload_path, use_isolated_safe_config_dir,
};
use std::path::PathBuf;

const TEST_FILE: &str = "../resources/testdata/test.md";
const LARGE_TEST_FILE: &str = "../resources/testdata/large_markdown_file.md";

type DataReplicasReport = Vec<(XorName, Vec<(usize, String)>)>;

#[test]
fn dog_should_resolve_files_container_from_nrs_url_without_safe_prefix() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;
    let content = safe_cmd_stdout(&config_dir, ["files", "put", TEST_FILE, "--json"], Some(0))?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let nrsurl = get_random_string();
    safe_cmd(
        &config_dir,
        ["nrs", "register", &nrsurl, "-l", &container_xorurl],
        Some(0),
    )?;

    let dog_output = safe_cmd_stdout(&config_dir, ["dog", &nrsurl, "--json"], Some(0))?;
    let (url, mut content, _): (String, Vec<SafeData>, DataReplicasReport) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog` on file");
    assert_eq!(url, format!("safe://{nrsurl}"));

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {content:?}");
    }
}

#[test]
fn dog_should_resolve_files_container_from_nrs_url_with_safe_prefix() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;
    let content = safe_cmd_stdout(&config_dir, ["files", "put", TEST_FILE, "--json"], Some(0))?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let site_name = get_random_string();
    let nrsurl = format!("safe://{site_name}");
    safe_cmd(
        &config_dir,
        ["nrs", "register", &site_name, "-l", &container_xorurl],
        Some(0),
    )?;

    let dog_output = safe_cmd_stdout(&config_dir, ["dog", &nrsurl, "--json"], Some(0))?;
    let (url, mut content, _): (String, Vec<SafeData>, DataReplicasReport) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog` on file");
    assert_eq!(url, nrsurl);

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {content:?}");
    }
}

#[test]
fn dog_should_resolve_files_container_using_json_compact_output_from_nrs_url() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;
    let content = safe_cmd_stdout(
        &config_dir,
        ["files", "put", TEST_FILE, "--output=jsoncompact"],
        Some(0),
    )?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let nrsurl = get_random_string();
    safe_cmd(
        &config_dir,
        ["nrs", "register", &nrsurl, "-l", &container_xorurl],
        Some(0),
    )?;

    let dog_output = safe_cmd_stdout(
        &config_dir,
        ["dog", &nrsurl, "--output=jsoncompact"],
        Some(0),
    )?;
    let (url, mut content, _): (String, Vec<SafeData>, DataReplicasReport) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog`");
    assert_eq!(url, format!("safe://{nrsurl}"));

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {content:?}");
    }
}

#[test]
fn dog_should_resolve_files_container_using_yaml_output_from_nrs_url() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;
    let content = safe_cmd_stdout(&config_dir, ["files", "put", TEST_FILE, "--json"], Some(0))?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let nrsurl = get_random_string();
    let _ = safe_cmd_stdout(
        &config_dir,
        ["nrs", "register", &nrsurl, "-l", &container_xorurl],
        Some(0),
    )?;
    let dog_output = safe_cmd_stdout(&config_dir, ["dog", &nrsurl, "--output=yaml"], Some(0))?;
    let (url, mut content, _): (String, Vec<SafeData>, DataReplicasReport) =
        serde_yaml::from_str(&dog_output).expect("Failed to parse output of `safe dog`");
    assert_eq!(url, format!("safe://{nrsurl}"));

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {content:?}");
    }
}

#[test]
fn dog_should_resolve_wallet_from_xor_url() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;
    let json_output = safe_cmd_stdout(&config_dir, ["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(&config_dir, ["dog", &wallet_xorurl], Some(0))?
        .assert()
        .stdout(predicate::str::contains("= Wallet ="))
        .stdout(predicate::str::contains(format!(
            "XOR-URL: {wallet_xorurl}"
        )))
        .stdout(predicate::str::contains("Native data type: Register"));

    Ok(())
}

#[test]
fn dog_should_resolve_nrs_container_from_nrs_url() -> Result<()> {
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

    safe_cmd(&config_dir, ["dog", &container_xorurl], Some(0))?
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

#[test]
fn dog_should_query_data_replicas_from_nrs_url() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;
    let content = safe_cmd_stdout(
        &config_dir,
        ["files", "put", LARGE_TEST_FILE, "/myfile", "--json"],
        Some(0),
    )?;
    let (container_xorurl, _) = parse_files_put_or_sync_output(&content)?;

    let nrsurl = get_random_string();
    let _ = safe_cmd_stdout(
        &config_dir,
        ["nrs", "register", &nrsurl, "-l", &container_xorurl],
        Some(0),
    )?;
    // let's query all valid four replicas, plus replica with index 99 we should get an error for.
    let out_of_index = 99;
    let dog_output = safe_cmd_stdout(
        &config_dir,
        [
            "dog",
            &format!("safe://{nrsurl}/myfile"),
            "-r0",
            "-r1",
            "-r2",
            "-r3",
            "-r",
            &out_of_index.to_string(),
            "--json",
        ],
        Some(0),
    )?;

    #[allow(clippy::type_complexity)]
    let (_, _, replicas_report): (String, Vec<SafeData>, DataReplicasReport) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog`");

    assert_eq!(replicas_report.len(), 4); // it's a 4-chunks file

    let expected_error = format!("expected: {}", out_of_index + 1);

    for (_, outcomes) in replicas_report {
        // assert!(replicas_report.iter().all(|(_, outcomes)| {
        // for each chunk, replica with index 0, 1, 2 and 3, shall be ok,
        // error is expected on index 99
        assert_eq!(outcomes.len(), 5, "we did not get the expected outcomes");

        assert_eq!(outcomes[0], (0, "".to_string()));
        assert_eq!(outcomes[1], (1, "".to_string()));
        assert_eq!(outcomes[2], (2, "".to_string()));
        assert_eq!(outcomes[3], (3, "".to_string()));

        // finally assert over our sad case
        let (index, bad_outcome) = &outcomes[4];

        assert_eq!(index, &out_of_index);

        println!("bad_ouctome: {bad_outcome:?}");
        assert!(
            bad_outcome.contains("InsufficientNodeCount"),
            "bad outcome error not InsifficientNodeCount"
        );
        assert!(
            bad_outcome.contains(&expected_error),
            "bad outcome not due to correct query index"
        );
    }

    // }));

    Ok(())
}
