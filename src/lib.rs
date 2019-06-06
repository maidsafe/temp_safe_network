// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod lib_helpers;
mod scl_mock;
use log::{info, debug};

pub use lib_helpers::vec_to_hex;
use lib_helpers::{
    pk_from_hex, pk_to_hex, sk_from_hex, xorname_to_xorurl, xorurl_to_xorname, KeyPair,
};
use scl_mock::MockSCL;
use serde::{Deserialize, Serialize};
use threshold_crypto::SecretKey;
use unwrap::unwrap;
use uuid::Uuid;

static WALLET_TYPE_TAG: u64 = 10000;
static WALLET_DEFAULT: &str = "_default";

// The XOR-URL type (in the future in can be a struct with different functions)
pub type XorUrl = String;

// We expose a BLS key pair as two hex encoded strings
// TODO: consider supporting other encodings like base32 or just expose Vec<u8>
#[derive(Clone)]
pub struct BlsKeyPair {
    pub pk: String,
    pub sk: String,
}

// Struct which is serialised and stored in Wallet MD for linking to a spendable balance (Key)
#[derive(Serialize, Deserialize, Debug)]
struct WalletSpendableBalance {
    xorurl: XorUrl,
    sk: String,
}

pub struct Safe {
    safe_app: MockSCL,
    xorurl_base: String,
}

impl Safe {
    pub fn new(xorurl_base: String) -> Self {
        Self {
            safe_app: MockSCL::new(), // TODO: this will need to be replaced by auth process
            xorurl_base,
        }
    }

    // Create a Key on the network and return its XOR-URL
    pub fn keys_create(
        &mut self,
        from: BlsKeyPair,
        preload_amount: Option<String>,
        pk: Option<String>,
    ) -> (XorUrl, Option<BlsKeyPair>) {
        let from_key_pair = KeyPair::from_hex_keys(&from.pk, &from.sk);

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

        let xorurl = xorname_to_xorurl(&xorname, &self.xorurl_base);
        (xorurl, key_pair)
    }

    // Create a Key on the network, allocates testcoins onto it, and return the Key's XOR-URL
    // This is avilable only when testing with mock-network
    // #[cfg(feature = "mock-network")]
    pub fn keys_create_preload_test_coins(
        &mut self,
        preload_amount: String,
        pk: Option<String>,
    ) -> (XorUrl, Option<BlsKeyPair>) {
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

        let xorurl = xorname_to_xorurl(&xorname, &self.xorurl_base);
        (xorurl, key_pair)
    }

    // Check Key's balance from the network from a given PublicKey
    pub fn keys_balance_from_pk(&self, key_pair: &BlsKeyPair) -> String {
        let pair = KeyPair::from_hex_keys(&key_pair.pk, &key_pair.sk);
        self.safe_app.get_balance_from_pk(&pair.pk, &pair.sk)
    }

    // Check Key's balance from the network from a given XOR-URL
    pub fn keys_balance_from_xorurl(&self, xorurl: &XorUrl, sk: &str) -> String {
        let secret_key: SecretKey = sk_from_hex(sk);
        let xorname = xorurl_to_xorname(xorurl);
        self.safe_app
            .get_balance_from_xorname(&xorname, &secret_key)
    }

    // Fetch Key's pk from the network from a given XOR-URL
    pub fn keys_fetch_pk(&self, xorurl: &XorUrl) -> String {
        let xorname = xorurl_to_xorname(xorurl);
        let public_key = self.safe_app.keys_fetch_pk(&xorname);
        pk_to_hex(&public_key)
    }

    // Create an empty Wallet and return its XOR-URL
    pub fn wallet_create(&mut self) -> XorUrl {
        let xorname = self.safe_app.mutable_data_put(None, None, None, false);
        xorname_to_xorurl(&xorname, &self.xorurl_base)
    }

    // Add a Key to a Wallet to make it spendable
    pub fn wallet_insert(
        &mut self,
        wallet_xorurl: &XorUrl,
        name: &str,
        default: bool,
        key_pair: &BlsKeyPair,
        key_xorurl: &XorUrl,
    ) {
        let value = WalletSpendableBalance {
            xorurl: key_xorurl.clone(),
            sk: key_pair.sk.clone(),
        };
        let serialised_value = unwrap!(serde_json::to_string(&value));
        // FIXME: it should return error if the name already exists
        let k = name.to_string().into_bytes();
        let wallet_xorname = xorurl_to_xorname(&wallet_xorurl);
        self.safe_app.mutable_data_insert(
            &wallet_xorname,
            WALLET_TYPE_TAG,
            &k,
            &serialised_value.into_bytes(),
        );

        if default {
            // add the _default key
            self.safe_app.mutable_data_insert(
                &wallet_xorname,
                WALLET_TYPE_TAG,
                &WALLET_DEFAULT.to_string().into_bytes(),
                &k,
            );
        }
    }

    // Check the total balance of a Wallet found at a given XOR-URL
    pub fn wallet_balance(&mut self, xorurl: &XorUrl, _sk: &str) -> String {
        let mut total_balance: f64 = 0.0;
        let wallet_xorname = xorurl_to_xorname(&xorurl);
        let spendable_balances = self
            .safe_app
            .mutable_data_get_entries(&wallet_xorname, WALLET_TYPE_TAG);

        // Iterate through the Keys and query the balance for each
        spendable_balances.iter().for_each(|(name, balance)| {
            let thename = String::from_utf8_lossy(name).to_string();
            // ignore the _default Wallet MDkey
            if thename != WALLET_DEFAULT {
                let current_balance = String::from_utf8_lossy(balance).to_string();
                let spendable_balance: WalletSpendableBalance =
                    unwrap!(serde_json::from_str(&current_balance));

                let current_balance =
                    self.keys_balance_from_xorurl(&spendable_balance.xorurl, &spendable_balance.sk);
                total_balance += unwrap!(current_balance.parse::<f64>());
            }
        });
        total_balance.to_string()
    }

    fn wallet_get_default_balance(
        &mut self,
        wallet_xorurl: &XorUrl
    ) -> Result<WalletSpendableBalance, String> {
        let xorname = xorurl_to_xorname(&wallet_xorurl);
        let mut default_key: String = "".to_string();

        if let Some(default) =
            unwrap!(self
                .safe_app
                .mutable_data_get_key(WALLET_DEFAULT, &xorname, WALLET_TYPE_TAG))
        {
            default_key = String::from_utf8_lossy(&default).to_string();

            info!(
                "The default WalletBalance {:?} is named \"{:?}\"",
                &wallet_xorurl, &default_key
            );

        } else {
            return Err(format!(
                "No default balance found at Wallet {:?}",
                &wallet_xorurl
            ));
        }

        let the_balance: WalletSpendableBalance = {
            let default_balance_vec = unwrap!(unwrap!(self.safe_app.mutable_data_get_key(
                &default_key,
                &xorname,
                WALLET_TYPE_TAG
            )));

            let default_balance = String::from_utf8_lossy(&default_balance_vec).to_string();
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
    /// let wallet_xorurl = safe.wallet_create();
    /// let wallet_xorurl2 = safe.wallet_create();
    /// let (key1_xorurl, key_pair1) = safe.keys_create_preload_test_coins("14".to_string(), None);
    /// let (key2_xorurl, key_pair2) = safe.keys_create_preload_test_coins("1".to_string(), None);
    /// safe.wallet_insert(
    ///     &wallet_xorurl,
    ///     "frombalance",
    ///     true,
    ///     &key_pair1.unwrap(),
    ///     &key1_xorurl,
    /// );
    /// let current_balance = safe.wallet_balance(&wallet_xorurl, &sk);
    /// assert_eq!("14", current_balance);
    ///
    /// safe.wallet_insert(
    ///     &wallet_xorurl2,
    ///     "tobalance",
    ///     true,
    ///     &key_pair2.unwrap(),
    ///     &key2_xorurl,
    /// );
    ///
    ///
    /// safe.wallet_transfer( "10", &wallet_xorurl2, Some(wallet_xorurl) );
    /// let from_balance = safe.keys_balance_from_xorurl(&key1_xorurl, &sk );
    /// assert_eq!("4.", from_balance);
    /// let to_balance = safe.keys_balance_from_xorurl(&key2_xorurl, &sk );
    /// assert_eq!("11.", to_balance);
    /// ```
    pub fn wallet_transfer(
        &mut self,
        amount: &str,
        to: &XorUrl,
        from: Option<XorUrl>
    ) -> Result<Uuid, String> {
        // from is not optional until we know default account container / Wallet location ("root")
        // if no FROM for now, ERR
        // FROM needs to be from default

        // TODO: Grab "from" stdin

        // TODO, check if to/from are WalletContainers or PKs (via safe:)
        let from_wallet_xorurl =
            match from {
                Some(wallet_xorurl) => wallet_xorurl,
                _ => return Err(
                    "A \"<from>\" wallet is required until default wallets have been configured."
                        .to_string(),
                ),
            };

        let from_wallet_balance =
            unwrap!(self.wallet_get_default_balance(&from_wallet_xorurl));
        let to_wallet_balance = unwrap!(self.wallet_get_default_balance(&to));

        let from_pk = self
            .safe_app
            .keys_fetch_pk(&xorurl_to_xorname(&from_wallet_balance.xorurl));

        let to_pk = self
            .safe_app
            .keys_fetch_pk(&xorurl_to_xorname(&to_wallet_balance.xorurl));

        let from_sk = sk_from_hex(&from_wallet_balance.sk);
        let tx_id = Uuid::new_v4();

        self.safe_app
            .safecoin_transfer(&from_pk, &from_sk, &to_pk, &tx_id, amount);

        Ok(tx_id)
    }
}

// Unit Tests

#[test]
fn test_keys_create_preload_test_coins() {
    let mut safe = Safe::new("base32".to_string());
    let (xorurl, key_pair) = safe.keys_create_preload_test_coins("12.23".to_string(), None);
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
    let (xorurl, key_pair) = safe.keys_create_preload_test_coins("1.1".to_string(), Some(pk));
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}

#[test]
fn test_keys_create() {
    let mut safe = Safe::new("base32".to_string());
    let (_, from_key_pair) = safe.keys_create_preload_test_coins("23.23".to_string(), None);

    let (xorurl, key_pair) = safe.keys_create(unwrap!(from_key_pair), None, None);
    assert!(xorurl.starts_with("safe://"));
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_preload() {
    let mut safe = Safe::new("base32".to_string());
    let (_, from_key_pair) = safe.keys_create_preload_test_coins("543.2312".to_string(), None);

    let preload_amount = "1.8";
    let (xorurl, key_pair) = safe.keys_create(
        unwrap!(from_key_pair),
        Some(preload_amount.to_string()),
        None,
    );
    assert!(xorurl.starts_with("safe://"));
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
    let mut safe = Safe::new("base32".to_string());
    let (_, from_key_pair) = safe.keys_create_preload_test_coins("1.1".to_string(), None);
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorurl, key_pair) = safe.keys_create(unwrap!(from_key_pair), None, Some(pk));
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
    let (_, key_pair) = safe.keys_create_preload_test_coins(preload_amount.to_string(), None);
    let current_balance = safe.keys_balance_from_pk(&unwrap!(key_pair));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_xorurl() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let preload_amount = "0.243";
    let (xorurl, key_pair) = safe.keys_create_preload_test_coins(preload_amount.to_string(), None);
    let current_balance = safe.keys_balance_from_xorurl(&xorurl, &unwrap!(key_pair).sk);
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_balance_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let preload_amount = "1743.234";
    let (_, from_key_pair) = safe.keys_create_preload_test_coins(preload_amount.to_string(), None);
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "1740";
    let (_, to_key_pair) = safe.keys_create(
        from_key_pair_unwrapped.clone(),
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
    let mut safe = Safe::new("base32".to_string());
    let preload_amount = "435.34";
    let (from_xorname, from_key_pair) =
        safe.keys_create_preload_test_coins(preload_amount.to_string(), None);
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "35.3";
    let (to_xorname, to_key_pair) = safe.keys_create(
        from_key_pair_unwrapped.clone(),
        Some(amount.to_string()),
        None,
    );

    let from_current_balance =
        safe.keys_balance_from_xorurl(&from_xorname, &from_key_pair_unwrapped.sk);
    assert_eq!("400.04" /*== 435.34 - 35.3*/, from_current_balance);

    let to_current_balance = safe.keys_balance_from_xorurl(&to_xorname, &unwrap!(to_key_pair).sk);
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_keys_fetch_pk_test_coins() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let (xorurl, key_pair) = safe.keys_create_preload_test_coins("23.22".to_string(), None);
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = safe.keys_fetch_pk(&xorurl);
    assert_eq!(pk, key_pair_unwrapped.pk);
}

#[test]
fn test_keys_fetch_pk() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32".to_string());
    let (_, from_key_pair) = safe.keys_create_preload_test_coins("0.56".to_string(), None);

    let (xorurl, key_pair) = safe.keys_create(unwrap!(from_key_pair), None, None);
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = safe.keys_fetch_pk(&xorurl);
    assert_eq!(pk, key_pair_unwrapped.pk);
}

#[test]
fn test_wallet_create() {
    let mut safe = Safe::new("base32".to_string());
    let xorurl = safe.wallet_create();
    assert!(xorurl.starts_with("safe://"));
}

#[test]
fn test_wallet_insert_and_balance() {
    let mut safe = Safe::new("base32".to_string());
    let sk = String::from("391987fd429b4718a59b165b5799eaae2e56c697eb94670de8886f8fb7387058");
    let wallet_xorurl = safe.wallet_create();
    println!("AA: {}", wallet_xorurl);
    let (key1_xorurl, key_pair1) = safe.keys_create_preload_test_coins("12.23".to_string(), None);
    let (key2_xorurl, key_pair2) = safe.keys_create_preload_test_coins("1.53".to_string(), None);
    safe.wallet_insert(
        &wallet_xorurl,
        "myfirstbalance",
        true,
        &unwrap!(key_pair1),
        &key1_xorurl,
    );
    let current_balance = safe.wallet_balance(&wallet_xorurl, &sk);
    assert_eq!("12.23", current_balance);

    safe.wallet_insert(
        &wallet_xorurl,
        "mysecondbalance",
        false,
        &unwrap!(key_pair2),
        &key2_xorurl,
    );

    let current_balance = safe.wallet_balance(&wallet_xorurl, &sk);
    assert_eq!("13.76" /*== 12.23 + 1.53*/, current_balance);
}
