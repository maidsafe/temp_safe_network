// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    action::Action, chunk_store::ImmutableChunkStore, rpc::Rpc, utils, vault::Init, Config, Result,
};
use log::{error, info};

use safe_nd::{Error as NdError, IData, IDataAddress, MessageId, NodePublicId, PublicId, Response};

use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(crate) struct IDataHolder {
    id: NodePublicId,
    immutable_chunks: ImmutableChunkStore,
}

impl IDataHolder {
    pub(crate) fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<RefCell<u64>>,
        init_mode: Init,
    ) -> Result<Self> {
        let root_dir = config.root_dir();
        let max_capacity = config.max_capacity();
        let immutable_chunks = ImmutableChunkStore::new(
            &root_dir,
            max_capacity,
            Rc::clone(total_used_space),
            init_mode,
        )?;
        Ok(Self {
            id,
            immutable_chunks,
        })
    }

    pub(crate) fn store_idata(
        &mut self,
        kind: IData,
        requester: PublicId,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = if self.immutable_chunks.has(kind.address()) {
            info!(
                "{}: Immutable chunk already exists, not storing: {:?}",
                self,
                kind.address()
            );
            Ok(())
        } else {
            self.immutable_chunks
                .put(&kind)
                .map_err(|error| error.to_string().into())
        };
        Some(Action::RespondToOurDataHandlers {
            sender: *self.id.name(),
            message: Rpc::Response {
                requester,
                response: Response::Mutation(result),
                message_id,
            },
        })
    }

    pub(crate) fn get_idata(
        &self,
        address: IDataAddress,
        client: PublicId,
        message_id: MessageId,
    ) -> Option<Action> {
        let client_pk = utils::own_key(&client)?;
        let result = self
            .immutable_chunks
            .get(&address)
            .map_err(|error| error.to_string().into())
            .and_then(|kind| match kind {
                IData::Unpub(ref data) => {
                    if data.owner() != client_pk {
                        Err(NdError::AccessDenied)
                    } else {
                        Ok(kind)
                    }
                }
                _ => Ok(kind),
            });
        Some(Action::RespondToOurDataHandlers {
            sender: *self.id.name(),
            message: Rpc::Response {
                requester: client.clone(),
                response: Response::GetIData(result),
                message_id,
            },
        })
    }

    pub(crate) fn delete_unpub_idata(
        &mut self,
        address: IDataAddress,
        client: PublicId,
        message_id: MessageId,
    ) -> Option<Action> {
        let client_pk = utils::own_key(&client)?;

        // First we need to read the chunk to verify the permissions
        let result = self
            .immutable_chunks
            .get(&address)
            .map_err(|error| error.to_string().into())
            .and_then(|kind| match kind {
                IData::Unpub(ref data) => {
                    if data.owner() != client_pk {
                        Err(NdError::AccessDenied)
                    } else {
                        Ok(())
                    }
                }
                _ => {
                    error!(
                        "{}: Invalid DeleteUnpub(IData::Pub) encountered: {:?}",
                        self, message_id
                    );
                    Err(NdError::InvalidOperation)
                }
            })
            .and_then(|_| {
                self.immutable_chunks
                    .delete(&address)
                    .map_err(|error| error.to_string().into())
            });

        Some(Action::RespondToOurDataHandlers {
            sender: *self.id.name(),
            message: Rpc::Response {
                requester: client.clone(),
                response: Response::Mutation(result),
                message_id,
            },
        })
    }
}

impl Display for IDataHolder {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
