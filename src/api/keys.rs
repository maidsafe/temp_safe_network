// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{parse_coins_amount, pk_from_hex, pk_to_hex, sk_from_hex, KeyPair};
use super::xorurl::SafeContentType;
use super::{Error, ResultReturn, Safe, XorUrl, XorUrlEncoder};
use threshold_crypto::SecretKey;
use unwrap::unwrap;

// A trait that the Validate derive will impl
use validator::{Validate, ValidationErrors};

// We expose a BLS key pair as two hex encoded strings
// TODO: consider supporting other encodings like base32 or just expose Vec<u8>
#[derive(Clone, Validate)]
pub struct BlsKeyPair {
    #[validate(length(equal = "96"))]
    pub pk: String,
    #[validate(length(equal = "64"))]
    pub sk: String,
}

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
    // Generate a key pair without creating and/or storing a Key on the network
    pub fn keypair(&self) -> ResultReturn<BlsKeyPair> {
        let key_pair = KeyPair::random();
        let (pk, sk) = key_pair.to_hex_key_pair()?;
        Ok(BlsKeyPair { pk, sk })
    }

    // Create a Key on the network and return its XOR-URL.
    pub fn keys_create(
        &mut self,
        from: Option<String>,
        preload_amount: Option<String>,
        pk: Option<String>,
    ) -> ResultReturn<(XorUrl, Option<BlsKeyPair>)> {
        let from_sk = match from {
            Some(sk) => match sk_from_hex(&sk) {
                Ok(sk) => Some(sk),
                Err(_) => return Err(Error::InvalidInput("Secret key invalid".to_string())),
            },
            None => None,
        };
        let _ = parse_coins_amount(&preload_amount.clone().unwrap_or_else(|| "0".to_string()))?;

        let create_coin_balance =
            |to_pk, amount: &str| match self.safe_app.create_balance(from_sk, to_pk, amount) {
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

        let amount = preload_amount.unwrap_or_else(|| "0.0".to_string());

        let (xorname, key_pair) = match pk {
            Some(pk_str) => {
                let pk = unwrap!(pk_from_hex(&pk_str));
                let xorname = create_coin_balance(pk, &amount)?;
                (xorname, None)
            }
            None => {
                let key_pair = KeyPair::random();
                let (pk, sk) = key_pair.to_hex_key_pair()?;
                let xorname = create_coin_balance(key_pair.pk, &amount)?;
                (xorname, Some(BlsKeyPair { pk, sk }))
            }
        };

        let xorurl =
            XorUrlEncoder::encode(xorname, 0, SafeContentType::CoinBalance, &self.xorurl_base)?;
        Ok((xorurl, key_pair))
    }

    // Create a Key on the network, allocates testcoins onto it, and return the Key's XOR-URL
    // This is avilable only when testing with mock-network
    // #[cfg(feature = "mock-network")]
    pub fn keys_create_preload_test_coins(
        &mut self,
        preload_amount: String,
        pk: Option<String>,
    ) -> ResultReturn<(XorUrl, Option<BlsKeyPair>)> {
        let _ = parse_coins_amount(&preload_amount)?;
        let (xorname, key_pair) = match pk {
            Some(pk_str) => {
                let pk = unwrap!(pk_from_hex(&pk_str));
                let xorname = self.safe_app.allocate_test_coins(&pk, &preload_amount)?;
                (xorname, None)
            }
            None => {
                let key_pair = KeyPair::random();
                let xorname = self
                    .safe_app
                    .allocate_test_coins(&key_pair.pk, &preload_amount)?;
                let (pk, sk) = key_pair.to_hex_key_pair()?;
                (xorname, Some(BlsKeyPair { pk, sk }))
            }
        };

        let xorurl =
            XorUrlEncoder::encode(xorname, 0, SafeContentType::CoinBalance, &self.xorurl_base)?;
        Ok((xorurl, key_pair))
    }

    // Check Key's balance from the network from a given PublicKey
    pub fn keys_balance_from_pk(&self, key_pair: &BlsKeyPair) -> ResultReturn<String> {
        let pair = KeyPair::from_hex_keys(&key_pair.pk, &key_pair.sk)?;
        self.safe_app
            .get_balance_from_pk(&pair.pk, &pair.sk)
            .map_err(|_| Error::ContentNotFound("No Key found at specified location".to_string()))
    }

    // Check Key's balance from the network from a given XOR-URL
    pub fn keys_balance_from_xorurl(&self, xorurl: &str, sk: &str) -> ResultReturn<String> {
        let secret_key: SecretKey = sk_from_hex(sk)
            .map_err(|_| Error::InvalidInput("Invalid secret key provided".to_string()))?;
        let xorurl_encoder = XorUrlEncoder::from_url(xorurl)?;

        let balance = self
            .safe_app
            .get_balance_from_xorname(&xorurl_encoder.xorname(), &secret_key)
            .map_err(|_| {
                Error::ContentNotFound("No Key found at specified location".to_string())
            })?;

        Ok(balance)
    }

    // Fetch Key's pk from the network from a given XOR-URL
    pub fn fetch_pk_from_xorname(&self, xorurl: &str) -> ResultReturn<String> {
        let xorurl_encoder = XorUrlEncoder::from_url(xorurl)?;
        let public_key = self
            .safe_app
            .fetch_pk_from_xorname(&xorurl_encoder.xorname())
            .map_err(|_| {
                Error::ContentNotFound("No Key found at specified location".to_string())
            })?;

        Ok(pk_to_hex(&public_key))
    }
}

// Unit Tests

#[test]
fn test_keys_create_preload_test_coins() {
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
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
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
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
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let (_, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("23.23".to_string(), None));

    let (xorurl, key_pair) = unwrap!(safe.keys_create(Some(unwrap!(from_key_pair).sk), None, None));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_preload() {
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let (_, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("543.2312".to_string(), None));

    let preload_amount = "1.8";
    let (xorurl, key_pair) = unwrap!(safe.keys_create(
        Some(unwrap!(from_key_pair).sk),
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
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    match safe.keys_create_preload_test_coins(".45".to_string(), None) {
        Err(msg) => assert_eq!(
            msg,
            Error::InvalidAmount(
                "Invalid safecoins amount '.45', expected a numeric value".to_string()
            )
        ),
        Ok(_) => panic!("Key with test-coins was created unexpectedly"),
    };

    let (_, key_pair) = unwrap!(safe.keys_create_preload_test_coins("12".to_string(), None));
    match safe.keys_create(
        Some(unwrap!(key_pair.clone()).sk),
        Some(".003".to_string()),
        None,
    ) {
        Err(msg) => assert_eq!(
            msg,
            Error::InvalidAmount(
                "Invalid safecoins amount '.003', expected a numeric value".to_string()
            )
        ),
        Ok(_) => panic!("Key was created unexpectedly"),
    };

    // test fail with corrupted secret key
    let mut unwrapped_key_pair = unwrap!(key_pair.clone());
    unwrapped_key_pair.sk.replace_range(..6, "ababab");
    match safe.keys_create(Some(unwrapped_key_pair.sk), Some(".003".to_string()), None) {
        Err(msg) => assert_eq!(
            msg,
            Error::InvalidAmount(
                "Invalid safecoins amount '.003', expected a numeric value".to_string()
            )
        ),
        Ok(_) => panic!("Key was created unexpectedly"),
    };

    // test fail to preload with more than available balance in source (which has only 12 coins)
    match safe.keys_create(
        Some(unwrap!(key_pair).sk),
        Some("12.00001".to_string()),
        None,
    ) {
        Err(msg) => assert_eq!(
            msg,
            Error::NotEnoughBalance("Not enough balance at 'source' for the operation".to_string())
        ),
        Ok(_) => panic!("Key was created unexpectedly"),
    };
}

#[test]
fn test_keys_create_pk() {
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let (_, from_key_pair) = unwrap!(safe.keys_create_preload_test_coins("1.1".to_string(), None));
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorurl, key_pair) =
        unwrap!(safe.keys_create(Some(unwrap!(from_key_pair).sk), None, Some(pk)));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}

#[test]
fn test_keys_test_coins_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let preload_amount = "1.1542";
    let (_, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));
    let current_balance = unwrap!(safe.keys_balance_from_pk(&unwrap!(key_pair)));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_xorurl() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let preload_amount = "0.243";
    let (xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));
    let current_balance = unwrap!(safe.keys_balance_from_xorurl(&xorurl, &unwrap!(key_pair).sk));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_wrong_xorurl() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let (_xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("0".to_string(), None));

    let invalid_xorurl = "safe://this-is-not-a-valid-xor-url";
    let current_balance = safe.keys_balance_from_xorurl(&invalid_xorurl, &unwrap!(key_pair).sk);
    match current_balance {
        Err(Error::InvalidXorUrl(msg)) => assert!(msg.contains("Failed to decode XOR-URL")),
        Err(err) => panic!("Error returned is not the expected: {:?}", err),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_test_coins_balance_wrong_location() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let amount = "35312";
    let (mut xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(amount.to_string(), None));

    let current_balance =
        unwrap!(safe.keys_balance_from_xorurl(&xorurl, &unwrap!(key_pair.clone()).sk));
    assert_eq!(amount, current_balance);

    // let's corrupt the XOR-URL
    xorurl.replace_range(11..16, "ccccc");
    let current_balance = safe.keys_balance_from_xorurl(&xorurl, &unwrap!(key_pair).sk);
    match current_balance {
        Err(Error::ContentNotFound(msg)) => {
            assert!(msg.contains("No Key found at specified location"))
        }
        Err(err) => panic!("Error returned is not the expected: {:?}", err),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_test_coins_balance_wrong_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let (_xorurl, key_pair) = unwrap!(safe.keys_create_preload_test_coins("0".to_string(), None));
    let mut unwrapped_key_pair = unwrap!(key_pair);
    unwrapped_key_pair.pk.replace_range(..6, "ababab");
    let current_balance = safe.keys_balance_from_pk(&unwrapped_key_pair);
    match current_balance {
        Err(Error::InvalidInput(msg)) => assert!(msg.contains("Invalid public key bytes")),
        Err(err) => panic!("Error returned is not the expected: {:?}", err),
        Ok(balance) => panic!("Unexpected balance obtained: {:?}", balance),
    };
}

#[test]
fn test_keys_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let preload_amount = "1743.234";
    let (_, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "1740";
    let (_, to_key_pair) = unwrap!(safe.keys_create(
        Some(from_key_pair_unwrapped.clone().sk),
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
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let preload_amount = "435.34";
    let (from_xorname, from_key_pair) =
        unwrap!(safe.keys_create_preload_test_coins(preload_amount.to_string(), None));
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "35.3";
    let (to_xorname, to_key_pair) = unwrap!(safe.keys_create(
        Some(from_key_pair_unwrapped.clone().sk),
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
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let (xorurl, key_pair) =
        unwrap!(safe.keys_create_preload_test_coins("23.22".to_string(), None));
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = unwrap!(safe.fetch_pk_from_xorname(&xorurl));
    assert_eq!(pk, key_pair_unwrapped.pk);
}

#[test]
fn test_fetch_pk_from_xorname() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    unwrap!(safe.connect("", ""));
    let (_, from_key_pair) = unwrap!(safe.keys_create_preload_test_coins("0.56".to_string(), None));

    let (xorurl, key_pair) = unwrap!(safe.keys_create(Some(unwrap!(from_key_pair).sk), None, None));
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = unwrap!(safe.fetch_pk_from_xorname(&xorurl));
    assert_eq!(pk, key_pair_unwrapped.pk);
}
