// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::msg_decisions::AdultMsgDecisions;
use crate::{chunk_store::ImmutableChunkStore, cmd::NodeCmd, node::Init, Config, Result};
use log::{error, info};
use safe_nd::{
    AdultDuty, CmdError, Duty, Error as NdError, IData, IDataAddress, Message, MessageId,
    MsgEnvelope, MsgSender, NetworkCmdError, NodePublicId, PublicKey, QueryResponse,
};
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};
use threshold_crypto::Signature;

pub(crate) struct ChunkStorage {
    id: NodePublicId,
    chunks: ImmutableChunkStore,
    decisions: AdultMsgDecisions,
}

impl ChunkStorage {
    pub(crate) fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let max_capacity = config.max_capacity();
        let chunks = ImmutableChunkStore::new(
            &root_dir,
            max_capacity,
            Rc::clone(total_used_space),
            init_mode,
        )?;
        let decisions = AdultMsgDecisions::new(id, AdultDuty::ChunkStorage);
        Ok(Self {
            id,
            chunks,
            decisions,
        })
    }

    pub(crate) fn store(
        &mut self,
        data: &IData,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        if let Err(error) = self.try_store(data) {
            self.decision.error(
                CmdError::Data(error),
                msg.message.id(),
                msg.origin.address(),
            )
        }
        None
    }

    pub(crate) fn take_duplicate(
        &mut self,
        data: &IData,
        msg_id: MessageId,
        origin: MsgSender,
        accumulated_signature: &Signature,
    ) -> Option<NodeCmd> {
        let message = match self.try_store(data) {
            Ok(()) => Message::NetworkEvent {
                cmd: NetworkEvent::DuplicationComplete {
                    chunk: data.address(),
                    proof: accumulated_signature.clone(),
                },
                id: MessageId::new(),
                correlation_id: message_id,
            },
            Err(error) => Message::NetworkCmdError {
                id: MessageId::new(),
                error: NetworkCmdError::DuplicateChunk {
                    address: data.address(),
                    error,
                },
                correlation_id: msg_id,
                cmd_origin: origin,
            },
        };
        self.decisions.send(message)
    }

    fn try_store(&mut self, data: &IData) -> Result<()> {
        if self.chunks.has(data.address()) {
            info!(
                "{}: Immutable chunk already exists, not storing: {:?}",
                self,
                data.address()
            );
            return None;
        }
        self.chunks.put(&data)
    }

    pub(crate) fn get(
        &self,
        address: IDataAddress,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let result = self.chunks.get(&address);
        self.decisions.send(Message::QueryResponse {
            id: MessageId::new(),
            response: QueryResponse::GetBlob(result),
            correlation_id: msg_id,
            query_origin: origin,
        })
    }

    // pub(crate) fn get_for_duplciation(
    //     &self,
    //     address: IDataAddress,
    //     msg: MsgEnvelope,
    // ) -> Option<NodeCmd> {

    //     match self.chunks.get(&address) {

    //     }

    //     let mut targets: BTreeSet<XorName> = Default::default();
    //     let _ = targets.insert(XorName(xorname.0));
    //     Some(NodeCmd::SendToNode {
    //         targets,
    //         msg: Message::QueryResponse {
    //             requester: requester.clone(),
    //             response: Response::GetIData(result),
    //             message_id,
    //             proof: Some((request, (accumulated_signature?).clone())),
    //         },
    //     })
    // }

    pub(crate) fn delete(
        &mut self,
        address: IDataAddress,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        if !self.chunks.has(&address) {
            info!("{}: Immutable chunk doesn't exist: {:?}", self, address);
            return None;
        }
        let result = self
            .chunks
            .get(&address)
            .and_then(|data| match data {
                IData::Unpub(_) => Ok(()),
                _ => {
                    error!(
                        "{}: Invalid DeletePrivate(IData::Public) encountered: {:?}",
                        self, message_id
                    );
                    Err(NdError::InvalidOperation)
                }
            })
            .and_then(|_| self.chunks.delete(&address));

        if let Err(error) = result {
            self.decision.error(
                CmdError::Data(error),
                msg.message.id(),
                msg.origin.address(),
            )
        }
        None
    }
}

impl Display for ChunkStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
