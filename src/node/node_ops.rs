// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    client::{
        ChunkRead, ChunkWrite, ClientMsg, DataCmd, DataExchange, DataQuery, ProcessingError,
        QueryResponse, SupportingInfo,
    },
    node::NodeMsg,
    ClientSigned, DstLocation, EndUser, MessageId, SrcLocation,
};
use crate::routing::Prefix;
use crate::types::{Chunk, PublicKey};
use std::{
    collections::BTreeSet,
    fmt::{Debug, Formatter},
};
use xor_name::XorName;

/// Internal messages are what is passed along
/// within a node, between the entry point and
/// exit point of remote messages.
/// In other words, when communication from another
/// participant at the network arrives, it is mapped
/// to an internal message, that can
/// then be passed along to its proper processing module
/// at the node. At a node module, the result of such a call
/// is also an internal message.
/// Finally, an internal message might be destined for messaging
/// module, by which it leaves the process boundary of this node
/// and is sent on the wire to some other dst(s) on the network.

/// Vec of NodeDuty
pub type NodeDuties = Vec<NodeDuty>;

/// Common duties run by all nodes.
#[allow(clippy::large_enum_variant)]
pub enum NodeDuty {
    ReadChunk {
        read: ChunkRead,
        msg_id: MessageId,
    },
    WriteChunk {
        write: ChunkWrite,
        msg_id: MessageId,
        client_signed: ClientSigned,
    },
    ProcessRepublish {
        chunk: Chunk,
        msg_id: MessageId,
    },
    /// Run at data-section Elders on receiving the result of
    /// read operations from Adults
    RecordAdultReadLiveness {
        response: QueryResponse,
        correlation_id: MessageId,
        src: XorName,
    },
    /// Get section elders.
    GetSectionElders {
        msg_id: MessageId,
        origin: SrcLocation,
    },
    Genesis,
    EldersChanged {
        /// Our section prefix.
        our_prefix: Prefix,
        /// Our section public key.
        our_key: PublicKey,
        /// The new Elders.
        new_elders: BTreeSet<XorName>,
        /// Oldie or newbie?
        newbie: bool,
    },
    AdultsChanged {
        /// Remaining Adults in our section.
        remaining: BTreeSet<XorName>,
        /// New Adults in our section.
        added: BTreeSet<XorName>,
        /// Removed Adults in our section.
        removed: BTreeSet<XorName>,
    },
    SectionSplit {
        /// Our section prefix.
        our_prefix: Prefix,
        /// our section public key
        our_key: PublicKey,
        /// The new Elders of our section.
        our_new_elders: BTreeSet<XorName>,
        /// The new Elders of our sibling section.
        their_new_elders: BTreeSet<XorName>,
        /// The PK of the sibling section, as this event is fired during a split.
        sibling_key: PublicKey,
        /// oldie or newbie?
        newbie: bool,
    },
    /// When demoted, node levels down
    LevelDown,
    /// Initiates the node with state from peers.
    SynchState {
        /// The metadata stored on Elders.
        metadata: DataExchange,
    },
    /// As members are lost for various reasons
    /// there are certain things nodes need
    /// to do, to update for that.
    ProcessLostMember {
        name: XorName,
        age: u8,
    },
    /// Storage reaching max capacity.
    ReachingMaxCapacity,
    /// Increment count of full nodes in the network
    IncrementFullNodeCount {
        /// Node ID of node that reached max capacity.
        node_id: PublicKey,
    },
    /// Sets joining allowed to true or false.
    SetNodeJoinsAllowed(bool),
    /// Send a message to the specified dst.
    Send(OutgoingMsg),
    /// Send a lazy error as a result of a specific message.
    /// The aim here is for the sender to respond with any missing state
    SendError(OutgoingLazyError),
    /// Send supporting info for a given processing error.
    /// This should be any missing state required to proceed at the erring node.
    SendSupport(OutgoingSupportingInfo),
    /// Send the same request to each individual node.
    SendToNodes {
        id: MessageId,
        msg: NodeMsg,
        targets: BTreeSet<XorName>,
        aggregation: bool,
    },
    /// Process read of data
    ProcessRead {
        query: DataQuery,
        msg_id: MessageId,
        client_signed: ClientSigned,
        origin: EndUser,
    },
    /// Process write of data
    ProcessWrite {
        cmd: DataCmd,
        msg_id: MessageId,
        client_signed: ClientSigned,
        origin: EndUser,
    },
    /// Receive a chunk that is being replicated.
    /// This is run at an Adult (the new holder).
    ReplicateChunk {
        chunk: Chunk,
        msg_id: MessageId,
    },
    /// Create proposals to vote unresponsive nodes as offline
    ProposeOffline(Vec<XorName>),
    NoOp,
}

impl From<NodeDuty> for NodeDuties {
    fn from(duty: NodeDuty) -> Self {
        if matches!(duty, NodeDuty::NoOp) {
            vec![]
        } else {
            vec![duty]
        }
    }
}

impl Debug for NodeDuty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Genesis { .. } => write!(f, "Genesis"),
            Self::ReadChunk { .. } => write!(f, "ReadChunk"),
            Self::WriteChunk { .. } => write!(f, "WriteChunk"),
            Self::ProcessRepublish { .. } => write!(f, "ProcessRepublish"),
            Self::RecordAdultReadLiveness {
                correlation_id,
                response,
                src,
            } => write!(
                f,
                "RecordAdultReadLiveness {{ correlation_id: {}, response: {:?}, src: {} }}",
                correlation_id, response, src
            ),
            Self::LevelDown => write!(f, "LevelDown"),
            Self::SynchState { .. } => write!(f, "SynchState"),
            Self::EldersChanged { .. } => write!(f, "EldersChanged"),
            Self::AdultsChanged { .. } => write!(f, "AdultsChanged"),
            Self::SectionSplit { .. } => write!(f, "SectionSplit"),
            Self::GetSectionElders { .. } => write!(f, "GetSectionElders"),
            Self::NoOp => write!(f, "No op."),
            Self::ReachingMaxCapacity => write!(f, "ReachingMaxCapacity"),
            Self::ProcessLostMember { .. } => write!(f, "ProcessLostMember"),
            //Self::ProcessRelocatedMember { .. } => write!(f, "ProcessRelocatedMember"),
            Self::IncrementFullNodeCount { .. } => write!(f, "IncrementFullNodeCount"),
            Self::SetNodeJoinsAllowed(_) => write!(f, "SetNodeJoinsAllowed"),
            Self::Send(msg) => write!(f, "Send [ msg: {:?} ]", msg),
            Self::SendError(msg) => write!(f, "SendError [ msg: {:?} ]", msg),
            Self::SendSupport(msg) => write!(f, "SendSupport [ msg: {:?} ]", msg),
            Self::SendToNodes {
                id,
                msg,
                targets,
                aggregation,
            } => write!(
                f,
                "SendToNodes [ id: {}, msg: {:?}, targets: {:?}, aggregation: {:?} ]",
                id, msg, targets, aggregation
            ),
            Self::ProcessRead { .. } => write!(f, "ProcessRead"),
            Self::ProcessWrite { .. } => write!(f, "ProcessWrite"),
            Self::ReplicateChunk { .. } => write!(f, "ReplicateChunk"),
            Self::ProposeOffline(nodes) => write!(f, "ProposeOffline({:?})", nodes),
        }
    }
}

// --------------- Messaging ---------------

#[derive(Debug, Clone)]
pub struct OutgoingMsg {
    pub id: MessageId,
    pub msg: MsgType,
    pub dst: DstLocation,
    pub aggregation: bool,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MsgType {
    Node(NodeMsg),
    Client(ClientMsg),
}

#[derive(Debug, Clone)]
pub struct OutgoingLazyError {
    pub msg: ProcessingError,
    pub dst: DstLocation,
}

#[derive(Debug, Clone)]
pub struct OutgoingSupportingInfo {
    pub msg: SupportingInfo,
    pub dst: DstLocation,
}
