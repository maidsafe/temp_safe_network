// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use safe_nd::{HandshakeResponse, MsgEnvelope, XorName, Address, MessageId, SignedTransfer, DebitAgreementProof};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::SocketAddr};
use routing::{event::Event as NetworkEvent, TransportEvent as ClientEvent};
// /// Node internal cmds, about what requests to make.
// /// Any network node
// #[derive(Debug)]
// #[allow(clippy::large_enum_variant)]
// pub(crate) enum MessagingChain {
//     Single(MessagingDuty),
//     Multiple(Vec<MessagingDuty>),
// }

/// Internal messages are what is passed along
/// within a node, between the entry point and
/// exit point of remote messages.
/// In other words, when communication from another
/// participant at the network arrives, it is analysed
/// and interpreted into an internal message, that can 
/// then be passed along to its proper processing module
/// at the node. At a node module, the result of such a call
/// is also an internal message.
/// Finally, an internal message might be destined for Messaging
/// module, by which it leaves the physical boundary of this node
/// and is sent on the wire to some other destination(s) on the network.
/// 
// #[derive(Debug)]
// #[allow(clippy::large_enum_variant)]
// pub(crate) enum NodeOperations {
//     Single(NodeOperation),
//     Multiple(Vec<NodeOperation>),
// }

pub enum NodeOperation {
    RunAsGateway(GatewayDuty),
    RunAsPayment(PaymentDuty),
    RunAsMetadata(MetadataDuty),
    RunAsRewards(RewardDuty),
    RunAsTransfers(TransferDuty),
    RunAsAdult(AdultDuty),
    RunAsElder(ElderDuty),
    RunAsNode(NodeDuty),
    Unknown,
}

// Need to Serialize/Deserialize to go through the consensus process.
/// A GroupDecision is something only
/// taking place at the network Gateways.
#[derive(Debug, Clone, Serialize, Deserialize)] // Debug,
pub enum GroupDecision {
    /// When Gateway nodes consider a request
    /// valid, they will vote for it to be forwarded.
    /// As they reach consensus, this is then carried out.
    Forward(MsgEnvelope),
}

// --------------- Messaging ---------------

/// This duty is at the border of infrastructural
/// and domain duties. Messaging is such a fundamental
/// part of the system, that it can be considered domain.
//#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum MessagingDuty {
    /// Send to a client.
    SendToClient { address: SocketAddr, msg: MsgEnvelope },
    /// Send to a single node.
    SendToNode(MsgEnvelope),
    /// Send to a section.
    SendToSection(MsgEnvelope),
    /// Send the same request to each individual Adult.
    SendToAdults {
        targets: BTreeSet<XorName>,
        msg: MsgEnvelope,
    },
    /// Vote for a cmd so we can process the deferred action on consensus.
    /// (Currently immediately.)
    VoteFor(GroupDecision),
    /// 
    SendHandshake { address: SocketAddr, response: HandshakeResponse },
    ///
    DisconnectClient(SocketAddr),
}

pub enum NodeDuty {
    BecomeAdult,
    BecomeElder,
    ProcessMessaging(MessagingDuty),
    ProcessNetworkEvent(NetworkEvent),
}

pub enum ElderDuty {
    ProcessLostMember { 
        name: XorName,
        age: u8,
    },
    ProcessElderChange {
        /// The prefix of our section.
        prefix: Prefix,
        /// The BLS public key of our section.
        key: PublicKey,
        /// The set of elders of our section.
        elders: BTreeSet<XorName>,
    },
    ProcessJoinedMember {
        old_node_id: XorName,
        new_node_id: XorName,
    },
    // RunAsGateway(GatewayDuty),
    // RunAsPayment(PaymentDuty),
    // RunAsMetadata(MetadataDuty),
    // RunAsRewards(RewardDuty),
    // RunAsTransfers(TransferDuty),
}

pub enum AdultDuty {
    RunAsChunks(ChunkDuty),
}

// -- All duties below solely process and produce internal msgs
// -- for further handling locally - of which MessagingDuty is the
// -- most common next local step. From there, it is sent out on the network.
// -- It is important to stress that _only_ MessagingDuty does sending on to the network.

// --------------- Gateway ---------------

/// Gateway duties are run at Elders.
pub enum GatewayDuty {
    ///
    ProcessMsg(MsgEnvelope),
    ///
    ProcessClientEvent(ClientEvent),
    ///
    ProcessGroupDecision(GroupDecision),
}

// --------------- Payment ---------------

///
pub enum PaymentDuty {
    ///
    ProcessPayment(MsgEnvelope),
}

// --------------- Metadata ---------------

///
pub enum MetadataDuty {
    ///
    ProcessRead(MsgEnvelope),
    ///
    ProcessWrite(MsgEnvelope),
}

// --------------- Chunks ---------------

/// Chunk duties.
pub enum ChunkDuty {
    ///
    ReadChunk(MsgEnvelope),
    ///
    WriteChunk(MsgEnvelope),
}

// --------------- Rewards ---------------

///
pub enum RewardDuty {
    ///
    AccumulateReward {
        ///
        data: Vec<u8>
    }
    ///
    AddNewAccount {
        ///
        id: AccountId, 
        ///
        node_id: XorName
    }
    ///
    AddRelocatedAccount {
        ///
        old_node_id: XorName,
        ///
        new_node_id: XorName,
    },
    ///
    ClaimRewardCounter {
        ///
        old_node_id: XorName, 
        ///
        new_node_id: XorName, 
        ///
        msg_id: MessageId, 
        ///
        origin: Address,
    },
    ///
    ReceiveClaimedRewards {
        ///
        id: AccountId,
        ///
        node_id: XorName,
        ///
        counter: RewardCounter,
    }
}

// --------------- Transfers ---------------

///
pub enum TransferDuty {
    ///
    ProcessQuery(InternalTransferQuery),
    ///
    ProcessCmd(InternalTransferCmd)
}

pub enum InternalTransferQuery {
    /// Get the PublicKeySet for replicas of a given PK
    GetReplicaKeys {
        ///
        account_id: PublicKey,
        ///
        msg_id: MessageId, 
        ///
        origin: Address,
    },
    /// Get key balance.
    GetBalance {
        ///
        account_id: PublicKey,
        ///
        msg_id: MessageId, 
        ///
        origin: Address,
    },
    /// Get key transfers since specified version.
    GetHistory {
        /// The balance key.
        at: PublicKey,
        /// The last version of transfers we know of.
        since_version: usize,
        ///
        msg_id: MessageId, 
        ///
        origin: Address,
    },
}

pub enum InternalTransferCmd {
    #[cfg(feature = "simulated-payouts")]
    /// Cmd to simulate a farming payout
    SimulatePayout {
        ///
        transfer: Transfer,
        ///
        msg_id: MessageId, 
        ///
        origin: Address,
    },
    /// The cmd to validate a transfer.
    ValidateTransfer {
        ///
        signed_transfer: SignedTransfer,
        ///
        msg_id: MessageId, 
        ///
        origin: Address,
    },
    /// The cmd to register the consensused transfer.
    RegisterTransfer {
        ///
        debit_agreement: DebitAgreementProof,
        ///
        msg_id: MessageId, 
        ///
        origin: Address,
    },
    ///
    PropagateTransfer {
        ///
        debit_agreement: DebitAgreementProof,
        ///
        msg_id: MessageId, 
        ///
        origin: Address,
    },
    // ///
    // InitiateRewardPayout {
    //     signed_transfer: SignedTransfer,
    //     msg_id: MessageId, 
    //     origin: Address,
    // },
    // ///
    // FinaliseRewardPayout {
    //     debit_agreement: DebitAgreementProof,
    //     msg_id: MessageId, 
    //     origin: Address,
    // },
}
