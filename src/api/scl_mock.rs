// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::xorurl::{create_random_xorname, xorurl_to_xorname};
use crate::api::helpers::{parse_hex, vec_to_hex, xorname_from_pk, xorname_to_hex};
use log::debug;
use safe_nd::{MDataValue, XorName};

use safecoin::{Coins, NanoCoins};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::io::Write;
use std::str::FromStr;
use std::{fs, str};
pub use threshold_crypto::{PublicKey as PublicKeyMock, SecretKey as SecretKeyMock};
use unwrap::unwrap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct CoinBalance {
    owner: PublicKeyMock,
    value: String,
}

type AppendOnlyDataMock = Vec<(Vec<u8>, Vec<u8>)>;
type TxStatusList = BTreeMap<String, String>;
type XorNameStr = String;
type SeqMutableDataMock = BTreeMap<String, MDataValue>;

static MOCK_FILE: &str = "./mock_data.txt";

#[derive(Default, Serialize, Deserialize)]
struct MockData {
    coin_balances: BTreeMap<XorNameStr, CoinBalance>,
    txs: BTreeMap<XorNameStr, TxStatusList>, // keep track of TX status per tx ID, per xorname
    unpublished_append_only: BTreeMap<XorNameStr, AppendOnlyDataMock>, // keep a versioned map of data per xorname
    published_seq_append_only: BTreeMap<XorNameStr, AppendOnlyDataMock>, // keep a versioned map of data per xorname
    mutable_data: BTreeMap<XorNameStr, SeqMutableDataMock>,
    published_immutable_data: BTreeMap<XorNameStr, Vec<u8>>,
}

pub struct SafeApp {
    mock_data: MockData,
}

/// Writes the mock data onto the mock file
impl Drop for SafeApp {
    fn drop(&mut self) {
        let serialised = unwrap!(serde_json::to_string(&self.mock_data));
        debug!("serialised = {}", serialised);

        let mut file = unwrap!(fs::File::create(&MOCK_FILE));
        unwrap!(file.write(serialised.as_bytes()));
    }
}

impl SafeApp {
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

    pub fn connect(&mut self, _app_id: &str, _auth_credentials: &str) -> Result<(), String> {
        debug!("Using mock so there is no connection to network");
        Ok(())
    }

    pub fn create_balance(
        &mut self,
        from_pk: &PublicKeyMock,
        from_sk: &SecretKeyMock,
        new_balance_owner: &PublicKeyMock,
        amount: &str,
    ) -> Result<XorName, &str> {
        let from_xorname = xorname_from_pk(from_pk);
        let from_balance = unwrap!(Coins::from_str(&unwrap!(
            self.get_balance_from_xorname(&from_xorname, from_sk)
        )));
        let from_nano_balance = unwrap!(NanoCoins::try_from(from_balance));
        let amount_coin = unwrap!(Coins::from_str(amount));
        let amount_nano = unwrap!(NanoCoins::try_from(amount_coin));
        if from_nano_balance.num() < amount_nano.num() {
            return Err("NotEnoughBalance");
        }
        let from_new_amount = unwrap!(NanoCoins::new(from_nano_balance.num() - amount_nano.num()));
        self.mock_data.coin_balances.insert(
            xorname_to_hex(&from_xorname),
            CoinBalance {
                owner: (*from_pk),
                value: Coins::from(from_new_amount).to_string(),
            },
        );

        let to_xorname = xorname_from_pk(new_balance_owner);
        self.mock_data.coin_balances.insert(
            xorname_to_hex(&to_xorname),
            CoinBalance {
                owner: (*new_balance_owner),
                value: amount.to_string(),
            },
        );

        Ok(to_xorname)
    }

    pub fn allocate_test_coins(&mut self, to_pk: &PublicKeyMock, amount: &str) -> XorName {
        let xorname = xorname_from_pk(to_pk);
        self.mock_data.coin_balances.insert(
            xorname_to_hex(&xorname),
            CoinBalance {
                owner: (*to_pk),
                value: amount.to_string(),
            },
        );

        xorname
    }

    pub fn get_balance_from_pk(
        &self,
        pk: &PublicKeyMock,
        sk: &SecretKeyMock,
    ) -> Result<String, &str> {
        let xorname = xorname_from_pk(pk);
        self.get_balance_from_xorname(&xorname, &sk)
    }

    pub fn get_balance_from_xorname(
        &self,
        xorname: &XorName,
        _sk: &SecretKeyMock,
    ) -> Result<String, &str> {
        match &self.mock_data.coin_balances.get(&xorname_to_hex(&xorname)) {
            None => Err("CoinBalance data not found"),
            Some(coin_balance) => Ok(coin_balance
                .value
                .to_string()
                .replace("Coins(", "")
                .replace(")", "")),
        }
    }

    pub fn fetch_pk_from_xorname(&self, xorname: &XorName) -> Result<PublicKeyMock, &str> {
        match &self.mock_data.coin_balances.get(&xorname_to_hex(&xorname)) {
            None => Err("CoinBalance data not found"),
            Some(coin_balance) => Ok(coin_balance.owner),
        }
    }

    pub fn safecoin_transfer(
        &mut self,
        from_pk: &PublicKeyMock,
        from_sk: &SecretKeyMock,
        to_pk: &PublicKeyMock,
        tx_id: &Uuid,
        amount: &str,
    ) -> Result<Uuid, &str> {
        let to_xorname = xorname_from_pk(to_pk);
        let from_xorname = xorname_from_pk(from_pk);

        let the_tx_id = *tx_id;
        // generate TX in destination section (to_pk)
        let mut txs_for_xorname = match self.mock_data.txs.get(&xorname_to_hex(&to_xorname)) {
            Some(txs) => txs.clone(),
            None => BTreeMap::new(),
        };
        txs_for_xorname.insert(
            tx_id.to_string(),
            format!("Success({})", amount).to_string(),
        );
        self.mock_data
            .txs
            .insert(xorname_to_hex(&to_xorname), txs_for_xorname);

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
            xorname_to_hex(&from_xorname),
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
            xorname_to_hex(&to_xorname),
            CoinBalance {
                owner: (*to_pk),
                value: Coins::from(to_new_amount).to_string(),
            },
        );

        Ok(the_tx_id)
    }

    #[allow(dead_code)]
    pub fn get_transaction(&self, tx_id: &Uuid, pk: &PublicKeyMock, _sk: &SecretKeyMock) -> String {
        let xorname = xorname_from_pk(pk);
        let txs_for_xorname = &self.mock_data.txs[&xorname_to_hex(&xorname)];
        let tx_state = unwrap!(txs_for_xorname.get(&tx_id.to_string()));
        tx_state.to_string()
    }

    pub fn put_published_immutable(&mut self, data: &[u8]) -> Result<XorName, String> {
        let xorname = create_random_xorname();
        // TODO: hash to get xorname.
        self.mock_data
            .published_immutable_data
            .insert(xorname_to_hex(&xorname), data.to_vec());

        Ok(xorname)
    }

    pub fn get_published_immutable(&mut self, xorname: XorName) -> Result<Vec<u8>, String> {
        let data = match self
            .mock_data
            .published_immutable_data
            .get(&xorname_to_hex(&xorname))
        {
            Some(data) => data.clone(),
            None => return Err("No immutable data found at this address".to_string()),
        };

        Ok(data)
    }

    #[allow(dead_code)]
    pub fn unpublished_append_only_put(
        &mut self,
        pk: &PublicKeyMock,
        _sk: &SecretKeyMock,
        data: &[u8],
    ) -> XorName {
        let xorname = xorname_from_pk(pk);

        //TODO: Convert to updated AppendOnlyDataMock Vec setup

        // let mut unpublished_append_only = match self
        //     .mock_data
        //     .unpublished_append_only
        //     .get(&xorname_to_hex(&xorname))
        // {
        //     Some(uao) => uao.clone(),
        //     None => BTreeMap::new(),
        // };
        // unpublished_append_only.insert(unpublished_append_only.len(), data.to_vec());
        // self.mock_data
        //     .unpublished_append_only
        //     .insert(xorname_to_hex(&xorname), unpublished_append_only);

        xorname
    }

    #[allow(dead_code)]
    pub fn unpublished_append_only_get(
        &self,
        pk: &PublicKeyMock,
        _sk: &SecretKeyMock,
        version: Option<usize>,
    ) -> Vec<u8> {
        let xorname = xorname_from_pk(pk);

        //TODO: Convert to updated AppendOnlyDataMock Vec setup

        // let unpublished_append_only =
        //     &self.mock_data.unpublished_append_only[&xorname_to_hex(&xorname)];
        // let data = match version {
        //     Some(version) => unwrap!(unpublished_append_only.get(&version)),
        //     None => {
        //         unwrap!(unpublished_append_only.get(&self.mock_data.unpublished_append_only.len()))
        //     }
        // };

        // data.to_vec()
        b"placeholder".to_vec()
    }

    pub fn put_seq_appendable_data(
        &mut self,
        data: AppendOnlyDataMock, // Vec( Key, Value )
        name: Option<XorName>,
        _tag: u64,
        _permissions: Option<String>,
    ) -> Result<XorName, String> {
        let xorname = name.unwrap_or_else(|| create_random_xorname());

        self.mock_data
            .published_seq_append_only
            .insert(xorname_to_hex(&xorname), data);

        Ok(xorname)
    }

    pub fn put_seq_mutable_data(
        &mut self,
        name: Option<XorName>,
        _tag: u64,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> Result<XorName, String> {
        let xorname = name.unwrap_or_else(|| create_random_xorname());

        let seq_md = match self.mock_data.mutable_data.get(&xorname_to_hex(&xorname)) {
            Some(uao) => uao.clone(),
            None => BTreeMap::new(),
        };

        self.mock_data
            .mutable_data
            .insert(xorname_to_hex(&xorname), seq_md);

        Ok(xorname)
    }

    fn get_seq_mdata(&self, xorname: &XorName, _tag: u64) -> Result<SeqMutableDataMock, String> {
        debug!(
            "attempting to locate scl mock mdata: {:?}",
            &xorname_to_hex(&xorname)
        );

        match self.mock_data.mutable_data.get(&xorname_to_hex(&xorname)) {
            Some(seq_md) => Ok(seq_md.clone()),
            None => Err("SeqMutableDataNotFound".to_string()),
        }
    }

    pub fn seq_mutable_data_insert(
        &mut self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
        value: &[u8],
    ) -> Result<(), String> {
        let xorname = xorurl_to_xorname(xorurl)?;
        let mut seq_md = self.get_seq_mdata(&xorname, tag)?;

        seq_md.insert(
            vec_to_hex(key.to_vec()),
            MDataValue {
                data: value.to_vec(),
                version: 0,
            },
        );

        self.mock_data
            .mutable_data
            .insert(xorname_to_hex(&xorname), seq_md);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn mutable_data_delete(&mut self, _xorname: &XorName, _tag: u64, _key: &[u8]) {}

    pub fn seq_mutable_data_get_value(
        &mut self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
    ) -> Result<MDataValue, String> {
        let xorname = xorurl_to_xorname(xorurl)?;
        let seq_md = self.get_seq_mdata(&xorname, tag)?;
        match seq_md.get(&vec_to_hex(key.to_vec())) {
            Some(value) => Ok(value.clone()),
            None => Err("EntryNotFound".to_string()),
        }
    }

    pub fn list_seq_mdata_entries(
        &self,
        xorurl: &str,
        tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, MDataValue>, String> {
        debug!("Listing seq_mdata_entries for: {}", xorurl);
        let xorname = xorurl_to_xorname(xorurl)?;
        let seq_md = self.get_seq_mdata(&xorname, tag)?;

        let mut res = BTreeMap::new();
        seq_md.iter().for_each(|elem| {
            res.insert(parse_hex(elem.0), elem.1.clone());
        });

        Ok(res)
    }

    //TODO: Replace with real mock code
    #[allow(dead_code)]
    pub fn seq_mutable_data_update(
        &self,
        _xorurl: &str,
        _type_tag: u64,
        _key: &[u8],
        _value: &[u8],
        _version: u64,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn test_unpublished_append_only_put() {
    use self::SafeApp;
    use threshold_crypto::SecretKey as SecretKeyMock;

    let mut mock = SafeApp::new();

    let sk = SecretKeyMock::random();
    let pk = sk.public_key();
    println!(
        "New Unpublished AppendOnlyData at: {:?}",
        mock.unpublished_append_only_put(&pk, &sk, &vec![])
    );
}

#[test]
fn test_unpublished_append_only_get() {
    use self::SafeApp;
    use threshold_crypto::SecretKey as SecretKeyMock;

    let mut mock = SafeApp::new();

    let sk = SecretKeyMock::random();
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
    use self::SafeApp;
    use threshold_crypto::SecretKey as SecretKeyMock;

    let mut mock = SafeApp::new();

    let sk_to = SecretKeyMock::random();
    let pk_to = sk_to.public_key();

    let balance = "2.345678912";
    mock.allocate_test_coins(&pk_to, balance);
    let current_balance = unwrap!(mock.get_balance_from_pk(&pk_to, &sk_to));
    println!("Current balance: {}", current_balance);
    assert_eq!(balance, &current_balance);
}

#[test]
fn test_create_balance() {
    use self::SafeApp;
    use threshold_crypto::SecretKey as SecretKeyMock;

    let mut mock = SafeApp::new();

    let sk = SecretKeyMock::random();
    let pk = sk.public_key();

    let balance = "2.345678912";
    mock.allocate_test_coins(&pk, balance);

    let sk_to = SecretKeyMock::random();
    let pk_to = sk_to.public_key();
    println!(
        "New CoinBalance at: {:?}",
        mock.create_balance(&pk, &sk, &pk_to, "1.234567891")
    );
}

#[test]
fn test_check_balance() {
    use self::SafeApp;
    use threshold_crypto::SecretKey as SecretKeyMock;

    let mut mock = SafeApp::new();

    let sk = SecretKeyMock::random();
    let pk = sk.public_key();

    let balance = "2.3";
    mock.allocate_test_coins(&pk, balance);
    let current_balance = unwrap!(mock.get_balance_from_pk(&pk, &sk));
    println!("Current balance: {}", current_balance);
    assert_eq!(balance, &current_balance);

    let sk_to = SecretKeyMock::random();
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
    use self::SafeApp;
    use threshold_crypto::SecretKey as SecretKeyMock;

    let mut mock = SafeApp::new();

    let sk1 = SecretKeyMock::random();
    let pk1 = sk1.public_key();

    let sk2 = SecretKeyMock::random();
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
