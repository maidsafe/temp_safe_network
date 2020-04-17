// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{auth::ClientInfo, Responder, COST_OF_PUT};
use crate::{
    action::{Action, ConsensusAction},
    rpc::Rpc,
    utils,
};
use log::trace;
use safe_nd::{
    AData, ADataAddress, Coins, Error as NdError, IData, IDataAddress, IDataKind, MData, MessageId,
    NodePublicId, Request, Response,
};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

#[derive(Clone)]
pub(crate) struct ElderData {
    idata: ElderIData,
    mdata: ElderMData,
    adata: ElderAData,
}

impl ElderData {
    pub fn new(id: NodePublicId, responder: Rc<RefCell<Responder>>) -> Self {
        Self {
            idata: ElderIData::new(id.clone(), responder.clone()),
            mdata: ElderMData::new(id.clone(), responder.clone()),
            adata: ElderAData::new(id.clone(), responder.clone()),
        }
    }

    pub fn idata(self) -> ElderIData {
        self.idata
    }

    pub fn mdata(self) -> ElderMData {
        self.mdata
    }

    pub fn adata(self) -> ElderAData {
        self.adata
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct ElderAData {
    id: NodePublicId,
    responder: Rc<RefCell<Responder>>,
}

impl ElderAData {
    pub fn new(id: NodePublicId, responder: Rc<RefCell<Responder>>) -> Self {
        Self { id, responder }
    }

    // on client request
    pub fn initiate_adata_creation(
        &mut self,
        client: &ClientInfo,
        chunk: AData,
        message_id: MessageId,
    ) -> Option<Action> {
        let owner = utils::owner(&client.public_id)?;
        // TODO - Should we replace this with a adata.check_permission call in data_handler.
        // That would be more consistent, but on the other hand a check here stops spam earlier.
        if chunk.check_is_last_owner(*owner.public_key()).is_err() {
            trace!(
                "{}: {} attempted Put AppendOnlyData with invalid owners.",
                self,
                client.public_id
            );
            self.responder
                .borrow_mut()
                .respond_to_client(message_id, Response::Mutation(Err(NdError::InvalidOwners)));
            return None;
        }

        let request = Request::PutAData(chunk);
        Some(Action::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.public_id.clone(),
            message_id,
            cost: COST_OF_PUT,
        }))
    }

    // on client request
    pub fn initiate_adata_deletion(
        &mut self,
        client: &ClientInfo,
        address: ADataAddress,
        message_id: MessageId,
    ) -> Option<Action> {
        if address.is_pub() {
            self.responder.borrow_mut().respond_to_client(
                message_id,
                Response::Mutation(Err(NdError::InvalidOperation)),
            );
            return None;
        }

        Some(Action::VoteFor(ConsensusAction::Forward {
            request: Request::DeleteAData(address),
            client_public_id: client.public_id.clone(),
            message_id,
            cost: Coins::from_nano(0),
        }))
    }

    // on client request
    pub fn initiate_adata_mutation(
        &mut self,
        client: &ClientInfo,
        request: Request,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.public_id.clone(),
            message_id,
            cost: COST_OF_PUT,
        }))
    }

    pub fn get_adata(
        &mut self,
        client: &ClientInfo,
        request: Request,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request,
            message_id,
        }))
    }
}

impl Display for ElderAData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct ElderIData {
    id: NodePublicId,
    responder: Rc<RefCell<Responder>>,
}

impl ElderIData {
    pub fn new(id: NodePublicId, responder: Rc<RefCell<Responder>>) -> Self {
        Self { id, responder }
    }

    // on client request
    pub fn initiate_idata_creation(
        &mut self,
        client: &ClientInfo,
        chunk: IData,
        message_id: MessageId,
    ) -> Option<Action> {
        let owner = utils::owner(&client.public_id)?;

        // Assert that if the request was for UnpubIData, that the owner's public key has
        // been added to the chunk, to avoid Apps putting chunks which can't be retrieved
        // by their Client owners.
        if let IData::Unpub(ref unpub_chunk) = &chunk {
            if unpub_chunk.owner() != owner.public_key() {
                trace!(
                    "{}: {} attempted Put UnpubIData with invalid owners field.",
                    self,
                    client.public_id
                );
                self.responder
                    .borrow_mut()
                    .respond_to_client(message_id, Response::Mutation(Err(NdError::InvalidOwners)));
                return None;
            }
        }

        let request = Request::PutIData(chunk);
        Some(Action::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.public_id.clone(),
            message_id,
            cost: COST_OF_PUT,
        }))
    }

    // on client request
    pub fn initiate_unpub_idata_deletion(
        &mut self,
        client: &ClientInfo,
        address: IDataAddress,
        message_id: MessageId,
    ) -> Option<Action> {
        if address.kind() == IDataKind::Pub {
            self.responder.borrow_mut().respond_to_client(
                message_id,
                Response::Mutation(Err(NdError::InvalidOperation)),
            );
            return None;
        }
        Some(Action::VoteFor(ConsensusAction::Forward {
            request: Request::DeleteUnpubIData(address),
            client_public_id: client.public_id.clone(),
            message_id,
            cost: Coins::from_nano(0),
        }))
    }

    pub fn get_idata(
        &mut self,
        client: &ClientInfo,
        address: IDataAddress,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request: Request::GetIData(address),
            message_id,
        }))
    }
}

impl Display for ElderIData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct ElderMData {
    id: NodePublicId,
    responder: Rc<RefCell<Responder>>,
}

impl ElderMData {
    pub fn new(id: NodePublicId, responder: Rc<RefCell<Responder>>) -> Self {
        Self { id, responder }
    }

    // on client request
    pub fn initiate_mdata_mutation(
        &mut self,
        request: Request,
        client: &ClientInfo,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.public_id.clone(),
            message_id,
            cost: COST_OF_PUT,
        }))
    }

    // on client request
    pub fn initiate_mdata_deletion(
        &mut self,
        request: Request,
        client: &ClientInfo,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.public_id.clone(),
            message_id,
            cost: Coins::from_nano(0),
        }))
    }

    // on client request
    pub fn initiate_mdata_creation(
        &mut self,
        client: &ClientInfo,
        chunk: MData,
        message_id: MessageId,
    ) -> Option<Action> {
        let owner = utils::owner(&client.public_id)?;

        // Assert that the owner's public key has been added to the chunk, to avoid Apps
        // putting chunks which can't be retrieved by their Client owners.
        if chunk.owner() != *owner.public_key() {
            trace!(
                "{}: {} attempted PutMData with invalid owners field.",
                self,
                client.public_id
            );
            self.responder
                .borrow_mut()
                .respond_to_client(message_id, Response::Mutation(Err(NdError::InvalidOwners)));
            return None;
        }

        let request = Request::PutMData(chunk);

        Some(Action::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client.public_id.clone(),
            message_id,
            cost: COST_OF_PUT,
        }))
    }

    pub fn get_mdata(
        &mut self,
        request: Request,
        client: &ClientInfo,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request,
            message_id,
        }))
    }
}

impl Display for ElderMData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
