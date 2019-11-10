// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::api::fetch::Range;
use crate::Result;
use async_trait::async_trait;
use safe_nd::{Coins, MDataSeqValue, SeqMutableData, Transaction, TransactionId, XorName};
use std::collections::BTreeMap;
use threshold_crypto::{PublicKey, SecretKey};

pub type AppendOnlyDataRawData = (Vec<u8>, Vec<u8>);

#[async_trait]
pub trait SafeApp {
    fn new() -> Self;

    fn connect(&mut self, app_id: &str, auth_credentials: Option<&str>) -> Result<()>;

    async fn create_balance(
        &mut self,
        from_sk: Option<SecretKey>,
        new_balance_owner: PublicKey,
        amount: Coins,
    ) -> Result<XorName>;

    async fn allocate_test_coins(&mut self, owner_sk: SecretKey, amount: Coins) -> Result<XorName>;

    async fn get_balance_from_sk(&self, sk: SecretKey) -> Result<Coins>;

    async fn safecoin_transfer_to_xorname(
        &mut self,
        from_sk: Option<SecretKey>,
        to_xorname: XorName,
        tx_id: TransactionId,
        amount: Coins,
    ) -> Result<Transaction>;

    async fn safecoin_transfer_to_pk(
        &mut self,
        from_sk: Option<SecretKey>,
        to_pk: PublicKey,
        tx_id: TransactionId,
        amount: Coins,
    ) -> Result<Transaction>;

    async fn get_transaction(&self, tx_id: u64, pk: PublicKey, sk: SecretKey) -> Result<String>;

    async fn files_put_published_immutable(
        &mut self,
        data: &[u8],
        dry_run: bool,
    ) -> Result<XorName>;

    async fn files_get_published_immutable(
        &self,
        xorname: XorName,
        range: Range,
    ) -> Result<Vec<u8>>;

    async fn put_seq_append_only_data(
        &mut self,
        data: Vec<(Vec<u8>, Vec<u8>)>,
        name: Option<XorName>,
        tag: u64,
        permissions: Option<String>,
    ) -> Result<XorName>;

    async fn append_seq_append_only_data(
        &mut self,
        data: Vec<(Vec<u8>, Vec<u8>)>,
        new_version: u64,
        name: XorName,
        tag: u64,
    ) -> Result<u64>;

    async fn get_latest_seq_append_only_data(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<(u64, AppendOnlyDataRawData)>;

    async fn get_current_seq_append_only_data_version(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<u64>;

    async fn get_seq_append_only_data(
        &self,
        name: XorName,
        tag: u64,
        version: u64,
    ) -> Result<AppendOnlyDataRawData>;

    async fn put_seq_mutable_data(
        &mut self,
        name: Option<XorName>,
        tag: u64,
        // data: Option<String>,
        permissions: Option<String>,
    ) -> Result<XorName>;

    async fn get_seq_mdata(&self, name: XorName, tag: u64) -> Result<SeqMutableData>;

    async fn seq_mutable_data_insert(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
    ) -> Result<()>;

    async fn seq_mutable_data_get_value(
        &self,
        name: XorName,
        tag: u64,
        key: &[u8],
    ) -> Result<MDataSeqValue>;

    async fn list_seq_mdata_entries(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, MDataSeqValue>>;

    async fn seq_mutable_data_update(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
        version: u64,
    ) -> Result<()>;
}
