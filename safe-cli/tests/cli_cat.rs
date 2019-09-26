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
    create_preload_and_get_keys, get_bin_location, get_random_nrs_string, parse_cat_wallet_output,
    parse_files_put_or_sync_output, CLI,
};
use predicates::prelude::*;
use safe_api::{BlsKeyPair, SafeContentType, SafeData, SafeDataType, XorUrlEncoder};
use std::process::Command;
use unwrap::unwrap;

const TEST_FILE: &str = "../testdata/test.md";
const TEST_FILE_CONTENT: &str = "hello tests!";
const ANOTHER_FILE: &str = "../testdata/another.md";
const ANOTHER_FILE_CONTENT: &str = "exists";

#[test]
fn calling_safe_cat() {
    let content = cmd!(get_bin_location(), "files", "put", TEST_FILE, "--json")
        .read()
        .unwrap();

    let (_container_xorurl, map) = parse_files_put_or_sync_output(&content);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &map[TEST_FILE].1])
        .assert()
        .stdout(predicate::str::contains(TEST_FILE_CONTENT))
        .success();

    let xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&map[TEST_FILE].1));
    assert_eq!(
        xorurl_encoder.content_type(),
        SafeContentType::MediaType("text/x-markdown".to_string())
    );
    assert_eq!(
        xorurl_encoder.data_type(),
        SafeDataType::PublishedImmutableData
    );
}

#[test]
fn calling_safe_cat_xorurl_url_with_version() {
    let content = cmd!(get_bin_location(), "files", "put", TEST_FILE, "--json")
        .read()
        .unwrap();
    let (container_xorurl, _files_map) = parse_files_put_or_sync_output(&content);

    // let's sync with another file so we get a new version, and a different content in the file
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&container_xorurl));
    xorurl_encoder.set_path("/test.md");
    xorurl_encoder.set_content_version(None);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "files",
        "sync",
        ANOTHER_FILE,
        &unwrap!(xorurl_encoder.to_string()),
    ])
    .assert()
    .success();

    xorurl_encoder.set_content_version(None);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &unwrap!(xorurl_encoder.to_string())])
        .assert()
        .stdout(predicate::str::contains(ANOTHER_FILE_CONTENT))
        .success();

    xorurl_encoder.set_content_version(Some(0));
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &unwrap!(xorurl_encoder.to_string())])
        .assert()
        .stdout(predicate::str::contains(TEST_FILE_CONTENT))
        .success();

    xorurl_encoder.set_content_version(Some(1));
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &unwrap!(xorurl_encoder.to_string())])
        .assert()
        .stdout(predicate::str::contains(ANOTHER_FILE_CONTENT))
        .success();

    xorurl_encoder.set_content_version(Some(2));
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &unwrap!(xorurl_encoder.to_string())])
        .assert()
        .failure();
}

#[test]
fn calling_safe_cat_nrsurl_with_version() {
    let content = cmd!(get_bin_location(), "files", "put", TEST_FILE, "--json")
        .read()
        .unwrap();
    let (container_xorurl, _files_map) = parse_files_put_or_sync_output(&content);

    let nrsurl = format!("safe://{}", get_random_nrs_string());
    let _ = cmd!(
        get_bin_location(),
        "nrs",
        "create",
        &nrsurl,
        "-l",
        &container_xorurl,
    )
    .read()
    .unwrap();

    let nrsurl_with_path = format!("{}/test.md", nrsurl);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &nrsurl_with_path])
        .assert()
        .stdout(predicate::str::contains(TEST_FILE_CONTENT))
        .success();

    // let's sync with another file so we get a new version, and a different content in the file
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&container_xorurl));
    xorurl_encoder.set_path("/test.md");
    xorurl_encoder.set_content_version(None);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec![
        "files",
        "sync",
        ANOTHER_FILE,
        &unwrap!(xorurl_encoder.to_string()),
    ])
    .assert()
    .success();

    // NRS name was not updated (with --updated-nrs) when doing files sync,
    // so our file should not have been updated
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &nrsurl_with_path])
        .assert()
        .stdout(predicate::str::contains(TEST_FILE_CONTENT))
        .success();

    // NRS name has only one version which is 0, so using version 0 should also fetch the file
    let nrsurl_with_path_v0 = format!("{}/test.md?v=0", nrsurl);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &nrsurl_with_path_v0])
        .assert()
        .stdout(predicate::str::contains(TEST_FILE_CONTENT))
        .success();

    // there is no version 1 of NRS name
    let invalid_version_nrsurl = format!("{}/test.md?v=1", nrsurl);
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["cat", &invalid_version_nrsurl])
        .assert()
        .failure();
}

#[test]
fn calling_safe_cat_nrsurl_with_info_level_2() {
    let content = cmd!(get_bin_location(), "files", "put", TEST_FILE, "--json")
        .read()
        .unwrap();
    let (container_xorurl, _files_map) = parse_files_put_or_sync_output(&content);

    let nrsurl = format!("safe://{}", get_random_nrs_string());
    let _ = cmd!(
        get_bin_location(),
        "nrs",
        "create",
        &nrsurl,
        "-l",
        &container_xorurl,
    )
    .read()
    .unwrap();

    let cat_output = cmd!(
        get_bin_location(),
        "cat",
        &nrsurl,
        "--json",
        "--info",
        "--info"
    )
    .read()
    .unwrap();

    let content_info: (String, SafeData) = serde_json::from_str(&cat_output)
        .expect("Failed to parse output of `safe cat` with -ii on file");
    assert_eq!(content_info.0, nrsurl);
    if let SafeData::FilesContainer { resolved_from, .. } = content_info.1 {
        let unwrapped_resolved_from = resolved_from.unwrap();
        assert_eq!(
            unwrapped_resolved_from.public_name,
            nrsurl.replace("safe://", "")
        );
        assert_eq!(unwrapped_resolved_from.type_tag, 1500);
        assert_eq!(unwrapped_resolved_from.version, 0);
        assert_eq!(
            unwrapped_resolved_from.data_type,
            SafeDataType::PublishedSeqAppendOnlyData
        );
    } else {
        panic!("Content retrieved was unexpected: {:?}", content_info);
    }
}

#[test]
fn calling_safe_cat_wallet_xorurl() {
    let wallet_create = cmd!(
        get_bin_location(),
        "wallet",
        "create",
        "--test-coins",
        "--json"
    )
    .read()
    .unwrap();
    let (wallet_xorurl, key_xorurl, key_pair): (String, String, Option<BlsKeyPair>) =
        serde_json::from_str(&wallet_create)
            .expect("Failed to parse output of `safe wallet create`");

    let (key_pk_xor, sk) = create_preload_and_get_keys("7");
    let _wallet_insert_result = cmd!(
        get_bin_location(),
        "wallet",
        "insert",
        &wallet_xorurl,
        "--keyurl",
        &key_pk_xor,
        "--sk",
        &sk,
    )
    .read()
    .unwrap();

    let wallet_cat = cmd!(get_bin_location(), "cat", &wallet_xorurl, "--json")
        .read()
        .unwrap();
    let (xorurl, balances) = parse_cat_wallet_output(&wallet_cat);

    assert_eq!(wallet_xorurl, xorurl);
    assert_eq!(balances.len(), 2);

    assert_eq!(balances[&key_xorurl].0, true);
    assert_eq!(balances[&key_xorurl].1.xorurl, key_xorurl);
    assert_eq!(balances[&key_xorurl].1.sk, unwrap!(key_pair).sk);

    assert_eq!(balances[&key_pk_xor].0, false);
    assert_eq!(balances[&key_pk_xor].1.xorurl, key_pk_xor);
    assert_eq!(balances[&key_pk_xor].1.sk, sk);
}

#[test]
fn calling_safe_cat_safekey() {
    let (safekey_xorurl, _sk) = create_preload_and_get_keys("0");

    let cat_output = cmd!(get_bin_location(), "cat", &safekey_xorurl,)
        .read()
        .unwrap();

    assert_eq!(cat_output, "No content to show since the URL targets a SafeKey. Use -i / --info flag to obtain additional information about the targeted SafeKey.");
}

#[test]
fn calling_safe_cat_safekey_nrsurl_with_info_level_3() {
    let (safekey_xorurl, _sk) = create_preload_and_get_keys("0");

    let nrsurl = format!("safe://{}", get_random_nrs_string());
    let _ = cmd!(
        get_bin_location(),
        "nrs",
        "create",
        &nrsurl,
        "-l",
        &safekey_xorurl,
    )
    .read()
    .unwrap();

    let cat_output = cmd!(
        get_bin_location(),
        "cat",
        &nrsurl,
        "--json",
        "--info",
        "--info",
        "--info"
    )
    .read()
    .unwrap();

    let content_info: (String, SafeData) = serde_json::from_str(&cat_output)
        .expect("Failed to parse output of `safe cat` with -ii on file");
    assert_eq!(content_info.0, nrsurl);
    if let SafeData::SafeKey { resolved_from, .. } = content_info.1 {
        let unwrapped_resolved_from = resolved_from.unwrap();
        assert_eq!(
            unwrapped_resolved_from.public_name,
            nrsurl.replace("safe://", "")
        );
        assert_eq!(
            unwrapped_resolved_from.data_type,
            SafeDataType::PublishedSeqAppendOnlyData
        );
    } else {
        panic!("Content retrieved was unexpected: {:?}", content_info);
    }
}
