// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    node::node_ops::{
        AdultDuty, AdultDuty::NoOp as AdultNoOp, ChunkReplicationCmd, ChunkReplicationDuty,
        ChunkReplicationQuery, ChunkStoreDuty, ElderDuty, GatewayDuty, MetadataDuty, NodeDuty,
        NodeOperation, ReceivedMsg, RewardCmd, RewardDuty, RewardQuery, TransferCmd, TransferDuty,
        TransferQuery,
    },
    Error, Network, Result,
};
use log::{debug, error, info, trace};
use sn_messaging::client::{
    Cmd, DataQuery, Message, MessageId, NodeCmd, NodeDataCmd, NodeDataQuery, NodeDataQueryResponse,
    NodeEvent, NodeQuery, NodeQueryResponse, NodeRewardQuery, NodeRewardQueryResponse,
    NodeSystemCmd, NodeTransferCmd, NodeTransferQuery, NodeTransferQueryResponse, Query,
};

use sn_routing::{DstLocation, SrcLocation, MIN_AGE};

// NB: This approach is not entirely good, so will need to be improved.

/// Evaluates remote msgs from the network,
/// i.e. not msgs sent directly from a client.
pub struct ReceivedMsgAnalysis {
    network: Network,
}

impl ReceivedMsgAnalysis {
    pub fn new(network: Network) -> Self {
        Self { network }
    }

    pub async fn is_dst_for(&self, msg: &ReceivedMsg) -> Result<bool> {
        let are_we_dst = if let DstLocation::Node(name) = msg.dst {
            name == self.network.our_name().await
        } else {
            false
        };
        let are_we_origin = self.are_we_origin(&msg).await;
        let is_genesis_node_msg_to_self = are_we_origin && self.is_genesis_request().await;
        let are_we_handler_for_dst = self.self_is_handler_for(&msg.dst).await;
        let is_genesis_section_msg_to_section =
            msg.dst.is_section() && self.network.our_prefix().await.is_empty();

        let is_dst = are_we_dst
            || (are_we_handler_for_dst && !are_we_origin)
            || is_genesis_node_msg_to_self
            || is_genesis_section_msg_to_section;
        debug!("is_dst: {}", is_dst);
        Ok(is_dst)
    }

    async fn is_genesis_request(&self) -> bool {
        let elders = self.network.our_elder_names().await;
        if elders.len() == 1 {
            elders.contains(&self.network.our_name().await)
        } else {
            false
        }
    }

    async fn are_we_origin(&self, msg: &ReceivedMsg) -> bool {
        if let SrcLocation::Node(origin) = msg.src {
            origin == self.network.our_name().await
        } else {
            false
        }
    }

    pub async fn evaluate(&mut self, msg: ReceivedMsg) -> Result<NodeOperation> {
        // match self.try_messaging(&msg).await? {
        //     // Identified as an outbound msg, to be sent on the wire.
        //     NodeMessagingDuty::NoOp => (),
        //     op => return Ok(op.into()),
        // };
        if !self.is_dst_for(&msg).await? {
            error!(
                "Unknown message destination: {:?}, for {:?}",
                msg.dst,
                msg.id(),
            );
            return Err(Error::Logic("Unknown message destination".to_string()));
        }
        match self.try_system_cmd(&msg).await? {
            NodeOperation::NoOp => (),
            op => return Ok(op),
        };
        match self.try_client_entry(&msg).await? {
            // Client auth cmd finalisation (Temporarily handled here, will be at app layer (Authenticator)).
            // The auth cmd has been agreed by the Gateway section.
            // (All other client msgs are handled when received from client).
            GatewayDuty::NoOp => (),
            op => return Ok(op.into()),
        };
        match self.try_transfers(&msg).await? {
            TransferDuty::NoOp => (),
            op => return Ok(op.into()),
        };
        match self.try_metadata(&msg).await? {
            // Accumulated msg from `Payment`!
            MetadataDuty::NoOp => (),
            op => return Ok(op.into()),
        };
        match self.try_adult(&msg).await? {
            // Accumulated msg from `Metadata`!
            AdultNoOp => (),
            op => return Ok(op.into()),
        };
        match self.try_chunk_replication(&msg).await? {
            // asdf aSdF AsDf `..`!
            AdultNoOp => (),
            op => return Ok(op.into()),
        }
        match self.try_rewards(&msg).await? {
            // Identified as a Rewards msg
            RewardDuty::NoOp => (),
            op => return Ok(op.into()),
        };
        match self.try_node_duties(&msg).await? {
            // Identified as a NodeCfg msg
            NodeDuty::NoOp => (),
            op => return Ok(op.into()),
        }
        error!("Unknown message destination: {:?}", msg.id());
        Err(Error::Logic("Unknown message destination".to_string()))
    }

    async fn try_system_cmd(&self, msg: &ReceivedMsg) -> Result<NodeOperation> {
        trace!("Msg analysis: try_system_cmd..");
        use NodeCmd::*;
        use NodeSystemCmd::*;
        // Check if it a message from adult
        // todo: more verifications..
        if msg.src.is_section() {
            return Ok(NodeOperation::NoOp);
        }
        if let Message::NodeCmd {
            cmd: System(StorageFull { node_id, .. }),
            ..
        } = &msg.msg
        {
            Ok(ElderDuty::StorageFull { node_id: *node_id }.into())
        } else {
            Ok(NodeOperation::NoOp)
        }
    }

    // async fn try_messaging(&self, msg: &ReceivedMsg) -> Result<NodeMessagingDuty> {
    //     trace!("Msg analysis: try_messaging..");
    //     if !self.is_dst_for(msg).await? {
    //         Ok(NodeMessagingDuty::SendToSection {
    //             msg: Msg {
    //                 msg: msg.msg.clone(),
    //                 dst: msg.dst,
    //             },
    //             as_node: !msg.src.is_section(),
    //         })
    //     } else {
    //         Ok(NodeMessagingDuty::NoOp)
    //     }
    // }

    async fn try_node_duties(&self, msg: &ReceivedMsg) -> Result<NodeDuty> {
        trace!("Msg analysis: try_node_duties..");
        // From Transfer module, we get
        // `CatchUpWithSectionWallet` query response.
        use NodeQueryResponse::Transfers;
        use NodeTransferQueryResponse::*;
        match &msg.msg {
            Message::NodeCmd {
                cmd: NodeCmd::System(NodeSystemCmd::ProposeGenesis { credit, sig }),
                ..
            } => Ok(NodeDuty::ReceiveGenesisProposal {
                credit: credit.clone(),
                sig: sig.clone(),
            }),
            Message::NodeCmd {
                cmd: NodeCmd::System(NodeSystemCmd::AccumulateGenesis { signed_credit, sig }),
                ..
            } => Ok(NodeDuty::ReceiveGenesisAccumulation {
                signed_credit: signed_credit.clone(),
                sig: sig.clone(),
            }),
            Message::NodeQueryResponse {
                response: Transfers(CatchUpWithSectionWallet(Ok(info))),
                ..
            } => {
                info!("We have a CatchUpWithSectionWallet query response!");
                Ok(NodeDuty::InitSectionWallet(info.clone()))
            }
            Message::NodeEvent {
                event: NodeEvent::SectionPayoutRegistered { from, to },
                ..
            } => Ok(NodeDuty::FinishElderChange {
                previous_key: *from,
                new_key: *to,
            }),
            _ => Ok(NodeDuty::NoOp),
        }
    }

    async fn try_client_entry(&self, msg: &ReceivedMsg) -> Result<GatewayDuty> {
        trace!("Msg analysis: try_client_entry..");
        let is_our_client_msg = self.self_is_handler_for(&msg.dst).await;

        if is_our_client_msg {
            Ok(GatewayDuty::FindClientFor(crate::node::node_ops::Msg {
                msg: msg.msg.clone(),
                dst: DstLocation::Client(*msg.dst.name().unwrap()),
            }))
        } else {
            Ok(GatewayDuty::NoOp)
        }
    }

    /// After the data write sent from Payment Elders has been
    /// accumulated (can be seen since the sender is `Section`),
    /// it is time to actually carry out the write operation.
    async fn try_metadata(&self, msg: &ReceivedMsg) -> Result<MetadataDuty> {
        trace!("Msg analysis: try_metadata..");
        let is_data_query = matches!(msg.msg, Message::Query {
            query: Query::Data(_),
            ..
        });
        let is_data_cmd = matches!(msg.msg, Message::Cmd {
            cmd: Cmd::Data { .. },
            ..
        });

        let origin = *msg.src.to_dst().name().unwrap();

        let duty = if is_data_query {
            MetadataDuty::ProcessRead {
                msg: msg.msg.clone(),
                origin,
            } // TODO: Fix these for type safety
        } else if is_data_cmd {
            MetadataDuty::ProcessWrite {
                msg: msg.msg.clone(),
                origin,
            } // TODO: Fix these for type safety
        } else {
            return Ok(MetadataDuty::NoOp);
        };
        Ok(duty)
    }

    /// When the write requests from Elders has been accumulated
    /// at an Adult, it is time to carry out the write operation.
    async fn try_adult(&self, msg: &ReceivedMsg) -> Result<AdultDuty> {
        trace!("Msg analysis: try_adult..");

        // TODO: Should not accumulate queries, just pass them through.
        let is_chunk_query = matches!(msg.msg, Message::Query {
            query: Query::Data(DataQuery::Blob(_)),
            ..
        });

        let is_data_cmd = matches!(msg.msg,
        Message::NodeCmd {
            cmd: NodeCmd::Data(NodeDataCmd::Blob(_)),
            ..
        });

        use AdultDuty::*;
        use ChunkStoreDuty::*;
        let duty = if is_data_cmd {
            RunAsChunkStore(WriteChunk(msg.clone()))
        } else if is_chunk_query {
            RunAsChunkStore(ReadChunk(msg.clone()))
        } else {
            return Ok(AdultNoOp);
        };
        Ok(duty)
    }

    async fn try_chunk_replication(&self, msg: &ReceivedMsg) -> Result<AdultDuty> {
        trace!("Msg analysis: try_chunk_replication..");
        use ChunkReplicationDuty::*;

        use ChunkReplicationCmd::*;
        use ChunkReplicationQuery::*;
        let chunk_replication = match &msg.msg {
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
                Some(ProcessCmd {
                    cmd: ReplicateChunk {
                        current_holders: current_holders.clone(),
                        address: *address,
                        //section_authority: msg.most_recent_sender().clone(),
                    },
                    msg_id: *id,
                    origin: *msg.src.to_dst().name().unwrap(),
                })
            }
            Message::NodeQueryResponse {
                response: NodeQueryResponse::Data(NodeDataQueryResponse::GetChunk(result)),
                correlation_id,
                ..
            } => {
                let blob = result.to_owned()?;
                info!("Verifying GetChunk NodeQueryResponse!");
                // Recreate original MessageId from Section
                let msg_id =
                    MessageId::combine(vec![*blob.address().name(), self.network.our_name().await]);
                if msg_id == *correlation_id {
                    Some(ProcessCmd {
                        cmd: StoreReplicatedBlob(blob),
                        msg_id,
                        origin: *msg.src.to_dst().name().unwrap(),
                    })
                } else {
                    info!("Given blob is incorrect.");
                    None
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
                let proof_chain = self.network.our_history().await;

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

                // // Verify that the message was sent from the section
                // let verify_section_authority = section_authority.verify(&message.serialize()?);

                // let given_section_pk = &section_authority
                //     .id()
                //     .public_key()
                //     .bls()
                //     .ok_or_else(|| Error::Logic("Section Key cannot be non-BLS".to_string()))?;

                // // Verify that the original ReplicateChunk cmd was sent with SectionAuthority
                // if section_authority.is_section()
                //     && verify_section_authority
                //     && proof_chain.has_key(given_section_pk)
                // {
                info!("Internal ChunkReplicationQuery ProcessQuery");
                Some(ProcessQuery {
                    query: GetChunk(*address),
                    msg_id,
                    origin: *msg.src.to_dst().name().unwrap(),
                })
                // } else {
                //     None
                // }
            }
            _ => None,
        };

        use AdultDuty::*;
        let duty = if let Some(request) = chunk_replication {
            RunAsChunkReplication(request)
        } else {
            return Ok(AdultNoOp);
        };
        Ok(duty)
    }

    async fn try_rewards(&self, msg: &ReceivedMsg) -> Result<RewardDuty> {
        match self.try_nonacc_rewards(msg).await? {
            RewardDuty::NoOp => (),
            op => return Ok(op),
        };
        match self.try_accumulated_rewards(msg).await? {
            RewardDuty::NoOp => (),
            op => return Ok(op),
        };
        self.try_wallet_register(msg).await
    }

    async fn try_wallet_register(&self, msg: &ReceivedMsg) -> Result<RewardDuty> {
        trace!("Msg analysis: try_wallet_register..");

        match &msg.msg {
            Message::NodeCmd {
                cmd: NodeCmd::System(NodeSystemCmd::RegisterWallet { wallet, .. }),
                id,
                ..
            } => Ok(RewardDuty::ProcessCmd {
                cmd: RewardCmd::SetNodeWallet {
                    wallet_id: *wallet,
                    node_id: *msg.src.to_dst().name().unwrap(), //msg.origin.address().xorname(),
                },
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            _ => Ok(RewardDuty::NoOp),
        }
    }

    // Check non-accumulated reward msgs.
    async fn try_nonacc_rewards(&self, msg: &ReceivedMsg) -> Result<RewardDuty> {
        trace!("Msg analysis: try_nonacc_rewards..");

        use NodeRewardQuery::GetNodeWalletId;
        // GetNodeWalletId
        // does not need accumulation since its accumulated in the domain logic.

        match &msg.msg {
            Message::NodeQuery {
                query:
                    NodeQuery::Rewards(GetNodeWalletId {
                        old_node_id,
                        new_node_id,
                    }),
                id,
            } => Ok(RewardDuty::ProcessQuery {
                query: RewardQuery::GetNodeWalletId {
                    old_node_id: *old_node_id,
                    new_node_id: *new_node_id,
                },
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeEvent {
                event: NodeEvent::SectionPayoutValidated(validation),
                id,
                ..
            } => Ok(RewardDuty::ProcessCmd {
                cmd: RewardCmd::ReceivePayoutValidation(validation.clone()),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            _ => Ok(RewardDuty::NoOp),
        }

        // SectionPayoutValidated
        // does not need accumulation since its accumulated in the domain logic.
    }

    // Check accumulated reward msgs.
    async fn try_accumulated_rewards(&self, msg: &ReceivedMsg) -> Result<RewardDuty> {
        trace!("Msg analysis: try_accumulated_rewards..");
        use NodeQueryResponse::Rewards;
        use NodeQueryResponse::Transfers;
        use NodeRewardQueryResponse::GetNodeWalletId;
        use NodeTransferQueryResponse::*;
        match &msg.msg {
            Message::NodeQueryResponse {
                response: Rewards(GetNodeWalletId(Ok((wallet_id, new_node_id)))),
                id,
                ..
            } => Ok(RewardDuty::ProcessCmd {
                cmd: RewardCmd::ActivateNodeRewards {
                    id: *wallet_id,
                    node_id: *new_node_id,
                },
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeQueryResponse {
                response: Transfers(GetNewSectionWallet(result)),
                id,
                ..
            } => Ok(RewardDuty::ProcessCmd {
                cmd: RewardCmd::InitiateSectionWallet(result.clone()?),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeQueryResponse {
                response: Transfers(CatchUpWithSectionWallet(result)),
                id,
                ..
            } => Ok(RewardDuty::ProcessCmd {
                cmd: RewardCmd::InitiateSectionWallet(result.clone()?),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            _ => Ok(RewardDuty::NoOp),
        }
    }

    // Check internal transfer cmds.
    async fn try_transfers(&self, msg: &ReceivedMsg) -> Result<TransferDuty> {
        match self.try_nonacc_transfers(msg).await? {
            TransferDuty::NoOp => (),
            op => return Ok(op),
        };
        self.try_accumulated_transfers(msg).await
    }

    // Check accumulated transfer msgs.
    async fn try_accumulated_transfers(&self, msg: &ReceivedMsg) -> Result<TransferDuty> {
        trace!("Msg analysis: try_accumulated_transfers..");
        use NodeQueryResponse::Transfers;
        use NodeTransferQueryResponse::*;
        match &msg.msg {
            Message::NodeQueryResponse {
                response: Transfers(GetReplicaEvents(events)),
                id,
                ..
            } => Ok(TransferDuty::ProcessCmd {
                cmd: TransferCmd::InitiateReplica(events.clone()?),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            _ => Ok(TransferDuty::NoOp),
        }
    }

    // Check non accumulated transfer msgss.
    async fn try_nonacc_transfers(&self, msg: &ReceivedMsg) -> Result<TransferDuty> {
        trace!("Msg analysis: try_nonacc_transfers..");
        // // From Transfer module we get `PropagateTransfer` and `GetReplicaEvents`.
        use NodeTransferCmd::*;
        use NodeTransferQuery::CatchUpWithSectionWallet;
        use NodeTransferQuery::*;
        match &msg.msg {
            Message::NodeCmd {
                cmd: NodeCmd::Transfers(PropagateTransfer(proof)),
                id,
            } => Ok(TransferDuty::ProcessCmd {
                cmd: TransferCmd::PropagateTransfer(proof.credit_proof()),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeQuery {
                query: NodeQuery::Transfers(GetReplicaEvents(public_key)),
                id,
            } => {
                // This comparison is a good example of the need to use `lazy messaging`,
                // as to handle that the expected public key is not the same as the current.
                if let Some(section_pk) = self.network.section_public_key().await {
                    if public_key == &section_pk {
                        Ok(TransferDuty::ProcessQuery {
                            query: TransferQuery::GetReplicaEvents,
                            msg_id: *id,
                            origin: *msg.src.to_dst().name().unwrap(),
                        })
                    } else {
                        error!("Unexpected public key!");
                        Err(Error::Logic("Unexpected PK".to_string()))
                    }
                } else {
                    error!("No section public key found!");
                    Err(Error::Logic("No section PK found".to_string()))
                }
            }
            Message::NodeCmd {
                cmd: NodeCmd::Transfers(ValidateSectionPayout(signed_transfer)),
                id,
            } => Ok(TransferDuty::ProcessCmd {
                cmd: TransferCmd::ValidateSectionPayout(signed_transfer.clone()),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeCmd {
                cmd: NodeCmd::Transfers(RegisterSectionPayout(debit_agreement)),
                id,
            } => Ok(TransferDuty::ProcessCmd {
                cmd: TransferCmd::RegisterSectionPayout(debit_agreement.clone()),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeQuery {
                query: NodeQuery::Transfers(CatchUpWithSectionWallet(public_key)),
                id,
            } => Ok(TransferDuty::ProcessQuery {
                query: TransferQuery::CatchUpWithSectionWallet(*public_key),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeQuery {
                query: NodeQuery::Transfers(GetNewSectionWallet(public_key)),
                id,
            } => Ok(TransferDuty::ProcessQuery {
                query: TransferQuery::GetNewSectionWallet(*public_key),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeQuery {
                query: NodeQuery::Transfers(CatchUpWithSectionWallet(public_key)),
                id,
            } => Ok(TransferDuty::ProcessQuery {
                query: TransferQuery::CatchUpWithSectionWallet(*public_key),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            Message::NodeQuery {
                query: NodeQuery::Transfers(GetNewSectionWallet(public_key)),
                id,
            } => Ok(TransferDuty::ProcessQuery {
                query: TransferQuery::GetNewSectionWallet(*public_key),
                msg_id: *id,
                origin: *msg.src.to_dst().name().unwrap(),
            }),
            _ => Ok(TransferDuty::NoOp),
        }

        // From Rewards module, we get
        // `ValidateSectionPayout`, `RegisterSectionPayout`, `CatchUpWithSectionWallet` and `GetNewSectionWallet`.
    }

    async fn self_is_handler_for(&self, dst: &DstLocation) -> bool {
        if let Some(name) = dst.name() {
            self.network.matches_our_prefix(*name).await
        } else {
            false
        }
    }

    pub async fn is_elder(&self) -> bool {
        self.network.is_elder().await
    }

    pub async fn is_adult(&self) -> bool {
        !self.network.is_elder().await && self.network.age().await > MIN_AGE
    }

    pub async fn no_of_elders(&self) -> usize {
        self.network.our_elder_addresses().await.len()
    }

    pub async fn no_of_adults(&self) -> usize {
        self.network.our_adults().await.len()
    }
}
