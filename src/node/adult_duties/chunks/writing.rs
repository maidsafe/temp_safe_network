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
use safe_nd::{BlobWrite, MsgEnvelope, Write};
use serde::Serialize;

pub(super) struct Writing {
    write: Write,
    msg: MsgEnvelope,
}

impl Writing {
    pub fn new(write: Write, msg: MsgEnvelope) -> Self {
        Self { write, msg }
    }

    pub fn get_result(&self, storage: &mut ChunkStorage) -> Option<NodeCmd> {
        use Write::*;
        match &self.write {
            Blob(write) => self.blob(write, storage),
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

    fn blob(&self, write: &BlobWrite, storage: &mut ChunkStorage) -> Option<NodeCmd> {
        use BlobWrite::*;
        match write {
            New(data) => {
                if self.verify(&self.msg) {
                    storage.store(&data, self.msg.id(), self.msg.origin)
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
                    storage.delete(*address, self.msg.id(), self.msg.origin)
                } else {
                    error!("Accumulated signature is invalid!");
                    None
                }
            }
        }
    }
}
