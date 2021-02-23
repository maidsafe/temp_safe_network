// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    node::node_ops::{
        AdultDuty, ChunkReplicationCmd, ChunkReplicationDuty, ChunkReplicationQuery,
        ChunkStoreDuty, ElderDuty, GatewayDuty, MetadataDuty, NodeDuty, NodeOperation, ReceivedMsg,
        RewardCmd, RewardDuty, RewardQuery, TransferCmd, TransferDuty, TransferQuery,
    },
    AdultState, ElderState, Error, NodeState, Result,
};
use log::{debug, error, info, trace};
use sn_messaging::{
    client::{
        Cmd, DataQuery, Message, MessageId, NodeCmd, NodeDataCmd, NodeDataQuery,
        NodeDataQueryResponse, NodeEvent, NodeQuery, NodeQueryResponse, NodeRewardQuery,
        NodeRewardQueryResponse, NodeSystemCmd, NodeTransferCmd, NodeTransferQuery,
        NodeTransferQueryResponse, Query,
    },
    DstLocation, SrcLocation,
};

use sn_routing::Prefix;

// NB: This approach is not entirely good, so will need to be improved.

/// Evaluates remote msgs from the network,
/// i.e. not msgs sent directly from a client.
pub struct ReceivedMsgAnalysis {
    state: NodeState,
}

impl ReceivedMsgAnalysis {
    pub fn new(state: NodeState) -> Self {
        Self { state }
    }

    pub async fn is_dst_for(&self, msg: &ReceivedMsg) -> Result<bool> {
        let are_we_dst = if let DstLocation::Node(name) = msg.dst {
            name == self.state.node_name()
        } else {
            false
        };
        let are_we_origin = self.are_we_origin(&msg).await;
        let is_genesis_node_msg_to_self = are_we_origin && self.is_genesis_request().await;
        let are_we_handler_for_dst = self.self_is_handler_for(&msg.dst);
        let is_genesis_section_msg_to_section =
            msg.dst.is_section() && self.is_elder() && self.prefix().is_empty();

        let is_dst = are_we_dst
            || (are_we_handler_for_dst && !are_we_origin)
            || is_genesis_node_msg_to_self
            || is_genesis_section_msg_to_section;
        debug!("is_dst: {}", is_dst);
        Ok(is_dst)
    }

    async fn is_genesis_request(&self) -> bool {
        if let Ok(state) = self.elder_state() {
            let elders = state.elder_names();
            if elders.len() == 1 {
                return elders.contains(&state.node_name());
            }
        }

        false
    }

    async fn are_we_origin(&self, msg: &ReceivedMsg) -> bool {
        if let SrcLocation::Node(origin) = msg.src {
            origin == self.state.node_name()
        } else {
            false
        }
    }

    pub async fn evaluate(&self, msg: &ReceivedMsg) -> Result<NodeOperation> {
        use AdultDuty::*;
        use ChunkStoreDuty::*;
        use DstLocation::*;

        //let origin = msg.dst.name();

        let res = match &msg.dst {
            Direct => unimplemented!(),
            Node(_name) => {
                match &msg.msg {
                    //
                    // ------ system cmd ------
                    Message::NodeCmd {
                        cmd: NodeCmd::System(NodeSystemCmd::StorageFull { node_id, .. }),
                        ..
                    } => ElderDuty::StorageFull { node_id: *node_id }.into(),
                    //
                    // ------ node duties ------
                    Message::NodeCmd {
                        cmd: NodeCmd::System(NodeSystemCmd::ProposeGenesis { credit, sig }),
                        ..
                    } => NodeDuty::ReceiveGenesisProposal {
                        credit: credit.clone(),
                        sig: sig.clone(),
                    }
                    .into(),
                    Message::NodeCmd {
                        cmd:
                            NodeCmd::System(NodeSystemCmd::AccumulateGenesis { signed_credit, sig }),
                        ..
                    } => NodeDuty::ReceiveGenesisAccumulation {
                        signed_credit: signed_credit.clone(),
                        sig: sig.clone(),
                    }
                    .into(),
                    Message::NodeQueryResponse {
                        response:
                            NodeQueryResponse::Transfers(
                                NodeTransferQueryResponse::CatchUpWithSectionWallet(Ok(info)),
                            ),
                        ..
                    } => {
                        info!("We have a CatchUpWithSectionWallet query response!");
                        NodeDuty::InitSectionWallet(info.clone()).into()
                    }
                    Message::NodeEvent {
                        event: NodeEvent::SectionPayoutRegistered { from, to },
                        ..
                    } => NodeDuty::FinishElderChange {
                        previous_key: *from,
                        new_key: *to,
                    }
                    .into(),

                    //
                    // ------ adult ------
                    Message::Query {
                        query: Query::Data(DataQuery::Blob(_)),
                        ..
                    } => RunAsChunkStore(ReadChunk(msg.clone())).into(),
                    Message::NodeCmd {
                        cmd: NodeCmd::Data(NodeDataCmd::Blob(_)),
                        ..
                    } => RunAsChunkStore(WriteChunk(msg.clone())).into(),
                    //
                    // ------ chunk replication ------
                    Message::NodeCmd {
                        cmd:
                            NodeCmd::Data(NodeDataCmd::ReplicateChunk {
                                address,
                                current_holders,
                                ..
                            }),
                        id,
                        ..
                    } => {
                        //info!("Origin of Replicate Chunk: {:?}", msg.origin.clone());
                        RunAsChunkReplication(ChunkReplicationDuty::ProcessCmd {
                            cmd: ChunkReplicationCmd::ReplicateChunk {
                                current_holders: current_holders.clone(),
                                address: *address,
                                //section_authority: msg.most_recent_sender().clone(),
                            },
                            msg_id: *id,
                            origin: msg.src,
                        })
                        .into()
                    }
                    Message::NodeQueryResponse {
                        response: NodeQueryResponse::Data(NodeDataQueryResponse::GetChunk(result)),
                        correlation_id,
                        ..
                    } => {
                        let blob = result.to_owned()?;
                        info!("Verifying GetChunk NodeQueryResponse!");
                        // Recreate original MessageId from Section
                        let msg_id = MessageId::combine(vec![
                            *blob.address().name(),
                            self.state.node_name(),
                        ]);
                        if msg_id == *correlation_id {
                            RunAsChunkReplication(ChunkReplicationDuty::ProcessCmd {
                                cmd: ChunkReplicationCmd::StoreReplicatedBlob(blob),
                                msg_id,
                                origin: msg.src,
                            })
                            .into()
                        } else {
                            info!("Given blob is incorrect.");
                            panic!()
                        }
                    }
                    Message::NodeQuery {
                        query:
                            NodeQuery::Data(NodeDataQuery::GetChunk {
                                //section_authority,
                                new_holder,
                                address,
                                current_holders,
                            }),
                        ..
                    } => {
                        info!("Verifying GetChunk query!");
                        let proof_chain = self.adult_state()?.section_proof_chain();

                        // Recreate original MessageId from Section
                        let msg_id = MessageId::combine(vec![*address.name(), *new_holder]);

                        // Recreate cmd that was sent by the section.
                        let message = Message::NodeCmd {
                            cmd: NodeCmd::Data(NodeDataCmd::ReplicateChunk {
                                new_holder: *new_holder,
                                address: *address,
                                current_holders: current_holders.clone(),
                            }),
                            id: msg_id,
                        };

                        info!("Internal ChunkReplicationQuery ProcessQuery");
                        RunAsChunkReplication(ChunkReplicationDuty::ProcessQuery {
                            query: ChunkReplicationQuery::GetChunk(*address),
                            msg_id,
                            origin: msg.src,
                        })
                        .into()
                    }
                    //
                    // ------ wallet register ------
                    Message::NodeCmd {
                        cmd: NodeCmd::System(NodeSystemCmd::RegisterWallet { wallet, .. }),
                        id,
                        ..
                    } => RewardDuty::ProcessCmd {
                        cmd: RewardCmd::SetNodeWallet {
                            wallet_id: *wallet,
                            node_id: msg.src.to_dst().name().unwrap(),
                        },
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    //
                    // ------ nonacc rewards ------
                    Message::NodeQuery {
                        query:
                            NodeQuery::Rewards(NodeRewardQuery::GetNodeWalletId {
                                old_node_id,
                                new_node_id,
                            }),
                        id,
                    } => RewardDuty::ProcessQuery {
                        query: RewardQuery::GetNodeWalletId {
                            old_node_id: *old_node_id,
                            new_node_id: *new_node_id,
                        },
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    Message::NodeEvent {
                        event: NodeEvent::SectionPayoutValidated(validation),
                        id,
                        ..
                    } => RewardDuty::ProcessCmd {
                        cmd: RewardCmd::ReceivePayoutValidation(validation.clone()),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    //
                    // ------ nacc rewards ------
                    Message::NodeQueryResponse {
                        response:
                            NodeQueryResponse::Rewards(NodeRewardQueryResponse::GetNodeWalletId(Ok((
                                wallet_id,
                                new_node_id,
                            )))),
                        id,
                        ..
                    } => RewardDuty::ProcessCmd {
                        cmd: RewardCmd::ActivateNodeRewards {
                            id: *wallet_id,
                            node_id: *new_node_id,
                        },
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    Message::NodeQueryResponse {
                        response:
                            NodeQueryResponse::Transfers(
                                NodeTransferQueryResponse::GetNewSectionWallet(result),
                            ),
                        id,
                        ..
                    } => RewardDuty::ProcessCmd {
                        cmd: RewardCmd::InitiateSectionWallet(result.clone()?),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    Message::NodeQueryResponse {
                        response:
                            NodeQueryResponse::Transfers(
                                NodeTransferQueryResponse::CatchUpWithSectionWallet(result),
                            ),
                        id,
                        ..
                    } => RewardDuty::ProcessCmd {
                        cmd: RewardCmd::InitiateSectionWallet(result.clone()?),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    //
                    // ------ acc transfers ------
                    Message::NodeQueryResponse {
                        response:
                            NodeQueryResponse::Transfers(NodeTransferQueryResponse::GetReplicaEvents(
                                events,
                            )),
                        id,
                        ..
                    } => TransferDuty::ProcessCmd {
                        cmd: TransferCmd::InitiateReplica(events.clone()?),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    //
                    // ------ nonacc transfers ------
                    Message::NodeCmd {
                        cmd: NodeCmd::Transfers(NodeTransferCmd::PropagateTransfer(proof)),
                        id,
                    } => TransferDuty::ProcessCmd {
                        cmd: TransferCmd::PropagateTransfer(proof.credit_proof()),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    Message::NodeQuery {
                        query: NodeQuery::Transfers(NodeTransferQuery::GetReplicaEvents(public_key)),
                        id,
                    } => {
                        // This comparison is a good example of the need to use `lazy messaging`,
                        // as to handle that the expected public key is not the same as the current.
                        if public_key == &self.elder_state()?.section_public_key() {
                            TransferDuty::ProcessQuery {
                                query: TransferQuery::GetReplicaEvents,
                                msg_id: *id,
                                origin: msg.src,
                            }
                            .into()
                        } else {
                            error!("Unexpected public key!");
                            return Err(Error::Logic("Unexpected PK".to_string()));
                        }
                    }
                    Message::NodeCmd {
                        cmd:
                            NodeCmd::Transfers(NodeTransferCmd::ValidateSectionPayout(signed_transfer)),
                        id,
                    } => TransferDuty::ProcessCmd {
                        cmd: TransferCmd::ValidateSectionPayout(signed_transfer.clone()),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    Message::NodeCmd {
                        cmd:
                            NodeCmd::Transfers(NodeTransferCmd::RegisterSectionPayout(debit_agreement)),
                        id,
                    } => TransferDuty::ProcessCmd {
                        cmd: TransferCmd::RegisterSectionPayout(debit_agreement.clone()),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    Message::NodeQuery {
                        query:
                            NodeQuery::Transfers(NodeTransferQuery::CatchUpWithSectionWallet(
                                public_key,
                            )),
                        id,
                    } => TransferDuty::ProcessQuery {
                        query: TransferQuery::CatchUpWithSectionWallet(*public_key),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    Message::NodeQuery {
                        query:
                            NodeQuery::Transfers(NodeTransferQuery::GetNewSectionWallet(public_key)),
                        id,
                    } => TransferDuty::ProcessQuery {
                        query: TransferQuery::GetNewSectionWallet(*public_key),
                        msg_id: *id,
                        origin: msg.src,
                    }
                    .into(),
                    // Message::NodeQuery {
                    //     query:
                    //         NodeQuery::Transfers(NodeTransferQuery::CatchUpWithSectionWallet(
                    //             public_key,
                    //         )),
                    //     id,
                    // } => TransferDuty::ProcessQuery {
                    //     query: TransferQuery::CatchUpWithSectionWallet(*public_key),
                    //     msg_id: *id,
                    //     origin: msg.src,
                    // }
                    // .into(),
                    // Message::NodeQuery {
                    //     query:
                    //         NodeQuery::Transfers(NodeTransferQuery::GetNewSectionWallet(public_key)),
                    //     id,
                    // } => TransferDuty::ProcessQuery {
                    //     query: TransferQuery::GetNewSectionWallet(*public_key),
                    //     msg_id: *id,
                    //     origin: msg.src,
                    // }
                    // .into(),
                    _ => unimplemented!(),
                }
            }
            Section(_name) => {
                unimplemented!()
            }
            User(user) => {
                match &msg.msg {
                    //
                    // ------ metadata ------
                    Message::Query {
                        query: Query::Data(_),
                        ..
                    } => MetadataDuty::ProcessRead {
                        msg: msg.msg.clone(),
                        origin: *user,
                    }
                    .into(),
                    Message::Cmd {
                        cmd: Cmd::Data { .. },
                        ..
                    } => MetadataDuty::ProcessWrite {
                        msg: msg.msg.clone(),
                        origin: *user,
                    }
                    .into(),

                    _ => unimplemented!(),
                }
            }
        };

        Ok(res)
    }

    // async fn try_client_entry(&self, msg: &ReceivedMsg) -> Result<GatewayDuty> {
    //     trace!("Msg analysis: try_client_entry..");
    //     let is_our_client_msg = self.self_is_handler_for(&msg.dst).await;

    //     if is_our_client_msg {
    //         Ok(GatewayDuty::FindClientFor(crate::node::node_ops::Msg {
    //             msg: msg.msg.clone(),
    //             dst: DstLocation::Client(*msg.dst.name().unwrap()),
    //         }))
    //     } else {
    //         Ok(GatewayDuty::NoOp)
    //     }
    // }

    fn self_is_handler_for(&self, dst: &DstLocation) -> bool {
        if let Some(name) = dst.name() {
            self.is_elder() && self.prefix().matches(&name)
        } else {
            false
        }
    }

    fn is_elder(&self) -> bool {
        matches!(self.state, NodeState::Elder(_))
    }

    fn is_adult(&self) -> bool {
        matches!(self.state, NodeState::Adult(_))
    }

    // fn no_of_elders(&self) -> Result<usize> {
    //     if let NodeState::Elder(state) = self.state {
    //         Ok(state.elder_names().len())
    //     } else {
    //         Err(Error::InvalidOperation)
    //     }
    // }

    fn elder_state(&self) -> Result<&ElderState> {
        if let NodeState::Elder(state) = &self.state {
            Ok(state)
        } else {
            Err(Error::InvalidOperation)
        }
    }

    fn adult_state(&self) -> Result<&AdultState> {
        if let NodeState::Adult(state) = &self.state {
            Ok(state)
        } else {
            Err(Error::InvalidOperation)
        }
    }

    // async fn no_of_adults(&self) -> Result<usize> {
    //     if let NodeState::Elder(state) = self.state {
    //         Ok(state.adults().await.len())
    //     } else {
    //         Err(Error::InvalidOperation)
    //     }
    // }

    fn prefix(&self) -> &Prefix {
        match &self.state {
            NodeState::Elder(state) => state.prefix(),
            NodeState::Adult(_state) => unimplemented!(), // state.prefix(),
        }
    }
}
