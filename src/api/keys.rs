// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub use super::helpers::vec_to_hex;
use super::helpers::{
    parse_coins_amount, pk_from_hex, pk_to_hex, sk_from_hex,
    xorname_to_xorurl, xorurl_to_xorname, KeyPair,
};

use log::{debug, info};
// #[cfg(not(feature = "scl-mock"))]
// use safe_client_libs::SafeApp;
// use safe_core::ipc::{AppExchangeInfo, AuthReq, IpcReq};
// use safe_nd::AppPermissions;
// #[cfg(feature = "scl-mock")]
// use scl_mock::SafeApp;

use threshold_crypto::SecretKey;
use unwrap::unwrap;
use uuid::Uuid;

// Type tag used for the Wallet container
static WALLET_TYPE_TAG: u64 = 10_000;

static WALLET_DEFAULT: &str = "_default";
static WALLET_DEFAULT_BYTES: &[u8] = b"_default";

// use safe_cli::Safe;
use super::safe::{Safe, BlsKeyPair, WalletSpendableBalance, XorUrl};


impl Safe {

    // Generate a key pair without creating and/or storing a Key on the network
    pub fn keypair(&self) -> Result<BlsKeyPair, String> {
        let key_pair = KeyPair::random();
        let (pk, sk) = key_pair.to_hex_key_pair();
        Ok(BlsKeyPair { pk, sk })
    }

    // Create a Key on the network and return its XOR-URL
    pub fn keys_create(
        &mut self,
        from: BlsKeyPair,
        preload_amount: Option<String>,
        pk: Option<String>,
    ) -> Result<(XorUrl, Option<BlsKeyPair>), String> {
        let from_key_pair = KeyPair::from_hex_keys(&from.pk, &from.sk)?;
        let _ = parse_coins_amount(&preload_amount.clone().unwrap_or_else(|| "0".to_string()))?;

        let mut create_coin_balance = |to_pk, amount: &str| match self.safe_app.create_balance(
            &from_key_pair.pk,
            &from_key_pair.sk,
            &to_pk,
            amount,
        ) {
            Err("InvalidAmount") => Err(format!(
                "The amount '{}' specified for the transfer is invalid",
                amount
            )),
            Err("NotEnoughBalance") => {
                Err("Not enough balance at 'source' for the operation".to_string())
            }
            Err(other_error) => Err(format!(
                "Unexpected error when attempting to create Key: {}",
                other_error
            )),
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
                let (pk, sk) = key_pair.to_hex_key_pair();
                let xorname = create_coin_balance(key_pair.pk, &amount)?;
                (xorname, Some(BlsKeyPair { pk, sk }))
            }
        };

        let xorurl = xorname_to_xorurl(&xorname, &self.xorurl_base)?;
        Ok((xorurl, key_pair))
    }

    // Create a Key on the network, allocates testcoins onto it, and return the Key's XOR-URL
    // This is avilable only when testing with mock-network
    // #[cfg(feature = "mock-network")]
    pub fn keys_create_preload_test_coins(
        &mut self,
        preload_amount: String,
        pk: Option<String>,
    ) -> Result<(XorUrl, Option<BlsKeyPair>), String> {
        let _ = parse_coins_amount(&preload_amount)?;
        let (xorname, key_pair) = match pk {
            Some(pk_str) => {
                let pk = unwrap!(pk_from_hex(&pk_str));
                let xorname = self.safe_app.allocate_test_coins(&pk, &preload_amount);
                (xorname, None)
            }
            None => {
                let key_pair = KeyPair::random();
                let xorname = self
                    .safe_app
                    .allocate_test_coins(&key_pair.pk, &preload_amount);
                let (pk, sk) = key_pair.to_hex_key_pair();
                (xorname, Some(BlsKeyPair { pk, sk }))
            }
        };

        let xorurl = xorname_to_xorurl(&xorname, &self.xorurl_base)?;
        Ok((xorurl, key_pair))
    }

    // Check Key's balance from the network from a given PublicKey
    pub fn keys_balance_from_pk(&self, key_pair: &BlsKeyPair) -> Result<String, String> {
        let pair = KeyPair::from_hex_keys(&key_pair.pk, &key_pair.sk)?;
        self.safe_app
            .get_balance_from_pk(&pair.pk, &pair.sk)
            .map_err(|_| "No Key found at specified location".to_string())
    }

    // Check Key's balance from the network from a given XOR-URL
    pub fn keys_balance_from_xorurl(&self, xorurl: &str, sk: &str) -> Result<String, String> {
        let secret_key: SecretKey =
            sk_from_hex(sk).map_err(|_| "Invalid secret key provided".to_string())?;
        let xorname = xorurl_to_xorname(xorurl)?;

        Ok(self
            .safe_app
            .get_balance_from_xorname(&xorname, &secret_key)
            .map_err(|_| "No Key found at specified location".to_string())?)
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
