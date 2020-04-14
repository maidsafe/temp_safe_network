// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

extern crate safe_cmd_test_utilities;

#[macro_use]
extern crate duct;

use safe_api::fetch::{SafeData, SafeDataType};
use safe_cmd_test_utilities::{
    create_preload_and_get_keys, get_bin_location, get_random_nrs_string,
    parse_files_put_or_sync_output,
};

const TEST_FILE: &str = "../testdata/test.md";

#[test]
fn calling_safe_dog_files_container_nrsurl() {
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

    let dog_output = cmd!(get_bin_location(), "dog", &nrsurl, "--json",)
        .read()
        .unwrap();

    let content_info: (String, SafeData) = serde_json::from_str(&dog_output)
        .expect("Failed to parse output of `safe dog` with -ii on file");
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
fn calling_safe_dog_files_container_nrsurl_jsoncompact() {
    let content = cmd!(
        get_bin_location(),
        "files",
        "put",
        TEST_FILE,
        "--output=jsoncompact"
    )
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

    let dog_output = cmd!(get_bin_location(), "dog", &nrsurl, "--output=jsoncompact",)
        .read()
        .unwrap();

    let content_info: (String, SafeData) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog`");
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
fn calling_safe_dog_files_container_nrsurl_yaml() {
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

    let dog_output = cmd!(get_bin_location(), "dog", &nrsurl, "--output=yaml",)
        .read()
        .unwrap();

    let content_info: (String, SafeData) =
        serde_yaml::from_str(&dog_output).expect("Failed to parse output of `safe dog`");
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
fn calling_safe_dog_safekey_nrsurl() {
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

    let dog_output = cmd!(get_bin_location(), "dog", &nrsurl, "--json",)
        .read()
        .unwrap();

    let content_info: (String, SafeData) = serde_json::from_str(&dog_output)
        .expect("Failed to parse output of `safe dog` with -ii on file");
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
