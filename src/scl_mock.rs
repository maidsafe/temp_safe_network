// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::lib_helpers::vec_to_hex;
use log::debug;
use rand::rngs::OsRng;
use rand::Rng;
use rand_core::RngCore;
use safe_nd::mutable_data::{MutableData, MutableDataKind, Permission, User, Value};
pub use safe_nd::{XorName, XOR_NAME_LEN};
use safecoin::{Coins, NanoCoins};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::io::Write;
use std::str::FromStr;
use std::{fs, str};
use threshold_crypto::{PublicKey, SecretKey};
use unwrap::unwrap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct CoinBalance {
    owner: PublicKey,
    value: String,
}

type AppendOnlyDataMock = BTreeMap<usize, Vec<u8>>;
type TxStatusList = BTreeMap<String, String>;
type XorNameStr = String;

static MOCK_FILE: &str = "./mock_data.txt";

#[derive(Default, Serialize, Deserialize)]
struct MockData {
    coin_balances: BTreeMap<XorNameStr, CoinBalance>,
    txs: BTreeMap<XorNameStr, TxStatusList>, // keep track of TX status per tx ID, per xorname
    unpublished_append_only: BTreeMap<XorNameStr, AppendOnlyDataMock>, // keep a versioned map of data per xorname
    mutable_data: BTreeMap<XorNameStr, MutableData>,
}

fn xorname_from_pk(pk: &PublicKey) -> XorName {
    let pk_as_bytes: [u8; 48] = pk.to_bytes();
    let mut xorname = XorName::default();
    xorname.copy_from_slice(&pk_as_bytes[..XOR_NAME_LEN]);
    xorname
}

pub struct MockSCL {
    mock_data: MockData,
}

/// Writes the mock data onto the mock file
impl Drop for MockSCL {
    fn drop(&mut self) {
        let serialised = unwrap!(serde_json::to_string(&self.mock_data));
        debug!("serialised = {}", serialised);

        let mut file = unwrap!(fs::File::create(&MOCK_FILE));
        unwrap!(file.write(serialised.as_bytes()));
    }
}

impl MockSCL {
    pub fn new() -> Self {
        let mock_data = match fs::File::open(&MOCK_FILE) {
            Ok(file) => {
                let deserialised: MockData = unwrap!(serde_json::from_reader(&file));
                deserialised
            }
            Err(error) => {
                debug!("Error reading mock file. {}", error.to_string());
                MockData::default()
            }
        };

        Self { mock_data }
    }

    pub fn create_balance(
        &mut self,
        from_pk: &PublicKey,
        from_sk: &SecretKey,
        new_balance_owner: &PublicKey,
        amount: &str,
    ) -> Result<XorName, &str> {
        let from_xorname = xorname_from_pk(from_pk);
        let from_balance = unwrap!(Coins::from_str(&unwrap!(
            self.get_balance_from_pk(from_pk, from_sk)
        )));
        let from_nano_balance = unwrap!(NanoCoins::try_from(from_balance));
        let amount_coin = unwrap!(Coins::from_str(amount));
        let amount_nano = unwrap!(NanoCoins::try_from(amount_coin));
        if from_nano_balance.num() < amount_nano.num() {
            return Err("NotEnoughBalance");
        }
        let from_new_amount = unwrap!(NanoCoins::new(from_nano_balance.num() - amount_nano.num()));
        self.mock_data.coin_balances.insert(
            vec_to_hex(from_xorname.to_vec()),
            CoinBalance {
                owner: (*from_pk),
                value: Coins::from(from_new_amount).to_string(),
            },
        );

        let to_xorname = xorname_from_pk(new_balance_owner);
        self.mock_data.coin_balances.insert(
            vec_to_hex(to_xorname.to_vec()),
            CoinBalance {
                owner: (*new_balance_owner),
                value: amount.to_string(),
            },
        );

        Ok(to_xorname)
    }

    pub fn allocate_test_coins(&mut self, to_pk: &PublicKey, amount: &str) -> XorName {
        let xorname = xorname_from_pk(to_pk);
        self.mock_data.coin_balances.insert(
            vec_to_hex(xorname.to_vec()),
            CoinBalance {
                owner: (*to_pk),
                value: amount.to_string(),
            },
        );

        xorname
    }

    pub fn get_balance_from_pk(&self, pk: &PublicKey, sk: &SecretKey) -> Result<String, &str> {
        let xorname = xorname_from_pk(pk);
        self.get_balance_from_xorname(&xorname, &sk)
    }

    pub fn get_balance_from_xorname(
        &self,
        xorname: &XorName,
        _sk: &SecretKey,
    ) -> Result<String, &str> {
        match &self
            .mock_data
            .coin_balances
            .get(&vec_to_hex(xorname.to_vec()))
        {
            None => Err("CoinBalance data not found"),
            Some(coin_balance) => Ok(coin_balance
                .value
                .to_string()
                .replace("Coins(", "")
                .replace(")", "")),
        }
    }

    pub fn keys_fetch_pk(&self, xorname: &XorName) -> Result<PublicKey, &str> {
        match &self
            .mock_data
            .coin_balances
            .get(&vec_to_hex(xorname.to_vec()))
        {
            None => Err("CoinBalance data not found"),
            Some(coin_balance) => Ok(coin_balance.owner),
        }
    }

    pub fn safecoin_transfer(
        &mut self,
        from_pk: &PublicKey,
        from_sk: &SecretKey,
        to_pk: &PublicKey,
        tx_id: &Uuid,
        amount: &str,
    ) -> Result<Uuid, &str> {
        let to_xorname = xorname_from_pk(to_pk);
        let from_xorname = xorname_from_pk(from_pk);

        let the_tx_id = *tx_id;
        // generate TX in destination section (to_pk)
        let mut txs_for_xorname = match self.mock_data.txs.get(&vec_to_hex(to_xorname.to_vec())) {
            Some(txs) => txs.clone(),
            None => BTreeMap::new(),
        };
        txs_for_xorname.insert(
            tx_id.to_string(),
            format!("Success({})", amount).to_string(),
        );
        self.mock_data
            .txs
            .insert(vec_to_hex(to_xorname.to_vec()), txs_for_xorname);

        let amount_coin = (Coins::from_str(amount)).map_err(|_| "InvalidAmount")?;

        // reduce balance from safecoin_transferer
        let from_balance = unwrap!(Coins::from_str(&unwrap!(
            self.get_balance_from_pk(from_pk, from_sk)
        )));
        let from_nano_balance = unwrap!(NanoCoins::try_from(from_balance));
        let amount_nano = unwrap!(NanoCoins::try_from(amount_coin));
        if from_nano_balance.num() < amount_nano.num() {
            return Err("NotEnoughBalance");
        }
        let from_new_amount = unwrap!(NanoCoins::new(from_nano_balance.num() - amount_nano.num()));
        self.mock_data.coin_balances.insert(
            vec_to_hex(from_xorname.to_vec()),
            CoinBalance {
                owner: (*from_pk),
                value: Coins::from(from_new_amount).to_string(),
            },
        );

        // credit destination
        let to_balance = unwrap!(Coins::from_str(&unwrap!(
            self.get_balance_from_pk(to_pk, from_sk /*incorrect but doesn't matter for now*/)
        ),));
        let to_nano_balance = unwrap!(NanoCoins::try_from(to_balance));
        let to_new_amount = unwrap!(NanoCoins::new(to_nano_balance.num() + amount_nano.num()));
        self.mock_data.coin_balances.insert(
            vec_to_hex(to_xorname.to_vec()),
            CoinBalance {
                owner: (*to_pk),
                value: Coins::from(to_new_amount).to_string(),
            },
        );

        Ok(the_tx_id)
    }

    #[allow(dead_code)]
    pub fn get_transaction(&self, tx_id: &Uuid, pk: &PublicKey, _sk: &SecretKey) -> String {
        let xorname = xorname_from_pk(pk);
        let txs_for_xorname = &self.mock_data.txs[&vec_to_hex(xorname.to_vec())];
        let tx_state = unwrap!(txs_for_xorname.get(&tx_id.to_string()));
        tx_state.to_string()
    }

    #[allow(dead_code)]
    pub fn unpublished_append_only_put(
        &mut self,
        pk: &PublicKey,
        _sk: &SecretKey,
        data: &[u8],
    ) -> XorName {
        let xorname = xorname_from_pk(pk);
        let mut uao_for_xorname = match self
            .mock_data
            .unpublished_append_only
            .get(&vec_to_hex(xorname.to_vec()))
        {
            Some(uao) => uao.clone(),
            None => BTreeMap::new(),
        };
        uao_for_xorname.insert(uao_for_xorname.len(), data.to_vec());
        self.mock_data
            .unpublished_append_only
            .insert(vec_to_hex(xorname.to_vec()), uao_for_xorname);

        xorname
    }

    #[allow(dead_code)]
    pub fn unpublished_append_only_get(
        &self,
        pk: &PublicKey,
        _sk: &SecretKey,
        version: Option<usize>,
    ) -> Vec<u8> {
        let xorname = xorname_from_pk(pk);
        let uao_for_xorname =
            &self.mock_data.unpublished_append_only[&vec_to_hex(xorname.to_vec())];
        let data = match version {
            Some(version) => unwrap!(uao_for_xorname.get(&version)),
            None => unwrap!(uao_for_xorname.get(&self.mock_data.unpublished_append_only.len())),
        };

        data.to_vec()
    }

    pub fn mutable_data_put(
        &mut self,
        name: Option<XorName>,
        tag: Option<u64>,
        // _data: Option<String>,
        permissions: Option<String>,
        sequenced: bool,
    ) -> XorName {
        let xorname = name.unwrap_or_else(|| {
            let mut os_rng = OsRng::new().unwrap();
            let mut xorname = [0u8; 32];
            os_rng.fill_bytes(&mut xorname);
            xorname
        });

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

        let md_kind = if sequenced {
            // if let Some(data_string) = data {
            // }
            let inner: BTreeMap<String, Vec<u8>> = BTreeMap::new();
            //inner.insert(String::from("testkeyseq"), b"testvalueseq".to_vec());
            MutableDataKind::Sequenced { data: inner }
        } else {
            // An unsequenced MD doesn't need data versioning. Noted here: https://github.com/maidsafe/safe-nd/issues/7

            // if let Some(data_string) = data {
            // }
            let inner: BTreeMap<String, Value> = BTreeMap::new();
            /*inner.insert(
                String::from("testkeyunseq"),
                Value {
                    data: b"testvalueunseq".to_vec(),
                    version: 0,
                },
            );*/
            MutableDataKind::Unsequenced { data: inner }
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

        let xorname_as_string: String = vec_to_hex(md.name().to_vec());
        self.mock_data
            .mutable_data
            .insert(xorname_as_string.clone(), md.clone());

        md.name()
    }

    pub fn mutable_data_insert(&mut self, xorname: &XorName, _tag: u64, key: &[u8], value: &[u8]) {
        let xorname_as_string: String = vec_to_hex(xorname.to_vec());
        let md = &self.mock_data.mutable_data[&xorname_as_string];
        if let MutableDataKind::Unsequenced { data } = &md.data {
            let mut inner: BTreeMap<String, Value> = data.clone();
            inner.insert(
                String::from_utf8_lossy(key).to_string(),
                Value {
                    data: value.to_vec(),
                    version: 0,
                },
            );
            let mut updated_md = md.clone();
            updated_md.data = MutableDataKind::Unsequenced { data: inner };
            self.mock_data
                .mutable_data
                .insert(xorname_as_string, updated_md);
        }
    }

    #[allow(dead_code)]
    pub fn mutable_data_delete(&mut self, xorname: &XorName, _tag: u64, key: &[u8]) {
        let xorname_as_string: String = vec_to_hex(xorname.to_vec());
        let md = &self.mock_data.mutable_data[&xorname_as_string];
        if let MutableDataKind::Unsequenced { data } = &md.data {
            let mut inner: BTreeMap<String, Value> = data.clone();
            inner.remove(&String::from_utf8_lossy(key).to_string());
            let mut updated_md = md.clone();
            updated_md.data = MutableDataKind::Unsequenced { data: inner };
            self.mock_data
                .mutable_data
                .insert(xorname_as_string, updated_md);
        }
    }

    pub fn mutable_data_get_key(
        &mut self,
        key: &str,
        xorname: &XorName,
        _tag: u64,
    ) -> Result<Option<Vec<u8>>, String> {
        let xorname_as_string: String = vec_to_hex(xorname.to_vec());
        let md = &self.mock_data.mutable_data[&xorname_as_string];
        let mut res = None;
        if let MutableDataKind::Unsequenced { data } = &md.data {
            data.iter().for_each(|elem| {
                let this_key = elem.0.clone();

                if this_key == key {
                    res = Some(elem.1.data.clone())
                }
                // res.insert(elem.0.clone().into_bytes(), elem.1.data.clone());
            });
        }
        // let the_key = unwrap!(res[$key]);

        Ok(res)
    }

    pub fn mutable_data_get_entries(
        &mut self,
        xorname: &XorName,
        _tag: u64,
    ) -> BTreeMap<Vec<u8>, Vec<u8>> {
        let xorname_as_string: String = vec_to_hex(xorname.to_vec());
        let md = &self.mock_data.mutable_data[&xorname_as_string];
        let mut res = BTreeMap::new();
        if let MutableDataKind::Unsequenced { data } = &md.data {
            data.iter().for_each(|elem| {
                res.insert(elem.0.clone().into_bytes(), elem.1.data.clone());
            });
        }
        res.clone()
    }
}

#[test]
fn test_unpublished_append_only_put() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk = SecretKey::random();
    let pk = sk.public_key();
    println!(
        "New Unpublished AppendOnlyData at: {:?}",
        mock.unpublished_append_only_put(&pk, &sk, &vec![])
    );
}

#[test]
fn test_unpublished_append_only_get() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk = SecretKey::random();
    let pk = sk.public_key();
    let data = vec![1, 2, 3];
    println!(
        "New Unpublished AppendOnlyData at: {:?}",
        mock.unpublished_append_only_put(&pk, &sk, &data)
    );

    let curr_data = mock.unpublished_append_only_get(&pk, &sk, Some(0));

    println!(
        "Current data at Unpublished AppendOnlyData at: {:?}",
        curr_data
    );

    assert_eq!(data, curr_data);
}

#[test]
fn test_allocate_test_coins() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();

    let balance = "2.345678912";
    mock.allocate_test_coins(&pk_to, balance);
    let current_balance = unwrap!(mock.get_balance_from_pk(&pk_to, &sk_to));
    println!("Current balance: {}", current_balance);
    assert_eq!(balance, &current_balance);
}

#[test]
fn test_create_balance() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk = SecretKey::random();
    let pk = sk.public_key();

    let balance = "2.345678912";
    mock.allocate_test_coins(&pk, balance);

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();
    println!(
        "New CoinBalance at: {:?}",
        mock.create_balance(&pk, &sk, &pk_to, "1.234567891")
    );
}

#[test]
fn test_check_balance() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk = SecretKey::random();
    let pk = sk.public_key();

    let balance = "2.3";
    mock.allocate_test_coins(&pk, balance);
    let current_balance = unwrap!(mock.get_balance_from_pk(&pk, &sk));
    println!("Current balance: {}", current_balance);
    assert_eq!(balance, &current_balance);

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();
    let preload = "1.234567891";
    println!(
        "New CoinBalance at: {:?}",
        mock.create_balance(&pk, &sk, &pk_to, preload)
    );
    let current_balance = unwrap!(mock.get_balance_from_pk(&pk_to, &sk_to));
    println!("Current balance: {}", current_balance);
    assert_eq!(preload, &current_balance);

    let current_balance = unwrap!(mock.get_balance_from_pk(&pk, &sk));
    println!("Current balance: {}", current_balance);
    assert_eq!(
        "1.065432109", /* == 2.3 - 1.234567891*/
        &current_balance
    );
}

#[test]
fn test_safecoin_transfer() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk1 = SecretKey::random();
    let pk1 = sk1.public_key();

    let sk2 = SecretKey::random();
    let pk2 = sk2.public_key();

    let balance1 = "2.5";
    let balance2 = "5.7";
    println!(
        "Allocate testcoins in new CoinBalance 1 at: {:?}",
        mock.allocate_test_coins(&pk1, balance1)
    );

    println!(
        "Allocate testcoins in new CoinBalance 2 at: {:?}",
        mock.allocate_test_coins(&pk2, balance2)
    );

    let curr_balance1 = unwrap!(mock.get_balance_from_pk(&pk1, &sk1));
    let curr_balance2 = unwrap!(mock.get_balance_from_pk(&pk2, &sk2));
    println!(
        "Current balances before TX: {} and {}",
        curr_balance1, curr_balance2
    );

    assert_eq!(balance1, curr_balance1);
    assert_eq!(balance2, curr_balance2);

    let tx_id = Uuid::new_v4();
    println!("UUID {}", tx_id);

    let _ = mock.safecoin_transfer(&pk1, &sk1, &pk2, &tx_id, "1.4");
    println!(
        "Current TX state: {}",
        mock.get_transaction(&tx_id, &pk2, &sk2)
    );

    let curr_balance1 = unwrap!(mock.get_balance_from_pk(&pk1, &sk1));
    let curr_balance2 = unwrap!(mock.get_balance_from_pk(&pk2, &sk2));
    println!(
        "Current balances after TX: {} and {}",
        curr_balance1, curr_balance2
    );

    assert_eq!(curr_balance1, "1.1");
    assert_eq!(curr_balance2, "7.1");
}
