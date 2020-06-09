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

#[async_trait]
pub trait SafeApp {
    fn new() -> Self;

    async fn connect(&mut self, app_id: &str, auth_credentials: Option<&str>) -> Result<()>;

    // === Coins operations ===
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

    // === ImmutableData operations ===
    async fn put_published_immutable(&mut self, data: &[u8], dry_run: bool) -> Result<XorName>;

    async fn get_published_immutable(&self, xorname: XorName, range: Range) -> Result<Vec<u8>>;

    // === MutableData operations ===
    async fn put_mdata(
        &mut self,
        name: Option<XorName>,
        tag: u64,
        // data: Option<String>,
        permissions: Option<String>,
    ) -> Result<XorName>;

    async fn get_mdata(&self, name: XorName, tag: u64) -> Result<SeqMutableData>;

    async fn mdata_insert(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
    ) -> Result<()>;

    async fn mdata_get_value(&self, name: XorName, tag: u64, key: &[u8]) -> Result<MDataSeqValue>;

    async fn mdata_list_entries(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, MDataSeqValue>>;

    async fn mdata_update(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
        version: u64,
    ) -> Result<()>;

    // === Sequence data operations ===
    async fn store_sequence_data(
        &mut self,
        data: &[u8],
        name: Option<XorName>,
        tag: u64,
        permissions: Option<String>,
    ) -> Result<XorName>;

    async fn sequence_get_last_entry(&self, name: XorName, tag: u64) -> Result<(u64, Vec<u8>)>;

    async fn sequence_get_entry(&self, name: XorName, tag: u64, index: u64) -> Result<Vec<u8>>;

    async fn sequence_append(&mut self, data: &[u8], name: XorName, tag: u64) -> Result<()>;
}
