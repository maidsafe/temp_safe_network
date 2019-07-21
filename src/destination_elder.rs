// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod idata_op;

use crate::{
    action::Action,
    adata_handler::ADataHandler,
    chunk_store::{
        error::Error as ChunkStoreError, AppendOnlyChunkStore, ImmutableChunkStore,
        MutableChunkStore,
    },
    idata_holder::IDataHolder,
    mdata_handler::MDataHandler,
    rpc::Rpc,
    utils,
    vault::Init,
    Config, Result, ToDbKey,
};
use idata_op::{IDataOp, OpType};
use log::{error, info, trace, warn};
use pickledb::PickleDb;
use safe_nd::{
    AData, ADataAction, ADataAddress, ADataAppend, ADataIndex, ADataOwner, ADataPubPermissions,
    ADataUnpubPermissions, ADataUser, AppendOnlyData, Error as NdError, IData, IDataAddress, MData,
    MDataAction, MDataAddress, MDataPermissionSet, MDataSeqEntryActions, MDataUnseqEntryActions,
    MessageId, NodePublicId, PublicId, PublicKey, Request, Response, Result as NdResult,
    SeqAppendOnly, UnseqAppendOnly, XorName,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    iter,
    rc::Rc,
};
use unwrap::unwrap;

const IMMUTABLE_META_DB_NAME: &str = "immutable_data.db";
const FULL_ADULTS_DB_NAME: &str = "full_adults.db";
// The number of separate copies of an ImmutableData chunk which should be maintained.
const IMMUTABLE_DATA_COPY_COUNT: usize = 3;

#[derive(Default, Serialize, Deserialize)]
struct ChunkMetadata {
    holders: BTreeSet<XorName>,
}

// TODO - remove this
#[allow(unused)]
pub(crate) struct DestinationElder {
    id: NodePublicId,
    idata_ops: BTreeMap<MessageId, IDataOp>,
    immutable_metadata: PickleDb,
    full_adults: PickleDb,
    idata_holder: IDataHolder,
    mdata_handler: MDataHandler,
    adata_handler: ADataHandler,
}

impl DestinationElder {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<RefCell<u64>>,
        init_mode: Init,
    ) -> Result<Self> {
        let root_dir = config.root_dir();
        let immutable_metadata = utils::new_db(&root_dir, IMMUTABLE_META_DB_NAME, init_mode)?;
        let full_adults = utils::new_db(&root_dir, FULL_ADULTS_DB_NAME, init_mode)?;

        let max_capacity = config.max_capacity();
        let idata_holder = IDataHolder::new(id.clone(), config, total_used_space, init_mode)?;
        let mdata_handler = MDataHandler::new(id.clone(), config, total_used_space, init_mode)?;
        let adata_handler = ADataHandler::new(id.clone(), config, total_used_space, init_mode)?;
        Ok(Self {
            id,
            idata_ops: Default::default(),
            immutable_metadata,
            full_adults,
            idata_holder,
            mdata_handler,
            adata_handler,
        })
    }

    pub fn handle_vault_message(&mut self, src: XorName, message: Rpc) -> Option<Action> {
        match message {
            Rpc::Request {
                request,
                requester,
                message_id,
            } => self.handle_request(src, requester, request, message_id),
            Rpc::Response {
                response,
                message_id,
                ..
            } => self.handle_response(src, response, message_id),
        }
    }

    fn handle_request(
        &mut self,
        src: XorName,
        requester: PublicId,
        request: Request,
        message_id: MessageId,
    ) -> Option<Action> {
        use Request::*;
        trace!(
            "{}: Received ({:?} {:?}) from src {} (client {:?})",
            self,
            request,
            message_id,
            src,
            requester
        );
        // TODO - remove this
        #[allow(unused)]
        match request {
            //
            // ===== Immutable Data =====
            //
            PutIData(kind) => self.handle_put_idata_req(src, requester, kind, message_id),
            GetIData(address) => self.handle_get_idata_req(src, requester, address, message_id),
            DeleteUnpubIData(address) => {
                self.handle_delete_unpub_idata_req(src, requester, address, message_id)
            }
            //
            // ===== Mutable Data =====
            //
            PutMData(data) => self
                .mdata_handler
                .handle_put_mdata_req(requester, data, message_id),
            GetMData(address) => self
                .mdata_handler
                .handle_get_mdata_req(requester, address, message_id),
            GetMDataValue { address, ref key } => self
                .mdata_handler
                .handle_get_mdata_value_req(requester, address, key, message_id),
            DeleteMData(address) => self
                .mdata_handler
                .handle_delete_mdata_req(requester, address, message_id),
            GetMDataShell(address) => self
                .mdata_handler
                .handle_get_mdata_shell_req(requester, address, message_id),
            GetMDataVersion(address) => self
                .mdata_handler
                .handle_get_mdata_version_req(requester, address, message_id),
            ListMDataEntries(address) => self
                .mdata_handler
                .handle_list_mdata_entries_req(requester, address, message_id),
            ListMDataKeys(address) => self
                .mdata_handler
                .handle_list_mdata_keys_req(requester, address, message_id),
            ListMDataValues(address) => self
                .mdata_handler
                .handle_list_mdata_values_req(requester, address, message_id),
            ListMDataPermissions(address) => self
                .mdata_handler
                .handle_list_mdata_permissions_req(requester, address, message_id),
            ListMDataUserPermissions { address, user } => self
                .mdata_handler
                .handle_list_mdata_user_permissions_req(requester, address, user, message_id),
            SetMDataUserPermissions {
                address,
                user,
                ref permissions,
                version,
            } => self.mdata_handler.handle_set_mdata_user_permissions_req(
                requester,
                address,
                user,
                permissions,
                version,
                message_id,
            ),
            DelMDataUserPermissions {
                address,
                user,
                version,
            } => self.mdata_handler.handle_del_mdata_user_permissions_req(
                requester, address, user, version, message_id,
            ),
            MutateSeqMDataEntries { address, actions } => self
                .mdata_handler
                .handle_mutate_seq_mdata_entries_req(requester, address, actions, message_id),
            MutateUnseqMDataEntries { address, actions } => self
                .mdata_handler
                .handle_mutate_unseq_mdata_entries_req(requester, address, actions, message_id),
            //
            // ===== Append Only Data =====
            //
            PutAData(data) => self
                .adata_handler
                .handle_put_adata_req(requester, data, message_id),
            GetAData(address) => self
                .adata_handler
                .handle_get_adata_req(requester, address, message_id),
            GetADataValue { address, key } => self
                .adata_handler
                .handle_get_adata_value_req(requester, address, key, message_id),
            GetADataShell {
                address,
                data_index,
            } => self
                .adata_handler
                .handle_get_adata_shell_req(requester, address, data_index, message_id),
            GetADataRange { address, range } => self
                .adata_handler
                .handle_get_adata_range_req(requester, address, range, message_id),
            GetADataIndices(address) => self
                .adata_handler
                .handle_get_adata_indices_req(requester, address, message_id),
            GetADataLastEntry(address) => self
                .adata_handler
                .handle_get_adata_last_entry_req(requester, address, message_id),
            GetADataOwners {
                address,
                owners_index,
            } => self.adata_handler.handle_get_adata_owners_req(
                requester,
                address,
                owners_index,
                message_id,
            ),
            GetPubADataUserPermissions {
                address,
                permissions_index,
                user,
            } => self
                .adata_handler
                .handle_get_pub_adata_user_permissions_req(
                    requester,
                    address,
                    permissions_index,
                    user,
                    message_id,
                ),
            GetUnpubADataUserPermissions {
                address,
                permissions_index,
                public_key,
            } => self
                .adata_handler
                .handle_get_unpub_adata_user_permissions_req(
                    requester,
                    address,
                    permissions_index,
                    public_key,
                    message_id,
                ),
            GetADataPermissions {
                address,
                permissions_index,
            } => self.adata_handler.handle_get_adata_permissions_req(
                requester,
                address,
                permissions_index,
                message_id,
            ),
            DeleteAData(address) => self
                .adata_handler
                .handle_delete_adata_req(requester, address, message_id),
            AddPubADataPermissions {
                address,
                permissions,
                permissions_idx,
            } => self.adata_handler.handle_add_pub_adata_permissions_req(
                requester,
                address,
                permissions,
                permissions_idx,
                message_id,
            ),
            AddUnpubADataPermissions {
                address,
                permissions,
                permissions_idx,
            } => self.adata_handler.handle_add_unpub_adata_permissions_req(
                requester,
                address,
                permissions,
                permissions_idx,
                message_id,
            ),
            SetADataOwner {
                address,
                owner,
                owners_idx,
            } => self
                .adata_handler
                .handle_set_adata_owner_req(requester, address, owner, owners_idx, message_id),
            AppendSeq { append, index } => self
                .adata_handler
                .handle_append_seq_req(requester, append, index, message_id),
            AppendUnseq(operation) => self
                .adata_handler
                .handle_append_unseq_req(requester, operation, message_id),
            //
            // ===== Coins =====
            //
            TransferCoins {
                destination,
                amount,
                transaction_id,
            } => unimplemented!(),
            //
            // ===== Login packets =====
            //
            //
            // ===== Invalid =====
            //
            GetBalance
            | CreateBalance { .. }
            | CreateLoginPacket(_)
            | CreateLoginPacketFor { .. }
            | UpdateLoginPacket(_)
            | GetLoginPacket(_)
            | ListAuthKeysAndVersion
            | InsAuthKey { .. }
            | DelAuthKey { .. } => {
                error!(
                    "{}: Should not receive {:?} as a destination elder.",
                    self, request
                );
                None
            }
        }
    }

    fn handle_response(
        &mut self,
        src: XorName,
        response: Response,
        message_id: MessageId,
    ) -> Option<Action> {
        use Response::*;
        trace!(
            "{}: Received ({:?} {:?}) from {}",
            self,
            response,
            message_id,
            src
        );
        match response {
            Mutation(result) => self.handle_mutation_resp(src, result, message_id),
            GetIData(result) => self.handle_get_idata_resp(src, result, message_id),
            //
            // ===== Invalid =====
            //
            GetMData(_)
            | GetMDataShell(_)
            | GetMDataVersion(_)
            | ListUnseqMDataEntries(_)
            | ListSeqMDataEntries(_)
            | ListMDataKeys(_)
            | ListSeqMDataValues(_)
            | ListUnseqMDataValues(_)
            | ListMDataUserPermissions(_)
            | ListMDataPermissions(_)
            | GetSeqMDataValue(_)
            | GetUnseqMDataValue(_)
            | GetAData(_)
            | GetADataValue(_)
            | GetADataShell(_)
            | GetADataOwners(_)
            | GetADataRange(_)
            | GetADataIndices(_)
            | GetADataLastEntry(_)
            | GetUnpubADataPermissionAtIndex(_)
            | GetPubADataPermissionAtIndex(_)
            | GetPubADataUserPermissions(_)
            | GetUnpubADataUserPermissions(_)
            | Transaction(_)
            | GetBalance(_)
            | ListAuthKeysAndVersion(_)
            | GetLoginPacket(_) => {
                error!(
                    "{}: Should not receive {:?} as a destination elder.",
                    self, response
                );
                None
            }
        }
    }

    fn handle_put_idata_req(
        &mut self,
        src: XorName,
        requester: PublicId,
        kind: IData,
        message_id: MessageId,
    ) -> Option<Action> {
        if &src == kind.name() {
            // Since the src is the chunk's name, this message was sent by the dst elders to us as a
            // single dst elder, implying that we're a dst elder chosen to store the chunk.
            self.idata_holder.store_idata(kind, requester, message_id)
        } else {
            // We're acting as dst elder, received request from client handlers
            let data_name = *kind.name();

            let client_id = requester.clone();
            let respond = |result: NdResult<()>| {
                Some(Action::RespondToClientHandlers {
                    sender: data_name,
                    message: Rpc::Response {
                        requester: client_id,
                        response: Response::Mutation(result),
                        message_id,
                    },
                })
            };

            if self
                .immutable_metadata
                .exists(&(*kind.address()).to_db_key())
            {
                trace!(
                    "{}: Replying success for Put {:?}, it already exists.",
                    self,
                    kind
                );
                return respond(Ok(()));
            }
            let target_holders = self
                .non_full_adults_sorted(kind.name())
                .chain(self.elders_sorted(kind.name()))
                .take(IMMUTABLE_DATA_COPY_COUNT)
                .cloned()
                .collect::<BTreeSet<_>>();
            let data_name = *kind.name();
            // Can't fail
            let idata_op = unwrap!(IDataOp::new(
                requester.clone(),
                Request::PutIData(kind),
                target_holders.clone()
            ));
            match self.idata_ops.entry(message_id) {
                Entry::Occupied(_) => respond(Err(NdError::DuplicateMessageId)),
                Entry::Vacant(vacant_entry) => {
                    let idata_op = vacant_entry.insert(idata_op);
                    Some(Action::SendToPeers {
                        sender: data_name,
                        targets: target_holders,
                        message: Rpc::Request {
                            request: idata_op.request().clone(),
                            requester,
                            message_id,
                        },
                    })
                }
            }
        }
    }

    fn handle_delete_unpub_idata_req(
        &mut self,
        src: XorName,
        requester: PublicId,
        address: IDataAddress,
        message_id: MessageId,
    ) -> Option<Action> {
        if &src == address.name() {
            // Since the src is the chunk's name, this message was sent by the dst elders to us as a
            // single dst elder, implying that we're a dst elder where the chunk is stored.
            let client = self.client_id(&message_id)?.clone();
            self.idata_holder
                .delete_unpub_idata(address, client, message_id)
        } else {
            // We're acting as dst elder, received request from client handlers
            let client_id = requester.clone();
            let respond = |result: NdResult<()>| {
                Some(Action::RespondToClientHandlers {
                    sender: *address.name(),
                    message: Rpc::Response {
                        requester: client_id,
                        response: Response::Mutation(result),
                        message_id,
                    },
                })
            };

            let metadata = match self.get_metadata_for(address) {
                Ok(metadata) => metadata,
                Err(error) => return respond(Err(error)),
            };

            // Can't fail
            let idata_op = unwrap!(IDataOp::new(
                requester.clone(),
                Request::DeleteUnpubIData(address),
                metadata.holders.clone()
            ));
            match self.idata_ops.entry(message_id) {
                Entry::Occupied(_) => respond(Err(NdError::DuplicateMessageId)),
                Entry::Vacant(vacant_entry) => {
                    let idata_op = vacant_entry.insert(idata_op);
                    Some(Action::SendToPeers {
                        sender: *address.name(),
                        targets: metadata.holders,
                        message: Rpc::Request {
                            request: idata_op.request().clone(),
                            requester,
                            message_id,
                        },
                    })
                }
            }
        }
    }

    fn handle_mutation_resp(
        &mut self,
        sender: XorName,
        result: NdResult<()>,
        message_id: MessageId,
    ) -> Option<Action> {
        let own_id = format!("{}", self);
        let (idata_address, op_type) = self.idata_op_mut(&message_id).and_then(|idata_op| {
            let op_type = idata_op.op_type();
            idata_op
                .handle_mutation_resp(sender, own_id, message_id)
                .map(|address| (address, op_type))
        })?;

        if op_type == OpType::Put {
            self.handle_put_idata_resp(idata_address, sender, result, message_id)
        } else {
            self.handle_delete_unpub_idata_resp(idata_address, sender, result, message_id)
        }
    }

    fn handle_put_idata_resp(
        &mut self,
        idata_address: IDataAddress,
        sender: XorName,
        _result: NdResult<()>,
        message_id: MessageId,
    ) -> Option<Action> {
        // TODO -
        // - if Ok, and this is the final of the three responses send success back to client handlers and
        //   then on to the client.  Note: there's no functionality in place yet to know whether
        //   this is the last response or not.
        // - if Ok, and this is not the last response, just return `None` here.
        // - if Err, we need to flag this sender as "full" (i.e. add to self.full_adults, try on
        //   next closest non-full adult, or elder if none.  Also update the metadata for this
        //   chunk.  Not known yet where we'll get the chunk from to do that.
        //
        // For phase 1, we can leave many of these unanswered.

        // TODO - we'll assume `result` is success for phase 1.
        let db_key = idata_address.to_db_key();
        let mut metadata = self
            .immutable_metadata
            .get::<ChunkMetadata>(&db_key)
            .unwrap_or_default();
        if !metadata.holders.insert(sender) {
            warn!(
                "{}: {} already registered as a holder for {:?}",
                self,
                sender,
                self.idata_op(&message_id)?
            );
        }
        if let Err(error) = self.immutable_metadata.set(&db_key, &metadata) {
            warn!("{}: Failed to write metadata to DB: {:?}", self, error);
            // TODO - send failure back to client handlers (hopefully won't accumulate), or
            //        maybe self-terminate if we can't fix this error?
        }

        self.remove_idata_op_if_concluded(&message_id)
            .map(|idata_op| Action::RespondToClientHandlers {
                sender: *idata_address.name(),
                message: Rpc::Response {
                    requester: idata_op.client().clone(),
                    response: Response::Mutation(Ok(())),
                    message_id,
                },
            })
    }

    fn handle_delete_unpub_idata_resp(
        &mut self,
        idata_address: IDataAddress,
        sender: XorName,
        _result: NdResult<()>,
        message_id: MessageId,
    ) -> Option<Action> {
        // TODO - Assume deletion on Adult nodes was success for phase 1.
        let db_key = idata_address.to_db_key();
        let metadata = self
            .immutable_metadata
            .get::<ChunkMetadata>(&db_key)
            .or_else(|| {
                warn!(
                    "{}: Failed to get metadata from DB: {:?}",
                    self, idata_address
                );
                None
            });

        if let Some(mut metadata) = metadata {
            if !metadata.holders.remove(&sender) {
                warn!(
                    "{}: {} is not registered as a holder for {:?}",
                    self,
                    sender,
                    self.idata_op(&message_id)?
                );
            }
            if metadata.holders.is_empty() {
                if let Err(error) = self.immutable_metadata.rem(&db_key) {
                    warn!("{}: Failed to delete metadata from DB: {:?}", self, error);
                    // TODO - Send failure back to client handlers?
                }
            } else if let Err(error) = self.immutable_metadata.set(&db_key, &metadata) {
                warn!("{}: Failed to write metadata to DB: {:?}", self, error);
                // TODO - Send failure back to client handlers?
            }
        };
        self.remove_idata_op_if_concluded(&message_id)
            .map(|idata_op| Action::RespondToClientHandlers {
                sender: *idata_address.name(),
                message: Rpc::Response {
                    requester: idata_op.client().clone(),
                    response: Response::Mutation(Ok(())),
                    message_id,
                },
            })
    }

    fn handle_get_idata_req(
        &mut self,
        src: XorName,
        requester: PublicId,
        address: IDataAddress,
        message_id: MessageId,
    ) -> Option<Action> {
        if &src == address.name() {
            // The message was sent by the dst elders to us as the one who is supposed to store the
            // chunk. See the sent Get request below.
            let client = self.client_id(&message_id)?.clone();
            self.idata_holder.get_idata(address, client, message_id)
        } else {
            let client_id = requester.clone();
            let respond = |result: NdResult<IData>| {
                Some(Action::RespondToClientHandlers {
                    sender: *address.name(),
                    message: Rpc::Response {
                        requester: client_id,
                        response: Response::GetIData(result),
                        message_id,
                    },
                })
            };

            // We're acting as dst elder, received request from client handlers
            let metadata = match self.get_metadata_for(address) {
                Ok(metadata) => metadata,
                Err(error) => return respond(Err(error)),
            };

            // Can't fail
            let idata_op = unwrap!(IDataOp::new(
                requester.clone(),
                Request::GetIData(address),
                metadata.holders.clone()
            ));
            match self.idata_ops.entry(message_id) {
                Entry::Occupied(_) => respond(Err(NdError::DuplicateMessageId)),
                Entry::Vacant(vacant_entry) => {
                    let idata_op = vacant_entry.insert(idata_op);
                    Some(Action::SendToPeers {
                        sender: *address.name(),
                        targets: metadata.holders,
                        message: Rpc::Request {
                            request: idata_op.request().clone(),
                            requester,
                            message_id,
                        },
                    })
                }
            }
        }
    }

    fn handle_get_idata_resp(
        &mut self,
        sender: XorName,
        result: NdResult<IData>,
        message_id: MessageId,
    ) -> Option<Action> {
        let own_id = format!("{}", self);
        let action = self.idata_op_mut(&message_id).and_then(|idata_op| {
            idata_op.handle_get_idata_resp(sender, result, own_id, message_id)
        });
        let _ = self.remove_idata_op_if_concluded(&message_id);
        action
    }

    // Returns an iterator over all of our section's non-full adults' names, sorted by closest to
    // `target`.
    fn non_full_adults_sorted(&self, _target: &XorName) -> impl Iterator<Item = &XorName> {
        None.iter()
    }

    // Returns an iterator over all of our section's elders' names, sorted by closest to `target`.
    fn elders_sorted(&self, _target: &XorName) -> impl Iterator<Item = &XorName> {
        iter::once(self.id.name())
    }

    fn get_metadata_for(&self, address: IDataAddress) -> NdResult<ChunkMetadata> {
        match self
            .immutable_metadata
            .get::<ChunkMetadata>(&address.to_db_key())
        {
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

    fn client_id(&self, message_id: &MessageId) -> Option<&PublicId> {
        self.idata_op(message_id).map(IDataOp::client)
    }

    fn idata_op(&self, message_id: &MessageId) -> Option<&IDataOp> {
        self.idata_ops.get(message_id).or_else(|| {
            warn!(
                "{}: No current ImmutableData operation for {:?}",
                self, message_id
            );
            None
        })
    }

    fn idata_op_mut(&mut self, message_id: &MessageId) -> Option<&mut IDataOp> {
        let own_id = format!("{}", self);
        self.idata_ops.get_mut(message_id).or_else(|| {
            warn!(
                "{}: No current ImmutableData operation for {:?}",
                own_id, message_id
            );
            None
        })
    }

    /// Removes and returns the op if it has concluded.
    fn remove_idata_op_if_concluded(&mut self, message_id: &MessageId) -> Option<IDataOp> {
        let is_concluded = self
            .idata_op(message_id)
            .map(IDataOp::concluded)
            .unwrap_or(false);
        if is_concluded {
            return self.idata_ops.remove(message_id);
        }
        None
    }
}

impl Display for DestinationElder {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
