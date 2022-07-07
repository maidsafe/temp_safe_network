// Copyright 2022 MaidSafe.net limited.
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
use sn_cmd_test_utilities::util::{
    parse_keys_create_output, parse_wallet_create_output, safe_cmd, safe_cmd_stdout,
    DBC_WITH_12_230_000_000,
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
fn wallet_deposit_should_deposit_a_bearer_dbc() -> Result<()> {
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
        "Spendable DBC deposited (12.230000000 safecoins) with name '{}' in wallet located at \"{}\"\n",
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
        "Spendable DBC deposited (12.230000000 safecoins) with name '{}' in wallet located at \"{}\"\n",
        "my-first-dbc", wallet_xorurl
    ))
    .success();

    Ok(())
}

#[test]
fn wallet_deposit_should_deposit_a_dbc_from_a_file_with_whitespace_at_the_end() -> Result<()> {
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let dbc_data_file = tmp_data_dir.child("dbc_with_12_300_000_000");
    dbc_data_file.write_str(&format!("{}  \n", DBC_WITH_12_230_000_000))?;

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
        "Spendable DBC deposited (12.230000000 safecoins) with name '{}' in wallet located at \"{}\"\n",
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

/// Deposit an owned DBC that uses the secret key that's configured for use with the CLI.
/// Therefore, this test requires the generation of those credentials in advance. We should
/// probably refactor the CLI test suite so that each test case that requires credentials creates
/// its own new keypair, but we don't have that infrastructure available at the moment.
#[test]
fn wallet_deposit_should_deposit_an_owned_dbc_with_configured_secret_key() -> Result<()> {
    let keys_show_output = safe_cmd_stdout(["keys", "show", "--json"], Some(0))?;
    let (pk_hex, _) = parse_keys_create_output(&keys_show_output)?;
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;

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

    let reissue_output = safe_cmd_stdout(
        [
            "wallet",
            "reissue",
            "7.15",
            "--from",
            &wallet_xorurl,
            "--public-key",
            &pk_hex,
            "--json",
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "reissued-dbc",
            "--dbc",
            &reissue_output,
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited (7.150000000 safecoins) with name '{}' in wallet located at \"{}\"\n",
        "reissued-dbc", wallet_xorurl
    ))
    .success();
    Ok(())
}

#[test]
fn wallet_deposit_should_deposit_an_owned_dbc_with_secret_key_arg() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let pk_hex = "8664d6e363117516a6816ad62a0960fdef274c784934e208a7134594888e2454\
                  f3ed300ba87c1783c79cb671edcf4d95";
    let sk_hex = "20d58316dc97d533a798dafa08ac46811219e532ee8ba419e9be2ebe7a1e1f24";

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

    let reissue_output = safe_cmd_stdout(
        [
            "wallet",
            "reissue",
            "7.15",
            "--from",
            &wallet_xorurl,
            "--public-key",
            pk_hex,
            "--json",
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "reissued-dbc",
            "--dbc",
            &reissue_output,
            "--secret-key",
            sk_hex,
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited (7.150000000 safecoins) with name '{}' in wallet located at \"{}\"\n",
        "reissued-dbc", wallet_xorurl
    ))
    .success();
    Ok(())
}

/// This test is a special case that requires the deletion of the credentials file at
/// ~/.safe/cli/credentials. The scenario is, we are trying to deposit a DBC, but a secret key
/// hasn't been supplied and there aren't any credentials are available for the CLI to try.
///
/// Some of the remaining tests in the CLI test suite require the existence of the generated
/// credentials, so at the end of this test, we will generate a new set. The best solution would
/// probably be for each test case that requires credentials to generate its own keypair, but at
/// the moment we don't have that infrastructure.
#[test]
#[ignore = "this test is problematic when running in parallel"]
fn wallet_deposit_owned_dbc_with_no_secret_key_or_credentials_should_fail_with_suggestion(
) -> Result<()> {
    let pk_hex = bls::SecretKey::random().public_key().to_hex();
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;

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

    let reissue_output = safe_cmd_stdout(
        [
            "wallet",
            "reissue",
            "7.15",
            "--from",
            &wallet_xorurl,
            "--public-key",
            &pk_hex,
            "--json",
        ],
        Some(0),
    )?;

    // In a real world scenario, there would have been some long period of time between the reissue
    // and depositing the DBC. In that period of time, the credentials file would have been
    // deleted, most likely by accident, but it could have been intentional.
    let home_path =
        dirs_next::home_dir().ok_or_else(|| eyre!("Couldn't find user's home directory"))?;
    std::fs::remove_file(home_path.join(".safe/cli/credentials"))?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "reissued-dbc",
            "--dbc",
            &reissue_output,
            &wallet_xorurl,
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "This is an owned DBC. To deposit, it requires a secret key.",
    ))
    .stderr(predicate::str::contains(
        "A secret key was not supplied and there were no credentials configured for use with safe.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again using the --secret-key argument to specify the key.",
    ))
    .failure();

    safe_cmd(["keys", "create", "--for-cli"], Some(0))?;
    Ok(())
}

#[test]
fn wallet_deposit_owned_dbc_with_secret_key_that_does_not_match_should_fail_with_suggestion(
) -> Result<()> {
    let sk = bls::SecretKey::random();
    let sk2 = bls::SecretKey::random();
    let pk_hex = sk.public_key().to_hex();
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;

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

    let reissue_output = safe_cmd_stdout(
        [
            "wallet",
            "reissue",
            "7.15",
            "--from",
            &wallet_xorurl,
            "--public-key",
            &pk_hex,
            "--json",
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "reissued-dbc",
            "--dbc",
            &reissue_output,
            "--secret-key",
            &sk2.to_hex(),
            &wallet_xorurl,
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The supplied secret key did not match the public key for this DBC.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the correct key for the --secret-key argument.",
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

#[test]
fn wallet_reissue_with_owned_arg_should_reissue_with_configured_public_key() -> Result<()> {
    let keys_show_output = safe_cmd_stdout(["keys", "show", "--json"], Some(0))?;
    let (pk_hex, _) = parse_keys_create_output(&keys_show_output)?;
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;

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
            "--owned",
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

/// This test is a special case that requires the deletion of the credentials file at
/// ~/.safe/cli/credentials. The scenario is, we are trying to reissue an owned DBC using the
/// public key with the credentials configured for the CLI, but there are no credentials at the
/// point of reissue.
///
/// Some of the remaining tests in the CLI test suite require the existence of the generated
/// credentials, so at the end of this test, we will generate a new set. The best solution would
/// probably be for each test case that requires credentials to generate its own keypair, but at
/// the moment we don't have that infrastructure.
#[test]
#[ignore = "this test is problematic when running in parallel"]
fn wallet_reissue_with_owned_arg_should_fail_if_credentials_are_not_configured() -> Result<()> {
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;

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

    // In a real world scenario, there would have been some long period of time between the wallet
    // being created and the reissue. In that period of time, the credentials file would have been
    // deleted, most likely by accident, but it could have been intentional.
    let home_path =
        dirs_next::home_dir().ok_or_else(|| eyre!("Couldn't find user's home directory"))?;
    std::fs::remove_file(home_path.join(".safe/cli/credentials"))?;

    safe_cmd(
        [
            "wallet",
            "reissue",
            "7.15",
            "--from",
            &wallet_xorurl,
            "--owned",
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The --owned argument requires credentials to be configured for safe.",
    ))
    .stderr(predicate::str::contains(
        "Run the 'keys create --for-cli' command to generate a credentials then run this command again.",
    ))
    .failure();

    safe_cmd(["keys", "create", "--for-cli"], Some(0))?;
    Ok(())
}

#[test]
fn wallet_reissue_with_owned_and_public_key_args_should_fail_with_suggestion() -> Result<()> {
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;
    let sk = bls::SecretKey::random();

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
            "--owned",
            "--public-key",
            &sk.public_key().to_hex(),
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The --owned and --public-key arguments are mutually exclusive.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again and use one or the other, but not both, of these arguments.",
    ))
    .failure();

    Ok(())
}
