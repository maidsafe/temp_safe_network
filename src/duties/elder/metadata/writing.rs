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
use safe_nd::{BlobWrite, MapWrite, MessageId, PublicId, Request, SequenceWrite, Write};
use threshold_crypto::{PublicKey, Signature};

pub(super) struct Writing {
    request: Request,
    src: SrcLocation,
    requester: PublicId,
    write: Write,
    message_id: MessageId,
    accumulated_signature: Option<Signature>,
    public_key: Option<PublicKey>,
}

impl Writing {
    pub fn new(
        write: Write,
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
            write,
            message_id,
            accumulated_signature,
            public_key,
        }
    }

    pub fn get_result(&mut self, stores: &mut ElderStores) -> Option<Action> {
        use Write::*;
        match self.write.clone() {
            Blob(write) => self.blob(write, stores.blob_register_mut()),
            Map(write) => self.map(write, stores.map_storage_mut()),
            Sequence(write) => self.sequence(write, stores.sequence_storage_mut()),
            _ => None,
        }
    }

    fn blob(&mut self, write: BlobWrite, register: &mut BlobRegister) -> Option<Action> {
        use BlobWrite::*;
        match write {
            New(data) => register.store(
                self.requester.clone(),
                data,
                self.message_id,
                self.request.clone(),
            ),
            DeletePrivate(address) => register.delete(
                self.requester.clone(),
                address,
                self.message_id,
                self.request.clone(),
            ),
        }
    }

    fn map(&mut self, write: MapWrite, storage: &mut MapStorage) -> Option<Action> {
        storage.write(self.requester.clone(), write, self.message_id)
    }

    fn sequence(&mut self, write: SequenceWrite, storage: &mut SequenceStorage) -> Option<Action> {
        storage.write(self.requester.clone(), write, self.message_id)
    }
}
