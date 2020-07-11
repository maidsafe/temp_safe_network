// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::chunk_storage::ChunkStorage;
use crate::{cmd::NodeCmd, utils};
use log::error;
use safe_nd::{BlobRead, MsgEnvelope, MsgSender, Read};
use serde::Serialize;

pub(super) struct Reading {
    read: Read,
    msg: MsgEnvelope,
}

impl Reading {
    pub fn new(read: Read, msg: MsgEnvelope) -> Self {
        Self { read, msg }
    }

    pub fn get_result(&self, storage: &ChunkStorage) -> Option<NodeCmd> {
        use Read::*;
        match &self.read {
            Blob(read) => self.blob(read, storage),
            _ => None,
        }
    }

    fn verify<T: Serialize>(&self, data: &T) -> bool {
        match self.msg.most_recent_sender() {
            MsgSender::Section { id, signature, .. } => {
                id.verify(signature, &utils::serialise(data))
            }
            _ => false,
        }
    }

    fn blob(&self, read: &BlobRead, storage: &ChunkStorage) -> Option<NodeCmd> {
        let BlobRead::Get(address) = read;
        if self.src.is_section() {
            if self.verify(&self.msg.message) {
                storage.get(*address, self.msg.id(), self.msg.origin)
            } else {
                error!("Accumulated signature is invalid!");
                None
            }
        // } else if matches!(self.requester, PublicId::Node(_)) {
        //     if self.verify(&address) {
        //         storage.get(
        //             self.src,
        //             *address,
        //             &self.requester,
        //             self.message_id,
        //             self.request.clone(),
        //             self.accumulated_signature.as_ref(),
        //         )
        //     } else {
        //         error!("Accumulated signature is invalid!");
        //         None
        //     }
        } else {
            None
        }
    }
}
