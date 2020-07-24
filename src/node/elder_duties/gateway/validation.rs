// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{node::msg_wrapping::ElderMsgWrapping, node::node_ops::MessagingDuty};
use log::trace;
use safe_nd::{
    Account, AccountWrite, Blob, BlobAddress, BlobKind, BlobRead, BlobWrite, Cmd, CmdError,
    DataCmd, DataQuery, Error as NdError, Map, MapRead, MapWrite, Message, MsgEnvelope, Query,
    Sequence, SequenceAddress, SequenceRead, SequenceWrite,
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
    pub fn new(wrapping: ElderMsgWrapping) -> Self {
        Self {
            blobs: Blobs::new(wrapping.clone()),
            maps: Maps::new(wrapping.clone()),
            sequences: Sequences::new(wrapping.clone()),
            accounts: Accounts::new(wrapping),
        }
    }

    // pub fn receive_msg(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
    //     match &msg.message {
    //         Message::Cmd {
    //             cmd: Cmd::Data { cmd, .. },
    //             ..
    //         } => self.initiate_write(cmd, msg),
    //         Message::Query {
    //             query: Query::Data(query),
    //             ..
    //         } => self.initiate_read(query, msg),
    //         _ => return None,
    //     }
    // }

    // Called directly from
    pub fn initiate_write(&mut self, cmd: &DataCmd, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        match cmd {
            DataCmd::Blob(_) => self.blobs.initiate_write(msg),
            DataCmd::Map(_) => self.maps.initiate_write(msg),
            DataCmd::Sequence(_) => self.sequences.initiate_write(msg),
            DataCmd::Account(_) => self.accounts.initiate_write(msg),
        }
    }

    pub fn initiate_read(&mut self, query: &DataQuery, msg: &MsgEnvelope) -> Option<MessagingDuty> {
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
    wrapping: ElderMsgWrapping,
}

impl Sequences {
    pub fn new(wrapping: ElderMsgWrapping) -> Self {
        Self { wrapping }
    }

    // client query
    pub fn initiate_read(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        let _ = self.extract_read(msg)?;
        self.wrapping.forward(msg)
    }

    // on client request
    pub fn initiate_write(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        let write = self.extract_write(msg)?;
        use SequenceWrite::*;
        match write {
            New(chunk) => self.initiate_creation(chunk, msg),
            Delete(address) => self.initiate_deletion(address, msg),
            SetPublicPermissions { .. }
            | SetPrivatePermissions { .. }
            | SetOwner { .. }
            | Edit(..) => self.initiate_edit(msg),
        }
    }

    // on client request
    fn initiate_creation(&mut self, chunk: Sequence, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        // TODO - Should we replace this with a sequence.check_permission call in data_handler.
        // That would be more consistent, but on the other hand a check here stops spam earlier.
        if chunk.check_is_last_owner(msg.origin.id()).is_err() {
            trace!(
                "{}: {} attempted to store Sequence with invalid owners.",
                self,
                msg.origin.id()
            );
            return self.wrapping.error(
                CmdError::Data(NdError::InvalidOwners),
                msg.id(),
                &msg.origin.address(),
            );
        }
        self.wrapping.forward(msg)
    }

    // on client request
    fn initiate_deletion(
        &mut self,
        address: SequenceAddress,
        msg: &MsgEnvelope,
    ) -> Option<MessagingDuty> {
        if address.is_pub() {
            return self.wrapping.error(
                CmdError::Data(NdError::InvalidOperation),
                msg.id(),
                &msg.origin.address(),
            );
        }
        self.wrapping.forward(msg)
    }

    // on client request
    fn initiate_edit(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        self.wrapping.forward(msg)
    }

    fn extract_read(&self, msg: &MsgEnvelope) -> Option<SequenceRead> {
        match &msg.message {
            Message::Query {
                query: Query::Data(DataQuery::Sequence(query)),
                ..
            } => Some(query.clone()),
            _ => None,
        }
    }

    fn extract_write(&self, msg: &MsgEnvelope) -> Option<SequenceWrite> {
        match &msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Sequence(write),
                        ..
                    },
                ..
            } => Some(write.clone()),
            _ => None,
        }
    }
}

impl Display for Sequences {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", "Sequences")
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Blobs {
    wrapping: ElderMsgWrapping,
}

impl Blobs {
    pub fn new(wrapping: ElderMsgWrapping) -> Self {
        Self { wrapping }
    }

    // on client request
    pub fn initiate_read(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        let _read = self.extract_read(msg)?;
        self.wrapping.forward(msg)
        // TODO: We don't check for the existence of a valid signature for published data,
        // since it's free for anyone to get.  However, as a means of spam prevention, we
        // could change this so that signatures are required, and the signatures would need
        // to match a pattern which becomes increasingly difficult as the client's
        // behaviour is deemed to become more "spammy". (e.g. the get requests include a
        // `seed: [u8; 32]`, and the client needs to form a sig matching a required pattern
        // by brute-force attempts with varying seeds)
    }

    // on client request
    pub fn initiate_write(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        let write = self.extract_write(msg)?;
        use BlobWrite::*;
        match write {
            New(chunk) => self.initiate_creation(chunk, msg),
            DeletePrivate(address) => self.initiate_deletion(address, msg),
        }
    }

    // on client request
    fn initiate_creation(&mut self, chunk: Blob, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        // Assert that if the request was for UnpubBlob, that the owner's public key has
        // been added to the chunk, to avoid Apps putting chunks which can't be retrieved
        // by their Client owners.
        if let Blob::Private(ref unpub_chunk) = &chunk {
            if unpub_chunk.owner() != &msg.origin.id() {
                trace!(
                    "{}: {} attempted Put UnpubBlob with invalid owners field.",
                    self,
                    msg.origin.id()
                );
                return self.wrapping.error(
                    CmdError::Data(NdError::InvalidOwners),
                    msg.id(),
                    &msg.origin.address(),
                );
            }
        }
        self.wrapping.forward(msg)
    }

    // on client request
    fn initiate_deletion(
        &mut self,
        address: BlobAddress,
        msg: &MsgEnvelope,
    ) -> Option<MessagingDuty> {
        if address.kind() == BlobKind::Private {
            self.wrapping.forward(msg)
        } else {
            self.wrapping.error(
                CmdError::Data(NdError::InvalidOperation),
                msg.id(),
                &msg.origin.address(),
            )
        }
    }

    fn extract_read(&self, msg: &MsgEnvelope) -> Option<BlobRead> {
        match &msg.message {
            Message::Query {
                query: Query::Data(DataQuery::Blob(query)),
                ..
            } => Some(query.clone()),
            _ => None,
        }
    }

    fn extract_write(&self, msg: &MsgEnvelope) -> Option<BlobWrite> {
        match &msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Blob(write),
                        ..
                    },
                ..
            } => Some(write.clone()),
            _ => None,
        }
    }
}

impl Display for Blobs {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", "Blobs")
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Maps {
    wrapping: ElderMsgWrapping,
}

impl Maps {
    pub fn new(wrapping: ElderMsgWrapping) -> Self {
        Self { wrapping }
    }

    // on client request
    pub fn initiate_read(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        let _read = self.extract_read(msg)?;
        self.wrapping.forward(msg)
    }

    // on client request
    pub fn initiate_write(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        use MapWrite::*;
        let write = self.extract_write(msg)?;
        match write {
            New(chunk) => self.initiate_creation(chunk, msg),
            Delete(..) | Edit { .. } | SetUserPermissions { .. } | DelUserPermissions { .. } => {
                self.wrapping.forward(msg)
            }
        }
    }

    // on client request
    fn initiate_creation(&mut self, chunk: Map, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        // Assert that the owner's public key has been added to the chunk, to avoid Apps
        // putting chunks which can't be retrieved by their Client owners.
        if chunk.owner() != msg.origin.id() {
            trace!(
                "{}: {} attempted to store Map with invalid owners field.",
                self,
                msg.origin.id()
            );
            return self.wrapping.error(
                CmdError::Data(NdError::InvalidOwners),
                msg.id(),
                &msg.origin.address(),
            );
        }

        self.wrapping.forward(msg)
    }

    fn extract_read(&self, msg: &MsgEnvelope) -> Option<MapRead> {
        match &msg.message {
            Message::Query {
                query: Query::Data(DataQuery::Map(query)),
                ..
            } => Some(query.clone()),
            _ => None,
        }
    }

    fn extract_write(&self, msg: &MsgEnvelope) -> Option<MapWrite> {
        match &msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Map(write),
                        ..
                    },
                ..
            } => Some(write.clone()),
            _ => return None,
        }
    }
}

impl Display for Maps {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", "Maps")
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(super) struct Accounts {
    wrapping: ElderMsgWrapping,
}

impl Accounts {
    pub fn new(wrapping: ElderMsgWrapping) -> Self {
        Self { wrapping }
    }

    // on client request
    pub fn initiate_read(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        if self.is_account_read(msg) {
            self.wrapping.forward(msg)
        } else {
            None
        }
    }

    // on client request
    pub fn initiate_write(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        let account = self.extract_account_write(msg)?;
        if !account.size_is_valid() {
            return self.wrapping.error(
                CmdError::Data(NdError::ExceededSize),
                msg.id(),
                &msg.origin.address(),
            );
        }
        self.wrapping.forward(msg)
    }

    fn is_account_read(&self, msg: &MsgEnvelope) -> bool {
        match msg.message {
            Message::Query {
                query: Query::Data(DataQuery::Account(_)),
                ..
            } => true,
            _ => false,
        }
    }

    fn extract_account_write(&self, msg: &MsgEnvelope) -> Option<Account> {
        use AccountWrite::*;
        match &msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Account(write),
                        ..
                    },
                ..
            } => match write {
                New(account) => Some(account.clone()),
                Update(updated_account) => Some(updated_account.clone()),
            },
            _ => None,
        }
    }
}

impl Display for Accounts {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", "Accounts")
    }
}
