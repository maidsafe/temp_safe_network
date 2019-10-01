// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{parse_coins_amount, sk_from_hex, xorname_from_pk, xorname_to_hex, KeyPair};
use super::xorurl::{SafeContentType, SafeDataType};
use super::{Error, ResultReturn, Safe, SafeApp, XorUrl, XorUrlEncoder};
use log::debug;
use rand_core::RngCore;
use safe_nd::{Coins, XorName};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Type tag used for the Wallet container
const WALLET_TYPE_TAG: u64 = 1_000;

const WALLET_DEFAULT_BYTES: &[u8] = b"_default";

// Struct which is serialised and stored in Wallet MD for linking to a spendable balance (Key)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WalletSpendableBalance {
    pub xorurl: XorUrl,
    pub sk: String,
}

pub type WalletSpendableBalances = BTreeMap<String, (bool, WalletSpendableBalance)>;

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

    // Add a SafeKey to a Wallet to make it spendable, and returns the friendly name set for it
    pub fn wallet_insert(
        &mut self,
        url: &str,
        name: Option<&str>,
        default: bool,
        sk: &str,
    ) -> ResultReturn<String> {
        let key_pair = KeyPair::from_hex_sk(sk)?;
        let xorname = xorname_from_pk(key_pair.pk);
        let xorurl = XorUrlEncoder::encode(
            xorname,
            0,
            SafeDataType::SafeKey,
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

        let md_key = name.unwrap_or_else(|| &xorurl);
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
                    "A spendable balance already exists in the Wallet with the same name: '{}'",
                    md_key
                )),
                other => other,
            })?;

        debug!(
            "Wallet at {} had a spendable balance added with name: {}.",
            &url, md_key
        );

        if default {
            match self.safe_app.seq_mutable_data_insert(
                xorurl_encoder.xorname(),
                WALLET_TYPE_TAG,
                WALLET_DEFAULT_BYTES,
                &md_key.to_string().into_bytes(),
            ) {
                Err(Error::EntryExists(_)) => {
                    let (_, version) = self.wallet_get_default_balance(url)?;
                    self.safe_app.seq_mutable_data_update(
                        xorurl_encoder.xorname(),
                        WALLET_TYPE_TAG,
                        WALLET_DEFAULT_BYTES,
                        &md_key.to_string().into_bytes(),
                        version + 1,
                    )
                }
                other => other,
            }?;

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
        let (xorurl_encoder, nrs_xorurl_encoder) = self.parse_and_resolve_url(url)?;
        debug!(
            "Wallet URL was parsed and resolved to: {:?}",
            xorurl_encoder
        );
        let url_path = if let Some(nrs_url) = nrs_xorurl_encoder {
            nrs_url.path().to_string()
        } else {
            xorurl_encoder.path().to_string()
        };

        let balances = if url_path.is_empty() {
            debug!("We'll check the total balance of the Wallet");
            gen_wallet_spendable_balances_list(&self, xorurl_encoder.xorname(), url)?
        } else {
            let balance_name = &url_path[1..]; // we get rid of starting '/'
            debug!(
                "We'll check only the balance for spendable balance named: '{}'",
                balance_name
            );
            let (spendable_balance, _) = wallet_get_spendable_balance(
                &self,
                xorurl_encoder.xorname(),
                balance_name.as_bytes(),
            )?;
            let mut balances = WalletSpendableBalances::default();
            balances.insert(balance_name.to_string(), (false, spendable_balance));
            balances
        };

        debug!("Spendable balances to check: {:?}", balances);
        // Iterate through the Keys and query the balance for each
        for (name, (_, balance)) in balances.iter() {
            // Ignore the _default Wallet MD entry key
            debug!("Checking wallet of name: {:?}", name);
            let secret_key = sk_from_hex(&balance.sk)?;
            let current_balance = self.safe_app.get_balance_from_sk(secret_key).map_err(|_| {
                Error::ContentNotFound("One of the SafeKey's was not found".to_string())
            })?;

            debug!("{}: balance is {}", name, current_balance);
            match total_balance.checked_add(current_balance) {
                None => {
                    return Err(Error::Unexpected(
                        "Failed to calculate total balance due to overflow".to_string(),
                    ))
                }
                Some(new_balance_coins) => total_balance = new_balance_coins,
            };
        }

        Ok(total_balance.to_string())
    }

    pub fn wallet_get_default_balance(
        &self,
        url: &str,
    ) -> ResultReturn<(WalletSpendableBalance, u64)> {
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url)?;
        let default = self
            .safe_app
            .seq_mutable_data_get_value(
                xorurl_encoder.xorname(),
                WALLET_TYPE_TAG,
                WALLET_DEFAULT_BYTES,
            )
            .map_err(|err| match err {
                Error::AccessDenied(_) => Error::AccessDenied(format!(
                    "Couldn't read source Wallet for the transfer at \"{}\"",
                    url
                )),
                Error::ContentNotFound(_) => {
                    Error::ContentError(format!("No Wallet found at Wallet \"{}\"", url))
                }
                _other => {
                    Error::ContentError(format!("No default balance found at Wallet \"{}\"", url))
                }
            })?;

        wallet_get_spendable_balance(&self, xorurl_encoder.xorname(), &default.data)
    }

    /// # Transfer safecoins from one Wallet to another
    ///
    /// Using established Wallet and SpendableBalances you can send safecoins between Wallets.
    ///
    /// ## Example
    /// ```
    /// # use safe_api::Safe;
    /// # use unwrap::unwrap;
    /// let mut safe = Safe::new("base32z");
    /// # unwrap!(safe.connect("", Some("fake-credentials")));
    /// let wallet_xorurl = unwrap!(safe.wallet_create());
    /// let wallet_xorurl2 = unwrap!(safe.wallet_create());
    /// let (key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("14"));
    /// let (key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("1"));
    /// unwrap!(safe.wallet_insert(
    ///     &wallet_xorurl,
    ///     Some("frombalance"),
    ///     true,
    ///     &key_pair1.clone().unwrap().sk,
    /// ));
    /// let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    /// assert_eq!("14.000000000", current_balance);
    ///
    /// unwrap!(safe.wallet_insert(
    ///     &wallet_xorurl2,
    ///     Some("tobalance"),
    ///     true,
    ///     &key_pair2.clone().unwrap().sk,
    /// ));
    ///
    ///
    /// unwrap!(safe.wallet_transfer( "10", Some(&wallet_xorurl), &wallet_xorurl2, None ));
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
        tx_id: Option<u64>,
    ) -> ResultReturn<u64> {
        // Parse and validate the amount is a valid
        let amount_coins = parse_coins_amount(amount)?;

        // 'from_url' is not optional until we know the account's default Wallet
        let (from_wallet_url, from_xorurl_encoder, from_nrs_xorurl_encoder) = match from_url {
            Some(url) => {
                // Check if 'from_url' is a valid Wallet URL
                let (xorurl_encoder, nrs_xorurl_encoder) = self.parse_and_resolve_url(&url).map_err(|_| {
                    Error::InvalidInput(format!("Failed to parse the 'from_url' URL: {}", url))
                })?;

                if xorurl_encoder.content_type() == SafeContentType::Wallet {
                    Ok((url, xorurl_encoder, nrs_xorurl_encoder))
                } else {
                    Err(Error::InvalidInput(format!(
                        "The 'from_url' URL doesn't target a Wallet, it is: {:?} ({})",
                        xorurl_encoder.content_type(),
                        xorurl_encoder.data_type()
                    )))
                }
            }
            None => Err(Error::InvalidInput(
                "A 'from_url' Wallet is required until a default Wallet has been configured in the Account, which is currently not supported/possible."
                    .to_string(),
            )),
        }?;

        // Now check if the 'to_url' is a valid Wallet or a SafeKey URL
        let (to_xorurl_encoder, to_nrs_xorurl_encoder) =
            self.parse_and_resolve_url(to_url).map_err(|_| {
                Error::InvalidInput(format!("Failed to parse the 'to_url' URL: {}", to_url))
            })?;

        let to_xorname = if to_xorurl_encoder.content_type() == SafeContentType::Wallet {
            let to_wallet_balance =
                resolve_wallet_url(&self, to_url, to_xorurl_encoder, to_nrs_xorurl_encoder)?;
            XorUrlEncoder::from_url(&to_wallet_balance.xorurl)?.xorname()
        } else if to_xorurl_encoder.content_type() == SafeContentType::Raw
            && to_xorurl_encoder.data_type() == SafeDataType::SafeKey
        {
            to_xorurl_encoder.xorname()
        } else {
            return Err(Error::InvalidInput(format!(
                "The destination URL doesn't target a SafeKey or Wallet, target is: {:?} ({})",
                to_xorurl_encoder.content_type(),
                to_xorurl_encoder.data_type()
            )));
        };

        // Generate a random transfer TX ID
        let tx_id = tx_id.unwrap_or_else(|| rand::thread_rng().next_u64());

        let from_wallet_balance = resolve_wallet_url(
            &self,
            from_wallet_url,
            from_xorurl_encoder,
            from_nrs_xorurl_encoder,
        )?;
        let from_sk = sk_from_hex(&from_wallet_balance.sk)?;

        // Finally, let's make the transfer
        match self.safe_app.safecoin_transfer_to_xorname(
            Some(from_sk),
            to_xorname,
            tx_id,
            amount_coins,
        ) {
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
            Ok(tx) => Ok(tx.id),
        }
    }

    pub fn wallet_get(&self, url: &str) -> ResultReturn<WalletSpendableBalances> {
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url)?;
        gen_wallet_spendable_balances_list(&self, xorurl_encoder.xorname(), url)
    }
}

// Private helper to generate the list of SpendableBalances which is used for different purposes
fn gen_wallet_spendable_balances_list(
    safe: &Safe,
    xorname: XorName,
    url: &str,
) -> ResultReturn<WalletSpendableBalances> {
    let entries = match safe
        .safe_app
        .list_seq_mdata_entries(xorname, WALLET_TYPE_TAG)
    {
        Ok(entries) => entries,
        Err(Error::AccessDenied(_)) => {
            return Err(Error::AccessDenied(format!(
                "Couldn't read Wallet at \"{}\"",
                url
            )))
        }
        Err(Error::ContentNotFound(_)) => {
            return Err(Error::ContentNotFound(format!(
                "No Wallet found at {}",
                url
            )))
        }
        Err(err) => {
            return Err(Error::ContentError(format!(
                "Failed to read balances from Wallet: {}",
                err
            )))
        }
    };

    let mut balances = WalletSpendableBalances::default();
    let mut default_balance = "".to_string();
    for (key, value) in entries.iter() {
        let value_str = String::from_utf8_lossy(&value.data).to_string();
        if key.as_slice() == WALLET_DEFAULT_BYTES {
            default_balance = value_str;
        } else {
            let spendable_balance: WalletSpendableBalance = serde_json::from_str(&value_str)
                .map_err(|_| {
                    Error::ContentError(
                        "Couldn't deserialise data stored in the Wallet".to_string(),
                    )
                })?;
            let thename = String::from_utf8_lossy(key).to_string();
            balances.insert(thename, (false, spendable_balance));
        }
    }

    if !default_balance.is_empty() {
        let mut default = balances.get_mut(&default_balance).ok_or_else(|| {
            Error::Unexpected(format!(
                "Failed to get default spendable balance from Wallet at \"{}\"",
                url
            ))
        })?;
        default.0 = true;
    }
    Ok(balances)
}

// Private helper to fetch a specific spendable balance from a Wallet usng its assigned frienly name
// TODO: move this out to a WalletRdf API
fn wallet_get_spendable_balance(
    safe: &Safe,
    xorname: XorName,
    balance_name: &[u8],
) -> ResultReturn<(WalletSpendableBalance, u64)> {
    let the_balance: (WalletSpendableBalance, u64) = {
        let default_balance_vec = safe
            .safe_app
            .seq_mutable_data_get_value(xorname, WALLET_TYPE_TAG, balance_name)
            .map_err(|_| {
                Error::ContentError(format!(
                    "Default balance set but not found at Wallet \"{}\"",
                    xorname_to_hex(&xorname)
                ))
            })?;

        let default_balance = String::from_utf8_lossy(&default_balance_vec.data).to_string();
        let spendable_balance: WalletSpendableBalance = serde_json::from_str(&default_balance)
            .map_err(|_| {
                Error::ContentError("Couldn't deserialise data stored in the Wallet".to_string())
            })?;
        (spendable_balance, default_balance_vec.version)
    };

    Ok(the_balance)
}

// Get a specific spendable balance if the URL has a path, or the default
fn resolve_wallet_url(
    safe: &Safe,
    wallet_url: &str,
    xorurl_encoder: XorUrlEncoder,
    nrs_xorurl_encoder: Option<XorUrlEncoder>,
) -> ResultReturn<WalletSpendableBalance> {
    let url_path = if let Some(nrs_url) = nrs_xorurl_encoder {
        nrs_url.path().to_string()
    } else {
        xorurl_encoder.path().to_string()
    };

    let (wallet_balance, _) = if url_path.is_empty() {
        // Figure out which is the default spendable balance
        safe.wallet_get_default_balance(&wallet_url)?
    } else {
        // Get the spendable balance which name matches the path we found in the URL
        wallet_get_spendable_balance(
            safe,
            xorurl_encoder.xorname(),
            url_path[1..].as_bytes(), // we get rid of starting '/' from the URL path
        )?
    };

    Ok(wallet_balance)
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
    let (_key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("12.23"));
    let (_key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("1.53"));

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("my-first-balance"),
        true,
        &unwrap!(key_pair1).sk,
    ));

    let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    assert_eq!("12.230000000", current_balance);

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("my-second-balance"),
        false,
        &unwrap!(key_pair2).sk,
    ));

    let current_balance = unwrap!(safe.wallet_balance(&wallet_xorurl));
    assert_eq!("13.760000000" /*== 12.23 + 1.53*/, current_balance);
}

#[test]
fn test_wallet_insert_and_get() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let wallet_xorurl = unwrap!(safe.wallet_create());
    let (key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("12.23"));
    let (key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("1.53"));

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("my-first-balance"),
        true,
        &unwrap!(key_pair1.clone()).sk,
    ));

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("my-second-balance"),
        false,
        &unwrap!(key_pair2.clone()).sk,
    ));

    let wallet_balances = unwrap!(safe.wallet_get(&wallet_xorurl));
    assert_eq!(wallet_balances["my-first-balance"].0, true);
    assert_eq!(wallet_balances["my-first-balance"].1.xorurl, key1_xorurl);
    assert_eq!(
        wallet_balances["my-first-balance"].1.sk,
        unwrap!(key_pair1).sk
    );

    assert_eq!(wallet_balances["my-second-balance"].0, false);
    assert_eq!(wallet_balances["my-second-balance"].1.xorurl, key2_xorurl);
    assert_eq!(
        wallet_balances["my-second-balance"].1.sk,
        unwrap!(key_pair2).sk
    );
}

#[test]
fn test_wallet_insert_and_set_default() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let wallet_xorurl = unwrap!(safe.wallet_create());
    let (key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("65.82"));
    let (key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("11.44"));

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("my-first-balance"),
        true,
        &unwrap!(key_pair1.clone()).sk,
    ));

    unwrap!(safe.wallet_insert(
        &wallet_xorurl,
        Some("my-second-balance"),
        true,
        &unwrap!(key_pair2.clone()).sk,
    ));

    let wallet_balances = unwrap!(safe.wallet_get(&wallet_xorurl));
    assert_eq!(wallet_balances["my-first-balance"].0, false);
    assert_eq!(wallet_balances["my-first-balance"].1.xorurl, key1_xorurl);
    assert_eq!(
        wallet_balances["my-first-balance"].1.sk,
        unwrap!(key_pair1).sk
    );

    assert_eq!(wallet_balances["my-second-balance"].0, true);
    assert_eq!(wallet_balances["my-second-balance"].1.xorurl, key2_xorurl);
    assert_eq!(
        wallet_balances["my-second-balance"].1.sk,
        unwrap!(key_pair2).sk
    );
}

#[test]
fn test_wallet_transfer_no_default() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create()); // this one won't have a default balance

    let to_wallet_xorurl = unwrap!(safe.wallet_create()); // we'll insert a default balance
    let (_key_xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("43523"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("my-first-balance"),
        true, // set --default
        &unwrap!(key_pair).sk,
    ));

    // test no default balance at wallet in <from> argument
    match safe.wallet_transfer("10", Some(&from_wallet_xorurl), &to_wallet_xorurl, None) {
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
    match safe.wallet_transfer("10", Some(&to_wallet_xorurl), &from_wallet_xorurl, None) {
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
fn test_wallet_transfer_from_zero_balance() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl1, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("0.0"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("my-first-balance"),
        true, // set --default
        &unwrap!(key_pair1).sk,
    ));

    let (to_key_xorurl, _key_pair2) = unwrap!(safe.keys_create_preload_test_coins("0.5"));

    // test fail to transfer with 0 balance at wallet in <from> argument
    match safe.wallet_transfer("0", Some(&from_wallet_xorurl), &to_key_xorurl, None) {
        Err(Error::InvalidAmount(msg)) => assert_eq!(
            msg,
            "The amount '0' specified for the transfer is invalid".to_string()
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl2, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("0.5"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("also-my-balance"),
        true, // set --default
        &unwrap!(key_pair2).sk,
    ));

    // test fail to transfer with 0 balance at wallet in <from> argument
    match safe.wallet_transfer("0", Some(&from_wallet_xorurl), &to_wallet_xorurl, None) {
        Err(Error::InvalidAmount(msg)) => assert_eq!(
            msg,
            "The amount '0' specified for the transfer is invalid".to_string()
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
    let (_key_xorurl1, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("100.5"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("my-first-balance"),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl2, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("0.5"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("also-my-balance"),
        true, // set --default
        &unwrap!(key_pair2.clone()).sk,
    ));

    // test fail to transfer more than current balance at wallet in <from> argument
    match safe.wallet_transfer("100.6", Some(&from_wallet_xorurl), &to_wallet_xorurl, None) {
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
    match safe.wallet_transfer(".06", Some(&from_wallet_xorurl), &to_wallet_xorurl, None) {
        Err(Error::InvalidAmount(msg)) => assert_eq!(
            msg,
            "Invalid safecoins amount '.06' (Can\'t parse coin units)"
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test successful transfer
    match safe.wallet_transfer("100.4", Some(&from_wallet_xorurl), &to_wallet_xorurl, None) {
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
fn test_wallet_transfer_to_safekey() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));

    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("4621.45"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("my-first-balance"),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let (key_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("10.0"));

    // test successful transfer
    match safe.wallet_transfer("523.87", Some(&from_wallet_xorurl), &key_xorurl, None) {
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
fn test_wallet_transfer_from_safekey() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));

    let (safekey_xorurl1, _) = unwrap!(safe.keys_create_preload_test_coins("7"));
    let (safekey_xorurl2, _) = unwrap!(safe.keys_create_preload_test_coins("0"));

    match safe.wallet_transfer("1", Some(&safekey_xorurl1), &safekey_xorurl2, None) {
        Ok(_) => panic!("Transfer from SafeKey was expected to fail".to_string()),
        Err(Error::InvalidInput(msg)) => {
            assert_eq!(
                msg,
                "The 'from_url' URL doesn't target a Wallet, it is: Raw (SafeKey)"
            );
        }
        Err(err) => panic!(format!("Error is not the expected one: {:?}", err)),
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
    let (_, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("0.2"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("my-first-balance"),
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

    let (key_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("0.1"));
    let to_nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    let _ = unwrap!(safe.nrs_map_container_create(&to_nrsurl, &key_xorurl, false, true, false));

    // test successful transfer
    match safe.wallet_transfer("0.2", Some(&from_nrsurl), &to_nrsurl, None) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance = unwrap!(safe.wallet_balance(&from_nrsurl));
            assert_eq!("0.000000000" /* 0.2 - 0.2 */, from_current_balance);
            let key_current_balance = unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair2).sk));
            assert_eq!("0.300000000" /* 0.1 + 0.2 */, key_current_balance);
        }
    };
}

#[test]
fn test_wallet_transfer_from_specific_balance() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl1, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("100.5"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("from-first-balance"),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let (_key_xorurl2, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("200.5"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("from-second-balance"),
        false,
        &unwrap!(key_pair2.clone()).sk,
    ));

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl3, key_pair3) = unwrap!(safe.keys_create_preload_test_coins("10.5"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("to-first-balance"),
        true, // set --default
        &unwrap!(key_pair3.clone()).sk,
    ));

    // test fail to transfer more than current balance at 'from-firstbaance'
    let mut from_wallet_spendable_balance = unwrap!(XorUrlEncoder::from_url(&from_wallet_xorurl));
    from_wallet_spendable_balance.set_path("from-second-balance");
    let from_spendable_balance = unwrap!(from_wallet_spendable_balance.to_string());
    match safe.wallet_transfer(
        "200.6",
        Some(&from_spendable_balance),
        &to_wallet_xorurl,
        None,
    ) {
        Err(Error::NotEnoughBalance(msg)) => assert_eq!(
            msg,
            format!(
                "Not enough balance for the transfer at Wallet \"{}\"",
                from_spendable_balance
            )
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test successful transfer
    match safe.wallet_transfer(
        "100.3",
        Some(&from_spendable_balance),
        &to_wallet_xorurl,
        None,
    ) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_first_current_balance =
                unwrap!(safe.wallet_balance(&format!("{}/from-first-balance", from_wallet_xorurl)));
            assert_eq!("100.500000000", from_first_current_balance);
            let from_second_current_balance = unwrap!(safe.wallet_balance(&from_spendable_balance));
            assert_eq!(
                "100.200000000", /* 200.5 - 100.3 */
                from_second_current_balance
            );
            let from_current_balance = unwrap!(safe.wallet_balance(&from_wallet_xorurl));
            assert_eq!("200.700000000" /* 301 - 100.3 */, from_current_balance);
            let to_current_balance = unwrap!(safe.wallet_balance(&to_wallet_xorurl));
            assert_eq!("110.800000000" /* 10.5 + 100.3 */, to_current_balance);
        }
    };
}

#[test]
fn test_wallet_transfer_to_specific_balance() {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl1, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("100.7"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("from-first-balance"),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl2, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("10.2"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("to-first-balance"),
        true, // set --default
        &unwrap!(key_pair2.clone()).sk,
    ));

    let (_key_xorurl3, key_pair3) = unwrap!(safe.keys_create_preload_test_coins("20.2"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("to-second-balance"),
        false,
        &unwrap!(key_pair3.clone()).sk,
    ));

    // test successful transfer to 'to-second-balance'
    let mut to_wallet_spendable_balance = unwrap!(XorUrlEncoder::from_url(&to_wallet_xorurl));
    to_wallet_spendable_balance.set_path("to-second-balance");
    let to_spendable_balance = unwrap!(to_wallet_spendable_balance.to_string());
    match safe.wallet_transfer(
        "100.5",
        Some(&from_wallet_xorurl),
        &to_spendable_balance,
        None,
    ) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance = unwrap!(safe.wallet_balance(&from_wallet_xorurl));
            assert_eq!("0.200000000" /* 100.7 - 100.5 */, from_current_balance);
            let to_first_current_balance =
                unwrap!(safe.wallet_balance(&format!("{}/to-first-balance", to_wallet_xorurl)));
            assert_eq!("10.200000000", to_first_current_balance);
            let to_second_current_balance = unwrap!(safe.wallet_balance(&to_spendable_balance));
            assert_eq!(
                "120.700000000", /* 20.2 + 100.5 */
                to_second_current_balance
            );
            let to_current_balance = unwrap!(safe.wallet_balance(&to_wallet_xorurl));
            assert_eq!("130.900000000", /* 30.4 + 100.5 */ to_current_balance);
        }
    };

    // let's also test checking the balance with NRS URL of the destination spendable balances
    let to_wallet_nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    let _ = unwrap!(safe.nrs_map_container_create(
        &to_wallet_nrsurl,
        &to_wallet_xorurl,
        false,
        true,
        false
    ));

    let to_first_current_balance =
        unwrap!(safe.wallet_balance(&format!("{}/to-first-balance", to_wallet_nrsurl)));
    assert_eq!("10.200000000", to_first_current_balance);
    let to_second_current_balance =
        unwrap!(safe.wallet_balance(&format!("{}/to-second-balance", to_wallet_nrsurl)));
    assert_eq!("120.700000000", to_second_current_balance);
}

#[test]
fn test_wallet_transfer_specific_balances_with_nrs_urls() {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let from_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl1, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("10.1"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("from-first-balance"),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let (_key_xorurl2, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("20.2"));
    unwrap!(safe.wallet_insert(
        &from_wallet_xorurl,
        Some("from-second-balance"),
        false,
        &unwrap!(key_pair2.clone()).sk,
    ));

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl3, key_pair3) = unwrap!(safe.keys_create_preload_test_coins("30.3"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("to-first-balance"),
        true, // set --default
        &unwrap!(key_pair3.clone()).sk,
    ));

    let (_key_xorurl4, key_pair4) = unwrap!(safe.keys_create_preload_test_coins("40.4"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("to-second-balance"),
        false,
        &unwrap!(key_pair4.clone()).sk,
    ));

    // create NRS URLs for both wallets
    let from_nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    let _ = unwrap!(safe.nrs_map_container_create(
        &from_nrsurl,
        &from_wallet_xorurl,
        false,
        true,
        false
    ));
    let to_nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    let _ =
        unwrap!(safe.nrs_map_container_create(&to_nrsurl, &to_wallet_xorurl, false, true, false));

    // test successful transfer from 'from-second-balance' to 'to-second-balance'
    let from_spendable_balance = format!("{}/from-second-balance", from_nrsurl);
    let to_spendable_balance = format!("{}/to-second-balance", to_nrsurl);
    match safe.wallet_transfer(
        "5.8",
        Some(&from_spendable_balance),
        &to_spendable_balance,
        None,
    ) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance = unwrap!(safe.wallet_balance(&from_wallet_xorurl));
            assert_eq!(
                "24.500000000", /* 10.1 + 20.2 - 5.8 */
                from_current_balance
            );
            let from_first_current_balance =
                unwrap!(safe.wallet_balance(&format!("{}/from-first-balance", from_wallet_xorurl)));
            assert_eq!("10.100000000", from_first_current_balance);
            let from_second_current_balance = unwrap!(safe.wallet_balance(&from_spendable_balance));
            assert_eq!(
                "14.400000000", /* 20.2 - 5.8 */
                from_second_current_balance
            );

            let to_current_balance = unwrap!(safe.wallet_balance(&to_wallet_xorurl));
            assert_eq!(
                "76.500000000",
                /* 30.3 + 40.4 + 5.8 */ to_current_balance
            );
            let to_first_current_balance =
                unwrap!(safe.wallet_balance(&format!("{}/to-first-balance", to_wallet_xorurl)));
            assert_eq!("30.300000000", to_first_current_balance);
            let to_second_current_balance = unwrap!(safe.wallet_balance(&to_spendable_balance));
            assert_eq!(
                "46.200000000", /* 40.4 + 5.8 */
                to_second_current_balance
            );
        }
    };
}

#[test]
#[cfg(not(feature = "scl-mock"))]
fn test_wallet_transfer_from_not_owned_wallet() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let account1_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_key_xorurl1, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("100.5"));
    unwrap!(safe.wallet_insert(
        &account1_wallet_xorurl,
        Some("my-first-balance"),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let mut another_safe = Safe::new("base32z");
    unwrap!(another_safe.connect("", Some("another-fake-credentials")));
    let (key_xorurl, _key_pair) = unwrap!(another_safe.keys_create_preload_test_coins("100.5"));

    // test fail to transfer from a not owned wallet in <from> argument
    match another_safe.wallet_transfer("0.2", Some(&account1_wallet_xorurl), &key_xorurl, None) {
        Err(Error::AccessDenied(msg)) => assert_eq!(
            msg,
            format!(
                "Couldn't read source Wallet for the transfer at \"{}\"",
                account1_wallet_xorurl
            )
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };
}
