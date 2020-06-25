// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    common::parse_hex,
    fetch::Range,
    helpers::{parse_coins_amount, vec_to_hex, xorname_from_pk, xorname_to_hex},
    SafeApp,
};
use crate::{Error, Result};
use async_trait::async_trait;
use futures::lock::Mutex;
use lazy_static::lazy_static;
use log::{debug, trace};
use safe_nd::{
    Coins, MDataSeqValue, PublicKey as SafeNdPublicKey, SeqMutableData, Transaction, TransactionId,
    XorName,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::Write, str, sync::Arc};
use threshold_crypto::{PublicKey, SecretKey};
use tiny_keccak::sha3_256;

const FAKE_VAULT_FILE: &str = "./fake_vault_data.json";

#[derive(Debug, Serialize, Deserialize)]
struct SafeKey {
    owner: PublicKey,
    value: String,
}

type XorNameStr = String;
type SeqMutableDataFake = BTreeMap<String, MDataSeqValue>;
type SequenceDataFake = Vec<Vec<u8>>;

#[derive(Default, Serialize, Deserialize)]
struct FakeData {
    coin_balances: BTreeMap<XorNameStr, SafeKey>,
    public_sequence: BTreeMap<XorNameStr, SequenceDataFake>,
    private_sequence: BTreeMap<XorNameStr, SequenceDataFake>,
    mutable_data: BTreeMap<XorNameStr, SeqMutableDataFake>,
    public_immutable_data: BTreeMap<XorNameStr, Vec<u8>>,
}

lazy_static! {
    static ref FAKE_DATA_SINGLETON: Arc<Mutex<FakeData>> = {
        let fake_data = match fs::File::open(&FAKE_VAULT_FILE) {
            Ok(file) => {
                let deserialised: FakeData =
                    serde_json::from_reader(&file).expect("Failed to read fake vault DB file");
                deserialised
            }
            Err(error) => {
                debug!("Error reading mock file. {}", error.to_string());
                FakeData::default()
            }
        };
        Arc::new(Mutex::new(fake_data))
    };
}

#[derive(Default)]
pub struct SafeAppFake {
    fake_vault: Arc<Mutex<FakeData>>,
}

/// Writes the fake Vault data onto the file
impl Drop for SafeAppFake {
    fn drop(&mut self) {
        if Arc::strong_count(&self.fake_vault) <= 2 {
            // we are the last SafeAppFake instance going out of scope then,
            // the other ref is from FAKE_DATA_SINGLETON
            let fake_data: &FakeData = &futures::executor::block_on(self.fake_vault.lock());
            let serialised = serde_json::to_string(fake_data)
                .expect("Failed to serialised fake vault data to write on file");
            trace!("Writing serialised fake vault data = {}", serialised);

            let mut file =
                fs::File::create(&FAKE_VAULT_FILE).expect("Failed to create fake vault DB file");
            let _ = file
                .write(serialised.as_bytes())
                .expect("Failed to write fake vault DB file");
        }
    }
}

impl SafeAppFake {
    // private helpers
    async fn get_balance_from_xorname(&self, xorname: &XorName) -> Result<Coins> {
        match self
            .fake_vault
            .lock()
            .await
            .coin_balances
            .get(&xorname_to_hex(xorname))
        {
            None => Err(Error::ContentNotFound("SafeKey data not found".to_string())),
            Some(coin_balance) => parse_coins_amount(&coin_balance.value),
        }
    }

    async fn fetch_pk_from_xorname(&self, xorname: &XorName) -> Result<PublicKey> {
        match self
            .fake_vault
            .lock()
            .await
            .coin_balances
            .get(&xorname_to_hex(xorname))
        {
            None => Err(Error::ContentNotFound("SafeKey data not found".to_string())),
            Some(coin_balance) => Ok(coin_balance.owner),
        }
    }

    async fn substract_coins(&mut self, sk: SecretKey, amount: Coins) -> Result<()> {
        let from_balance = self.get_balance_from_sk(sk.clone()).await?;
        match from_balance.checked_sub(amount) {
            None => Err(Error::NotEnoughBalance(from_balance.to_string())),
            Some(new_balance_coins) => {
                let from_pk = sk.public_key();
                self.fake_vault.lock().await.coin_balances.insert(
                    xorname_to_hex(&xorname_from_pk(from_pk)),
                    SafeKey {
                        owner: from_pk,
                        value: new_balance_coins.to_string(),
                    },
                );
                Ok(())
            }
        }
    }
}

#[async_trait]
impl SafeApp for SafeAppFake {
    fn new() -> Self {
        Self {
            fake_vault: Arc::clone(&FAKE_DATA_SINGLETON),
        }
    }

    async fn connect(&mut self, _app_id: &str, _auth_credentials: Option<&str>) -> Result<()> {
        debug!("Using mock so there is no connection to network");
        Ok(())
    }

    // === Coins operations ===
    async fn create_balance(
        &mut self,
        from_sk: Option<SecretKey>,
        new_balance_owner: PublicKey,
        amount: Coins,
    ) -> Result<XorName> {
        if let Some(sk) = from_sk {
            // 1 nano is the creation cost
            let amount_with_cost = Coins::from_nano(amount.as_nano() + 1);
            self.substract_coins(sk, amount_with_cost).await?;
        };

        let to_xorname = xorname_from_pk(new_balance_owner);
        self.fake_vault.lock().await.coin_balances.insert(
            xorname_to_hex(&to_xorname),
            SafeKey {
                owner: new_balance_owner,
                value: amount.to_string(),
            },
        );

        Ok(to_xorname)
    }

    async fn allocate_test_coins(&mut self, owner_sk: SecretKey, amount: Coins) -> Result<XorName> {
        let to_pk = owner_sk.public_key();
        let xorname = xorname_from_pk(to_pk);
        self.fake_vault.lock().await.coin_balances.insert(
            xorname_to_hex(&xorname),
            SafeKey {
                owner: (to_pk),
                value: amount.to_string(),
            },
        );

        Ok(xorname)
    }

    async fn get_balance_from_sk(&self, sk: SecretKey) -> Result<Coins> {
        let pk = sk.public_key();
        let xorname = xorname_from_pk(pk);
        self.get_balance_from_xorname(&xorname).await
    }

    async fn safecoin_transfer_to_xorname(
        &mut self,
        from_sk: Option<SecretKey>,
        to_xorname: XorName,
        tx_id: TransactionId,
        amount: Coins,
    ) -> Result<Transaction> {
        if amount.as_nano() == 0 {
            return Err(Error::InvalidAmount(amount.to_string()));
        }

        if let Some(sk) = from_sk {
            // reduce balance from safecoin_transferer
            self.substract_coins(sk, amount).await?;
        }

        // credit destination
        let to_balance = self.get_balance_from_xorname(&to_xorname).await?;
        match to_balance.checked_add(amount) {
            None => Err(Error::Unexpected(
                "Failed to credit destination due to overflow...maybe a millionaire's problem?!"
                    .to_string(),
            )),
            Some(new_balance_coins) => {
                let safekey = SafeKey {
                    owner: self.fetch_pk_from_xorname(&to_xorname).await?,
                    value: new_balance_coins.to_string(),
                };

                self.fake_vault
                    .lock()
                    .await
                    .coin_balances
                    .insert(xorname_to_hex(&to_xorname), safekey);
                Ok(Transaction { id: tx_id, amount })
            }
        }
    }

    async fn safecoin_transfer_to_pk(
        &mut self,
        from_sk: Option<SecretKey>,
        to_pk: PublicKey,
        tx_id: TransactionId,
        amount: Coins,
    ) -> Result<Transaction> {
        let to_xorname = xorname_from_pk(to_pk);
        self.safecoin_transfer_to_xorname(from_sk, to_xorname, tx_id, amount)
            .await
    }

    // === ImmutableData operations ===
    async fn put_public_immutable(&mut self, data: &[u8], dry_run: bool) -> Result<XorName> {
        // We create a XorName based on a hash of the content, not a real one as
        // it doesn't apply self-encryption, but a unique one for our fake SCL
        let vec_hash = sha3_256(&data);
        let xorname = XorName(vec_hash);

        if !dry_run {
            self.fake_vault
                .lock()
                .await
                .public_immutable_data
                .insert(xorname_to_hex(&xorname), data.to_vec());
        }

        Ok(xorname)
    }

    async fn get_public_immutable(&self, xorname: XorName, range: Range) -> Result<Vec<u8>> {
        let data = match self
            .fake_vault
            .lock()
            .await
            .public_immutable_data
            .get(&xorname_to_hex(&xorname))
        {
            Some(data) => data.clone(),
            None => {
                return Err(Error::NetDataError(
                    "No ImmutableData found at this address".to_string(),
                ))
            }
        };

        let data = match range {
            Some((start, end)) => data
                [start.unwrap_or_default() as usize..end.unwrap_or(data.len() as u64) as usize]
                .to_vec(),
            None => data.to_vec(),
        };

        Ok(data)
    }

    // === MutableData operations ===
    async fn put_mdata(
        &mut self,
        name: Option<XorName>,
        _tag: u64,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> Result<XorName> {
        let xorname = name.unwrap_or_else(rand::random);
        let seq_md = match self
            .fake_vault
            .lock()
            .await
            .mutable_data
            .get(&xorname_to_hex(&xorname))
        {
            Some(uao) => uao.clone(),
            None => BTreeMap::new(),
        };

        self.fake_vault
            .lock()
            .await
            .mutable_data
            .insert(xorname_to_hex(&xorname), seq_md);

        Ok(xorname)
    }

    async fn get_mdata(&self, name: XorName, tag: u64) -> Result<SeqMutableData> {
        let xorname_hex = xorname_to_hex(&name);
        debug!("attempting to locate scl mock mdata: {}", xorname_hex);

        match self.fake_vault.lock().await.mutable_data.get(&xorname_hex) {
            Some(seq_md) => {
                let mut seq_md_with_vec: BTreeMap<Vec<u8>, MDataSeqValue> = BTreeMap::new();
                seq_md.iter().for_each(|(k, v)| {
                    seq_md_with_vec.insert(parse_hex(k), v.clone());
                });

                Ok(SeqMutableData::new_with_data(
                    name,
                    tag,
                    seq_md_with_vec,
                    BTreeMap::default(),
                    SafeNdPublicKey::Bls(SecretKey::random().public_key()),
                ))
            }
            None => Err(Error::ContentNotFound(format!(
                "Sequenced MutableData not found at Xor name: {}",
                xorname_hex
            ))),
        }
    }

    async fn mdata_insert(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        let seq_md = self.get_mdata(name, tag).await?;
        let mut data = seq_md.entries().clone();

        data.insert(
            key.to_vec(),
            MDataSeqValue {
                data: value.to_vec(),
                version: 0,
            },
        );

        let mut seq_md_with_str: BTreeMap<String, MDataSeqValue> = BTreeMap::new();
        data.iter().for_each(|(k, v)| {
            seq_md_with_str.insert(vec_to_hex(k.to_vec()), v.clone());
        });
        self.fake_vault
            .lock()
            .await
            .mutable_data
            .insert(xorname_to_hex(&name), seq_md_with_str);

        Ok(())
    }

    async fn mdata_get_value(&self, name: XorName, tag: u64, key: &[u8]) -> Result<MDataSeqValue> {
        let seq_md = self.get_mdata(name, tag).await?;
        match seq_md.get(&key.to_vec()) {
            Some(value) => Ok(value.clone()),
            None => Err(Error::EntryNotFound(format!(
                "Entry not found in Sequenced MutableData found at Xor name: {}",
                xorname_to_hex(&name)
            ))),
        }
    }

    async fn mdata_list_entries(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, MDataSeqValue>> {
        debug!("Listing seq_mdata_entries for: {}", name);
        let seq_md = self.get_mdata(name, tag).await?;
        let mut res = BTreeMap::new();
        seq_md.entries().iter().for_each(|elem| {
            res.insert(elem.0.clone(), elem.1.clone());
        });

        Ok(res)
    }

    async fn mdata_update(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
        _version: u64,
    ) -> Result<()> {
        let _ = self.mdata_get_value(name, tag, key).await;
        self.mdata_insert(name, tag, key, value).await
    }

    // === Sequence data operations ===
    async fn store_sequence_data(
        &mut self,
        data: &[u8],
        name: Option<XorName>,
        _tag: u64,
        _permissions: Option<String>,
        private: bool,
    ) -> Result<XorName> {
        let xorname = name.unwrap_or_else(rand::random);
        let xorname_hex = xorname_to_hex(&xorname);
        let initial_data = vec![data.to_vec()];
        if private {
            self.fake_vault
                .lock()
                .await
                .private_sequence
                .insert(xorname_hex, initial_data);
        } else {
            self.fake_vault
                .lock()
                .await
                .public_sequence
                .insert(xorname_hex, initial_data);
        }

        Ok(xorname)
    }

    async fn sequence_get_last_entry(
        &self,
        name: XorName,
        _tag: u64,
        private: bool,
    ) -> Result<(u64, Vec<u8>)> {
        let xorname_hex = xorname_to_hex(&name);
        debug!("Attempting to locate Sequence in scl mock: {}", xorname_hex);

        let mutex = self.fake_vault.lock().await;
        let seq = if private {
            mutex.private_sequence.get(&xorname_hex)
        } else {
            mutex.public_sequence.get(&xorname_hex)
        }
        .ok_or_else(|| {
            Error::ContentNotFound(format!("Sequence not found at Xor name: {}", xorname_hex))
        })?;

        if seq.is_empty() {
            Err(Error::EmptyContent(format!(
                "Empty Sequence found at Xor name {}",
                xorname_hex
            )))
        } else {
            let latest_index = seq.len() - 1;
            let last_entry = seq.get(latest_index).ok_or_else(|| {
                Error::Unexpected(format!(
                    "Failed to get latest entry from Sequence found at Xor name {}",
                    xorname_hex
                ))
            })?;
            Ok((latest_index as u64, last_entry.clone()))
        }
    }

    async fn sequence_get_entry(
        &self,
        name: XorName,
        _tag: u64,
        index: u64,
        private: bool,
    ) -> Result<Vec<u8>> {
        let xorname_hex = xorname_to_hex(&name);
        debug!("Attempting to locate Sequence in scl mock: {}", xorname_hex);

        let mutex = self.fake_vault.lock().await;
        let seq = if private {
            mutex.private_sequence.get(&xorname_hex)
        } else {
            mutex.public_sequence.get(&xorname_hex)
        }
        .ok_or_else(|| {
            Error::ContentNotFound(format!("Sequence not found at Xor name: {}", xorname_hex))
        })?;

        if seq.is_empty() {
            Err(Error::EmptyContent(format!(
                "Empty Sequence found at Xor name {}",
                xorname_hex
            )))
        } else {
            let last_entry = seq.get(index as usize).ok_or_else(|| {
                Error::VersionNotFound(format!(
                    "Failed to get entry (index: {}) from Sequence found at Xor name {}",
                    index, xorname_hex
                ))
            })?;
            Ok(last_entry.clone())
        }
    }

    async fn sequence_append(
        &mut self,
        data: &[u8],
        name: XorName,
        _tag: u64,
        private: bool,
    ) -> Result<()> {
        let xorname_hex = xorname_to_hex(&name);
        debug!(
            "Attempting to append to Sequence in scl mock: {}",
            xorname_hex
        );

        let mut mutex = self.fake_vault.lock().await;

        let seq = if private {
            mutex.private_sequence.get_mut(&xorname_hex)
        } else {
            mutex.public_sequence.get_mut(&xorname_hex)
        }
        .ok_or_else(|| {
            Error::ContentNotFound(format!("Sequence not found at Xor name: {}", xorname_hex))
        })?;

        seq.push(data.to_vec());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // Helper function to instantiate Coins form a string and handle any error
    fn coins_from_str(str: &str) -> Result<Coins> {
        Coins::from_str(str).map_err(|err| {
            Error::Unexpected(format!(
                "Failed to instantiate Coins from str '{}': {}",
                str, err
            ))
        })
    }

    #[tokio::test]
    async fn test_allocate_test_coins() -> Result<()> {
        use threshold_crypto::SecretKey;

        let mut mock = SafeAppFake::new();
        let sk_to = SecretKey::random();

        let balance = coins_from_str("2.345678912")?;
        mock.allocate_test_coins(sk_to.clone(), balance).await?;
        let current_balance = mock.get_balance_from_sk(sk_to).await?;
        assert_eq!(balance, current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_balance() -> Result<()> {
        use threshold_crypto::SecretKey;

        let mut mock = SafeAppFake::new();
        let sk = SecretKey::random();

        let balance = coins_from_str("2.345678912")?;
        mock.allocate_test_coins(sk.clone(), balance).await?;

        let sk_to = SecretKey::random();
        let pk_to = sk_to.public_key();
        assert!(mock
            .create_balance(Some(sk), pk_to, coins_from_str("1.234567891")?)
            .await
            .is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_check_balance() -> Result<()> {
        use threshold_crypto::SecretKey;

        let mut mock = SafeAppFake::new();
        let sk = SecretKey::random();

        let balance = coins_from_str("2.3")?;
        mock.allocate_test_coins(sk.clone(), balance).await?;
        let current_balance = mock.get_balance_from_sk(sk.clone()).await?;
        assert_eq!(balance, current_balance);

        let sk_to = SecretKey::random();
        let pk_to = sk_to.public_key();
        let preload = coins_from_str("1.234567891")?;
        mock.create_balance(Some(sk.clone()), pk_to, preload)
            .await?;
        let current_balance = mock.get_balance_from_sk(sk_to).await?;
        assert_eq!(preload, current_balance);

        let current_balance = mock.get_balance_from_sk(sk).await?;
        assert_eq!(
            coins_from_str("1.065432108")?, /* == 2.3 - 1.234567891 - 0.000000001 (creation cost) */
            current_balance
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_safecoin_transfer() -> Result<()> {
        use threshold_crypto::SecretKey;

        let mut mock = SafeAppFake::new();
        let sk1 = SecretKey::random();
        let sk2 = SecretKey::random();
        let pk2 = sk2.public_key();

        let balance1 = coins_from_str("2.5")?;
        let balance2 = coins_from_str("5.7")?;
        mock.allocate_test_coins(sk1.clone(), balance1).await?;
        mock.allocate_test_coins(sk2.clone(), balance2).await?;

        let curr_balance1 = mock.get_balance_from_sk(sk1.clone()).await?;
        let curr_balance2 = mock.get_balance_from_sk(sk2.clone()).await?;

        assert_eq!(balance1, curr_balance1);
        assert_eq!(balance2, curr_balance2);

        let tx_id = rand::random();
        let _ = mock
            .safecoin_transfer_to_xorname(
                Some(sk1.clone()),
                xorname_from_pk(pk2),
                tx_id,
                coins_from_str("1.4")?,
            )
            .await?;

        let curr_balance1 = mock.get_balance_from_sk(sk1).await?;
        let curr_balance2 = mock.get_balance_from_sk(sk2).await?;

        assert_eq!(curr_balance1, coins_from_str("1.1")?);
        assert_eq!(curr_balance2, coins_from_str("7.1")?);
        Ok(())
    }
}
