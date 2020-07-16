// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{keys::NodeKeys, msg_wrapping::AdultMsgWrapping};
use crate::{chunk_store::BlobChunkStor, cmd::MessagingDuty, node::Init, Config, Result};
use log::{error, info};
use safe_nd::{
    AdultDuties, CmdError, Error as NdError, Blob, BlobAddress, Message, MessageId, MsgSender,
    NetworkCmdError, NetworkDataError, NetworkEvent, QueryResponse, Result as NdResult, Signature,
};
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(crate) struct ChunkStorage {
    keys: NodeKeys,
    chunks: BlobChunkStor,
    decisions: AdultMsgWrapping,
}

impl ChunkStorage {
    pub(crate) fn new(
        keys: NodeKeys,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let max_capacity = config.max_capacity();
        let chunks = BlobChunkStor::new(
            &root_dir,
            max_capacity,
            Rc::clone(total_used_space),
            init_mode,
        )?;
        let decisions = AdultMsgWrapping::new(keys.clone(), AdultDuties::ChunkStorage);
        Ok(Self {
            keys,
            chunks,
            decisions,
        })
    }

    pub(crate) fn store(
        &mut self,
        data: &Blob,
        msg_id: MessageId,
        origin: &MsgSender,
    ) -> Option<MessagingDuty> {
        if let Err(error) = self.try_store(data) {
            return self.decisions.error(CmdError::Data(error), msg_id, &origin);
        }
        None
    }

    pub(crate) fn take_duplicate(
        &mut self,
        data: &Blob,
        msg_id: MessageId,
        origin: &MsgSender,
        accumulated_signature: &Signature,
    ) -> Option<MessagingDuty> {
        let message = match self.try_store(data) {
            Ok(()) => Message::NetworkEvent {
                event: NetworkEvent::DuplicationComplete {
                    chunk: *data.address(),
                    proof: accumulated_signature.clone(),
                },
                id: MessageId::new(),
                correlation_id: msg_id,
            },
            Err(error) => Message::NetworkCmdError {
                id: MessageId::new(),
                error: NetworkCmdError::Data(NetworkDataError::ChunkDuplication {
                    address: *data.address(),
                    error,
                }),
                correlation_id: msg_id,
                cmd_origin: origin.address(),
            },
        };
        self.decisions.send(message)
    }

    fn try_store(&mut self, data: &Blob) -> NdResult<()> {
        if self.chunks.has(data.address()) {
            info!(
                "{}: Immutable chunk already exists, not storing: {:?}",
                self,
                data.address()
            );
            return Err(NdError::DataExists);
        }
        self.chunks
            .put(&data)
            .map_err(|error| error.to_string().into())
    }

    pub(crate) fn get(
        &self,
        address: BlobAddress,
        msg_id: MessageId,
        origin: &MsgSender,
    ) -> Option<MessagingDuty> {
        let result = self
            .chunks
            .get(&address)
            .map_err(|error| error.to_string().into());
        self.decisions.send(Message::QueryResponse {
            id: MessageId::new(),
            response: QueryResponse::GetBlob(result),
            correlation_id: msg_id,
            query_origin: origin.address(),
        })
    }

    // pub(crate) fn get_for_duplciation(
    //     &self,
    //     address: BlobAddress,
    //     msg: &MsgEnvelope,
    // ) -> Option<MessagingDuty> {

    //     match self.chunks.get(&address) {

    //     }

    //     let mut targets: BTreeSet<XorName> = Default::default();
    //     let _ = targets.insert(XorName(xorname.0));
    //     Some(MessagingDuty::SendToNode {
    //         targets,
    //         msg: Message::QueryResponse {
    //             requester: requester.clone(),
    //             response: Response::GetBlob(result),
    //             message_id,
    //             proof: Some((request, (accumulated_signature?).clone())),
    //         },
    //     })
    // }

    pub(crate) fn delete(
        &mut self,
        address: BlobAddress,
        msg_id: MessageId,
        origin: &MsgSender,
    ) -> Option<MessagingDuty> {
        if !self.chunks.has(&address) {
            info!("{}: Immutable chunk doesn't exist: {:?}", self, address);
            return None;
        }

        let result = match self.chunks.get(&address) {
            Ok(Blob::Private(_)) => self
                .chunks
                .delete(&address)
                .map_err(|error| error.to_string().into()),
            Ok(_) => {
                error!(
                    "{}: Invalid DeletePrivate(Blob::Public) encountered: {:?}",
                    self, msg_id
                );
                Err(NdError::InvalidOperation)
            }
            _ => Err(NdError::NoSuchKey),
            //err @ Err(_) => err.map_err(|error| error.to_string().into()),
        };

        if let Err(error) = result {
            return self.decisions.error(CmdError::Data(error), msg_id, &origin);
        }
        None
    }
}

impl Display for ChunkStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
