// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{helpers::serialise_output, OutputFmt};
use crate::operations::auth_and_connect::{create_credentials_file, read_credentials};
use crate::operations::config::Config;
use color_eyre::{eyre::bail, eyre::eyre, eyre::WrapErr, Result};
use hex::encode;
use sn_api::{
    resolver::{SafeData, SafeUrl},
    sk_to_hex, Keypair, PublicKey, Safe, XorName,
};
use sn_dbc::{rng, Owner};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum KeysSubCommands {
    /// Show information about a SafeKey. By default it will show the one owned by CLI (if found).
    Show {
        /// Set this flag to show the secret key
        #[structopt(long = "show-sk")]
        show_sk: bool,
        /// The SafeKey's URL to decode and show its Public Key. If this is not provided, the SafeKey owned by CLI (if found) will be shown
        keyurl: Option<String>,
    },
    #[structopt(name = "create")]
    /// Create a new SafeKey. Currently generates an Ed25519 keypair.
    Create {
        /// Set this flag to output the generated keypair to file at ~/.safe/cli/credentials. The
        /// CLI will then sign all commands using this keypair.
        #[structopt(long = "for-cli")]
        for_cli: bool,
    },
    /// Generate a secret key for use with DBC reissues.
    #[structopt(name = "create-dbc-owner")]
    CreateDbcOwner {},
}

pub async fn key_commander(
    cmd: KeysSubCommands,
    output_fmt: OutputFmt,
    safe: &Safe,
    config: &Config,
) -> Result<()> {
    match cmd {
        KeysSubCommands::Show { show_sk, keyurl } => {
            if let Some(url) = keyurl {
                if show_sk {
                    bail!("The 'show-sk' flag cannot be set when providing a SafeKey URL");
                }

                match safe.fetch(&url, None).await {
                    Ok(SafeData::SafeKey {
                        xorurl, xorname, ..
                    }) => {
                        // Get pk from xorname. We assume Ed25519 key for now, which is
                        // 32 bytes long, just like a xorname.
                        // TODO: support for BLS keys which are longer.
                        let pk = ed25519_dalek::PublicKey::from_bytes(&xorname).map_err(|err| {
                            eyre!(
                                "Failed to derive Ed25519 PublicKey from SafeKey at '{}': {:?}",
                                url,
                                err
                            )
                        })?;

                        println!("SafeKey found at {}:", url);
                        println!("XOR-URL: {}", xorurl);
                        println!("Public Key: {}", encode(pk));
                    }
                    Ok(other) => bail!(format!(
                        "The Safe-URL provided is not targetting a SafeKey: {:?}",
                        other
                    )),
                    Err(err) => bail!(err),
                }
            } else {
                match read_credentials(config)? {
                    (file_path, Some(keypair)) => {
                        let xorname = XorName::from(keypair.public_key());
                        let xorurl = SafeUrl::encode_safekey(xorname, safe.xorurl_base)?;
                        let (pk_hex, sk_hex) = keypair_to_hex_strings(&keypair)?;

                        println!("Current CLI's SafeKey found at {}:", file_path.display());
                        println!("XOR-URL: {}", xorurl);
                        println!("Public Key: {}", pk_hex);
                        if show_sk {
                            println!("Secret Key: {}", sk_hex);
                        }
                    }
                    (file_path, None) => println!("No SafeKey found at {}", file_path.display()),
                }
            }

            Ok(())
        }
        KeysSubCommands::Create { for_cli } => {
            let (xorurl, key_pair) = create_new_key(safe).await?;
            print_new_key_output(output_fmt, xorurl, Some(&key_pair));

            if for_cli {
                println!("Setting new SafeKey to be used by CLI...");
                let (mut file, file_path) = create_credentials_file(config)?;
                let serialised_keypair = serde_json::to_string(&key_pair)
                    .wrap_err("Unable to serialise the credentials created")?;

                file.write_all(serialised_keypair.as_bytes())
                    .wrap_err_with(|| {
                        format!("Unable to write credentials in {}", file_path.display(),)
                    })?;

                println!(
                    "New credentials were successfully stored in {}",
                    file_path.display()
                );
                println!("Safe CLI now has write access to the network");
            }
            Ok(())
        }
        KeysSubCommands::CreateDbcOwner { .. } => {
            println!("Generating secret key for DBC reissues...");
            let sk = Owner::from_random_secret_key(&mut rng::thread_rng());
            let path = dbc_owner_to_file(&sk, config)?;
            println!("Saved key at {}", path.display());
            Ok(())
        }
    }
}

pub fn print_new_key_output(output_fmt: OutputFmt, xorurl: String, key_pair: Option<&Keypair>) {
    if OutputFmt::Pretty == output_fmt {
        println!("New SafeKey created: \"{}\"", xorurl);

        if let Some(pair) = &key_pair {
            println!("Key pair generated:");
            match keypair_to_hex_strings(pair) {
                Ok((pk_hex, sk_hex)) => {
                    println!("Public Key = {}", pk_hex);
                    println!("Secret Key = {}", sk_hex);
                }
                Err(err) => println!("{}", err),
            }
        }
    } else if let Some(pair) = &key_pair {
        match keypair_to_hex_strings(pair) {
            Ok((pk_hex, sk_hex)) => println!(
                "{}",
                serialise_output(&(xorurl, (pk_hex, sk_hex)), output_fmt)
            ),
            Err(err) => println!("{}", err),
        }
    }
}

pub fn keypair_to_hex_strings(keypair: &Keypair) -> Result<(String, String)> {
    let pk_hex = match keypair.public_key() {
        PublicKey::Ed25519(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
        PublicKey::Bls(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
        PublicKey::BlsShare(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
    };

    let sk_hex = sk_to_hex(
        keypair
            .secret_key()
            .context("Failed to obtain secret key")?,
    );

    Ok((pk_hex, sk_hex))
}

#[cfg(feature = "testing")]
pub async fn create_new_key(safe: &Safe) -> Result<(String, Keypair)> {
    let key_pair = safe.new_keypair();
    let xorname = XorName::from(key_pair.public_key());
    let xorurl = SafeUrl::encode_safekey(xorname, safe.xorurl_base)?;
    Ok((xorurl, key_pair))
}

fn dbc_owner_to_file(owner: &Owner, config: &Config) -> Result<PathBuf> {
    let hex = hex::encode(owner.to_bytes());
    let mut file_path = config.cli_config_path.clone();
    file_path.pop();
    file_path.push("dbc_sk");
    let mut file = File::create(&file_path).with_context(|| {
        format!(
            "Unable to open dbc secret key file at {}",
            file_path.display()
        )
    })?;
    file.write_all(hex.as_bytes())?;
    Ok(file_path)
}

#[cfg(test)]
mod create_command {
    use super::{key_commander, KeysSubCommands};
    use crate::operations::auth_and_connect::read_credentials;
    use crate::operations::config::Config;
    use crate::subcommands::OutputFmt;
    use assert_fs::prelude::*;
    use color_eyre::{eyre::eyre, Result};
    use predicates::prelude::*;
    use sn_api::{Keypair, Safe};

    #[tokio::test]
    async fn should_create_ed25519_keypair() -> Result<()> {
        let config_dir = assert_fs::TempDir::new()?;
        let credentials_file = config_dir.child(".safe/cli/credentials");
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )
        .await?;
        let safe = Safe::dry_runner(None);

        let result = key_commander(
            KeysSubCommands::Create { for_cli: false },
            OutputFmt::Pretty,
            &safe,
            &config,
        )
        .await;

        assert!(result.is_ok());
        credentials_file.assert(predicate::path::missing());
        Ok(())
    }

    #[tokio::test]
    async fn should_create_ed25519_keypair_saved_to_credentials_file() -> Result<()> {
        let config_dir = assert_fs::TempDir::new()?;
        let credentials_file = config_dir.child(".safe/cli/credentials");
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )
        .await?;
        let safe = Safe::dry_runner(None);

        let result = key_commander(
            KeysSubCommands::Create { for_cli: true },
            OutputFmt::Pretty,
            &safe,
            &config,
        )
        .await;

        assert!(result.is_ok());
        credentials_file.assert(predicate::path::is_file());

        let (_, keypair) = read_credentials(&config)?;
        let keypair =
            keypair.ok_or_else(|| eyre!("The command should have generated a keypair"))?;
        match keypair {
            Keypair::Ed25519(_) => Ok(()),
            _ => Err(eyre!("The command should generate a Ed25519 keypair")),
        }
    }
}

#[cfg(test)]
mod create_dbc_owner_command {
    use super::{key_commander, KeysSubCommands};
    use crate::operations::config::Config;
    use crate::subcommands::OutputFmt;
    use assert_fs::prelude::*;
    use bls::SecretKey;
    use color_eyre::{eyre::eyre, Result};
    use predicates::prelude::*;
    use sn_api::Safe;
    use sn_dbc::Owner;

    #[tokio::test]
    async fn should_create_a_dbc_owner_secret_key() -> Result<()> {
        let config_dir = assert_fs::TempDir::new()?;
        let db_sk_file = config_dir.child(".safe/cli/dbc_sk");
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )
        .await?;
        let safe = Safe::dry_runner(None);

        let result = key_commander(
            KeysSubCommands::CreateDbcOwner {},
            OutputFmt::Pretty,
            &safe,
            &config,
        )
        .await;

        assert!(result.is_ok());
        db_sk_file.assert(predicate::path::is_file());

        // There's no real properties we can check on the key/owner, so just make sure it
        // deserializes without error. That at least verifies that the file is a valid SecretKey.
        let hex = std::fs::read_to_string(db_sk_file.path())?;
        let sk: SecretKey = bincode::deserialize(hex.as_bytes()).map_err(|e| eyre!(e))?;
        let _owner = Owner::from(sk);

        Ok(())
    }
}
