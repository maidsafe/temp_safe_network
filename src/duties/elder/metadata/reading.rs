// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    blob_register::BlobRegister, elder_stores::ElderStores, map_storage::MapStorage,
    sequence_storage::SequenceStorage,
};
use crate::action::Action;
use routing::SrcLocation;
use safe_nd::{BlobRead, MapRead, MessageId, PublicId, Read, Request, SequenceRead};
use threshold_crypto::{PublicKey, Signature};

pub(super) struct Reading {
    request: Request,
    src: SrcLocation,
    requester: PublicId,
    read: Read,
    message_id: MessageId,
    accumulated_signature: Option<Signature>,
    public_key: Option<PublicKey>,
}

impl Reading {
    pub fn new(
        read: Read,
        src: SrcLocation,
        requester: PublicId,
        request: Request,
        message_id: MessageId,
        accumulated_signature: Option<Signature>,
        public_key: Option<PublicKey>,
    ) -> Self {
        Self {
            request,
            src,
            requester,
            read,
            message_id,
            accumulated_signature,
            public_key,
        }
    }

    pub fn get_result(&self, stores: &ElderStores) -> Option<Action> {
        use Read::*;
        match &self.read {
            Blob(read) => self.blob(read, stores.blob_register()),
            Map(read) => self.map(read, stores.map_storage()),
            Sequence(read) => self.sequence(read, stores.sequence_storage()),
            _ => None,
        }
    }

    fn blob(&self, read: &BlobRead, register: &BlobRegister) -> Option<Action> {
        use BlobRead::*;
        match read {
            Get(address) => register.get(
                &self.requester,
                *address,
                self.message_id,
                self.request.clone(),
            ),
        }
    }

    fn map(&self, read: &MapRead, storage: &MapStorage) -> Option<Action> {
        storage.read(self.requester.clone(), read, self.message_id)
    }

    fn sequence(&self, read: &SequenceRead, storage: &SequenceStorage) -> Option<Action> {
        storage.read(self.requester.clone(), read, self.message_id)
    }
}
