// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::chunk_storage::ChunkStorage;
use crate::{action::Action, utils};
use log::error;
use routing::SrcLocation;
use safe_nd::{BlobRead, MessageId, PublicId, Read, Request};
use serde::Serialize;
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

    pub fn get_result(&self, storage: &ChunkStorage) -> Option<Action> {
        use Read::*;
        match &self.read {
            Blob(read) => self.blob(read, storage),
            _ => None,
        }
    }

    fn verify<T: Serialize>(&self, data: &T) -> bool {
        if let Some(sig) = self.accumulated_signature.as_ref() {
            match self.public_key {
                Some(key) => key.verify(sig, &utils::serialise(data)),
                None => false,
            }
        } else {
            false
        }
    }

    fn blob(&self, read: &BlobRead, storage: &ChunkStorage) -> Option<Action> {
        let BlobRead::Get(address) = read;
        if self.src.is_section() {
            // Since the requester is a node, this message was sent by the data handlers to us
            // as a single data handler, implying that we're a data holder where the chunk is
            // stored.
            if self.verify(&self.request) {
                storage.get(
                    self.src,
                    *address,
                    &self.requester,
                    self.message_id,
                    self.request.clone(),
                    self.accumulated_signature.as_ref(),
                )
            } else {
                error!("Accumulated signature is invalid!");
                None
            }
        } else if matches!(self.requester, PublicId::Node(_)) {
            if self.verify(&address) {
                storage.get(
                    self.src,
                    *address,
                    &self.requester,
                    self.message_id,
                    self.request.clone(),
                    self.accumulated_signature.as_ref(),
                )
            } else {
                error!("Accumulated signature is invalid!");
                None
            }
        } else {
            None
        }
    }
}
