// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod lib_helpers;
mod scl_mock;

pub use lib_helpers::hash_to_hex;
use lib_helpers::{/*name_and_tag_xorurl, */ pk_from_hex, pk_to_hex, sk_from_hex, KeyPair};
use rand::rngs::OsRng;
use rand::Rng;
use rand_core::RngCore;
use safe_nd::mutable_data::{MutableData, MutableDataKind, Permission, User, Value};
use scl_mock::{MockSCL, XorHash, XorName};
use std::collections::{BTreeMap, BTreeSet};
use threshold_crypto::SecretKey;
use tiny_keccak::sha3_256;

// We expose a BLS key pair as two hex encoded strings
// TODO: consider supporting other encodings like base32 or just expose Vec<u8>
#[derive(Clone)]
pub struct BlsKeyPair {
    pub pk: String,
    pub sk: String,
}

// pub type XorUrl = String;

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

        let (xorname, key_pair) = match pk {
            Some(pk_str) => {
                let pk = pk_from_hex(&pk_str);
                (create_key(pk), None)
            }
            None => {
                let key_pair = KeyPair::random();
                let (pk, sk) = key_pair.to_hex_key_pair();
                (create_key(key_pair.pk), Some(BlsKeyPair { pk, sk }))
            }
        };

        (xorname, key_pair)
    }

    // Create a Key on the network, allocates testcoins onto it, and return the Key's XOR name
    // This is avilable only when testing with mock-network
    // #[cfg(feature = "mock-network")]
    pub fn keys_create_test_coins(
        &mut self,
        preload_amount: String,
        pk: Option<String>,
    ) -> (XorName, Option<BlsKeyPair>) {
        let (xorname, key_pair) = match pk {
            Some(pk_str) => {
                let pk = pk_from_hex(&pk_str);
                let xorhash = self.safe_app.allocate_test_coins(&pk, &preload_amount);
                (xorhash, None)
            }
            None => {
                let key_pair = KeyPair::random();
                let xorhash = self
                    .safe_app
                    .allocate_test_coins(&key_pair.pk, &preload_amount);
                let (pk, sk) = key_pair.to_hex_key_pair();
                (xorhash, Some(BlsKeyPair { pk, sk }))
            }
        };

        (xorname, key_pair)
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

    // Create an empty Wallet and return its XOR name
    pub fn wallet_create(&mut self) -> String {
        self.md_create(None, None, None, true)
    }

    // Check the total balance of a Wallet found at a given XOR name
    pub fn wallet_balance(&mut self, _xorname: &str, _sk: &str) -> String {
        let total_balance = "0";
        // let wallet_md = self.md_fetch(&target, &sk);
        // Iterate through the Keys and query the balance for each
        // foreach current_key in wallet_md {
        //      let current_key_balance = self.keys_balance_from_xorname(&current_key, &current_sk);
        //      total_balance += current_key_balance;
        // }
        total_balance.to_string()
    }

    pub fn md_create(
        &mut self,
        name: Option<String>,
        tag: Option<u64>,
        // _data: Option<String>,
        permissions: Option<String>,
        sequenced: bool,
    ) -> XorName {
        let mut xorname: XorHash;
        if let Some(n) = name {
            xorname = sha3_256(n.as_bytes());
        } else {
            let mut os_rng = OsRng::new().unwrap();
            xorname = [0u8; 32];
            os_rng.fill_bytes(&mut xorname);
        }

        let mut permission_map: BTreeMap<User, BTreeSet<Permission>> = BTreeMap::new();
        let perms_string: String;
        if let Some(perms) = permissions {
            perms_string = perms;
        } else {
            perms_string = String::from("read insert update delete permissions");
        }
        let permission_set: BTreeSet<Permission> = perms_string
            .split_whitespace()
            .map(|permission| match permission.to_lowercase().as_str() {
                "read" => Ok(Permission::Read),
                "insert" => Ok(Permission::Insert),
                "update" => Ok(Permission::Update),
                "delete" => Ok(Permission::Delete),
                "permissions" => Ok(Permission::ManagePermissions),
                _ => Err("Invalid permission"),
            })
            .filter_map(Result::ok)
            .collect();
        permission_map.insert(User::Anyone, permission_set);

        let md_kind = match sequenced {
            true => {
                // if let Some(data_string) = data {
                // }
                let mut inner: BTreeMap<String, Vec<u8>> = BTreeMap::new();
                inner.insert(String::from("test"), String::from("test").into_bytes());
                MutableDataKind::Sequenced { data: inner }
            }
            false => {
                // An unsequenced MD doesn't need data versioning. Noted here: https://github.com/maidsafe/safe-nd/issues/7

                // if let Some(data_string) = data {
                // }
                let mut inner: BTreeMap<String, Value> = BTreeMap::new();
                inner.insert(
                    String::from("test"),
                    Value {
                        data: String::from("test").into_bytes(),
                        version: 0,
                    },
                );
                MutableDataKind::Unsequenced { data: inner }
            }
        };
        let md_tag: u64;
        if let Some(t) = tag {
            md_tag = t;
        } else {
            let mut rng = rand::thread_rng();
            md_tag = rng.gen();
        }
        let sk = SecretKey::random();
        let pk = sk.public_key();
        let md = MutableData::new(xorname, md_tag, md_kind, permission_map, pk);
        self.safe_app.mutable_data_put(&md);

        hash_to_hex(xorname.to_vec())
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

#[test]
fn test_md_create() {
    use lib_helpers::hash_to_hex;
    let mut safe = Safe::new();
    let name = String::from("test");
    let xorname = safe.md_create(Some(name.clone()), None, None, false);
    let hash = sha3_256(name.as_bytes());
    let hash_as_string: String = hash_to_hex(hash.to_vec());
    assert_eq!(xorname, hash_as_string);
}
