// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod api;

// mod lib_helpers;
#[cfg(not(feature = "scl-mock"))]
use api::safe_client_libs;
//#[cfg(feature = "scl-mock")]
use api::scl_mock;

pub use crate::api::helpers::vec_to_hex;
use crate::api::helpers::{
    decode_ipc_msg, encode_ipc_msg, parse_coins_amount, pk_from_hex, pk_to_hex, sk_from_hex,
    xorname_to_xorurl, xorurl_to_xorname, KeyPair,
};

// pub use crate::api::safe::Safe;
pub use crate::api::safe::{Safe, BlsKeyPair, XorUrl};
use crate::api::safe::{WalletSpendableBalance};

use log::{debug, info};
use reqwest::get as httpget;
#[cfg(not(feature = "scl-mock"))]
use safe_client_libs::SafeApp;
use safe_core::ipc::{AppExchangeInfo, AuthReq, IpcReq};
use safe_nd::AppPermissions;
#[cfg(feature = "scl-mock")]
use scl_mock::SafeApp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use threshold_crypto::SecretKey;
use unwrap::unwrap;
use uuid::Uuid;

// Type tag used for the Wallet container
static WALLET_TYPE_TAG: u64 = 10_000;

static WALLET_DEFAULT: &str = "_default";
static WALLET_DEFAULT_BYTES: &[u8] = b"_default";



impl Safe {

    // Fetch Key's pk from the network from a given XOR-URL
    pub fn fetch_pk_from_xorname(&self, xorurl: &str) -> Result<String, String> {
        let xorname = xorurl_to_xorname(xorurl)?;
        let public_key = self
            .safe_app
            .fetch_pk_from_xorname(&xorname)
            .map_err(|_| "No Key found at specified location".to_string())?;
        Ok(pk_to_hex(&public_key))
    }

    // Create an empty Wallet and return its XOR-URL
    pub fn wallet_create(&mut self) -> Result<XorUrl, String> {
        let xorname = self
            .safe_app
            .put_seq_mutable_data(None, WALLET_TYPE_TAG, None);
        xorname_to_xorurl(&xorname.unwrap(), &self.xorurl_base)
    }

    // Add a Key to a Wallet to make it spendable
    pub fn wallet_insert(
        &mut self,
        wallet_xorurl: &str,
        name: &str,
        default: bool,
        key_pair: &BlsKeyPair,
        key_xorurl: &str,
    ) -> Result<(), String> {
        let value = WalletSpendableBalance {
            xorurl: key_xorurl.to_string(),
            sk: key_pair.sk.clone(),
        };
        let serialised_value = unwrap!(serde_json::to_string(&value));
        // FIXME: it should return error if the name already exists
        let md_key = name.to_string();

        // TODO, check if key already exists and throw errors or update
        self.safe_app.seq_mutable_data_insert(
            wallet_xorurl,
            WALLET_TYPE_TAG,
            md_key.clone().into_bytes().to_vec(),
            &serialised_value.into_bytes(),
        )?;

        if default {
            self.safe_app.seq_mutable_data_insert(
                wallet_xorurl,
                WALLET_TYPE_TAG,
                WALLET_DEFAULT_BYTES.to_vec(),
                &md_key.into_bytes(),
            )?;
        }

        Ok(())
    }

    // Check the total balance of a Wallet found at a given XOR-URL
    pub fn wallet_balance(&mut self, xorurl: &str, _sk: &str) -> Result<String, String> {
        let mut total_balance: f64 = 0.0;
        // Let's get the list of balances from the Wallet
        let spendable_balances = self
            .safe_app
            .list_seq_mdata_entries(xorurl, WALLET_TYPE_TAG)?;

        // Iterate through the Keys and query the balance for each
        spendable_balances.iter().for_each(|(name, balance)| {
            let thename = String::from_utf8_lossy(name).to_string();

            // Ignore the _default Wallet MD entry key
            if thename != WALLET_DEFAULT {
                let the_balance = String::from_utf8_lossy(&balance.data).to_string();
                let spendable_balance: WalletSpendableBalance =
                    unwrap!(serde_json::from_str(&the_balance));

                let current_balance = unwrap!(
                    self.keys_balance_from_xorurl(&spendable_balance.xorurl, &spendable_balance.sk)
                );

                total_balance += unwrap!(parse_coins_amount(&current_balance));
            }
        });
        Ok(total_balance.to_string())
    }

    fn wallet_get_default_balance(
        &mut self,
        wallet_xorurl: &str,
    ) -> Result<WalletSpendableBalance, String> {
        let default = self
            .safe_app
            .seq_mutable_data_get_value(
                wallet_xorurl,
                WALLET_TYPE_TAG,
                WALLET_DEFAULT_BYTES.to_vec(),
            )
            .map_err(|_| format!("No default balance found at Wallet \"{}\"", wallet_xorurl))?;

        let the_balance: WalletSpendableBalance = {
            let default_balance_vec = self
                .safe_app
                .seq_mutable_data_get_value(wallet_xorurl, WALLET_TYPE_TAG, default.data)
                .map_err(|_| {
                    format!(
                        "Default balance set but not found at Wallet \"{}\"",
                        wallet_xorurl
                    )
                })?;

            let default_balance = String::from_utf8_lossy(&default_balance_vec.data).to_string();
            let spendable_balance: WalletSpendableBalance =
                unwrap!(serde_json::from_str(&default_balance));
            spendable_balance
        };

        Ok(the_balance)
    }

    /// # Transfer safecoins from one Wallet to another
    ///
    /// Using established Wallet and SpendableBalances you can send safecoins between Wallets.
    ///
    /// ## Example
    /// ```
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// let mut safe = Safe::new("base32".to_string());
    /// let sk = String::from("391987fd429b4718a59b165b5799eaae2e56c697eb94670de8886f8fb7387058");
    /// let wallet_xorurl = unwrap!(safe.wallet_create());
    /// let wallet_xorurl2 = unwrap!(safe.wallet_create());
    /// let (key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("14".to_string(), None));
    /// let (key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("1".to_string(), None));
    /// unwrap!(safe.wallet_insert(
    ///     &wallet_xorurl,
    ///     "frombalance",
    ///     true,
    ///     &key_pair1.unwrap(),
    ///     &key1_xorurl,
    /// ));
    /// let current_balance = unwrap!(safe.wallet_balance( &wallet_xorurl, &sk ));
    /// assert_eq!("14", current_balance);
    ///
    /// unwrap!(safe.wallet_insert(
    ///     &wallet_xorurl2,
    ///     "tobalance",
    ///     true,
    ///     &key_pair2.unwrap(),
    ///     &key2_xorurl
    /// ));
    ///
    ///
    /// unwrap!(safe.wallet_transfer( "10", Some(wallet_xorurl), &wallet_xorurl2 ));
    /// let from_balance = unwrap!(safe.keys_balance_from_xorurl( &key1_xorurl, &sk ));
    /// assert_eq!("4.", from_balance);
    /// let to_balance = unwrap!(safe.keys_balance_from_xorurl( &key2_xorurl, &sk ));
    /// assert_eq!("11.", to_balance);
    /// ```
    pub fn wallet_transfer(
        &mut self,
        amount: &str,
        from: Option<XorUrl>,
        to: &str,
    ) -> Result<Uuid, String> {
        // from is not optional until we know default account container / Wallet location ("root")
        // if no FROM for now, ERR
        // FROM needs to be from default

        // TODO: Grab "from" stdin

        // TODO, check if to/from are Wallets or PKs (via safe:)
        let from_wallet_xorurl =
            match from {
                Some(wallet_xorurl) => wallet_xorurl,
                _ => return Err(
                    "A \"<from>\" Wallet is required until default wallets have been configured."
                        .to_string(),
                ),
            };

        let from_wallet_balance = self.wallet_get_default_balance(&from_wallet_xorurl)?;
        let to_wallet_balance = self.wallet_get_default_balance(&to)?;

        let from_pk = unwrap!(self
            .safe_app
            .fetch_pk_from_xorname(&xorurl_to_xorname(&from_wallet_balance.xorurl)?));

        let to_pk = unwrap!(self
            .safe_app
            .fetch_pk_from_xorname(&xorurl_to_xorname(&to_wallet_balance.xorurl)?));

        let from_sk = unwrap!(sk_from_hex(&from_wallet_balance.sk));
        let tx_id = Uuid::new_v4();

        match self
            .safe_app
            .safecoin_transfer(&from_pk, &from_sk, &to_pk, &tx_id, amount)
        {
            Err("InvalidAmount") => Err(format!(
                "The amount '{}' specified for the transfer is invalid",
                amount
            )),
            Err("NotEnoughBalance") => Err(format!(
                "Not enough balance for the transfer at Wallet \"{}\"",
                from_wallet_xorurl
            )),
            Err(other_error) => Err(format!(
                "Unexpected error when attempting to transfer: {}",
                other_error
            )),
            Ok(uuid) => Ok(uuid),
        }
    }
}

// Unit Tests

#[test]
fn test_keys_create_preload_test_coins() {
    let mut safe = Safe::new("base32".to_string());
    let (xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("12.23".to_string(), None));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_preload_test_coins_pk() {
    let mut safe = Safe::new("base32".to_string());
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("1.1".to_string(), Some(pk)));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}

#[test]
fn test_keys_create() {
    let mut safe = Safe::new("base32".to_string());
    let (_, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("23.23".to_string(), None));

    let (xorurl, key_pair) = unwrap!(safe.keys_create(unwrap!(from_key_pair), None, None));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_preload() {
    let mut safe = Safe::new("base32".to_string());
    let (_, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("543.2312".to_string(), None));

    let preload_amount = "1.8";
    let (xorurl, key_pair) = unwrap!(safe.keys_create(
        unwrap!(from_key_pair),
        Some(preload_amount.to_string()),
        None,
    ));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(kp) => {
            let balance = unwrap!(safe.keys_balance_from_pk(&BlsKeyPair {
                pk: kp.pk,
                sk: kp.sk,
            }));
            assert_eq!(balance, preload_amount);
        }
    };
}

#[test]
fn test_keys_create_preload_invalid_amounts() {
    let mut safe = Safe::new("base32".to_string());
    match safe.keys_create_preload_test_coins(".45".to_string(), None) {
        Err(msg) => assert_eq!(
            msg,
            "Invalid safecoins amount '.45', expected a numeric value"
        ),
        Ok(_) => panic!("Key with test-coins was created unexpectedly"),
    };

    let (_, key_pair) = unwrap!(safe.keys_create_preload_test_coins("12".to_string(), None));
    match safe.keys_create(unwrap!(key_pair.clone()), Some(".003".to_string()), None) {
        Err(msg) => assert_eq!(
            msg,
            "Invalid safecoins amount '.003', expected a numeric value"
        ),
        Ok(_) => panic!("Key was created unexpectedly"),
    };

    // test fail with corrupted secret key
    let mut unwrapped_key_pair = unwrap!(key_pair.clone());
    unwrapped_key_pair.sk.replace_range(..6, "ababab");
    match safe.keys_create(unwrapped_key_pair, Some(".003".to_string()), None) {
        Err(msg) => assert_eq!(
            msg,
            "Invalid safecoins amount '.003', expected a numeric value"
        ),
        Ok(_) => panic!("Key was created unexpectedly"),
    };

    // test fail to preload with more than available balance in source (which has only 12 coins)
    match safe.keys_create(unwrap!(key_pair), Some("12.00001".to_string()), None) {
        Err(msg) => assert_eq!(msg, "Not enough balance at 'source' for the operation"),
        Ok(_) => panic!("Key was created unexpectedly"),
    };
}

#[test]
fn test_keys_create_pk() {
    let mut safe = Safe::new("base32".to_string());
    let (_, from_key_pair) = unwrap!(safe.keys_create_preload_test_coins("1.1".to_string(), None));
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorurl, key_pair) = unwrap!(safe.keys_create(unwrap!(from_key_pair), None, Some(pk)));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}

#[test]
fn test_keys_test_coins_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let preload_amount = "1.1542";
    let (_, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));
    let current_balance = unwrap!(safe.keys_balance_from_pk(&unwrap!(key_pair)));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_xorurl() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let preload_amount = "0.243";
    let (xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));
    let current_balance = unwrap!(safe.keys_balance_from_xorurl(&xorurl, &unwrap!(key_pair).sk));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_wrong_xorurl() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let (_xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("0".to_string(), None));

    let invalid_xorurl = "safe://this-is-not-a-valid-xor-url";
    let current_balance = safe.keys_balance_from_xorurl(&invalid_xorurl, &unwrap!(key_pair).sk);
    match current_balance {
        Err(msg) => assert!(msg.contains("Failed to decode XOR-URL")),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_test_coins_balance_wrong_location() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let amount = "35312";
    let (mut xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(amount.to_string(), None));

    let current_balance =
        unwrap!(safe.keys_balance_from_xorurl(&xorurl, &unwrap!(key_pair.clone()).sk));
    assert_eq!(amount, current_balance);

    // let's corrupt the XOR-URL
    xorurl.replace_range(xorurl.len() - 5.., "ccccc");
    let current_balance = safe.keys_balance_from_xorurl(&xorurl, &unwrap!(key_pair).sk);
    match current_balance {
        Err(msg) => assert!(msg.contains("No Key found at specified location")),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_test_coins_balance_wrong_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let (_xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("0".to_string(), None));
    let mut unwrapped_key_pair = unwrap!(key_pair);
    unwrapped_key_pair.pk.replace_range(..6, "ababab");
    let current_balance = safe.keys_balance_from_pk(&unwrapped_key_pair);
    match current_balance {
        Err(msg) => assert!(msg.contains("Invalid public key string")),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let preload_amount = "1743.234";
    let (_, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "1740";
    let (_, to_key_pair) = unwrap!(safe.keys_create(
        from_key_pair_unwrapped.clone(),
        Some(amount.to_string()),
        None,
    ));

    let from_current_balance = unwrap!(safe.keys_balance_from_pk(&from_key_pair_unwrapped));
    assert_eq!("3.234" /*== 1743.234 - 1740*/, from_current_balance);

    let to_current_balance = unwrap!(safe.keys_balance_from_pk(&unwrap!(to_key_pair)));
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_keys_balance_xorname() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let preload_amount = "435.34";
    let (from_xorname, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "35.3";
    let (to_xorname, to_key_pair) = unwrap!(safe.keys_create(
        from_key_pair_unwrapped.clone(),
        Some(amount.to_string()),
        None,
    ));

    let from_current_balance =
        unwrap!(safe.keys_balance_from_xorurl(&from_xorname, &from_key_pair_unwrapped.sk));
    assert_eq!("400.04" /*== 435.34 - 35.3*/, from_current_balance);

    let to_current_balance =
        unwrap!(safe.keys_balance_from_xorurl(&to_xorname, &unwrap!(to_key_pair).sk));
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_fetch_pk_from_xorname_test_coins() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let (xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("23.22".to_string(), None));
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = unwrap!(safe.fetch_pk_from_xorname(&xorurl));
    assert_eq!(pk, key_pair_unwrapped.pk);
}

#[test]
fn test_fetch_pk_from_xorname() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let (_, from_key_pair) = unwrap!(safe.keys_create_preload_test_coins("0.56".to_string(), None));

    let (xorurl, key_pair) = unwrap!(safe.keys_create(unwrap!(from_key_pair), None, None));
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = unwrap!(safe.fetch_pk_from_xorname(&xorurl));
    assert_eq!(pk, key_pair_unwrapped.pk);
}

#[test]
fn test_wallet_create() {
    let mut safe = Safe::new("base32".to_string());
    let xorurl = unwrap!(safe.wallet_create());
    assert!(xorurl.starts_with("safe://"));

    let sk = String::from("391987fd429b4718a59b165b5799eaae2e56c697eb94670de8886f8fb7387058");
    let current_balance = unwrap!(safe.wallet_balance(&xorurl, &sk));
    assert_eq!("0", current_balance);
}

#[test]
fn test_wallet_insert_and_balance() {
    let mut safe = Safe::new("base32".to_string());
    let sk = String::from("391987fd429b4718a59b165b5799eaae2e56c697eb94670de8886f8fb7387058");
    let wallet_xorurl = unwrap!(safe.wallet_create());
    let (key1_xorurl, key_pair1) =
        unwrap!(safe.keys_create_preload_test_coins("12.23".to_string(), None));
    let (key2_xorurl, key_pair2) =
        unwrap!(safe.keys_create_preload_test_coins("1.53".to_string(), None));

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        "myfirstbalance",
        true,
        &unwrap!(key_pair1),
        &key1_xorurl,
    ));

    let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl, &sk));
    assert_eq!("12.23", current_balance);

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        "mysecondbalance",
        false,
        &unwrap!(key_pair2),
        &key2_xorurl,
    ));

    let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl, &sk));
    assert_eq!("13.76" /*== 12.23 + 1.53*/, current_balance);
}

#[test]
#[cfg(not(feature = "mock-network"))] // TODO: this is temporary until issue #82 is fixed
fn test_wallet_transfer_no_default() {
    let mut safe = Safe::new("base32".to_string());
    let from_wallet_xorurl = unwrap!(safe.wallet_create()); // this one won't have a default balance

    let to_wallet_xorurl = unwrap!(safe.wallet_create()); // we'll insert a default balance
    let (key_xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("43523".to_string(), None));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        "myfirstbalance",
        true, // set --default
        &unwrap!(key_pair),
        &key_xorurl,
    ));

    // test no default balance at wallet in <from> argument
    match safe.wallet_transfer("10", Some(from_wallet_xorurl.clone()), &to_wallet_xorurl) {
        Err(msg) => assert_eq!(
            msg,
            format!(
                "No default balance found at Wallet \"{}\"",
                from_wallet_xorurl
            )
        ),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // invert wallets and test no default balance at wallet in <to> argument
    match safe.wallet_transfer("10", Some(to_wallet_xorurl.clone()), &from_wallet_xorurl) {
        Err(msg) => assert_eq!(
            msg,
            format!(
                "No default balance found at Wallet \"{}\"",
                from_wallet_xorurl
            )
        ),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };
}

#[test]
fn test_wallet_transfer_diff_amounts() {
    let mut safe = Safe::new("base32".to_string());
    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (key_xorurl1, key_pair1) =
        unwrap!(safe.keys_create_preload_test_coins("100.5".to_string(), None));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        "myfirstbalance",
        true, // set --default
        &unwrap!(key_pair1.clone()),
        &key_xorurl1,
    ));

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (key_xorurl2, key_pair2) =
        unwrap!(safe.keys_create_preload_test_coins("0.5".to_string(), None));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        "alsomyfirstbalance",
        true, // set --default
        &unwrap!(key_pair2.clone()),
        &key_xorurl2,
    ));

    // test fail to transfer more than current balance at wallet in <from> argument
    match safe.wallet_transfer("100.6", Some(from_wallet_xorurl.clone()), &to_wallet_xorurl) {
        Err(msg) => assert_eq!(
            msg,
            format!(
                "Not enough balance for the transfer at Wallet \"{}\"",
                from_wallet_xorurl
            )
        ),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test fail to transfer as it's a invalid/non-numeric amount
    match safe.wallet_transfer(".06", Some(from_wallet_xorurl.clone()), &to_wallet_xorurl) {
        Err(msg) => assert_eq!(
            msg,
            "The amount '.06' specified for the transfer is invalid",
        ),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test successful transfer
    match safe.wallet_transfer("100.4", Some(from_wallet_xorurl.clone()), &to_wallet_xorurl) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance =
                unwrap!(safe.wallet_balance(&from_wallet_xorurl, &unwrap!(key_pair1).sk));
            assert_eq!("0.1", from_current_balance);
            let to_current_balance =
                unwrap!(safe.wallet_balance(&to_wallet_xorurl, &unwrap!(key_pair2).sk));
            assert_eq!("100.9", to_current_balance);
        }
    };
}
