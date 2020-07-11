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
use safe_nd::{AccountWrite, BlobWrite, DataCmd, MapWrite, MessageId, MsgEnvelope, SequenceWrite};
use threshold_crypto::{PublicKey, Signature};

pub(super) struct Writing {
    cmd: DataCmd,
    msg: MsgEnvelope,
}

impl Writing {
    pub fn new(cmd: DataCmd, msg: MsgEnveloper) -> Self {
        Self { cmd, msg }
    }

    pub fn get_result(&mut self, stores: &mut ElderStores) -> Option<NodeCmd> {
        use DataCmd::*;
        match self.cmd.clone() {
            Blob(write) => self.blob(write, stores.blob_register_mut()),
            Map(write) => self.map(write, stores.map_storage_mut()),
            Sequence(write) => self.sequence(write, stores.sequence_storage_mut()),
            Account(write) => self.account(write, stores.account_storage_mut()),
        }
    }

    fn blob(&mut self, write: BlobWrite, register: &mut BlobRegister) -> Option<NodeCmd> {
        register.write(write, self.msg)
    }

    fn map(&mut self, write: MapWrite, storage: &mut MapStorage) -> Option<NodeCmd> {
        storage.write(write, self.msg)
    }

    fn sequence(&mut self, write: SequenceWrite, storage: &mut SequenceStorage) -> Option<NodeCmd> {
        storage.write(self.requester.clone(), write, self.message_id)
    }

    fn account(&mut self, write: AccountWrite, storage: &mut AccountStorage) -> Option<NodeCmd> {
        storage.write(self.requester.clone(), write, self.message_id)
    }
}
