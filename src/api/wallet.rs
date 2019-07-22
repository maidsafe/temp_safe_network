// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{parse_coins_amount, sk_from_hex, xorname_from_pk, KeyPair};
use super::xorurl::SafeContentType;
use super::{Error, ResultReturn};
use super::{Safe, XorUrl, XorUrlEncoder};
use log::debug;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use unwrap::unwrap; // TODO: remove all unwraps from this file

// Type tag used for the Wallet container
const WALLET_TYPE_TAG: u64 = 10_000;

const WALLET_DEFAULT: &str = "_default";
const WALLET_DEFAULT_BYTES: &[u8] = b"_default";

// Struct which is serialised and stored in Wallet MD for linking to a spendable balance (Key)
#[derive(Serialize, Deserialize, Debug)]
struct WalletSpendableBalance {
    pub xorurl: XorUrl,
    pub sk: String,
}

#[allow(dead_code)]
impl Safe {
    // Create an empty Wallet and return its XOR-URL
    pub fn wallet_create(&mut self) -> ResultReturn<XorUrl> {
        let xorname = self
            .safe_app
            .put_seq_mutable_data(None, WALLET_TYPE_TAG, None)?;

        XorUrlEncoder::encode(
            xorname,
            WALLET_TYPE_TAG,
            SafeContentType::Wallet,
            None,
            &self.xorurl_base,
        )
    }

    // Add a Key to a Wallet to make it spendable, and returns the friendly name set for it
    pub fn wallet_insert(
        &mut self,
        wallet_xorurl: &str,
        name: Option<String>,
        default: bool,
        sk: &str,
    ) -> ResultReturn<String> {
        let key_pair = KeyPair::from_hex_sk(sk)?;
        let xorname = xorname_from_pk(&key_pair.pk);
        let xorurl = XorUrlEncoder::encode(
            xorname,
            0,
            SafeContentType::CoinBalance,
            None,
            &self.xorurl_base,
        )?;
        let value = WalletSpendableBalance {
            xorurl: xorurl.clone(),
            sk: sk.to_string(),
        };

        let serialised_value = serde_json::to_string(&value).map_err(|err| {
            Error::Unexpected(format!(
                "Failed to serialise data to insert in Wallet container: {:?}",
                err
            ))
        })?;

        // TODO, check if name/key already exists and throw proper error, or simply update
        let md_key = name.unwrap_or_else(|| xorurl);
        self.safe_app.seq_mutable_data_insert(
            wallet_xorurl,
            WALLET_TYPE_TAG,
            md_key.to_string().into_bytes().to_vec(),
            &serialised_value.into_bytes(),
        )?;

        debug!(
            "Wallet at {} had a spendable balance added with name: {}.",
            &wallet_xorurl, md_key
        );

        if default {
            self.safe_app.seq_mutable_data_insert(
                wallet_xorurl,
                WALLET_TYPE_TAG,
                WALLET_DEFAULT_BYTES.to_vec(),
                &md_key.to_string().into_bytes(),
            )?;

            debug!("Default wallet set.");
        }

        Ok(md_key.to_string())
    }

    // Check the total balance of a Wallet found at a given XOR-URL
    pub fn wallet_balance(&mut self, xorurl: &str) -> ResultReturn<String> {
        debug!("Finding total wallet balance for: {:?}", xorurl);
        let mut total_balance: f64 = 0.0;

        // Let's get the list of balances from the Wallet
        let spendable_balances = match self
            .safe_app
            .list_seq_mdata_entries(xorurl, WALLET_TYPE_TAG)
        {
            Ok(entries) => entries,
            Err(Error::ContentNotFound(_)) => {
                return Err(Error::ContentNotFound(format!(
                    "No Wallet found at {}",
                    xorurl
                )))
            }
            Err(Error::InvalidXorUrl(_)) => {
                return Err(Error::InvalidXorUrl(
                    "The XOR-URL provided is invalid and cannot be decoded".to_string(),
                ))
            }
            Err(err) => {
                return Err(Error::ContentError(format!(
                    "Failed to read balances from Wallet: {}",
                    err
                )))
            }
        };

        debug!("Spendable balances: {:?}", spendable_balances);
        // Iterate through the Keys and query the balance for each
        spendable_balances.iter().for_each(|(name, balance)| {
            let thename = String::from_utf8_lossy(name).to_string();

            // Ignore the _default Wallet MD entry key
            if thename != WALLET_DEFAULT {
                debug!("Checking wallet of name: {:?}", thename);
                let the_balance = String::from_utf8_lossy(&balance.data).to_string();
                let spendable_balance: WalletSpendableBalance =
                    unwrap!(serde_json::from_str(&the_balance));

                let current_balance = unwrap!(
                    self.keys_balance_from_xorurl(&spendable_balance.xorurl, &spendable_balance.sk)
                );
                debug!("{:?}: balance: {:?}", thename, current_balance);

                total_balance += unwrap!(parse_coins_amount(&current_balance));
            }
        });
        Ok(total_balance.to_string())
    }

    fn wallet_get_default_balance(
        &mut self,
        wallet_xorurl: &str,
    ) -> ResultReturn<WalletSpendableBalance> {
        let default = self
            .safe_app
            .seq_mutable_data_get_value(
                wallet_xorurl,
                WALLET_TYPE_TAG,
                WALLET_DEFAULT_BYTES.to_vec(),
            )
            .map_err(|_| {
                Error::ContentError(format!(
                    "No default balance found at Wallet \"{}\"",
                    wallet_xorurl
                ))
            })?;

        let the_balance: WalletSpendableBalance = {
            let default_balance_vec = self
                .safe_app
                .seq_mutable_data_get_value(wallet_xorurl, WALLET_TYPE_TAG, default.data)
                .map_err(|_| {
                    Error::ContentError(format!(
                        "Default balance set but not found at Wallet \"{}\"",
                        wallet_xorurl
                    ))
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
    /// let mut safe = Safe::new("base32z".to_string());
    /// # unwrap!(safe.connect("", Some("fake-credentials")));
    /// let wallet_xorurl = unwrap!(safe.wallet_create());
    /// let wallet_xorurl2 = unwrap!(safe.wallet_create());
    /// let (key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("14".to_string(), None));
    /// let (key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("1".to_string(), None));
    /// unwrap!(safe.wallet_insert(
    ///     &wallet_xorurl,
    ///     Some("frombalance".to_string()),
    ///     true,
    ///     &key_pair1.clone().unwrap().sk,
    /// ));
    /// let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    /// assert_eq!("14", current_balance);
    ///
    /// unwrap!(safe.wallet_insert(
    ///     &wallet_xorurl2,
    ///     Some("tobalance".to_string()),
    ///     true,
    ///     &key_pair2.clone().unwrap().sk,
    /// ));
    ///
    ///
    /// unwrap!(safe.wallet_transfer( "10", Some(wallet_xorurl), &wallet_xorurl2 ));
    /// let from_balance = unwrap!(safe.keys_balance_from_xorurl( &key1_xorurl, &key_pair1.unwrap().sk ));
    /// assert_eq!("4.000000000", from_balance);
    /// let to_balance = unwrap!(safe.keys_balance_from_xorurl( &key2_xorurl, &key_pair2.unwrap().sk ));
    /// assert_eq!("11.000000000", to_balance);
    /// ```
    pub fn wallet_transfer(
        &mut self,
        amount: &str,
        from: Option<XorUrl>,
        to: &str,
    ) -> ResultReturn<u64> {
        // from is not optional until we know default account container / Wallet location ("root")
        // if no FROM for now, ERR
        // FROM needs to be from default

        // TODO: Grab "from" stdin

        // TODO, check if to/from are Wallets or PKs (via safe:)
        let from_wallet_xorurl =
            match from {
                Some(wallet_xorurl) => wallet_xorurl,
                _ => return Err(Error::InvalidInput(
                    "A \"<from>\" Wallet is required until default wallets have been configured."
                        .to_string(),
                )),
            };

        let from_wallet_balance = self.wallet_get_default_balance(&from_wallet_xorurl)?;
        let to_wallet_balance = self.wallet_get_default_balance(&to)?;
        let to_xorname = XorUrlEncoder::from_url(&to_wallet_balance.xorurl)?.xorname();

        let from_sk = unwrap!(sk_from_hex(&from_wallet_balance.sk));
        let mut rng = rand::thread_rng();
        let tx_id = rng.next_u64();

        match self
            .safe_app
            .safecoin_transfer_to_xorname(from_sk, to_xorname, tx_id, amount)
        {
            Err(Error::InvalidAmount(_)) => Err(Error::InvalidAmount(format!(
                "The amount '{}' specified for the transfer is invalid",
                amount
            ))),
            Err(Error::NotEnoughBalance(_)) => Err(Error::NotEnoughBalance(format!(
                "Not enough balance for the transfer at Wallet \"{}\"",
                from_wallet_xorurl
            ))),
            Err(other_error) => Err(Error::Unexpected(format!(
                "Unexpected error when attempting to transfer: {}",
                other_error
            ))),
            Ok(tx_id) => Ok(tx_id),
        }
    }
}

// Unit Tests

#[test]
fn test_wallet_create() {
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let xorurl = unwrap!(safe.wallet_create());
    assert!(xorurl.starts_with("safe://"));

    let current_balance = unwrap!(safe.wallet_balance(&xorurl));
    assert_eq!("0", current_balance);
}

#[test]
fn test_wallet_insert_and_balance() {
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key1_xorurl, key_pair1) =
        unwrap!(safe.keys_create_preload_test_coins("12.23".to_string(), None));
    let (_key2_xorurl, key_pair2) =
        unwrap!(safe.keys_create_preload_test_coins("1.53".to_string(), None));

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true,
        &unwrap!(key_pair1).sk,
    ));

    let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    assert_eq!("12.23", current_balance);

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("mysecondbalance".to_string()),
        false,
        &unwrap!(key_pair2).sk,
    ));

    let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    assert_eq!("13.76" /*== 12.23 + 1.53*/, current_balance);
}

#[test]
fn test_wallet_transfer_no_default() {
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create()); // this one won't have a default balance

    let to_wallet_xorurl = unwrap!(safe.wallet_create()); // we'll insert a default balance
    let (_key_xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("43523".to_string(), None));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair).sk,
    ));

    // test no default balance at wallet in <from> argument
    match safe.wallet_transfer("10", Some(from_wallet_xorurl.clone()), &to_wallet_xorurl) {
        Err(Error::ContentError(msg)) => assert_eq!(
            msg,
            format!(
                "No default balance found at Wallet \"{}\"",
                from_wallet_xorurl
            )
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // invert wallets and test no default balance at wallet in <to> argument
    match safe.wallet_transfer("10", Some(to_wallet_xorurl.clone()), &from_wallet_xorurl) {
        Err(Error::ContentError(msg)) => assert_eq!(
            msg,
            format!(
                "No default balance found at Wallet \"{}\"",
                from_wallet_xorurl
            )
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };
}

#[test]
fn test_wallet_transfer_diff_amounts() {
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl1, key_pair1) =
        unwrap!(safe.keys_create_preload_test_coins("100.5".to_string(), None));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl2, key_pair2) =
        unwrap!(safe.keys_create_preload_test_coins("0.5".to_string(), None));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("alsomyfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair2.clone()).sk,
    ));

    // test fail to transfer more than current balance at wallet in <from> argument
    match safe.wallet_transfer("100.6", Some(from_wallet_xorurl.clone()), &to_wallet_xorurl) {
        Err(Error::NotEnoughBalance(msg)) => assert_eq!(
            msg,
            format!(
                "Not enough balance for the transfer at Wallet \"{}\"",
                from_wallet_xorurl
            )
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test fail to transfer as it's a invalid/non-numeric amount
    match safe.wallet_transfer(".06", Some(from_wallet_xorurl.clone()), &to_wallet_xorurl) {
        Err(Error::InvalidAmount(msg)) => assert_eq!(
            msg,
            "The amount '.06' specified for the transfer is invalid",
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test successful transfer
    match safe.wallet_transfer("100.4", Some(from_wallet_xorurl.clone()), &to_wallet_xorurl) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance = unwrap!(safe.wallet_balance(&from_wallet_xorurl));
            assert_eq!("0.1", from_current_balance);
            let to_current_balance = unwrap!(safe.wallet_balance(&to_wallet_xorurl));
            assert_eq!("100.9", to_current_balance);
        }
    };
}
