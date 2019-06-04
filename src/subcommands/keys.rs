// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.
use log::warn;

use safe_cli::{BlsKeyPair, Safe};

// TODO: move these to helper file
use crate::cli::{get_target_location, prompt_user};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum KeysSubCommands {
    #[structopt(name = "create")]
    /// Create a new KeyPair
    Create {
        /// The source wallet for funds
        from: String,
        /// Create a Key and allocate test-coins onto it
        #[structopt(long = "test-coins")]
        preload_test_coins: bool,
        /// Preload the key with a coinbalance
        #[structopt(long = "preload")]
        preload: Option<String>,
        /// Don't generate a key pair and just use the provided public key
        #[structopt(long = "pk")]
        pk: Option<String>,
    },
    #[structopt(name = "balance")]
    /// Query a Key's current balance
    Balance {},
}

pub fn key_commander(
    cmd: Option<KeysSubCommands>,
    target: Option<String>,
    safe: &mut Safe,
) -> Result<(), String> {
    // Is it a create subcommand?
    match cmd {
        Some(KeysSubCommands::Create {
            preload,
            pk,
            from,
            preload_test_coins,
            ..
        }) => {
            create_new_key(safe, preload_test_coins, from, preload, pk);
            Ok(())
        }
        Some(KeysSubCommands::Balance {}) => {
            let sk =
                String::from("391987fd429b4718a59b165b5799eaae2e56c697eb94670de8886f8fb7387058"); // FIXME: get sk from args or from the account
            let target = get_target_location(target)?;
            let current_balance = safe.keys_balance_from_xorname(&target, &sk);
            println!("Key's current balance: {}", current_balance);
            Ok(())
        }
        None => return Err("Missing keys sub-command. Use --help for details.".to_string()),
    }
}

pub fn create_new_key(
    safe: &mut Safe,
    preload_test_coins: bool,
    from: String,
    preload: Option<String>,
    pk: Option<String>,
) -> (String, Option<BlsKeyPair>) {
    // '--from' is either a Wallet XOR-URL, a Key XOR-URL, or a pk
    // TODO: support Key XOR-URL and pk, we now support only Key XOR name
    // Prompt the user for the secret key since 'from' is a Key and not a Wallet
    let sk = prompt_user(
        &format!(
            "Enter secret key corresponding to public key at XOR name \"{}\": ",
            from
        ),
        "Invalid input",
    );

    let pk_from_xor = safe.keys_fetch_pk(&from, &sk);
    let from_key_pair = BlsKeyPair {
        pk: pk_from_xor,
        sk,
    };

    let (xorname, key_pair) = if preload_test_coins {
        /*if cfg!(not(feature = "mock-network")) {
            warn!("Ignoring \"--test-coins\" flag since it's only available for \"mock-network\" feature");
            println!("Ignoring \"--test-coins\" flag since it's only available for \"mock-network\" feature");
            safe.keys_create(from, preload, pk)
        } else {*/
        warn!("Note that the Key to be created will be preloaded with **test coins** rather than real coins");
        println!("Note that the Key to be created will be preloaded with **test coins** rather than real coins");
        let amount = preload.unwrap_or("1000.111".to_string());

        if amount == "1000.111" {
            eprintln!("You must pass a preload amount with test-coins, 1000.111 will be added");
        }

        safe.keys_create_preload_test_coins(amount, pk)
    // }
    } else {
        safe.keys_create(from_key_pair, preload, pk)
    };

    println!("New Key created at XOR name: \"{}\"", xorname);
    if let Some(pair) = &key_pair {
        println!("Key pair generated: pk=\"{}\", sk=\"{}\"", pair.pk, pair.sk);
    }

    (xorname, key_pair)
}
