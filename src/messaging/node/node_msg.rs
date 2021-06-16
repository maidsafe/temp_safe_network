// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// FIXME: change NodeCmd defnintions to return Result and
// Error defined for the crate::node instead of client Result/Error
use crate::messaging::client::{CmdError, Error, Result};
use crate::messaging::{
    client::{
        ChunkRead, ChunkWrite, ClientSig, DataCmd as NodeDataCmd, DataExchange,
        DataQuery as NodeDataQuery,
    },
    EndUser, MessageId, MessageType, WireMsg,
};
use crate::types::{
    ActorHistory, Chunk, ChunkAddress, CreditAgreementProof, NodeAge, PublicKey, ReplicaEvent,
    SectionElders, Signature,
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use xor_name::XorName;

// -------------- Node Cmd Messages --------------
// TODO: this messages hierarchy needs to be merged into
// the NodeMessage hierarchy. It's temporarily here till
// all messages defined within sn_routing are migrated to
// this crate and within NodeMessage struct.

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeMsg {
    /// Cmds only sent internally in the network.
    NodeCmd {
        /// NodeCmd.
        cmd: NodeCmd,
        /// Message ID.
        id: MessageId,
    },
    /// An error of a NodeCmd.
    NodeCmdError {
        /// The error.
        error: NodeCmdError,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// Events only sent internally in the network.
    NodeEvent {
        /// Request.
        event: NodeEvent,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// Queries is a read-only operation.
    NodeQuery {
        /// Query.
        query: NodeQuery,
        /// Message ID.
        id: MessageId,
    },
    /// The response to a query, containing the query result.
    NodeQueryResponse {
        /// QueryResponse.
        response: NodeQueryResponse,
        /// Message ID.
        id: MessageId,
        /// ID of causing query.
        correlation_id: MessageId,
    },
    /// The returned error, from any msg handling on recipient node.
    NodeMsgError {
        /// The error.
        error: Error,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
}

impl NodeMsg {
    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        match self {
            Self::NodeCmd { id, .. }
            | Self::NodeQuery { id, .. }
            | Self::NodeEvent { id, .. }
            | Self::NodeQueryResponse { id, .. }
            | Self::NodeCmdError { id, .. }
            | Self::NodeMsgError { id, .. } => *id,
        }
    }

    /// Convenience function to deserialize a 'NodeMsg' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a node command message.
    pub fn from(bytes: Bytes) -> crate::messaging::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::Node { msg, .. } = deserialized {
            Ok(msg)
        } else {
            Err(crate::messaging::Error::FailedToParse(
                "bytes as a node command message".to_string(),
            ))
        }
    }

    /// serialize this NodeCmd message into bytes ready to be sent over the wire.
    pub fn serialize(
        &self,
        dst: XorName,
        dst_section_pk: BlsPublicKey,
        src_section_pk: Option<BlsPublicKey>,
    ) -> crate::messaging::Result<Bytes> {
        WireMsg::serialize_node_msg(self, dst, dst_section_pk, src_section_pk)
    }
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeCmd {
    /// Metadata is handled by Elders
    Metadata {
        /// The contianed command
        cmd: NodeDataCmd,
        /// Client pk and signature
        client_sig: ClientSig,
        /// Message source
        origin: EndUser,
    },
    /// Chunks are handled by Adults
    Chunks {
        /// The contianed command
        cmd: ChunkWrite,
        /// Client pk and signature
        client_sig: ClientSig,
        /// Message source
        origin: EndUser,
    },
    /// Transfers are handled by Elders
    Transfers(NodeTransferCmd),
    /// Cmds related to the running of a node.
    System(NodeSystemCmd),
}

/// Cmds related to the running of a node.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemCmd {
    /// Register a wallet for reward payouts.
    RegisterWallet(PublicKey),
    /// Notify Elders on nearing max capacity
    StorageFull {
        /// Node Id
        node_id: PublicKey,
        /// Section to which the message needs to be sent to. (NB: this is the section of the node id).
        section: XorName,
    },
    /// Replicate a given chunk at an Adult
    ReplicateChunk(Chunk),
    /// Tells the Elders to re-publish a chunk in the data section
    RepublishChunk(Chunk),
    /// When new section key, all propose a reward payout.
    ProposeRewardPayout(crate::types::RewardProposal),
    /// When proposal has been agreed, they all accumulate the reward payout.
    AccumulateRewardPayout(crate::types::RewardAccumulation),
    /// Sent to all promoted nodes (also sibling if any) after
    /// a completed transition to a new constellation.
    ReceiveExistingData {
        /// Age and reward wallets of registered nodes, keyed by node name.
        node_rewards: BTreeMap<XorName, (NodeAge, PublicKey)>,
        /// Transfer histories
        user_wallets: BTreeMap<PublicKey, ActorHistory>,
        /// Metadata
        metadata: DataExchange,
    },
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferCmd {
    ///
    PropagateTransfer(CreditAgreementProof),
}

// -------------- Node Events --------------

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeEvent {
    /// Replication completed event, emitted by a node, received by elders.
    ReplicationCompleted {
        ///
        chunk: ChunkAddress,
        /// The Elder's accumulated signature
        /// over the chunk address. This is sent back
        /// to them so that any uninformed Elder knows
        /// that this is all good.
        proof: Signature,
    },
    /// Adults ack read/write of chunks as to convey responsivity.
    ChunkWriteHandled(Result<(), CmdError>),
}

/// Query originating at a node
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeQuery {
    /// Metadata is handled by Elders
    Metadata {
        /// The actual query message
        query: NodeDataQuery,
        /// Client signature
        client_sig: ClientSig,
        /// The user that has initiated this query
        origin: EndUser,
    },
    /// Chunks are handled by Adults
    Chunks {
        /// The actual query message
        query: ChunkRead,
        /// The user that has initiated this query
        origin: EndUser,
    },
    /// Rewards handled by Elders
    Rewards(NodeRewardQuery),
    /// Transfers handled by Elders
    Transfers(NodeTransferQuery),
    /// Related to the running of a node
    System(NodeSystemQuery),
}

/// Reward query that is sent between sections.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeRewardQuery {
    /// Get the wallet key of a given node for receiving rewards
    GetNodeWalletKey(XorName),
    /// A new Section Actor share (i.e. a new Elder) needs to query
    /// its peer Elders for the replicas' public key set
    /// and the history of events of the section wallet.
    GetSectionWalletHistory,
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferQuery {
    /// Replicas starting up
    /// need to query for events of
    /// the existing Replicas. (Sent to the other Elders).
    GetReplicaEvents,
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemQuery {
    /// On Elder change, all Elders need to query
    /// network for the new wallet's replicas' public key set
    GetSectionElders,
    /// Acquire the chunk from current holders for replication.
    /// providing the address of the chunk to be replicated.
    GetChunk(ChunkAddress),
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemQueryResponse {
    /// On Elder change, all Elders need to query
    /// network for the new wallet's replicas' public key set
    GetSectionElders(SectionElders),
    /// Respond elders with the requested chunk for replication
    GetChunk(Chunk),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeQueryResponse {
    ///
    Data(NodeDataQueryResponse),
    ///
    Transfers(NodeTransferQueryResponse),
    ///
    System(NodeSystemQueryResponse),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferQueryResponse {
    /// Replicas starting up
    /// need to query for events of
    /// the existing Replicas.
    GetReplicaEvents(Result<Vec<ReplicaEvent>>),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeDataQueryResponse {
    /// Elder to Adult Get.
    GetChunk(Result<Chunk>),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeCmdError {
    ///
    Data(NodeDataError),
    ///
    Transfers(NodeTransferError),
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeDataError {
    ///
    ChunkReplication {
        ///
        head_address: ChunkAddress,
        ///
        error: Error,
    },
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferError {
    /// The error of propagation of TransferRegistered event.
    TransferPropagation(Error),
}
