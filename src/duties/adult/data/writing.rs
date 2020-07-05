// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::chunk_storage::ChunkStorage;
use crate::{cmd::AdultCmd, utils};
use log::error;
use routing::SrcLocation;
use safe_nd::{BlobWrite, MessageId, PublicId, Request, Write};
use serde::Serialize;
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

    pub fn get_result(&self, storage: &mut ChunkStorage) -> Option<AdultCmd> {
        use Write::*;
        match &self.write {
            Blob(write) => self.blob(write, storage),
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

    fn blob(&self, write: &BlobWrite, storage: &mut ChunkStorage) -> Option<AdultCmd> {
        use BlobWrite::*;
        // Since the requester is a node, this message was sent by the data handlers to us
        // as a single data handler, implying that we're a data holder where the chunk is
        // stored.
        if !self.src.is_section() {
            return None;
        }
        match write {
            New(data) => {
                if self.verify(&self.request) {
                    storage.store(
                        self.src,
                        &data,
                        &self.requester,
                        self.message_id,
                        self.accumulated_signature.as_ref(),
                        self.request.clone(),
                    )
                } else {
                    error!(
                        "Accumulated signature for {:?} is invalid!",
                        &self.message_id
                    );
                    None
                }
            }
            DeletePrivate(address) => {
                if self.verify(&address) {
                    storage.delete(
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
            }
        }
    }
}
