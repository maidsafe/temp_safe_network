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
use sn_api::test_helpers::get_next_bearer_dbc;
use sn_cmd_test_utilities::util::{
    get_bearer_dbc_on_file, get_owned_dbc_on_file, get_random_string, parse_keys_create_output,
    parse_wallet_create_output, safe_cmd, safe_cmd_stdout,
};

#[test]
fn wallet_create_should_create_a_wallet() -> Result<()> {
    safe_cmd(["wallet", "create"], Some(0))?
        .assert()
        .stdout(predicate::str::contains("Wallet created at"))
        .success();

    Ok(())
}

#[tokio::test]
async fn wallet_deposit_should_deposit_a_bearer_dbc() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, balance) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.path().display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited ({} safecoins) with name 'my-first-dbc' in wallet located at \"{}\"\n",
        balance, wallet_xorurl
    ))
    .success();

    Ok(())
}

#[tokio::test]
async fn wallet_deposit_should_deposit_a_dbc_from_a_file() -> Result<()> {
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, balance) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited ({} safecoins) with name 'my-first-dbc' in wallet located at \"{}\"\n",
        balance, wallet_xorurl
    ))
    .success();

    Ok(())
}

#[tokio::test]
async fn wallet_deposit_should_deposit_a_dbc_from_a_file_with_whitespace_at_the_end() -> Result<()>
{
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let dbc_file_path = tmp_data_dir.child(get_random_string());
    let (dbc, balance) = get_next_bearer_dbc().await.map_err(|err| eyre!(err))?;
    dbc_file_path.write_str(&format!("{}  \n", dbc.to_hex()?))?;

    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.path().display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited ({} safecoins) with name 'my-first-dbc' in wallet located at \"{}\"\n",
        balance, wallet_xorurl
    ))
    .success();

    Ok(())
}

#[test]
fn wallet_deposit_should_fail_with_suggestion_when_file_does_not_contain_dbc_data() -> Result<()> {
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let dbc_file_path = tmp_data_dir.child("dbc_with_12_300_000_000");
    dbc_file_path.write_str("this file does not have dbc data")?;

    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.path().display().to_string(),
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
#[tokio::test]
async fn wallet_deposit_should_deposit_an_owned_dbc_with_configured_secret_key() -> Result<()> {
    let keys_show_output = safe_cmd_stdout(["keys", "show", "--json"], Some(0))?;
    let (pk_hex, _) = parse_keys_create_output(&keys_show_output)?;
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    let reissue_output = safe_cmd_stdout(
        [
            "wallet",
            "reissue",
            "3.15",
            "--from",
            &wallet_xorurl,
            "--to",
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
        "Spendable DBC deposited (3.150000000 safecoins) with name 'reissued-dbc' in wallet located at \"{}\"\n",
         wallet_xorurl
    ))
    .success();
    Ok(())
}

#[tokio::test]
async fn wallet_deposit_should_deposit_an_owned_dbc_with_secret_key_arg() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    let sk = bls::SecretKey::random();
    let pk = sk.public_key();

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
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
            "--to",
            &pk.to_hex(),
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
            &sk.to_hex(),
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(format!(
        "Spendable DBC deposited (7.150000000 safecoins) with name 'reissued-dbc' in wallet located at \"{}\"\n",
         wallet_xorurl
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
#[tokio::test]
#[ignore = "this test is problematic when running in parallel"]
async fn wallet_deposit_owned_dbc_with_no_secret_key_or_credentials_should_fail_with_suggestion(
) -> Result<()> {
    let pk_hex = bls::SecretKey::random().public_key().to_hex();
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    let reissue_output = safe_cmd_stdout(
        [
            "wallet",
            "reissue",
            "0.15",
            "--from",
            &wallet_xorurl,
            "--to",
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

#[tokio::test]
async fn wallet_deposit_owned_dbc_with_secret_key_that_does_not_match_should_fail_with_suggestion(
) -> Result<()> {
    let sk = bls::SecretKey::random();
    let sk2 = bls::SecretKey::random();
    let pk_hex = sk.public_key().to_hex();
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    let reissue_output = safe_cmd_stdout(
        [
            "wallet",
            "reissue",
            "0.15",
            "--from",
            &wallet_xorurl,
            "--to",
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

#[tokio::test]
async fn wallet_balance_should_report_the_balance_of_a_wallet() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, balance) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(["wallet", "balance", &wallet_xorurl], Some(0))?
        .assert()
        .stdout(format!(
            "Wallet at \"{}\" has a total balance of {} safecoins\n",
            wallet_xorurl, balance
        ))
        .success();

    Ok(())
}

#[tokio::test]
async fn wallet_reissue_should_reissue_a_bearer_dbc_from_a_deposited_dbc() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(
        ["wallet", "reissue", "1.15", "--from", &wallet_xorurl],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        "Reissued DBC with 1.15 safecoins.",
    ))
    .stdout(predicate::str::contains(
        "This is a bearer DBC that can be spent by anyone.",
    ))
    .success();

    Ok(())
}

#[tokio::test]
async fn wallet_reissue_should_reissue_an_owned_dbc_from_a_deposited_dbc() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    let pk_hex = bls::SecretKey::random().public_key().to_hex();

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "reissue",
            "2.15",
            "--from",
            &wallet_xorurl,
            "--to",
            &pk_hex,
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        "Reissued DBC with 2.15 safecoins.",
    ))
    .stdout(predicate::str::contains(format!(
        "This DBC is owned by public key {}",
        pk_hex
    )))
    .success();

    Ok(())
}

#[tokio::test]
async fn wallet_reissue_with_owned_arg_should_reissue_with_configured_public_key() -> Result<()> {
    let keys_show_output = safe_cmd_stdout(["keys", "show", "--json"], Some(0))?;
    let (pk_hex, _) = parse_keys_create_output(&keys_show_output)?;
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "reissue",
            "0.15",
            "--from",
            &wallet_xorurl,
            "--owned",
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        "Reissued DBC with 0.15 safecoins.",
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
#[tokio::test]
#[ignore = "this test is problematic when running in parallel"]
async fn wallet_reissue_with_owned_arg_should_fail_if_credentials_are_not_configured() -> Result<()>
{
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
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
            "0.15",
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

#[tokio::test]
async fn wallet_reissue_with_owned_and_public_key_args_should_fail_with_suggestion() -> Result<()> {
    let wallet_create_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&wallet_create_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    let sk = bls::SecretKey::random();

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "my-first-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "reissue",
            "1.15",
            "--from",
            &wallet_xorurl,
            "--owned",
            "--to",
            &sk.public_key().to_hex(),
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The --owned and --to arguments are mutually exclusive.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again and use one or the other, but not both, of these arguments.",
    ))
    .failure();

    Ok(())
}

#[tokio::test]
async fn wallet_deposit_should_fail_since_bearer_dbc_is_spent() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd_stdout(
        ["wallet", "reissue", "0.5", "--from", &wallet_xorurl],
        Some(0),
    )?;

    // trying to deposit the already spent DBC shall fail
    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The supplied DBC has been already spent on the network.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the --force flag if you still \
            wish to deposit it into the wallet.",
    ))
    .failure();

    Ok(())
}

#[tokio::test]
async fn wallet_deposit_should_fail_since_owned_dbc_is_spent() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, sk, _) = get_owned_dbc_on_file(&tmp_data_dir).await?;
    let sk_hex = sk.to_hex();

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &dbc_file_path.display().to_string(),
            "--secret-key",
            &sk_hex,
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd_stdout(
        ["wallet", "reissue", "0.555", "--from", &wallet_xorurl],
        Some(0),
    )?;

    // trying to deposit again the already spent DBC shall fail
    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &dbc_file_path.display().to_string(),
            "--secret-key",
            &sk_hex,
            &wallet_xorurl,
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The supplied DBC has been already spent on the network.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the --force flag if you still \
            wish to deposit it into the wallet.",
    ))
    .failure();

    Ok(())
}

#[tokio::test]
async fn wallet_deposit_spent_bearer_dbc_force() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, balance) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd_stdout(
        ["wallet", "reissue", "0.5", "--from", &wallet_xorurl],
        Some(0),
    )?;

    // force to deposit the already spent DBC
    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "spent-bearer-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
            "--force"
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        "WARNING: --force flag set, hence skipping verification to check if \
        supplied DBC has been already spent."
    ))
    .stdout(predicate::str::contains(
        format!(
            "Spendable DBC deposited ({} safecoins) with name 'spent-bearer-dbc' in wallet located at \"{}\"\n",
            balance, wallet_xorurl
        )
    ))
    .success();

    Ok(())
}

#[tokio::test]
async fn wallet_deposit_spent_owned_dbc_force() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, sk, balance) = get_owned_dbc_on_file(&tmp_data_dir).await?;
    let sk_hex = sk.to_hex();

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &dbc_file_path.display().to_string(),
            "--secret-key",
            &sk_hex,
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd_stdout(
        ["wallet", "reissue", "0.666", "--from", &wallet_xorurl],
        Some(0),
    )?;

    // force to deposit the already spent DBC
    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "spent-owned-dbc",
            "--dbc",
            &dbc_file_path.display().to_string(),
            "--secret-key",
            &sk_hex,
            &wallet_xorurl,
            "--force"
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        "WARNING: --force flag set, hence skipping verification to check if \
        supplied DBC has been already spent."
    ))
    .stdout(predicate::str::contains(
        format!(
            "Spendable DBC deposited ({} safecoins) with name 'spent-owned-dbc' in wallet located at \"{}\"\n",
            balance, wallet_xorurl
        )
    ))
    .success();

    Ok(())
}

#[tokio::test]
async fn wallet_reissue_to_file_succeeds() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (from_dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &from_dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    let to_dbc_file_path = tmp_data_dir.child(get_random_string());

    safe_cmd(
        [
            "wallet",
            "reissue",
            "0.987654321",
            "--from",
            &wallet_xorurl,
            "--save",
            &to_dbc_file_path.display().to_string(),
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(format!(
        "DBC content written at '{}'.",
        to_dbc_file_path.display()
    )))
    .stdout(predicate::str::contains("-------- DBC DATA --------").not())
    .stdout(predicate::str::contains(
        "Reissued DBC with 0.987654321 safecoins.",
    ))
    .success();

    Ok(())
}

#[tokio::test]
async fn wallet_reissue_to_file_fails() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "reissue",
            "0.123456789",
            "--from",
            &wallet_xorurl,
            "--save",
            "/invalid/file/path",
        ],
        Some(0),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "Error: Unable to write DBC at '/invalid/file/path'",
    ))
    .stdout(predicate::str::contains("-------- DBC DATA --------"))
    .stdout(predicate::str::contains(
        "Reissued DBC with 0.123456789 safecoins.",
    ))
    .success();

    Ok(())
}

#[tokio::test]
async fn wallet_deposit_dbc_reissued_to_file() -> Result<()> {
    let json_output = safe_cmd_stdout(["wallet", "create", "--json"], Some(0))?;
    let wallet_xorurl = parse_wallet_create_output(&json_output)?;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    let (from_dbc_file_path, _, _) = get_bearer_dbc_on_file(&tmp_data_dir).await?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--dbc",
            &from_dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?;

    let to_dbc_file_path = tmp_data_dir.child(get_random_string());

    safe_cmd(
        [
            "wallet",
            "reissue",
            "0.111",
            "--from",
            &wallet_xorurl,
            "--save",
            &to_dbc_file_path.display().to_string(),
        ],
        Some(0),
    )?;

    safe_cmd(
        [
            "wallet",
            "deposit",
            "--name",
            "dbc-reissued-to-file",
            "--dbc",
            &to_dbc_file_path.display().to_string(),
            &wallet_xorurl,
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(
        format!(
            "Spendable DBC deposited (0.111000000 safecoins) with name 'dbc-reissued-to-file' in wallet located at \"{}\"\n",
            wallet_xorurl
        )
    ))
    .success();

    Ok(())
}
