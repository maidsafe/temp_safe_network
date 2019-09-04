// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{
    parse_coins_amount, pk_from_hex, pk_to_hex, sk_from_hex, xorname_from_pk, KeyPair,
};
use super::xorurl::{SafeContentType, SafeDataType};
use super::{Error, ResultReturn, Safe, SafeApp, XorUrl, XorUrlEncoder};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use threshold_crypto::SecretKey;

// A trait that the Validate derive will impl
use validator::{Validate, ValidationErrors};

// We expose a BLS key pair as two hex encoded strings
// TODO: consider supporting other encodings like base32 or just expose Vec<u8>
#[derive(Clone, Validate, Serialize, Deserialize)]
pub struct BlsKeyPair {
    #[validate(length(equal = "96"))]
    pub pk: String,
    #[validate(length(equal = "64"))]
    pub sk: String,
}

#[allow(dead_code)]
pub fn validate_key_pair(key_pair: &BlsKeyPair) -> ResultReturn<()> {
    let validation = key_pair.validate();

    if ValidationErrors::has_error(&validation, "sk") {
        return Err(Error::InvalidInput(
            "The secret key must be 64 characters long".to_string(),
        ));
    }
    if ValidationErrors::has_error(&validation, "pk") {
        return Err(Error::InvalidInput(
            "The secret key must be 96 characters long".to_string(),
        ));
    }

    let secret_key = sk_from_hex(&key_pair.sk)?;

    let real_pk = secret_key.public_key();

    let real_pk_hex = pk_to_hex(&real_pk);

    if real_pk_hex != key_pair.pk {
        return Err(Error::InvalidInput(
            "The secret key provided does not match the public key.".to_string(),
        ));
    }

    Ok(())
}

#[allow(dead_code)]
impl Safe {
    // Generate a key pair without creating and/or storing a SafeKey on the network
    pub fn keypair(&self) -> ResultReturn<BlsKeyPair> {
        let key_pair = KeyPair::random();
        let (pk, sk) = key_pair.to_hex_key_pair()?;
        Ok(BlsKeyPair { pk, sk })
    }

    // Create a SafeKey on the network and return its XOR-URL.
    pub fn keys_create(
        &mut self,
        from: Option<String>,
        preload_amount: Option<String>,
        pk: Option<String>,
    ) -> ResultReturn<(XorUrl, Option<BlsKeyPair>)> {
        let from_sk = match from {
            Some(sk) => match sk_from_hex(&sk) {
                Ok(sk) => Some(sk),
                Err(_) => return Err(Error::InvalidInput("The source of funds needs to be a secret key. The secret key provided is invalid".to_string())),
            },
            None => None,
        };

        let create_coin_balance =
            |to_pk, amount| match self.safe_app.create_balance(from_sk, to_pk, amount) {
                Err(Error::InvalidAmount(_)) => Err(Error::InvalidAmount(format!(
                    "The amount '{}' specified for the transfer is invalid",
                    amount
                ))),
                Err(Error::NotEnoughBalance(_)) => Err(Error::NotEnoughBalance(
                    "Not enough balance at 'source' for the operation".to_string(),
                )),
                Err(other_error) => Err(Error::Unexpected(format!(
                    "Unexpected error when attempting to create Key: {}",
                    other_error
                ))),
                Ok(xorname) => Ok(xorname),
            };

        let amount = parse_coins_amount(&preload_amount.unwrap_or_else(|| "0.0".to_string()))?;

        let (xorname, key_pair) = match pk {
            Some(pk_str) => {
                let pk = pk_from_hex(&pk_str)?;
                let xorname = create_coin_balance(pk, amount)?;
                (xorname, None)
            }
            None => {
                let key_pair = KeyPair::random();
                let (pk, sk) = key_pair.to_hex_key_pair()?;
                let xorname = create_coin_balance(key_pair.pk, amount)?;
                (xorname, Some(BlsKeyPair { pk, sk }))
            }
        };

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
        Ok((xorurl, key_pair))
    }

    // Create a SafeKey on the network, allocates testcoins onto it, and return the SafeKey's XOR-URL
    pub fn keys_create_preload_test_coins(
        &mut self,
        preload_amount: &str,
    ) -> ResultReturn<(XorUrl, Option<BlsKeyPair>)> {
        let amount = parse_coins_amount(preload_amount)?;
        let key_pair = KeyPair::random();
        let xorname = self
            .safe_app
            .allocate_test_coins(key_pair.sk.clone(), amount)?;
        let (pk, sk) = key_pair.to_hex_key_pair()?;
        let key_pair = Some(BlsKeyPair { pk, sk });

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
        Ok((xorurl, key_pair))
    }

    // Check SafeKey's balance from the network from a given SecretKey string
    pub fn keys_balance_from_sk(&self, sk: &str) -> ResultReturn<String> {
        let secret_key = sk_from_hex(sk)?;
        let coins = self.safe_app.get_balance_from_sk(secret_key).map_err(|_| {
            Error::ContentNotFound("No SafeKey found at specified location".to_string())
        })?;
        Ok(coins.to_string())
    }

    // Check SafeKey's balance from the network from a given XOR/NRS-URL and secret key string.
    // The difference between this and 'keys_balance_from_sk' function is that this will additionally
    // check that the XOR/NRS-URL corresponds to the public key derived from the provided secret key
    pub fn keys_balance_from_url(&self, url: &str, sk: &str) -> ResultReturn<String> {
        self.validate_sk_for_url(sk, url)?;
        self.keys_balance_from_sk(sk)
    }

    // Check that the XOR/NRS-URL corresponds to the public key derived from the provided secret key
    pub fn validate_sk_for_url(&self, sk: &str, url: &str) -> ResultReturn<String> {
        let secret_key: SecretKey = sk_from_hex(sk)
            .map_err(|_| Error::InvalidInput("Invalid secret key provided".to_string()))?;
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url)?;
        let public_key = secret_key.public_key();
        let derived_xorname = xorname_from_pk(public_key);
        if xorurl_encoder.xorname() != derived_xorname {
            Err(Error::InvalidInput(
                "The URL doesn't correspond to the public key derived from the provided secret key"
                    .to_string(),
            ))
        } else {
            Ok(pk_to_hex(&public_key))
        }
    }

    /// # Transfer safecoins from one SafeKey to another, or to a Wallet
    ///
    /// Using a secret key you can send safecoins to a Wallet or to a SafeKey.
    ///
    /// ## Example
    /// ```
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// let mut safe = Safe::new("base32z");
    /// # unwrap!(safe.connect("", Some("fake-credentials")));
    /// let (key1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("14"));
    /// let (key2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("1"));
    /// let current_balance = unwrap!(safe.keys_balance_from_sk(&key_pair1.clone().unwrap().sk));
    /// assert_eq!("14.000000000", current_balance);
    ///
    /// unwrap!(safe.keys_transfer( "10", Some(key_pair1.clone().unwrap().sk), &key2_xorurl, None ));
    /// let from_balance = unwrap!(safe.keys_balance_from_url( &key1_xorurl, &key_pair1.unwrap().sk ));
    /// assert_eq!("4.000000000", from_balance);
    /// let to_balance = unwrap!(safe.keys_balance_from_url( &key2_xorurl, &key_pair2.unwrap().sk ));
    /// assert_eq!("11.000000000", to_balance);
    /// ```
    pub fn keys_transfer(
        &mut self,
        amount: &str,
        from_sk: Option<String>,
        to_url: &str,
        tx_id: Option<u64>,
    ) -> ResultReturn<u64> {
        // Parse and validate the amount is a valid
        let amount_coins = parse_coins_amount(amount)?;

        // Let's check if the 'to_url' is a valid Wallet or a SafeKey URL
        let (to_xorurl_encoder, _) = self.parse_and_resolve_url(to_url)?;
        let to_xorname = if to_xorurl_encoder.content_type() == SafeContentType::Wallet {
            let (to_balance, _) =
                self.wallet_get_default_balance(&to_xorurl_encoder.to_string()?)?;
            XorUrlEncoder::from_url(&to_balance.xorurl)?.xorname()
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

        let from = match &from_sk {
            Some(sk) => Some(sk_from_hex(sk)?),
            None => None,
        };

        // Finally, let's make the transfer
        match self
            .safe_app
            .safecoin_transfer_to_xorname(from, to_xorname, tx_id, amount_coins)
        {
            Err(Error::InvalidAmount(_)) => Err(Error::InvalidAmount(format!(
                "The amount '{}' specified for the transfer is invalid",
                amount
            ))),
            Err(Error::NotEnoughBalance(_)) => {
                let msg = if from_sk.is_some() {
                    "Not enough balance for the transfer at provided source SafeKey".to_string()
                } else {
                    "Not enough balance for the transfer at Account's default SafeKey".to_string()
                };

                Err(Error::NotEnoughBalance(msg))
            }
            Err(other_error) => Err(Error::Unexpected(format!(
                "Unexpected error when attempting to transfer: {}",
                other_error
            ))),
            Ok(tx) => Ok(tx.id),
        }
    }
}

// Unit Tests

#[test]
fn test_keys_create_preload_test_coins() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("12.23"));
    assert!(xorurl.starts_with("safe://"));
    assert!(key_pair.is_some());
}

#[test]
fn test_keys_create() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (_, from_key_pair) = unwrap!(safe.keys_create_preload_test_coins("23.23"));

    let (xorurl, key_pair) = unwrap!(safe.keys_create(Some(unwrap!(from_key_pair).sk), None, None));
    assert!(xorurl.starts_with("safe://"));
    assert!(key_pair.is_some());
}

#[test]
fn test_keys_create_preload() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (_, from_key_pair) = unwrap!(safe.keys_create_preload_test_coins("543.2312"));

    let preload_amount = "1.800000000";
    let (xorurl, key_pair) = unwrap!(safe.keys_create(
        Some(unwrap!(from_key_pair).sk),
        Some(preload_amount.to_string()),
        None,
    ));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(kp) => {
            let balance = unwrap!(safe.keys_balance_from_sk(&kp.sk));
            assert_eq!(balance, preload_amount);
        }
    };
}

#[test]
fn test_keys_create_preload_invalid_amounts() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    match safe.keys_create_preload_test_coins(".45") {
        Err(err) => assert_eq!(
            err,
            Error::InvalidAmount("Invalid safecoins amount '.45'".to_string())
        ),
        Ok(_) => panic!("Key with test-coins was created unexpectedly"),
    };

    let (_, key_pair) = unwrap!(safe.keys_create_preload_test_coins("12"));
    match safe.keys_create(
        Some(unwrap!(key_pair.clone()).sk),
        Some(".003".to_string()),
        None,
    ) {
        Err(err) => assert_eq!(
            err,
            Error::InvalidAmount("Invalid safecoins amount '.003'".to_string())
        ),
        Ok(_) => panic!("Key was created unexpectedly"),
    };

    // test fail with corrupted secret key
    let mut unwrapped_key_pair = unwrap!(key_pair.clone());
    unwrapped_key_pair.sk.replace_range(..6, "ababab");
    match safe.keys_create(Some(unwrapped_key_pair.sk), Some(".003".to_string()), None) {
        Err(err) => assert_eq!(
            err,
            Error::InvalidAmount("Invalid safecoins amount '.003'".to_string())
        ),
        Ok(_) => panic!("Key was created unexpectedly"),
    };

    // test fail to preload with more than available balance in source (which has only 12 coins)
    match safe.keys_create(
        Some(unwrap!(key_pair).sk),
        Some("12.000000001".to_string()),
        None,
    ) {
        Err(err) => assert_eq!(
            err,
            Error::NotEnoughBalance("Not enough balance at 'source' for the operation".to_string())
        ),
        Ok(_) => panic!("Key was created unexpectedly"),
    };
}

#[test]
fn test_keys_create_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (_, from_key_pair) = unwrap!(safe.keys_create_preload_test_coins("1.1"));
    let pk = pk_to_hex(&SecretKey::random().public_key());
    let (xorurl, key_pair) =
        unwrap!(safe.keys_create(Some(unwrap!(from_key_pair).sk), None, Some(pk)));
    assert!(xorurl.starts_with("safe://"));
    assert!(key_pair.is_none());
}

#[test]
fn test_keys_test_coins_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let preload_amount = "1.154200000";
    let (_, key_pair) = unwrap!(safe.keys_create_preload_test_coins(preload_amount));
    let current_balance = unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair).sk));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_xorurl() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let preload_amount = "0.243000000";
    let (xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins(preload_amount));
    let current_balance = unwrap!(safe.keys_balance_from_url(&xorurl, &unwrap!(key_pair).sk));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_wrong_url() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (_xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("0"));

    let invalid_xorurl = "safe://this-is-not-a-valid-xor-url";
    let current_balance = safe.keys_balance_from_url(&invalid_xorurl, &unwrap!(key_pair).sk);
    match current_balance {
        Err(Error::InvalidInput(msg)) => {
            assert!(msg.contains("The location couldn't be resolved from the NRS URL provided"))
        }
        Err(err) => panic!("Error returned is not the expected: {:?}", err),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_test_coins_balance_wrong_location() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let amount = "35312.000000000";
    let (mut xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins(amount));

    let current_balance =
        unwrap!(safe.keys_balance_from_url(&xorurl, &unwrap!(key_pair.clone()).sk));
    assert_eq!(amount, current_balance);

    // let's corrupt the XOR-URL right where the encoded xorname bytes are in the string
    xorurl.replace_range(13..18, "ccccc");
    let current_balance = safe.keys_balance_from_url(&xorurl, &unwrap!(key_pair).sk);
    match current_balance {
        Err(Error::InvalidInput(msg)) => assert!(msg.contains(
            "The URL doesn't correspond to the public key derived from the provided secret key"
        )),
        Err(err) => panic!("Error returned is not the expected: {:?}", err),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_test_coins_balance_wrong_sk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (_xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("0"));
    let mut unwrapped_sk = unwrap!(key_pair).sk;
    unwrapped_sk.replace_range(..6, "ababab");
    let current_balance = safe.keys_balance_from_sk(&unwrapped_sk);
    match current_balance {
        Err(Error::ContentNotFound(msg)) => {
            assert!(msg.contains("No SafeKey found at specified location"))
        }
        Err(err) => panic!("Error returned is not the expected: {:?}", err),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let preload_amount = "1743.234";
    let (_, from_key_pair) = unwrap!(safe.keys_create_preload_test_coins(preload_amount));
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "1740.000000000";
    let (_, to_key_pair) = unwrap!(safe.keys_create(
        Some(from_key_pair_unwrapped.clone().sk),
        Some(amount.to_string()),
        None,
    ));

    let from_current_balance = unwrap!(safe.keys_balance_from_sk(&from_key_pair_unwrapped.sk));
    assert_eq!(
        "3.233999999", /*== 1743.234 - 1740 - 0.000000001 (creation cost) */
        from_current_balance
    );

    let to_current_balance = unwrap!(safe.keys_balance_from_sk(&unwrap!(to_key_pair).sk));
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_keys_balance_xorname() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let preload_amount = "435.34";
    let (from_xorname, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount));
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "35.300000000";
    let (to_xorname, to_key_pair) = unwrap!(safe.keys_create(
        Some(from_key_pair_unwrapped.clone().sk),
        Some(amount.to_string()),
        None,
    ));

    let from_current_balance =
        unwrap!(safe.keys_balance_from_url(&from_xorname, &from_key_pair_unwrapped.sk));
    assert_eq!(
        "400.039999999", /*== 435.34 - 35.3 - 0.000000001 (creation cost)*/
        from_current_balance
    );

    let to_current_balance =
        unwrap!(safe.keys_balance_from_url(&to_xorname, &unwrap!(to_key_pair).sk));
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_validate_sk_for_url() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("23.22"));
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = unwrap!(safe.validate_sk_for_url(&key_pair_unwrapped.sk, &xorurl));
    assert_eq!(pk, key_pair_unwrapped.pk);
}

#[test]
fn test_keys_transfer_from_zero_balance() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (_from_safekey_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("0.0"));
    let (to_safekey_xorurl, _key_pair2) = unwrap!(safe.keys_create_preload_test_coins("0.5"));

    // test fail to transfer with 0 balance at SafeKey in <from> argument
    match safe.keys_transfer("0", Some(unwrap!(key_pair1).sk), &to_safekey_xorurl, None) {
        Err(Error::InvalidAmount(msg)) => assert_eq!(
            msg,
            "The amount '0' specified for the transfer is invalid".to_string()
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };
}

#[test]
fn test_keys_transfer_diff_amounts() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));
    let (safekey1_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("0.5"));
    let (safekey2_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("100.5"));

    // test fail to transfer more than current balance at SafeKey in <from> argument
    match safe.keys_transfer(
        "100.6",
        Some(unwrap!(key_pair1.clone()).sk),
        &safekey2_xorurl,
        None,
    ) {
        Err(Error::NotEnoughBalance(msg)) => assert_eq!(
            msg,
            "Not enough balance for the transfer at provided source SafeKey".to_string()
        ),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test fail to transfer as it's a invalid/non-numeric amount
    match safe.keys_transfer(
        ".06",
        Some(unwrap!(key_pair1.clone()).sk),
        &safekey2_xorurl,
        None,
    ) {
        Err(Error::InvalidAmount(msg)) => assert_eq!(msg, "Invalid safecoins amount '.06'"),
        Err(err) => panic!(format!("Error returned is not the expected: {:?}", err)),
        Ok(_) => panic!("Transfer succeeded unexpectedly"),
    };

    // test successful transfer
    match safe.keys_transfer(
        "100.4",
        Some(unwrap!(key_pair2.clone()).sk),
        &safekey1_xorurl,
        None,
    ) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance =
                unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair2.clone()).sk));
            assert_eq!("0.100000000", from_current_balance);
            let to_current_balance = unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair1).sk));
            assert_eq!("100.900000000", to_current_balance);
        }
    };
}

#[test]
fn test_keys_transfer_to_wallet() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));

    let to_wallet_xorurl = unwrap!(safe.wallet_create());
    let (_, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("10.0"));
    unwrap!(safe.wallet_insert(
        &to_wallet_xorurl,
        Some("myfirstbalance".to_string()),
        true, // set --default
        &unwrap!(key_pair1.clone()).sk,
    ));

    let (_safekey_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("4621.45"));

    // test successful transfer
    match safe.keys_transfer(
        "523.87",
        Some(unwrap!(key_pair2.clone()).sk),
        &to_wallet_xorurl.clone(),
        None,
    ) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance =
                unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair2.clone()).sk));
            assert_eq!(
                "4097.580000000", /* 4621.45 - 523.87 */
                from_current_balance
            );
            let wallet_current_balance = unwrap!(safe.wallet_balance(&to_wallet_xorurl));
            assert_eq!("533.870000000", wallet_current_balance);
        }
    };
}

#[test]
fn test_keys_transfer_to_nrs_urls() {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z");
    unwrap!(safe.connect("", Some("fake-credentials")));

    let (_from_safekey_xorurl, key_pair1) = unwrap!(safe.keys_create_preload_test_coins("0.2"));

    let (to_safekey_xorurl, key_pair2) = unwrap!(safe.keys_create_preload_test_coins("0.1"));
    let to_nrsurl: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    let _ =
        unwrap!(safe.nrs_map_container_create(&to_nrsurl, &to_safekey_xorurl, false, true, false));

    // test successful transfer
    match safe.keys_transfer("0.2", Some(unwrap!(key_pair1.clone()).sk), &to_nrsurl, None) {
        Err(msg) => panic!(format!("Transfer was expected to succeed: {}", msg)),
        Ok(_) => {
            let from_current_balance = unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair1).sk));
            assert_eq!("0.000000000" /* 0.2 - 0.2 */, from_current_balance);
            let to_current_balance = unwrap!(safe.keys_balance_from_sk(&unwrap!(key_pair2).sk));
            assert_eq!("0.300000000" /* 0.1 + 0.2 */, to_current_balance);
        }
    };
}
