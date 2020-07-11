// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{cmd::NodeCmd, node::Init, utils, Config, Result, ToDbKey};
use log::{info, trace, warn};
use pickledb::PickleDb;
use routing::Node as Routing;
use safe_nd::{
    BlobRead, BlobWrite, CmdError, DataError, Duty, ElderDuty, Error as NdError, IData,
    IDataAddress, Message, MessageId, MsgEnvelope, MsgSender, NetworkCmd, NodePublicId, PublicKey,
    QueryResponse, Read, Result as NdResult, Write, XorName,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    rc::Rc,
};
use tiny_keccak::sha3_256;

const IMMUTABLE_META_DB_NAME: &str = "immutable_data.db";
const HOLDER_META_DB_NAME: &str = "holder_data.db";
const FULL_ADULTS_DB_NAME: &str = "full_adults.db";
// The number of separate copies of an ImmutableData chunk which should be maintained.
const IMMUTABLE_DATA_COPY_COUNT: usize = 4;
const IMMUTABLE_DATA_ADULT_COPY_COUNT: usize = 3;

#[derive(Default, Debug, Serialize, Deserialize)]
struct ChunkMetadata {
    holders: BTreeSet<XorName>,
    owner: Option<PublicKey>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct HolderMetadata {
    chunks: BTreeSet<IDataAddress>,
}

pub(super) struct BlobRegister {
    id: NodePublicId,
    metadata: PickleDb,
    holders: PickleDb,
    #[allow(unused)]
    full_adults: PickleDb,
    #[allow(unused)]
    routing: Rc<RefCell<Routing>>,
}

impl BlobRegister {
    pub(super) fn new(
        id: NodePublicId,
        config: &Config,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let metadata = utils::new_db(&root_dir, IMMUTABLE_META_DB_NAME, init_mode)?;
        let holders = utils::new_db(&root_dir, HOLDER_META_DB_NAME, init_mode)?;
        let full_adults = utils::new_db(&root_dir, FULL_ADULTS_DB_NAME, init_mode)?;

        Ok(Self {
            id,
            metadata,
            holders,
            full_adults,
            routing,
        })
    }

    pub(super) fn write(&mut self, write: BlobWrite, msg: MsgEnvelope) -> Option<NodeCmd> {
        use BlobWrite::*;
        match write {
            New(data) => self.store(data, msg),
            DeletePrivate(address) => self.delete(address, msg),
        }
    }

    fn store(&mut self, data: IData, msg: MsgEnvelope) -> Option<NodeCmd> {
        let cmd_error = |error: NdError| {
            self.wrap(Message::CmdError {
                error: CmdError::Data(error),
                id: MessageId::new(),
                cmd_origin: msg.origin.address(),
                correlation_id: msg.id(),
            })
        };

        // If the data already exist, check the existing no of copies.
        // If no of copies are less then required, then continue with the put request.
        let metadata = self.get_metadata_for(*data.address());
        let target_holders = if let Ok(metadata) = metadata {
            if metadata.holders.len() == IMMUTABLE_DATA_COPY_COUNT {
                if data.is_pub() {
                    trace!("{}: All good, {:?}, chunk already exists.", self, data);
                    return None;
                } else {
                    return cmd_error(NdError::DataExists);
                }
            } else {
                let mut existing_holders = metadata.holders;
                let closest_holders = self
                    .get_holders_for_chunk(data.name())
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>();

                for holder_xorname in closest_holders {
                    if !existing_holders.contains(&holder_xorname)
                        && existing_holders.len() < IMMUTABLE_DATA_COPY_COUNT
                    {
                        let _ = existing_holders.insert(holder_xorname);
                    }
                }
                existing_holders
            }
        } else {
            self.get_holders_for_chunk(data.name())
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        };

        info!("Storing {} copies of the data", target_holders.len());

        let msg = self.set_proxy(msg);
        Some(NodeCmd::SendToAdults {
            targets: metadata.holders,
            msg,
        })
    }

    fn delete(&mut self, address: IDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let cmd_error = |error: NdError| {
            self.wrap(Message::CmdError {
                error: CmdError::Data(error),
                id: MessageId::new(),
                cmd_origin: msg.origin.address(),
                correlation_id: msg.id(),
            })
        };

        let metadata = match self.get_metadata_for(address) {
            Ok(metadata) => metadata,
            Err(error) => return cmd_error(error),
        };

        if let Some(data_owner) = metadata.owner {
            if data_owner != msg.origin.id() {
                return cmd_error(NdError::AccessDenied);
            }
        };

        let msg = self.set_proxy(msg);
        Some(NodeCmd::SendToAdults {
            targets: metadata.holders,
            msg,
        })
    }

    pub(super) fn duplicate_chunks(&mut self, holder: XorName) -> Option<Vec<NodeCmd>> {
        trace!("Duplicating chunks of holder {:?}", holder);

        let chunks_stored = match self.remove_holder(holder) {
            Ok(chunks) => chunks,
            _ => return None,
        };
        let cmds: Vec<_> = chunks_stored
            .into_iter()
            .map(|(address, holders)| self.get_duplication_msgs(address, holders))
            .flatten()
            .collect();

        Some(cmds)
    }

    fn get_duplication_msgs(
        &self,
        address: IDataAddress,
        current_holders: BTreeSet<XorName>,
    ) -> Vec<Message> {
        self.get_new_holders_for_chunk(&address)
            .into_iter()
            .map(|new_holder| {
                let mut hash_bytes = Vec::new();
                hash_bytes.extend_from_slice(&address.name().0);
                hash_bytes.extend_from_slice(&new_holder.0);
                let message_id = MessageId(XorName(sha3_256(&hash_bytes)));
                Message::NetworkCmd {
                    cmd: NetworkCmd::DuplicateChunk {
                        new_holder,
                        address,
                        fetch_from_holders: current_holders,
                    },
                    id: message_id,
                }
            })
            .filter_map(|message| self.wrap(message))
            .collect()
    }

    pub(super) fn read(&self, read: &BlobRead, msg: MsgEnvelope) -> Option<NodeCmd> {
        use BlobRead::*;
        match read {
            Get(address) => self.get(*address, msg),
        }
    }

    fn get(&self, address: IDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let query_error = |error: NdError| {
            self.wrap(Message::QueryResponse {
                response: QueryResponse::GetBlob(Err(error)),
                id: MessageId::new(),
                query_origin: msg.origin.address(),
                correlation_id: msg.id(),
            })
        };

        let metadata = match self.get_metadata_for(address) {
            Ok(metadata) => metadata,
            Err(error) => return query_error(error),
        };

        if let Some(data_owner) = metadata.owner {
            if data_owner != msg.origin.id() {
                return query_error(NdError::AccessDenied);
            }
        };

        let msg = self.set_proxy(msg);
        Some(NodeCmd::SendToAdults {
            targets: metadata.holders,
            msg,
        })
    }

    pub(super) fn update_holders(
        &mut self,
        address: IDataAddress,
        sender: XorName,
        result: NdResult<()>,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        if result.is_ok() {
            let mut chunk_metadata = self.get_metadata_for(address).unwrap_or_default();
            if !chunk_metadata.holders.insert(sender) {
                warn!(
                    "{}: {} already registered as a holder for {:?}",
                    self, sender, address
                );
            }
            if let Err(error) = self.metadata.set(&address.to_db_key(), &chunk_metadata) {
                warn!("{}: Failed to write metadata to DB: {:?}", self, error);
            }
            let mut holders_metadata = self.get_holder(sender).unwrap_or_default();
            if !holders_metadata.chunks.insert(address) {
                warn!(
                    "{}: {} already registered as a holder for {:?}",
                    self, sender, &address
                );
            }
            if let Err(error) = self.holders.set(&sender.to_db_key(), &holders_metadata) {
                warn!(
                    "{}: Failed to write holder metadata to DB: {:?}",
                    self, error
                );
            }
            info!("Duplication process completed for: {:?}", message_id);
        } else {
            // Todo: take care of the mutation failure case
        }
        None
    }

    // pub(super) fn handle_write_result(
    //     &mut self,
    //     sender: XorName,
    //     requester: PublicId,
    //     result: NdResult<()>,
    //     message_id: MessageId,
    //     request: Request,
    // ) -> Option<NodeCmd> {
    //     match &request {
    //         Request::Node(NodeRequest::Write(Write::Blob(BlobWrite::New(data)))) => {
    //             self.handle_store_result(*data.address(), sender, &result, message_id, requester)
    //         }
    //         Request::Node(NodeRequest::Write(Write::Blob(BlobWrite::DeletePrivate(address)))) => {
    //             self.handle_delete_result(*address, sender, result, message_id, requester)
    //         }
    //         _ => None,
    //     }
    // }

    // pub(super) fn handle_store_result(
    //     &mut self,
    //     idata_address: IDataAddress,
    //     sender: XorName,
    //     _result: &NdResult<()>,
    //     message_id: MessageId,
    //     requester: PublicId,
    // ) -> Option<NodeCmd> {
    //     // TODO -
    //     // - if Ok, and this is the final of the three responses send success back to client handlers and
    //     //   then on to the client.  Note: there's no functionality in place yet to know whether
    //     //   this is the last response or not.
    //     // - if Ok, and this is not the last response, just return `None` here.
    //     // - if Err, we need to flag this sender as "full" (i.e. add to self.full_adults, try on
    //     //   next closest non-full adult, or elder if none.  Also update the metadata for this
    //     //   chunk.  Not known yet where we'll get the chunk from to do that.
    //     //
    //     // For phase 1, we can leave many of these unanswered.

    //     // TODO - we'll assume `result` is success for phase 1.
    //     let db_key = idata_address.to_db_key();
    //     let mut metadata = self.get_metadata_for(idata_address).unwrap_or_default();
    //     if idata_address.is_unpub() {
    //         metadata.owner = Some(*utils::own_key(&requester)?);
    //     }

    //     if !metadata.holders.insert(sender) {
    //         warn!(
    //             "{}: {} already registered as a holder for {:?}",
    //             self, sender, &idata_address
    //         );
    //     }

    //     if let Err(error) = self.metadata.set(&db_key, &metadata) {
    //         warn!("{}: Failed to write metadata to DB: {:?}", self, error);
    //         // TODO - send failure back to client handlers (hopefully won't accumulate), or
    //         //        maybe self-terminate if we can't fix this error?
    //     }
    //     debug!(
    //         "{:?}, Entry {:?} has {:?} holders",
    //         self.id,
    //         idata_address,
    //         metadata.holders.len()
    //     );

    //     // We're acting as data handler, received request from client handlers
    //     let mut holders_metadata = self.get_holder(sender).unwrap_or_default();

    //     if !holders_metadata.chunks.insert(idata_address) {
    //         warn!(
    //             "{}: {} already registered as a holder for {:?}",
    //             self, sender, &idata_address
    //         );
    //     }

    //     if let Err(error) = self.holders.set(&sender.to_db_key(), &holders_metadata) {
    //         warn!("{}: Failed to write metadata to DB: {:?}", self, error);
    //     }

    //     // Should we wait for multiple responses
    //     wrap(MetadataCmd::RespondToGateway {
    //         sender: *idata_address.name(),
    //         msg: Message::Response {
    //             requester,
    //             response: Response::Write(Ok(())),
    //             message_id,
    //             proof: None,
    //         },
    //     })
    // }

    // pub(super) fn handle_delete_result(
    //     &mut self,
    //     idata_address: IDataAddress,
    //     sender: XorName,
    //     result: NdResult<()>,
    //     message_id: MessageId,
    //     requester: PublicId,
    // ) -> Option<NodeCmd> {
    //     if let Err(err) = &result {
    //         warn!("{}: Node reports error deleting: {}", self, err);
    //     } else {
    //         let db_key = idata_address.to_db_key();
    //         let metadata = self.get_metadata_for(idata_address);

    //         if let Ok(mut metadata) = metadata {
    //             let holder = self.get_holder(sender);

    //             // Remove the chunk from the holder metadata
    //             if let Ok(mut holder) = holder {
    //                 let _ = holder.chunks.remove(&idata_address);

    //                 if holder.chunks.is_empty() {
    //                     if let Err(error) = self.holders.rem(&sender.to_db_key()) {
    //                         warn!(
    //                             "{}: Failed to delete holder metadata from DB: {:?}",
    //                             self, error
    //                         );
    //                     }
    //                 } else if let Err(error) = self.holders.set(&sender.to_db_key(), &holder) {
    //                     warn!(
    //                         "{}: Failed to write holder metadata to DB: {:?}",
    //                         self, error
    //                     );
    //                 }
    //             }

    //             // Remove the holder from the chunk metadata
    //             if !metadata.holders.remove(&sender) {
    //                 warn!(
    //                     "{}: {} is not registered as a holder for {:?}",
    //                     self, sender, &idata_address
    //                 );
    //             }
    //             if metadata.holders.is_empty() {
    //                 if let Err(error) = self.metadata.rem(&db_key) {
    //                     warn!(
    //                         "{}: Failed to delete chunk metadata from DB: {:?}",
    //                         self, error
    //                     );
    //                     // TODO - Send failure back to client handlers?
    //                 }
    //             } else if let Err(error) = self.metadata.set(&db_key, &metadata) {
    //                 warn!(
    //                     "{}: Failed to write chunk metadata to DB: {:?}",
    //                     self, error
    //                 );
    //                 // TODO - Send failure back to client handlers?
    //             }
    //         };
    //     }

    //     // TODO: Different responses from adults?
    //     wrap(MetadataCmd::RespondToGateway {
    //         sender: *idata_address.name(),
    //         msg: Message::Response {
    //             requester,
    //             response: Response::Write(result),
    //             message_id,
    //             proof: None,
    //         },
    //     })
    // }

    // pub(super) fn handle_get_result(
    //     &self,
    //     result: NdResult<IData>,
    //     message_id: MessageId,
    //     requester: PublicId,
    //     proof: (Request, Signature),
    // ) -> Option<NodeCmd> {
    //     let response = Response::GetIData(result);
    //     wrap(MetadataCmd::RespondToGateway {
    //         sender: *self.id.name(),
    //         msg: Message::Response {
    //             requester,
    //             response,
    //             message_id,
    //             proof: Some(proof),
    //         },
    //     })
    // }

    // Updates the metadata of the chunks help by a node that left.
    // Returns the list of chunks that were held along with the remaining holders.
    pub fn remove_holder(
        &mut self,
        node: XorName,
    ) -> NdResult<BTreeMap<IDataAddress, BTreeSet<XorName>>> {
        let mut idata_addresses: BTreeMap<IDataAddress, BTreeSet<XorName>> = BTreeMap::new();
        let chunk_holder = self.get_holder(node);

        if let Ok(holder) = chunk_holder {
            for chunk_address in holder.chunks {
                let db_key = chunk_address.to_db_key();
                let chunk_metadata = self.get_metadata_for(chunk_address);

                if let Ok(mut metadata) = chunk_metadata {
                    if !metadata.holders.remove(&node) {
                        warn!("doesn't contain the holder",);
                    }

                    let _ = idata_addresses.insert(chunk_address, metadata.holders.clone());

                    if metadata.holders.is_empty() {
                        if let Err(error) = self.metadata.rem(&db_key) {
                            warn!("{}: Failed to write metadata to DB: {:?}", self, error);
                        }
                    } else if let Err(error) = self.metadata.set(&db_key, &metadata) {
                        warn!("{}: Failed to write metadata to DB: {:?}", self, error);
                    }
                }
            }
        }

        // Since the node has left the section, remove it from the holders DB
        if let Err(error) = self.holders.rem(&node.to_db_key()) {
            warn!("{}: Failed to delete metadata from DB: {:?}", self, error);
        };

        Ok(idata_addresses)
    }

    fn get_holder(&self, holder: XorName) -> NdResult<HolderMetadata> {
        match self.holders.get::<HolderMetadata>(&holder.to_db_key()) {
            Some(metadata) => {
                if metadata.chunks.is_empty() {
                    warn!("{}: is not responsible for any chunk", holder);
                    Err(NdError::NoSuchData)
                } else {
                    Ok(metadata)
                }
            }
            None => {
                info!("{}: is not responsible for any chunk", holder);
                Err(NdError::NoSuchData)
            }
        }
    }

    fn get_metadata_for(&self, address: IDataAddress) -> NdResult<ChunkMetadata> {
        match self.metadata.get::<ChunkMetadata>(&address.to_db_key()) {
            Some(metadata) => {
                if metadata.holders.is_empty() {
                    warn!("{}: Metadata holders is empty for: {:?}", self, address);
                    Err(NdError::NoSuchData)
                } else {
                    Ok(metadata)
                }
            }
            None => {
                warn!("{}: Failed to get metadata from DB: {:?}", self, address);
                Err(NdError::NoSuchData)
            }
        }
    }

    // Returns `XorName`s of the target holders for an idata chunk.
    // Used to fetch the list of holders for a new chunk.
    fn get_holders_for_chunk(&self, target: &XorName) -> Vec<XorName> {
        let routing = self.routing.borrow_mut();
        let mut closest_adults = routing
            .our_adults_sorted_by_distance_to(&routing::XorName(target.0))
            .iter()
            .take(IMMUTABLE_DATA_ADULT_COPY_COUNT)
            .map(|p2p_node| XorName(p2p_node.name().0))
            .collect::<Vec<_>>();

        if closest_adults.len() < IMMUTABLE_DATA_COPY_COUNT {
            let mut closest_elders = routing
                .our_elders_sorted_by_distance_to(&routing::XorName(target.0))
                .into_iter()
                .take(IMMUTABLE_DATA_COPY_COUNT - closest_adults.len())
                .map(|p2p_node| XorName(p2p_node.name().0))
                .collect::<Vec<_>>();

            closest_adults.append(&mut closest_elders);
            closest_adults
        } else {
            closest_adults
        }
    }

    // Returns `XorName`s of the new target holders for an idata chunk.
    // Used to fetch the additional list of holders for existing chunks.
    fn get_new_holders_for_chunk(&self, target: &IDataAddress) -> BTreeSet<XorName> {
        let closest_holders = self
            .get_holders_for_chunk(target.name())
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Ok(metadata) = self.get_metadata_for(*target) {
            return closest_holders
                .difference(&metadata.holders)
                .cloned()
                .collect();
        }
        closest_holders
    }

    fn set_proxy(&self, msg: &mut MsgEnvelope) {
        // origin signs the message, while proxies sign the envelope
        msg.add_proxy(self.sign(msg))
    }

    fn ok_or_error(
        &self,
        result: Result<()>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let error = match result {
            Ok(()) => return None,
            Err(error) => error,
        };
        self.wrap(Message::CmdError {
            id: MessageId::new(),
            error: CmdError::Data(error),
            correlation_id: msg_id,
            cmd_origin: origin,
        })
    }

    fn wrap(&self, message: Message) -> Option<NodeCmd> {
        let msg = MsgEnvelope {
            message,
            origin: self.sign(message),
            proxies: Default::default(),
        };
        Some(NodeCmd::SendToSection(msg))
    }

    fn sign<T: Serialize>(&self, data: &T) -> MsgSender {
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(data));
        MsgSender::Node {
            id: self.public_key(),
            duty: Duty::Elder(ElderDuty::Metadata),
            signature,
        }
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::Bls(self.id.public_id().bls_public_key())
    }
}

impl Display for BlobRegister {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
