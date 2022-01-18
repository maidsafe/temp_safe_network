// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use color_eyre::{eyre::eyre, Result};
use sn_api::resolver::SafeData;
use sn_cmd_test_utilities::util::{
    create_and_get_keys, get_random_nrs_string, parse_dog_output, parse_files_put_or_sync_output,
    safe_cmd, safe_cmd_stdout, safeurl_from,
};

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
    assert_eq!(url, nrsurl);

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
    assert_eq!(url, nrsurl);

    if let Some(SafeData::FilesContainer { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, container_xorurl);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {:?}", content);
    }
}

#[test]
#[ignore = "content version issue"]
fn calling_safe_dog_safekey_nrsurl() -> Result<()> {
    let (safekey_xorurl, _sk) = create_and_get_keys()?;

    let nrsurl = get_random_nrs_string();
    safe_cmd(["nrs", "register", &nrsurl, "-l", &safekey_xorurl], Some(0))?;
    let dog_output = safe_cmd_stdout(["dog", &nrsurl, "--json"], Some(0))?;
    let (url, mut content): (String, Vec<SafeData>) =
        serde_json::from_str(&dog_output).expect("Failed to parse output of `safe dog` on file");
    assert_eq!(url, nrsurl);

    if let Some(SafeData::SafeKey { resolved_from, .. }) = content.pop() {
        assert_eq!(resolved_from, safekey_xorurl);
        Ok(())
    } else {
        Err(eyre!("Content retrieved was unexpected: {:?}", content))
    }
}

#[test]
#[ignore = "still broken"]
fn calling_safe_dog_nrs_url_with_subnames() -> Result<()> {
    let (safekey_xorurl, _sk) = create_and_get_keys()?;

    let pub_name = get_random_nrs_string();
    let nrsurl = format!("subname.{}", pub_name);
    safe_cmd(["nrs", "register", &nrsurl, "-l", &safekey_xorurl], Some(0))?;

    // let's check the output with NRS-URL first
    let dog_output = safe_cmd_stdout(["dog", &nrsurl, "--json"], Some(0))?;
    let (url, safe_data_vec) = parse_dog_output(&dog_output)?;
    assert_eq!(url, nrsurl);
    let mut safeurl = safeurl_from(&nrsurl)?;
    safeurl.set_sub_names("").map_err(|e| eyre!(e))?;
    let nrs_map_xorurl = safeurl.to_xorurl_string();

    if let sn_api::resolver::SafeData::NrsMapContainer {
        resolved_from,
        xorurl,
        public_name,
        ..
    } = &safe_data_vec[0]
    {
        assert_eq!(*resolved_from, nrsurl);
        assert_eq!(*xorurl, nrs_map_xorurl);
        assert_eq!(*public_name, Some(pub_name));
    } else {
        panic!("Content retrieved was unexpected: {:?}", safe_data_vec);
    }

    // let's now check the output with its XOR-URL
    let dog_output = safe_cmd_stdout(["dog", &nrs_map_xorurl, "--json"], Some(0))?;
    let (url, safe_data_vec) = parse_dog_output(&dog_output)?;
    assert_eq!(url, *nrs_map_xorurl);
    if let sn_api::resolver::SafeData::NrsMapContainer {
        resolved_from,
        xorurl,
        public_name,
        ..
    } = &safe_data_vec[0]
    {
        assert_eq!(*resolved_from, nrs_map_xorurl);
        assert_eq!(*xorurl, nrs_map_xorurl);
        // it doesn't know the public name as it was resolved from a XOR-URL
        assert_eq!(*public_name, None);
        Ok(())
    } else {
        panic!("Content retrieved was unexpected: {:?}", safe_data_vec);
    }
}
