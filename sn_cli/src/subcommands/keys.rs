// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{helpers::serialise_output, OutputFmt};
use crate::operations::auth_and_connect::{get_credentials_file_path, read_credentials};
use crate::operations::config::Config;
use color_eyre::{eyre::WrapErr, Result};
use sn_api::{resolver::SafeUrl, Keypair, Safe, XorName};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum KeysSubCommands {
    /// Show information about a SafeKey. By default it will show the one owned by CLI (if found).
    Show {
        /// Set this flag to show the secret key
        #[structopt(long = "show-sk")]
        show_sk: bool,
    },
    #[structopt(name = "create")]
    /// Create a new SafeKey in BLS format.
    Create {
        /// Set this flag to output the generated keypair to file at ~/.safe/cli/credentials. The
        /// CLI will then sign all commands using this keypair.
        #[structopt(long = "for-cli")]
        for_cli: bool,
    },
}

pub async fn key_commander(
    cmd: KeysSubCommands,
    output_fmt: OutputFmt,
    safe: &Safe,
    config: &Config,
) -> Result<()> {
    match cmd {
        KeysSubCommands::Show { show_sk } => {
            match read_credentials(safe, config)? {
                (file_path, Some(keypair)) => {
                    let xorname = XorName::from(keypair.public_key());
                    let xorurl = SafeUrl::encode_safekey(xorname, safe.xorurl_base)?;
                    let (pk_hex, sk_hex) = keypair.to_hex()?;

                    println!("Current CLI's SafeKey found at {}:", file_path.display());
                    println!("XOR-URL: {}", xorurl);
                    println!("Public Key: {}", pk_hex);
                    if show_sk {
                        println!("Secret Key: {}", sk_hex);
                    }
                }
                (file_path, None) => println!("No SafeKey found at {}", file_path.display()),
            }

            Ok(())
        }
        KeysSubCommands::Create { for_cli } => {
            let (keypair, url) = safe.new_keypair_with_pk_url()?;
            print_new_key_output(output_fmt, url, Some(&keypair));

            if for_cli {
                let (_, path) = get_credentials_file_path(config)?;
                safe.serialize_keypair(&keypair, &path)
                    .wrap_err("Unable to serialize keypair to file")?;
                println!("Keypair saved at {}", path.display());
                println!("Safe CLI now has write access to the network");
            }
            Ok(())
        }
    }
}

pub fn print_new_key_output(output_fmt: OutputFmt, xorurl: SafeUrl, key_pair: Option<&Keypair>) {
    if OutputFmt::Pretty == output_fmt {
        println!("New SafeKey created: \"{}\"", xorurl);

        if let Some(pair) = &key_pair {
            match pair.to_hex() {
                Ok((pk_hex, sk_hex)) => {
                    println!("Public Key: {}", pk_hex);
                    println!("Secret Key: {}", sk_hex);
                }
                Err(err) => println!("{}", err),
            }
        }
    } else if let Some(pair) = &key_pair {
        match pair.to_hex() {
            Ok((pk_hex, sk_hex)) => println!(
                "{}",
                serialise_output(&(xorurl, (pk_hex, sk_hex)), output_fmt)
            ),
            Err(err) => println!("{}", err),
        }
    }
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
    async fn should_create_bls_keypair() -> Result<()> {
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
    async fn should_create_bls_keypair_saved_to_credentials_file() -> Result<()> {
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

        let (_, keypair) = read_credentials(&safe, &config)?;
        let keypair =
            keypair.ok_or_else(|| eyre!("The command should have generated a keypair"))?;
        match keypair {
            Keypair::Bls(_) => Ok(()),
            _ => Err(eyre!("The command should generate a BLS keypair")),
        }
    }
}
