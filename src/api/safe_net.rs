// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::errors::ResultReturn;
use safe_nd::XorName;
use safe_nd::{MDataValue, SeqMutableData};
use std::collections::BTreeMap;
use threshold_crypto::{PublicKey, SecretKey};

pub type AppendOnlyDataRawData = (Vec<u8>, Vec<u8>);

pub trait SafeApp {
    fn new() -> Self;

    fn connect(&mut self, _app_id: &str, _auth_credentials: Option<&str>) -> ResultReturn<()>;

    fn create_balance(
        &mut self,
        from_sk: Option<SecretKey>,
        new_balance_owner: PublicKey,
        amount: &str,
    ) -> ResultReturn<XorName>;

    fn allocate_test_coins(&mut self, to_pk: PublicKey, amount: &str) -> ResultReturn<XorName>;

    fn get_balance_from_sk(&self, sk: SecretKey) -> ResultReturn<String>;

    fn safecoin_transfer_to_xorname(
        &mut self,
        from_sk: SecretKey,
        to_xorname: XorName,
        tx_id: u64,
        amount: &str,
    ) -> ResultReturn<u64>;

    fn safecoin_transfer_to_pk(
        &mut self,
        from_sk: SecretKey,
        to_pk: PublicKey,
        tx_id: u64,
        amount: &str,
    ) -> ResultReturn<u64>;

    fn get_transaction(&self, tx_id: u64, pk: PublicKey, _sk: SecretKey) -> ResultReturn<String>;

    fn files_put_published_immutable(&mut self, data: &[u8]) -> ResultReturn<XorName>;

    fn files_get_published_immutable(&self, xorname: XorName) -> ResultReturn<Vec<u8>>;

    fn put_seq_append_only_data(
        &mut self,
        data: Vec<(Vec<u8>, Vec<u8>)>,
        name: Option<XorName>,
        _tag: u64,
        _permissions: Option<String>,
    ) -> ResultReturn<XorName>;

    fn append_seq_append_only_data(
        &mut self,
        data: Vec<(Vec<u8>, Vec<u8>)>,
        _new_version: u64,
        name: XorName,
        _tag: u64,
    ) -> ResultReturn<u64>;

    fn get_latest_seq_append_only_data(
        &self,
        name: XorName,
        _tag: u64,
    ) -> ResultReturn<(u64, AppendOnlyDataRawData)>;

    fn get_current_seq_append_only_data_version(
        &self,
        name: XorName,
        _tag: u64,
    ) -> ResultReturn<u64>;

    fn get_seq_append_only_data(
        &self,
        name: XorName,
        tag: u64,
        version: u64,
    ) -> ResultReturn<AppendOnlyDataRawData>;

    fn put_seq_mutable_data(
        &mut self,
        name: Option<XorName>,
        _tag: u64,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> ResultReturn<XorName>;

    fn get_seq_mdata(&self, xorname: XorName, _tag: u64) -> ResultReturn<SeqMutableData>;

    fn seq_mutable_data_insert(
        &mut self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
        value: &[u8],
    ) -> ResultReturn<()>;

    fn mutable_data_delete(
        &mut self,
        _xorname: &XorName,
        _tag: u64,
        _key: &[u8],
    ) -> ResultReturn<()>;

    fn seq_mutable_data_get_value(
        &mut self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
    ) -> ResultReturn<MDataValue>;

    fn list_seq_mdata_entries(
        &self,
        xorurl: &str,
        tag: u64,
    ) -> ResultReturn<BTreeMap<Vec<u8>, MDataValue>>;

    fn seq_mutable_data_update(
        &self,
        _xorurl: &str,
        _type_tag: u64,
        _key: &[u8],
        _value: &[u8],
        _version: u64,
    ) -> ResultReturn<()>;
}
