// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// Relaxing rules for this file since these tests are only run when 'node-ctrl' feature flag is set.
#![allow(dead_code, unused_imports)]

use color_eyre::{eyre::eyre, Result};
use sn_cmd_test_utilities::util::{safe_cmd_stdout, use_isolated_safe_config_dir};

// Address of the genesis node RPC service
const GENESIS_NODE_RPC_ADDR: &str = "127.0.0.1:12001";

#[test]
#[cfg(feature = "node-ctrl")]
fn node_info() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;
    let output = safe_cmd_stdout(
        &config_dir,
        ["node", GENESIS_NODE_RPC_ADDR, "info", "--json"],
        Some(0),
    )?;

    let (endpoint, name, is_elder, log_dir, bin_version): (String, String, bool, String, String) =
        serde_json::from_str(&output)
            .map_err(|_| eyre!("Failed to parse output of `safe node info`: {output}",))?;

    assert!(is_elder);
    assert!(!bin_version.is_empty());
    assert!(!name.is_empty());
    assert!(!log_dir.is_empty());
    assert_eq!(endpoint, format!("http://{GENESIS_NODE_RPC_ADDR}"));

    Ok(())
}

#[test]
#[cfg(feature = "node-ctrl")]
fn node_section_members() -> Result<()> {
    let config_dir = use_isolated_safe_config_dir()?;

    // let's find out the genesis node's name first
    let output = safe_cmd_stdout(
        &config_dir,
        ["node", GENESIS_NODE_RPC_ADDR, "info", "--json"],
        Some(0),
    )?;
    let (_, genesis_name, _, _, _): (String, String, bool, String, String) =
        serde_json::from_str(&output)
            .map_err(|_| eyre!("Failed to parse output of `safe node info`: {output}",))?;

    let output = safe_cmd_stdout(
        &config_dir,
        ["node", GENESIS_NODE_RPC_ADDR, "section", "--json"],
        Some(0),
    )?;
    let section_members: Vec<(String, bool, String)> = serde_json::from_str(&output)
        .map_err(|_| eyre!("Failed to parse output of `safe node section`: {output}",))?;

    assert!(section_members.len() > 7);
    assert!(section_members
        .iter()
        .any(|(name, is_elder, _)| *is_elder && name == &genesis_name));

    Ok(())
}
