// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    cmd::{ConsensusAction, GatewayCmd},
    msg::Message,
    utils,
};
use log::trace;
use safe_nd::{
    Account, AccountRead, AccountWrite, BlobRead, BlobWrite, DebitAgreementProof, Error as NdError,
    GatewayRequest, IData, IDataAddress, IDataKind, MData, MapRead, MapWrite, MessageId,
    NodePublicId, NodeRequest, PublicId, Read, SData, SDataAddress,
    SequenceRead, SequenceWrite, Write,
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
    pub fn new(id: NodePublicId) -> Self {
        Self {
            blobs: Blobs::new(id.clone()),
            maps: Maps::new(id.clone()),
            sequences: Sequences::new(id.clone()),
            accounts: Accounts::new(id),
        }
    }

    pub fn initiate_write(
        &mut self,
        cmd: DataCmd,
        client: PublicId,
        msg_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        match cmd {
            DataCmd::Blob(write) => self
                .blobs
                .initiate_write(client, write, msg_id, debit_proof),
            DataCmd::Map(write) => self.maps.initiate_write(client, write, msg_id, debit_proof),
            DataCmd::Sequence(write) => {
                self.sequences
                    .initiate_write(client, write, msg_id, debit_proof)
            }
            DataCmd::Account(write) => {
                self.accounts
                    .initiate_write(client, write, msg_id, debit_proof)
            }
        }
    }

    pub fn initiate_read(
        &mut self,
        read: DataQuery,
        client: PublicId,
        msg_id: MessageId,
    ) -> Option<NodeCmd> {
        match read {
            DataQuery::Blob(write) => self.blobs.initiate_read(client, write, msg_id),
            DataQuery::Map(write) => self.maps.initiate_read(client, write, msg_id),
            DataQuery::Sequence(write) => self.sequences.initiate_read(client, write, msg_id),
            DataQuery::Account(read) => self.accounts.initiate_read(client, read, msg_id),
        }
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Sequences {
    id: NodePublicId,
}

impl Sequences {
    pub fn new(id: NodePublicId) -> Self {
        Self { id }
    }

    // client query
    pub fn initiate_read(
        &mut self,
        requester: PublicId,
        request: SequenceRead,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        Some(GatewayCmd::ForwardClientRequest(Message::Request {
            requester,
            request: Request::Node(NodeRequest::Read(Read::Sequence(request))),
            message_id,
            signature: None,
        }))
    }

    // on client request
    pub fn initiate_write(
        &mut self,
        client: PublicId,
        write: SequenceWrite,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        use SequenceWrite::*;
        match write {
            New(chunk) => self.initiate_creation(client, chunk, message_id, debit_proof),
            Delete(address) => self.initiate_deletion(client, address, message_id, debit_proof),
            SetPubPermissions { .. } | SetPrivPermissions { .. } | SetOwner { .. } | Edit(..) => {
                self.initiate_mutation(client, write, message_id, debit_proof)
            }
        }
    }

    // on client request
    fn initiate_creation(
        &mut self,
        client: PublicId,
        chunk: SData,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        let owner = utils::owner(&client)?;
        // TODO - Should we replace this with a sequence.check_permission call in data_handler.
        // That would be more consistent, but on the other hand a check here stops spam earlier.
        if chunk.check_is_last_owner(*owner.public_key()).is_err() {
            trace!(
                "{}: {} attempted to store Sequence with invalid owners.",
                self,
                client
            );
            return Some(GatewayCmd::RespondToClient {
                message_id,
                response: Response::Write(Err(NdError::InvalidOwners)),
            });
        }

        let request = Self::wrap(SequenceWrite::New(chunk), debit_proof);
        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.clone(),
            message_id,
        }))
    }

    // on client request
    fn initiate_deletion(
        &mut self,
        client_public_id: PublicId,
        address: SDataAddress,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        if address.is_pub() {
            return Some(GatewayCmd::RespondToClient {
                message_id,
                response: Response::Write(Err(NdError::InvalidOperation)),
            });
        }

        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request: Self::wrap(SequenceWrite::Delete(address), debit_proof),
            client_public_id,
            message_id,
        }))
    }

    // on client request
    fn initiate_mutation(
        &mut self,
        client_public_id: PublicId,
        request: SequenceWrite,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request: Self::wrap(request, debit_proof),
            client_public_id,
            message_id,
        }))
    }

    fn wrap(write: SequenceWrite, debit_agreement: DebitAgreementProof) -> Request {
        Request::Gateway(GatewayRequest::Write {
            write: Write::Sequence(write),
            debit_agreement,
        })
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
    id: NodePublicId,
}

impl Blobs {
    pub fn new(id: NodePublicId) -> Self {
        Self { id }
    }

    // on client request
    pub fn initiate_read(
        &mut self,
        requester: PublicId,
        read: BlobRead,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        match read {
            BlobRead::Get(_) => {
                // TODO: We don't check for the existence of a valid signature for published data,
                // since it's free for anyone to get.  However, as a means of spam prevention, we
                // could change this so that signatures are required, and the signatures would need
                // to match a pattern which becomes increasingly difficult as the client's
                // behaviour is deemed to become more "spammy". (e.g. the get requests include a
                // `seed: [u8; 32]`, and the client needs to form a sig matching a required pattern
                // by brute-force attempts with varying seeds)
                Some(GatewayCmd::ForwardClientRequest(Message::Request {
                    requester,
                    request: Request::Node(NodeRequest::Read(Read::Blob(read))),
                    message_id,
                    signature: None,
                }))
            }
        }
    }

    // on client request
    pub fn initiate_write(
        &mut self,
        client: PublicId,
        write: BlobWrite,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        use BlobWrite::*;
        match write {
            New(chunk) => self.initiate_creation(client, chunk, message_id, debit_proof),
            DeletePrivate(address) => {
                self.initiate_deletion(client, address, message_id, debit_proof)
            }
        }
    }

    // on client request
    fn initiate_creation(
        &mut self,
        client: PublicId,
        chunk: IData,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        let owner = utils::owner(&client)?;

        // Assert that if the request was for UnpubIData, that the owner's public key has
        // been added to the chunk, to avoid Apps putting chunks which can't be retrieved
        // by their Client owners.
        if let IData::Unpub(ref unpub_chunk) = &chunk {
            if unpub_chunk.owner() != owner.public_key() {
                trace!(
                    "{}: {} attempted Put UnpubIData with invalid owners field.",
                    self,
                    client
                );
                return Some(GatewayCmd::RespondToClient {
                    message_id,
                    response: Response::Write(Err(NdError::InvalidOwners)),
                });
            }
        }

        let request = Self::wrap(BlobWrite::New(chunk), debit_proof);
        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.clone(),
            message_id,
        }))
    }

    // on client request
    fn initiate_deletion(
        &mut self,
        client_public_id: PublicId,
        address: IDataAddress,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        if address.kind() == IDataKind::Pub {
            return Some(GatewayCmd::RespondToClient {
                message_id,
                response: Response::Write(Err(NdError::InvalidOperation)),
            });
        }
        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request: Self::wrap(BlobWrite::DeletePrivate(address), debit_proof),
            client_public_id,
            message_id,
        }))
    }

    fn wrap(write: BlobWrite, debit_agreement: DebitAgreementProof) -> Request {
        Request::Gateway(GatewayRequest::Write {
            write: Write::Blob(write),
            debit_agreement,
        })
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
    id: NodePublicId,
}

impl Maps {
    pub fn new(id: NodePublicId) -> Self {
        Self { id }
    }

    // on client request
    pub fn initiate_read(
        &mut self,
        client: PublicId,
        read: MapRead,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        use MapRead::*;
        match read {
            Get(..)
            | GetVersion(..)
            | GetShell(..)
            | GetValue { .. }
            | ListPermissions(..)
            | ListUserPermissions { .. }
            | ListEntries(..)
            | ListKeys(..)
            | ListValues(..) => self.get(read, client, message_id),
        }
    }

    // on client request
    pub fn initiate_write(
        &mut self,
        client: PublicId,
        write: MapWrite,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        use MapWrite::*;
        match write {
            New(chunk) => self.initiate_creation(client, chunk, message_id, debit_proof),
            Edit { .. } | SetUserPermissions { .. } | DelUserPermissions { .. } => {
                self.initiate_mutation(write, client, message_id, debit_proof)
            }
            Delete(..) => self.initiate_deletion(write, client, message_id, debit_proof),
        }
    }

    // client query
    fn get(
        &mut self,
        read: MapRead,
        requester: PublicId,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        Some(GatewayCmd::ForwardClientRequest(Message::Request {
            requester,
            request: Request::Node(NodeRequest::Read(Read::Map(read))),
            message_id,
            signature: None,
        }))
    }

    // on client request
    fn initiate_mutation(
        &mut self,
        write: MapWrite,
        client_public_id: PublicId,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request: Self::wrap(write, debit_proof),
            client_public_id,
            message_id,
        }))
    }

    // on client request
    fn initiate_deletion(
        &mut self,
        write: MapWrite,
        client_public_id: PublicId,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request: Self::wrap(write, debit_proof),
            client_public_id,
            message_id,
        }))
    }

    // on client request
    fn initiate_creation(
        &mut self,
        client: PublicId,
        chunk: MData,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        let owner = utils::owner(&client)?;

        // Assert that the owner's public key has been added to the chunk, to avoid Apps
        // putting chunks which can't be retrieved by their Client owners.
        if chunk.owner() != *owner.public_key() {
            trace!(
                "{}: {} attempted PutMData with invalid owners field.",
                self,
                client
            );
            return Some(GatewayCmd::RespondToClient {
                message_id,
                response: Response::Write(Err(NdError::InvalidOwners)),
            });
        }

        let request = Self::wrap(MapWrite::New(chunk), debit_proof);

        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.clone(),
            message_id,
        }))
    }

    fn wrap(write: MapWrite, debit_agreement: DebitAgreementProof) -> Request {
        Request::Gateway(GatewayRequest::Write {
            write: Write::Map(write),
            debit_agreement,
        })
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
    id: NodePublicId,
}

impl Accounts {
    pub fn new(id: NodePublicId) -> Self {
        Self { id }
    }

    // on client request
    pub fn initiate_read(
        &mut self,
        requester: PublicId,
        read: AccountRead,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        Some(GatewayCmd::ForwardClientRequest(Message::Request {
            requester,
            request: Request::Gateway(GatewayRequest::Read(Read::Account(read))),
            message_id,
            signature: None,
        }))
    }

    // on client request
    pub fn initiate_write(
        &mut self,
        client: PublicId,
        write: AccountWrite,
        msg_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        use AccountWrite::*;
        match write {
            New(account) => self.initiate_creation(client, account, msg_id, debit_proof),
            Update(updated_account) => {
                self.initiate_update(client, updated_account, msg_id, debit_proof)
            }
        }
    }

    // on client request
    fn initiate_creation(
        &mut self,
        client_public_id: PublicId,
        account: Account,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        if !account.size_is_valid() {
            return Some(GatewayCmd::RespondToClient {
                message_id,
                response: Response::Write(Err(NdError::ExceededSize)),
            });
        }

        let request = Self::wrap(AccountWrite::New(account), debit_proof);

        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id,
            message_id,
        }))
    }

    // on client request
    fn initiate_update(
        &mut self,
        client_public_id: PublicId,
        updated_account: Account,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<NodeCmd> {
        if !updated_account.size_is_valid() {
            return Some(GatewayCmd::RespondToClient {
                message_id,
                response: Response::Write(Err(NdError::ExceededSize)),
            });
        }

        let request = Self::wrap(AccountWrite::Update(updated_account), debit_proof);

        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id,
            message_id,
        }))
    }

    fn wrap(write: AccountWrite, debit_agreement: DebitAgreementProof) -> Request {
        Request::Gateway(GatewayRequest::Write {
            write: Write::Account(write),
            debit_agreement,
        })
    }
}

impl Display for Accounts {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
