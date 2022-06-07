// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use color_eyre::Result;
use predicates::prelude::*;
use sn_cmd_test_utilities::util::{
    parse_wallet_create_output, safe_cmd, safe_cmd_stdout, DBC_WITH_12_230_000_000,
};

#[test]
fn wallet_create_should_create_a_wallet() -> Result<()> {
    safe_cmd(["wallet", "create"], Some(0))?
        .assert()
        .stdout(predicate::str::contains("Wallet created at"))
        .success();

    Ok(())
}

#[test]
fn wallet_deposit_should_deposit_a_dbc() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            DBC_WITH_12_230_000_000,
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited with name '{}' in wallet located at \"{}\"\n",
        "my-first-dbc", wallet_xorurl
    ))
    .success();

    Ok(())
}

#[test]
fn wallet_deposit_should_deposit_a_dbc_from_a_file() -> Result<()> {
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let dbc_data_file = tmp_data_dir.child("dbc_with_12_300_000_000");
    dbc_data_file.write_str(DBC_WITH_12_230_000_000)?;

    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_data_file.path().display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited with name '{}' in wallet located at \"{}\"\n",
        "my-first-dbc", wallet_xorurl
    ))
    .success();

    Ok(())
}

#[test]
fn wallet_deposit_should_fail_with_suggestion_when_file_does_not_contain_dbc_data() -> Result<()> {
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let dbc_data_file = tmp_data_dir.child("dbc_with_12_300_000_000");
    dbc_data_file.write_str("this file does not have dbc data")?;

    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_data_file.path().display().to_string(),
            &wallet_xorurl,
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "This file does not appear to have DBC data.",
    ))
    .stderr(predicate::str::contains(
        "Please select another file with valid hex-encoded DBC data.",
    ))
    .failure();

    Ok(())
}

#[test]
fn wallet_deposit_should_fail_with_suggestion_when_path_is_directory() -> Result<()> {
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &tmp_data_dir.path().display().to_string(),
            &wallet_xorurl,
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The path supplied refers to a directory.",
    ))
    .stderr(predicate::str::contains(
        "A file path must be specified for the DBC data.",
    ))
    .failure();

    Ok(())
}

#[test]
fn wallet_balance_should_report_the_balance_of_a_wallet() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            DBC_WITH_12_230_000_000,
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(["wallet", "balance", &wallet_xorurl], Some(0))?
        .assert()
        .stdout(format!(
            "Wallet at \"{}\" has a total balance of 12.230000000 safecoins\n",
            wallet_xorurl
        ))
        .success();

    Ok(())
}

#[test]
fn wallet_reissue_should_reissue_a_bearer_dbc_from_a_deposited_dbc() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            DBC_WITH_12_230_000_000,
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(
        ["wallet", "reissue", "7.15", "--from", &wallet_xorurl],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        "Reissued DBC with 7.15 safecoins.",
    ))
    .stdout(predicate::str::contains(
        "This is a bearer DBC that can be spent by anyone.",
    ))
    .success();

    Ok(())
}

#[test]
fn wallet_reissue_should_reissue_an_owned_dbc_from_a_deposited_dbc() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let pk_hex = "84da738d24dbf226bdcf4a9e27ac8a9f1beaa7c527f46774fb645b8d88850c0dab2159c2d646c35c53cf2fa940a9d6f3";

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            DBC_WITH_12_230_000_000,
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "reissue",
            "7.15",
            "--from",
            &wallet_xorurl,
            "--public-key",
            pk_hex,
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        "Reissued DBC with 7.15 safecoins.",
    ))
    .stdout(predicate::str::contains(format!(
        "This DBC is owned by public key {}",
        pk_hex
    )))
    .success();

    Ok(())
}
