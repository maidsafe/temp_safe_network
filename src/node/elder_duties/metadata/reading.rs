// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    account_storage::AccountStorage, blob_register::BlobRegister, elder_stores::ElderStores,
    map_storage::MapStorage, sequence_storage::SequenceStorage,
};
use crate::cmd::NodeCmd;
use routing::SrcLocation;
use safe_nd::{AccountRead, BlobRead, DataQuery, MapRead, MessageId, PublicId, SequenceRead};
use threshold_crypto::{PublicKey, Signature};

pub(super) struct Reading {
    _src: SrcLocation,
    requester: PublicId,
    read: DataQuery,
    message_id: MessageId,
    _accumulated_signature: Option<Signature>,
    _public_key: Option<PublicKey>,
}

impl Reading {
    pub fn new(
        query: DataQuery,
        _src: SrcLocation,
        requester: PublicId,
        message_id: MessageId,
        _accumulated_signature: Option<Signature>,
        _public_key: Option<PublicKey>,
    ) -> Self {
        Self {
            _src,
            requester,
            query,
            message_id,
            _accumulated_signature,
            _public_key,
        }
    }

    pub fn get_result(&self, stores: &ElderStores) -> Option<NodeCmd> {
        use DataQuery::*;
        match &self.query {
            Blob(read) => self.blob(read, stores.blob_register()),
            Map(read) => self.map(read, stores.map_storage()),
            Sequence(read) => self.sequence(read, stores.sequence_storage()),
            Account(read) => self.account(read, stores.account_storage()),
        }
    }

    fn blob(&self, read: &BlobRead, register: &BlobRegister) -> Option<NodeCmd> {
        register.read(self.requester.clone(), read, self.message_id)
    }

    fn map(&self, read: &MapRead, storage: &MapStorage) -> Option<NodeCmd> {
        storage.read(self.requester.clone(), read, self.message_id)
    }

    fn sequence(&self, read: &SequenceRead, storage: &SequenceStorage) -> Option<NodeCmd> {
        storage.read(self.requester.clone(), read, self.message_id)
    }

    fn account(&self, read: &AccountRead, storage: &AccountStorage) -> Option<NodeCmd> {
        storage.read(self.requester.clone(), read, self.message_id)
    }
}
