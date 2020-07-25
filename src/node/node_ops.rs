// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[cfg(feature = "simulated-payouts")]
use safe_nd::Transfer;

use routing::{event::Event as NetworkEvent, TransportEvent as ClientEvent};
use safe_nd::{
    AccountId, Address, AuthCmd, DebitAgreementProof, HandshakeResponse, MessageId, MsgEnvelope,
    MsgSender, PublicId, PublicKey, RewardCounter, SignedTransfer, TransferValidated, XorName,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::SocketAddr};

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
    Process {
        cmd: AuthCmd,
        msg_id: MessageId,
        origin: MsgSender,
    },
}

// --------------- Messaging ---------------

/// This duty is at the border of infrastructural
/// and domain duties. Messaging is such a fundamental
/// part of the system, that it can be considered domain.
//#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum MessagingDuty {
    /// Send to a client.
    SendToClient {
        address: SocketAddr,
        msg: MsgEnvelope,
    },
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
    SendHandshake {
        address: SocketAddr,
        response: HandshakeResponse,
    },
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
        // /// The prefix of our section.
        // prefix: Prefix,
        /// The BLS public key of our section.
        key: PublicKey,
        /// The set of elders of our section.
        elders: BTreeSet<XorName>,
    },
    ProcessJoinedMember {
        old_node_id: XorName,
        new_node_id: XorName,
    },
    RunAsKeySection(KeySectionDuty),
    RunAsDataSection(DataSectionDuty),
}

pub enum AdultDuty {
    RunAsChunks(ChunkDuty),
}

// -- All duties below solely process and produce internal msgs
// -- for further handling locally - of which MessagingDuty is the
// -- most common next local step. From there, it is sent out on the network.
// -- It is important to stress that _only_ MessagingDuty does sending on to the network.

pub enum KeySectionDuty {
    ///
    EvaluateClientMsg {
        public_id: PublicId,
        msg: MsgEnvelope,
    },
    ///
    ProcessGroupDecision(GroupDecision),
    ///
    RunAsAuth(AuthDuty),
    ///
    RunAsGateway(GatewayDuty),
    ///
    RunAsPayment(PaymentDuty),
    ///
    RunAsTransfers(TransferDuty),
}

pub enum DataSectionDuty {
    ///
    RunAsMetadata(MetadataDuty),
    ///
    RunAsRewards(RewardDuty),
}

pub enum AuthDuty {
    Process {
        cmd: AuthCmd,
        msg_id: MessageId,
        origin: MsgSender,
    },
    ListAuthKeysAndVersion {
        /// The Client id.
        client: PublicKey,
        msg_id: MessageId,
        origin: MsgSender,
    },
}

// --------------- Gateway ---------------

/// Gateway duties are run at Elders.
pub enum GatewayDuty {
    ///
    FindClientFor(MsgEnvelope),
    ///
    ProcessClientEvent(ClientEvent),
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
        data: Vec<u8>,
    },
    ///
    AddNewAccount {
        ///
        id: AccountId,
        ///
        node_id: XorName,
    },
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
    },
    ///
    PrepareAccountMove { node_id: XorName },
    ///
    ReceiveRewardValidation(TransferValidated),
}

// --------------- Transfers ---------------

///
pub enum TransferDuty {
    ///
    ProcessQuery {
        query: TransferQuery,
        ///
        msg_id: MessageId,
        ///
        origin: Address,
    },
    ///
    ProcessCmd {
        cmd: TransferCmd,
        ///
        msg_id: MessageId,
        ///
        origin: Address,
    },
}

pub enum TransferQuery {
    /// Get the PublicKeySet for replicas of a given PK
    GetReplicaKeys(AccountId),
    /// Get key balance.
    GetBalance(AccountId),
    /// Get key transfers since specified version.
    GetHistory {
        /// The balance key.
        at: AccountId,
        /// The last version of transfers we know of.
        since_version: usize,
    },
}

pub enum TransferCmd {
    #[cfg(feature = "simulated-payouts")]
    /// Cmd to simulate a farming payout
    SimulatePayout(Transfer),
    /// The cmd to validate a transfer.
    ValidateTransfer(SignedTransfer),
    /// The cmd to register the consensused transfer.
    RegisterTransfer(DebitAgreementProof),
    ///
    PropagateTransfer(DebitAgreementProof),
    ///
    ValidateRewardPayout(SignedTransfer),
    ///
    RegisterRewardPayout(DebitAgreementProof),
}

impl From<safe_nd::TransferCmd> for TransferCmd {
    fn from(cmd: safe_nd::TransferCmd) -> Self {
        match cmd {
            #[cfg(feature = "simulated-payouts")]
            safe_nd::TransferCmd::SimulatePayout(transfer) => Self::SimulatePayout(transfer),
            safe_nd::TransferCmd::ValidateTransfer(signed_transfer) => {
                Self::ValidateTransfer(signed_transfer)
            }
            safe_nd::TransferCmd::RegisterTransfer(debit_agreement) => {
                Self::RegisterTransfer(debit_agreement)
            }
        }
    }
}

impl From<safe_nd::TransferQuery> for TransferQuery {
    fn from(cmd: safe_nd::TransferQuery) -> Self {
        match cmd {
            safe_nd::TransferQuery::GetReplicaKeys(transfer) => Self::GetReplicaKeys(transfer),
            safe_nd::TransferQuery::GetBalance(signed_transfer) => {
                Self::GetBalance(signed_transfer)
            }
            safe_nd::TransferQuery::GetHistory { at, since_version } => {
                Self::GetHistory { at, since_version }
            }
        }
    }
}
