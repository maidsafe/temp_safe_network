// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::msg_util::ElderMsgUtil;
use crate::{cmd::ConsensusAction, utils};
use log::trace;
use safe_nd::{
    Account, AccountRead, AccountWrite, BlobRead, BlobWrite, Cmd, DataCmd, DebitAgreementProof,
    Duty, Duty, ElderDuty, ElderDuty, Error as NdError, IData, IDataAddress, IDataKind, MData,
    MapRead, MapWrite, Message, MessageId, MsgEnvelope, MsgSender, NodeCmd, NodePublicId, PublicId,
    Read, SData, SDataAddress, SequenceRead, SequenceWrite, Write,
};
use std::fmt::{self, Display, Formatter};

#[derive(Clone)]
pub(crate) struct Validation {
    blobs: Blobs,
    maps: Maps,
    sequences: Sequences,
    accounts: Accounts,
}

impl Validation {
    pub fn new(msg_util: ElderMsgUtil) -> Self {
        Self {
            blobs: Blobs::new(msg_util.clone()),
            maps: Maps::new(msg_util.clone()),
            sequences: Sequences::new(msg_util.clone()),
            accounts: Accounts::new(msg_util),
        }
    }

    pub fn receive_msg(&mut self, msg: MsgEnvelope) {
        let message = msg.message;
        match &message {
            Message::Cmd {
                cmd: Cmd::Data { cmd, .. },
                ..
            } => self.initiate_write(cmd, msg),
            Message::Query {
                query: Query::Data { query, .. },
                ..
            } => self.initiate_read(query, msg),
            _ => return None,
        }
    }

    fn initiate_write(&mut self, cmd: DataCmd, msg: MsgEnvelope) -> Option<NodeCmd> {
        match cmd {
            DataCmd::Blob(_) => self.blobs.initiate_write(msg),
            DataCmd::Map(_) => self.maps.initiate_write(msg),
            DataCmd::Sequence(_) => self.sequences.initiate_write(msg),
            DataCmd::Account(_) => self.accounts.initiate_write(msg),
        }
    }

    fn initiate_read(&mut self, query: DataQuery, msg: MsgEnvelope) -> Option<NodeCmd> {
        match query {
            DataQuery::Blob(_) => self.blobs.initiate_read(msg),
            DataQuery::Map(_) => self.maps.initiate_read(msg),
            DataQuery::Sequence(_) => self.sequences.initiate_read(msg),
            DataQuery::Account(_) => self.accounts.initiate_read(msg),
        }
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Sequences {
    msg_util: ElderMsgUtil,
}

impl Sequences {
    pub fn new(msg_util: ElderMsgUtil) -> Self {
        Self { msg_util }
    }

    // client query
    pub fn initiate_read(&mut self, read: SequenceRead, msg: MsgEnvelope) -> Option<NodeCmd> {
        self.msg_util.wrap_forward(msg)
    }

    // on client request
    pub fn initiate_write(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        use SequenceWrite::*;
        match write {
            New(chunk) => self.initiate_creation(chunk, msg),
            Delete(address) => self.initiate_deletion(address, msg),
            SetPubPermissions { .. } | SetPrivPermissions { .. } | SetOwner { .. } | Edit(..) => {
                self.initiate_edit(msg)
            }
        }
    }

    // on client request
    fn initiate_creation(&mut self, chunk: SData, msg: MsgEnvelope) -> Option<NodeCmd> {
        // TODO - Should we replace this with a sequence.check_permission call in data_handler.
        // That would be more consistent, but on the other hand a check here stops spam earlier.
        if chunk.check_is_last_owner(msg.origin.id()).is_err() {
            trace!(
                "{}: {} attempted to store Sequence with invalid owners.",
                self,
                client
            );
            return self
                .msg_util
                .error(NdError::InvalidOwners, msg.id(), msg.origin);
        }
        self.msg_util.vote(msg)
    }

    // on client request
    fn initiate_deletion(&mut self, address: SDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        if address.is_pub() {
            return self
                .msg_util
                .error(NdError::InvalidOperation, msg.id(), msg.origin);
        }
        self.msg_util.vote(msg)
    }

    // on client request
    fn initiate_edit(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        self.msg_util.vote(msg)
    }

    fn extract_read(&self, msg: MsgEnvelope) -> Option<SequenceRead> {
        let write = match msg.message {
            Message::Query {
                query:
                    Query::Data {
                        query: DataQuery::Sequence(query),
                        ..
                    },
                ..
            } => Some(query),
            _ => return None,
        };
    }

    fn extract_write(&self, msg: MsgEnvelope) -> Option<SequenceWrite> {
        let write = match msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Sequence(write),
                        ..
                    },
                ..
            } => Some(write),
            _ => return None,
        };
    }
}

impl Display for Sequences {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Blobs {
    msg_util: ElderMsgUtil,
}

impl Blobs {
    pub fn new(msg_util: ElderMsgUtil) -> Self {
        Self { msg_util }
    }

    // on client request
    pub fn initiate_read(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let read = self.extract_read(msg)?;
        self.msg_util.forward(msg)
        // TODO: We don't check for the existence of a valid signature for published data,
        // since it's free for anyone to get.  However, as a means of spam prevention, we
        // could change this so that signatures are required, and the signatures would need
        // to match a pattern which becomes increasingly difficult as the client's
        // behaviour is deemed to become more "spammy". (e.g. the get requests include a
        // `seed: [u8; 32]`, and the client needs to form a sig matching a required pattern
        // by brute-force attempts with varying seeds)
    }

    // on client request
    pub fn initiate_write(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let write = self.extract_write(msg)?;
        use BlobWrite::*;
        match write {
            New(chunk) => self.initiate_creation(chunk, msg),
            DeletePrivate(address) => self.initiate_deletion(address, msg),
        }
    }

    // on client request
    fn initiate_creation(&mut self, chunk: IData, msg: MsgEnvelope) -> Option<NodeCmd> {
        // Assert that if the request was for UnpubIData, that the owner's public key has
        // been added to the chunk, to avoid Apps putting chunks which can't be retrieved
        // by their Client owners.
        if let IData::Unpub(ref unpub_chunk) = &chunk {
            if unpub_chunk.owner() != origin.id() {
                trace!(
                    "{}: {} attempted Put UnpubIData with invalid owners field.",
                    self,
                    client
                );
                self.msg_util
                    .error(NdError::InvalidOwners, msg.id(), msg.origin)
            }
        }
        self.msg_util.vote(msg)
    }

    // on client request
    fn initiate_deletion(&mut self, address: IDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        if address.kind() == IDataKind::Pub {
            self.msg_util
                .error(NdError::InvalidOperation, msg.id(), msg.origin)
        }
        self.msg_util.vote(msg)
    }

    fn extract_read(&self, msg: MsgEnvelope) -> Option<BlobRead> {
        let write = match msg.message {
            Message::Query {
                query:
                    Query::Data {
                        query: DataQuery::Blob(query),
                        ..
                    },
                ..
            } => Some(query),
            _ => return None,
        };
    }

    fn extract_write(&self, msg: MsgEnvelope) -> Option<BlobWrite> {
        let write = match msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Blob(write),
                        ..
                    },
                ..
            } => Some(write),
            _ => return None,
        };
    }
}

impl Display for Blobs {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Maps {
    msg_util: ElderMsgUtil,
}

impl Maps {
    pub fn new(msg_util: ElderMsgUtil) -> Self {
        Self { msg_util }
    }

    // on client request
    pub fn initiate_read(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let read = self.extract_read(msg)?;
        self.msg_util.forward(msg)
    }

    // on client request
    pub fn initiate_write(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        use MapWrite::*;
        let write = self.extract_write(msg)?;
        match write {
            New(chunk) => self.initiate_creation(chunk, msg),
            Delete(..) | Edit { .. } | SetUserPermissions { .. } | DelUserPermissions { .. } => {
                self.msg_util.vote(msg)
            }
        }
    }

    // on client request
    fn initiate_creation(&mut self, chunk: MData, msg: MsgEnvelope) -> Option<NodeCmd> {
        // Assert that the owner's public key has been added to the chunk, to avoid Apps
        // putting chunks which can't be retrieved by their Client owners.
        if chunk.owner() != msg.origin.id() {
            trace!(
                "{}: {} attempted to store Map with invalid owners field.",
                self,
                client
            );
            return self.error(NdError::InvalidOwners, msg.id(), msg.origin);
        }

        self.msg_util.vote(msg)
    }

    fn extract_read(&self, msg: MsgEnvelope) -> Option<MapRead> {
        let write = match msg.message {
            Message::Query {
                query:
                    Query::Data {
                        query: DataQuery::Map(query),
                        ..
                    },
                ..
            } => Some(query),
            _ => return None,
        };
    }

    fn extract_write(&self, msg: MsgEnvelope) -> Option<MapWrite> {
        let write = match msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Map(write),
                        ..
                    },
                ..
            } => Some(write),
            _ => return None,
        };
    }
}

impl Display for Maps {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(super) struct Accounts {
    msg_util: ElderMsgUtil,
}

impl Accounts {
    pub fn new(msg_util: ElderMsgUtil) -> Self {
        Self { msg_util }
    }

    // on client request
    pub fn initiate_read(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        if self.is_account_read(msg) {
            self.msg_util.vote(msg)
        } else {
            None
        }
    }

    // on client request
    pub fn initiate_write(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let account = self.extract_account_write(msg)?;
        if !account.size_is_valid() {
            return self.error(NdError::ExceededSize, msg.id(), msg.origin);
        }
        self.msg_util.vote(msg)
    }

    fn is_account_read(&self, msg: MsgEnvelope) -> bool {
        match msg.message {
            Message::Query {
                query: Query::Data(DataQuery::Account(_)),
                ..
            } => true,
        }
    }

    fn extract_account_write(&self, msg: MsgEnvelope) -> Option<Account> {
        match msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Account(write),
                        ..
                    },
                ..
            } => match write {
                New(account) => Some(account),
                Update(updated_account) => Some(updated_account),
            },
            _ => None,
        }
    }
}

impl Display for Accounts {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
