// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod lib_helpers;
mod scl_mock;

use lib_helpers::{pk_from_hex, pk_to_hex, sk_from_hex, KeyPair};
use scl_mock::{MockSCL, XorName};

use threshold_crypto::SecretKey;

// We expose a BLS key pair as two hex encoded strings
// TODO: consider supporting other encodings like base32 or just expose Vec<u8>
#[derive(Clone)]
pub struct BlsKeyPair {
    pub pk: String,
    pub sk: String,
}

pub struct Safe {
    safe_app: MockSCL,
}

impl Safe {
    pub fn new() -> Self {
        Self {
            safe_app: MockSCL::new(), // TODO: this will need to be replaced by auth process
        }
    }

    // Create a Key on the network and return its XOR name
    pub fn keys_create(
        &mut self,
        from: Option<BlsKeyPair>,
        preload_amount: Option<String>,
        pk: Option<String>,
    ) -> (XorName, Option<BlsKeyPair>) {
        let from_key_pair: KeyPair = match from {
            Some(key_pair) => KeyPair::from_hex_keys(&key_pair.pk, &key_pair.sk),
            None => panic!("Missing coins' key pair to cover the costs of the operation"), // TODO: fetch default wallet from account if not provided
        };

        let create_key = |pk| match preload_amount {
            Some(amount) => {
                self.safe_app
                    .create_balance(&from_key_pair.pk, &from_key_pair.sk, &pk, &amount)
            }
            None => self
                .safe_app
                .create_balance(&from_key_pair.pk, &from_key_pair.sk, &pk, "0"),
        };

        if let Some(pk_str) = pk {
            let pk = pk_from_hex(&pk_str);
            (create_key(pk), None)
        } else {
            let key_pair = KeyPair::random();
            let (pk, sk) = key_pair.to_hex_key_pair();
            (create_key(key_pair.pk), Some(BlsKeyPair { pk, sk }))
        }
    }

    // Create a Key on the network, allocates testcoins onto it, and return the Key's XOR name
    // This is avilable only when testing with mock-network
    // #[cfg(feature = "mock-network")]
    pub fn keys_create_test_coins(
        &mut self,
        preload_amount: String,
        pk: Option<String>,
    ) -> (XorName, Option<BlsKeyPair>) {
        if let Some(pk_str) = pk {
            let pk = pk_from_hex(&pk_str);
            let xorname = self.safe_app.allocate_test_coins(&pk, &preload_amount);
            (xorname, None)
        } else {
            let key_pair = KeyPair::random();
            let xorname = self
                .safe_app
                .allocate_test_coins(&key_pair.pk, &preload_amount);
            let (pk, sk) = key_pair.to_hex_key_pair();
            (xorname, Some(BlsKeyPair { pk, sk }))
        }
    }

    // Check Key's balance from the network from a given PublicKey
    pub fn keys_balance_from_pk(&self, key_pair: &BlsKeyPair) -> String {
        let pair = KeyPair::from_hex_keys(&key_pair.pk, &key_pair.sk);
        self.safe_app.get_balance_from_pk(&pair.pk, &pair.sk)
    }

    // Check Key's balance from the network from a given XOR name
    pub fn keys_balance_from_xorname(&self, xorname: &XorName, sk: &str) -> String {
        let secret_key: SecretKey = sk_from_hex(sk);
        self.safe_app.get_balance_from_xorname(xorname, &secret_key)
    }

    // Fetch Key's pk from the network from a given XOR name
    pub fn fetch_key_pk(&self, xorname: &XorName, sk: &str) -> String {
        let secret_key: SecretKey = sk_from_hex(sk);
        let public_key = self.safe_app.fetch_key_pk(xorname, &secret_key);
        pk_to_hex(&public_key)
    }
}

// Unit Tests

#[test]
fn test_keys_create_test_coins() {
    let mut safe = Safe::new();
    let (xorname, key_pair) = safe.keys_create_test_coins("12.23".to_string(), None);
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_test_coins_pk() {
    let mut safe = Safe::new();
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorname, key_pair) = safe.keys_create_test_coins("1.1".to_string(), Some(pk));
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}

#[test]
fn test_keys_create() {
    let mut safe = Safe::new();
    let (_, from_key_pair) = safe.keys_create_test_coins("23.23".to_string(), None);

    let (xorname, key_pair) = safe.keys_create(from_key_pair, None, None);
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_preload() {
    let mut safe = Safe::new();
    let (_, from_key_pair) = safe.keys_create_test_coins("543.2312".to_string(), None);

    let preload_amount = "1.8";
    let (xorname, key_pair) =
        safe.keys_create(from_key_pair, Some(preload_amount.to_string()), None);
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(kp) => {
            let balance = safe.keys_balance_from_pk(&BlsKeyPair {
                pk: kp.pk,
                sk: kp.sk,
            });
            assert_eq!(balance, preload_amount);
        }
    };
}

#[test]
fn test_keys_create_pk() {
    let mut safe = Safe::new();
    let (_, from_key_pair) = safe.keys_create_test_coins("1.1".to_string(), None);
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorname, key_pair) = safe.keys_create(from_key_pair, None, Some(pk));
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}

#[test]
fn test_keys_test_coins_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new();
    let preload_amount = "1.1542";
    let (_, key_pair) = safe.keys_create_test_coins(preload_amount.to_string(), None);
    let current_balance = safe.keys_balance_from_pk(&unwrap!(key_pair));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_xorname() {
    use unwrap::unwrap;
    let mut safe = Safe::new();
    let preload_amount = "0.243";
    let (xorname, key_pair) = safe.keys_create_test_coins(preload_amount.to_string(), None);
    let current_balance = safe.keys_balance_from_xorname(&xorname, &unwrap!(key_pair).sk);
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new();
    let preload_amount = "1743.234";
    let (_, from_key_pair) = safe.keys_create_test_coins(preload_amount.to_string(), None);
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "1740";
    let (_, to_key_pair) = safe.keys_create(
        Some(from_key_pair_unwrapped.clone()),
        Some(amount.to_string()),
        None,
    );

    let from_current_balance = safe.keys_balance_from_pk(&from_key_pair_unwrapped);
    assert_eq!("3.234" /*== 1743.234 - 1740*/, from_current_balance);

    let to_current_balance = safe.keys_balance_from_pk(&unwrap!(to_key_pair));
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_keys_balance_xorname() {
    use unwrap::unwrap;
    let mut safe = Safe::new();
    let preload_amount = "435.34";
    let (from_xorname, from_key_pair) =
        safe.keys_create_test_coins(preload_amount.to_string(), None);
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "35.3";
    let (to_xorname, to_key_pair) = safe.keys_create(
        Some(from_key_pair_unwrapped.clone()),
        Some(amount.to_string()),
        None,
    );

    let from_current_balance =
        safe.keys_balance_from_xorname(&from_xorname, &from_key_pair_unwrapped.sk);
    assert_eq!("400.04" /*== 435.34 - 35.3*/, from_current_balance);

    let to_current_balance = safe.keys_balance_from_xorname(&to_xorname, &unwrap!(to_key_pair).sk);
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_fetch_key_pk_test_coins() {
    use unwrap::unwrap;
    let mut safe = Safe::new();
    let (xorname, key_pair) = safe.keys_create_test_coins("23.22".to_string(), None);
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = safe.fetch_key_pk(&xorname, &key_pair_unwrapped.sk);
    assert_eq!(pk, key_pair_unwrapped.pk);
}

#[test]
fn test_fetch_key_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new();
    let (_, from_key_pair) = safe.keys_create_test_coins("0.56".to_string(), None);

    let (xorname, key_pair) = safe.keys_create(from_key_pair, None, None);
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = safe.fetch_key_pk(&xorname, &key_pair_unwrapped.sk);
    assert_eq!(pk, key_pair_unwrapped.pk);
}
