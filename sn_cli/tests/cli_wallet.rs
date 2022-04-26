// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use assert_cmd::prelude::*;
use color_eyre::Result;
use predicates::prelude::*;
use sn_cmd_test_utilities::util::{
    parse_wallet_create_output, safe_cmd, safe_cmd_stdout, DBC_WITH_12_230_000_000,
};

#[test]
fn calling_safe_wallet_create() -> Result<()> {
    safe_cmd(["wallet", "create"], Some(0))?
        .assert()
        .stdout(predicate::str::contains("Wallet created at"))
        .success();

    Ok(())
}

#[test]
fn calling_safe_wallet_deposit() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            &wallet_xorurl,
            DBC_WITH_12_230_000_000,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited with name '{}' in Wallet located at \"{}\"\n",
        "my-first-dbc", wallet_xorurl
    ))
    .success();

    Ok(())
}

#[test]
fn calling_safe_wallet_balance() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            &wallet_xorurl,
            DBC_WITH_12_230_000_000,
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
fn calling_safe_wallet_reissue() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            &wallet_xorurl,
            DBC_WITH_12_230_000_000,
        ],
        Some(0),
    )?;

    safe_cmd(
        ["wallet", "reissue", "7.15", "--from", &wallet_xorurl],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        "Success. Reissued DBC with 7.15 safecoins",
    ))
    .success();

    Ok(())
}

#[test]
fn calling_safe_wallet_deposit_reissued() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            &wallet_xorurl,
            DBC_WITH_12_230_000_000,
        ],
        Some(0),
    )?;

    let reissued_dbc = safe_cmd_stdout(
        [
            "wallet",
            "reissue",
            "1.33",
            "--json",
            "--from",
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "reissued-dbc",
            &wallet_xorurl,
            &reissued_dbc,
        ],
        Some(0),
    )?;

    // Let's check the balance to make sure the deposited DBC was being
    // serialised (by the reissue cmd) and deserialised (by the deposit cmd) correctly
    safe_cmd(["wallet", "balance", &wallet_xorurl], Some(0))?
        .assert()
        .stdout(format!(
            "Wallet at \"{}\" has a total balance of 12.230000000 safecoins\n",
            wallet_xorurl
        ))
        .success();

    Ok(())
}
