// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{parse_coins_amount, sk_from_hex, xorname_from_pk, KeyPair};
use super::xorurl::{SafeContentType, SafeDataType};
use super::{Error, ResultReturn, Safe, SafeApp, XorUrl, XorUrlEncoder};
use log::debug;
use rand_core::RngCore;
use safe_nd::Coins;
use serde::{Deserialize, Serialize};

// Type tag used for the Wallet container
const WALLET_TYPE_TAG: u64 = 1_000;

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
            SafeDataType::SeqMutableData,
            SafeContentType::Wallet,
            None,
            None,
            None,
            &self.xorurl_base,
        )
    }

    // Add a Key to a Wallet to make it spendable, and returns the friendly name set for it
    pub fn wallet_insert(
        &mut self,
        url: &str,
        name: Option<String>,
        default: bool,
        sk: &str,
    ) -> ResultReturn<String> {
        let key_pair = KeyPair::from_hex_sk(sk)?;
        let xorname = xorname_from_pk(&key_pair.pk);
        let xorurl = XorUrlEncoder::encode(
            xorname,
            0,
            SafeDataType::CoinBalance,
            SafeContentType::Raw,
            None,
            None,
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

        let md_key = name.unwrap_or_else(|| xorurl);
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url)?;
        self.safe_app
            .seq_mutable_data_insert(
                xorurl_encoder.xorname(),
                WALLET_TYPE_TAG,
                &md_key.to_string().into_bytes(),
                &serialised_value.into_bytes(),
            )
            .map_err(|err| match err {
                Error::EntryExists(_) => Error::EntryExists(format!(
                    "An spendable balance already exists in the Wallet with the same name: '{}'",
                    md_key
                )),
                other => other,
            })?;

        debug!(
            "Wallet at {} had a spendable balance added with name: {}.",
            &url, md_key
        );

        if default {
            self.safe_app.seq_mutable_data_insert(
                xorurl_encoder.xorname(),
                WALLET_TYPE_TAG,
                WALLET_DEFAULT_BYTES,
                &md_key.to_string().into_bytes(),
            )?;

            debug!("Default wallet set.");
        }

        Ok(md_key.to_string())
    }

    // Check the total balance of a Wallet found at a given XOR-URL
    pub fn wallet_balance(&mut self, url: &str) -> ResultReturn<String> {
        debug!("Finding total wallet balance for: {:?}", url);
        let mut total_balance = Coins::from_nano(0).map_err(|err| {
            Error::Unexpected(format!(
                "Unexpected error when trying to instantiate a safe_nd::Coins object: {}",
                err
            ))
        })?;

        // Let's get the list of balances from the Wallet
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url)?;
        let spendable_balances = match self
            .safe_app
            .list_seq_mdata_entries(xorurl_encoder.xorname(), WALLET_TYPE_TAG)
        {
            Ok(entries) => entries,
            Err(Error::ContentNotFound(_)) => {
                return Err(Error::ContentNotFound(format!(
                    "No Wallet found at {}",
                    url
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
        for (name, balance) in spendable_balances.iter() {
            let thename = String::from_utf8_lossy(name).to_string();

            // Ignore the _default Wallet MD entry key
            if thename != WALLET_DEFAULT {
                debug!("Checking wallet of name: {:?}", thename);
                let the_balance = String::from_utf8_lossy(&balance.data).to_string();
                let spendable_balance: WalletSpendableBalance = serde_json::from_str(&the_balance)
                    .map_err(|_| {
                        Error::ContentError(
                            "Couldn't deserialise data stored in the Wallet".to_string(),
                        )
                    })?;

                let secret_key = sk_from_hex(&spendable_balance.sk)?;
                let current_balance =
                    self.safe_app.get_balance_from_sk(secret_key).map_err(|_| {
                        Error::ContentNotFound("One of the Key's was not found".to_string())
                    })?;

                debug!("{}: balance: {}", thename, current_balance);
                match total_balance.checked_add(current_balance) {
                    None => {
                        return Err(Error::Unexpected(
                            "Failed to calculate total balance due to overflow".to_string(),
                        ))
                    }
                    Some(new_balance_coins) => total_balance = new_balance_coins,
                };
            }
        }
        Ok(total_balance.to_string())
    }

    fn wallet_get_default_balance(&mut self, url: &str) -> ResultReturn<WalletSpendableBalance> {
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url)?;
        let default = self
            .safe_app
            .seq_mutable_data_get_value(
                xorurl_encoder.xorname(),
                WALLET_TYPE_TAG,
                WALLET_DEFAULT_BYTES,
            )
            .map_err(|_| {
                Error::ContentError(format!("No default balance found at Wallet \"{}\"", url))
            })?;

        let the_balance: WalletSpendableBalance = {
            let default_balance_vec = self
                .safe_app
                .seq_mutable_data_get_value(
                    xorurl_encoder.xorname(),
                    WALLET_TYPE_TAG,
                    &default.data,
                )
                .map_err(|_| {
                    Error::ContentError(format!(
                        "Default balance set but not found at Wallet \"{}\"",
                        url
                    ))
                })?;

            let default_balance = String::from_utf8_lossy(&default_balance_vec.data).to_string();
            let spendable_balance: WalletSpendableBalance = serde_json::from_str(&default_balance)
                .map_err(|_| {
                    Error::ContentError(
                        "Couldn't deserialise data stored in the Wallet".to_string(),
                    )
                })?;
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
    /// let mut safe = Safe::new("base32z");
    /// # unwrap!(safe.connect("", Some("fake-credentials")));
    /// let wallet_xorurl = unwrap!(safe.wallet_create());
    /// let wallet_xorurl2 = unwrap!(safe.wallet_create());
    /// let (key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("14", None));
    /// let (key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("1", None));
    /// unwrap!(safe.wallet_insert(
    ///     &wallet_xorurl,
    ///     Some("frombalance".to_string()),
    ///     true,
    ///     &key_pair1.clone().unwrap().sk,
    /// ));
    /// let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    /// assert_eq!("14.000000000", current_balance);
    ///
    /// unwrap!(safe.wallet_insert(
    ///     &wallet_xorurl2,
    ///     Some("tobalance".to_string()),
    ///     true,
    ///     &key_pair2.clone().unwrap().sk,
    /// ));
    ///
    ///
    /// unwrap!(safe.wallet_transfer( "10", Some(&wallet_xorurl), &wallet_xorurl2 ));
    /// let from_balance = unwrap!(safe.keys_balance_from_url( &key1_xorurl, &key_pair1.unwrap().sk ));
    /// assert_eq!("4.000000000", from_balance);
    /// let to_balance = unwrap!(safe.keys_balance_from_url( &key2_xorurl, &key_pair2.unwrap().sk ));
    /// assert_eq!("11.000000000", to_balance);
    /// ```
    pub fn wallet_transfer(
        &mut self,
        amount: &str,
        from_url: Option<&str>,
        to_url: &str,
    ) -> ResultReturn<u64> {
        // Parse and validate the amount is a valid
        let amount_coins = parse_coins_amount(amount)?;

        // 'from_url' is not optional until we know the account's default Wallet
        let from_wallet_url = match from_url {
            Some(url) => {
                // Check if 'from_url' is a valid Wallet URL
                let (xorurl_encoder, _) = self.parse_and_resolve_url(url).map_err(|_| {
                    Error::InvalidInput(format!("Failed to parse the 'from_url' URL: {}", url))
                })?;

                if xorurl_encoder.content_type() == SafeContentType::Wallet {
                    Ok(url)
                } else {
                    Err(Error::InvalidInput(format!(
                        "The 'from_url' URL doesn't target a Key or Wallet, it is: {:?} ({})",
                        xorurl_encoder.content_type(),
                        xorurl_encoder.data_type()
                    )))
                }
            }
            None => Err(Error::InvalidInput(
                "A 'from_url' Wallet is required until default wallets have been configured."
                    .to_string(),
            )),
        }?;

        // Now check if the 'to_url' is a valid Wallet or a Key URL
        let (to_xorurl_encoder, _) = self.parse_and_resolve_url(to_url)?;
        let to_xorname = if to_xorurl_encoder.content_type() == SafeContentType::Wallet {
            let to_balance = self.wallet_get_default_balance(&to_xorurl_encoder.to_string()?)?;
            XorUrlEncoder::from_url(&to_balance.xorurl)?.xorname()
        } else if to_xorurl_encoder.content_type() == SafeContentType::Raw
            && to_xorurl_encoder.data_type() == SafeDataType::CoinBalance
        {
            to_xorurl_encoder.xorname()
        } else {
            return Err(Error::InvalidInput(format!(
                "The destination URL doesn't target a Key or Wallet, target is: {:?} ({})",
                to_xorurl_encoder.content_type(),
                to_xorurl_encoder.data_type()
            )));
        };

        // Generate a random transfer TX ID
        let tx_id = rand::thread_rng().next_u64();

        // Figure out which is the default spendable balance we should use as the origin for the transfer
        let from_wallet_balance = self.wallet_get_default_balance(&from_wallet_url)?;
        let from_sk = sk_from_hex(&from_wallet_balance.sk)?;

        // Finally, let's make the transfer
        match self
            .safe_app
            .safecoin_transfer_to_xorname(from_sk, to_xorname, tx_id, amount_coins)
        {
            Err(Error::InvalidAmount(_)) => Err(Error::InvalidAmount(format!(
                "The amount '{}' specified for the transfer is invalid",
                amount
            ))),
            Err(Error::NotEnoughBalance(_)) => Err(Error::NotEnoughBalance(format!(
                "Not enough balance for the transfer at Wallet \"{}\"",
                from_wallet_url
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
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let xorurl = unwrap!(safe.wallet_create());
    assert!(xorurl.starts_with("safe://"));

    let current_balance = unwrap!(safe.wallet_balance(&xorurl));
    assert_eq!("0.000000000", current_balance);
}

#[test]
fn test_wallet_insert_and_balance() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("12.23", None));
    let (_key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("1.53", None));

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true,
        &unwrap!(key_pair1).sk,
    ));

    let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    assert_eq!("12.230000000", current_balance);

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("mysecondbalance".to_string()),
        false,
        &unwrap!(key_pair2).sk,
    ));

    let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    assert_eq!("13.760000000" /*== 12.23 + 1.53*/, current_balance);
}

#[test]
fn test_wallet_transfer_no_default() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create()); // this one won't have a default balance

    let to_wallet_xorurl = unwrap!(safe.wallet_create()); // we'll insert a default balance
    let (_key_xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("43523", None));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair).sk,
    ));

    // test no default balance at wallet in <from> argument
    match safe.wallet_transfer("10", Some(&from_wallet_xorurl), &to_wallet_xorurl) {
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
    match safe.wallet_transfer("10", Some(&to_wallet_xorurl), &from_wallet_xorurl) {
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
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl1, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("100.5", None));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl2, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("0.5", None));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("alsomyfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair2.clone()).sk,
    ));

    // test fail to transfer more than current balance at wallet in <from> argument
    match safe.wallet_transfer("100.6", Some(&from_wallet_xorurl), &to_wallet_xorurl) {
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
    match safe.wallet_transfer(".06", Some(&from_wallet_xorurl), &to_wallet_xorurl) {
        Err(Error::InvalidAmount(msg)) => assert_eq!(msg, "Invalid safecoins amount '.06'"),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test successful transfer
    match safe.wallet_transfer("100.4", Some(&from_wallet_xorurl), &to_wallet_xorurl) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance = unwrap!(safe.wallet_balance(&from_wallet_xorurl));
            assert_eq!("0.100000000", from_current_balance);
            let to_current_balance = unwrap!(safe.wallet_balance(&to_wallet_xorurl));
            assert_eq!("100.900000000", to_current_balance);
        }
    };
}

#[test]
fn test_wallet_transfer_to_key() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));

    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("4621.45", None));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let (key_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("10.0", None));

    // test successful transfer
    match safe.wallet_transfer("523.87", Some(&from_wallet_xorurl), &key_xorurl) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance = unwrap!(safe.wallet_balance(&from_wallet_xorurl));
            assert_eq!(
                "4097.580000000", /* 4621.45 - 523.87 */
                from_current_balance
            );
            let key_current_balance = unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair2).sk));
            assert_eq!("533.870000000", key_current_balance);
        }
    };
}

#[test]
fn test_wallet_transfer_with_nrs_urls() {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));

    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("0.2", None));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let from_nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    let _ = unwrap!(safe.nrs_map_container_create(
        &from_nrsurl,
        &from_wallet_xorurl,
        false,
        true,
        false
    ));

    let (key_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("0.1", None));
    let to_nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    let _ = unwrap!(safe.nrs_map_container_create(&to_nrsurl, &key_xorurl, false, true, false));

    // test successful transfer
    match safe.wallet_transfer("0.2", Some(&from_nrsurl), &to_nrsurl) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance = unwrap!(safe.wallet_balance(&from_nrsurl));
            assert_eq!(
                "0.000000000", /* 4621.45 - 523.87 */
                from_current_balance
            );
            let key_current_balance = unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair2).sk));
            assert_eq!("0.300000000", key_current_balance);
        }
    };
}
