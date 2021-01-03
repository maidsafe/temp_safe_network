// Copyright 2021MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::Address;
use serde::{Deserialize, Serialize};
use sn_data_types::{
    Blob, BlobAddress, BlobWrite, DebitId, Error, MsgSender, PublicKey, ReplicaEvent, Result,
    Signature, SignedTransfer, TransferAgreementProof, TransferValidated,
};
use std::collections::BTreeSet;
use xor_name::XorName;

// -------------- Node Cmds --------------

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeCmd {
    /// Cmds related to the running of a node.
    System(NodeSystemCmd),
    ///
    Data(NodeDataCmd),
    ///
    Transfers(NodeTransferCmd),
}

/// Cmds related to the running of a node.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemCmd {
    /// Register a wallet for reward payouts.
    RegisterWallet {
        /// The wallet to which rewards will be paid out by the network.
        wallet: PublicKey,
        /// The section where this wallet is to be registered (NB: this is the section of the node id).
        section: XorName,
    },
    /// Notify Elders on nearing max capacity
    StorageFull {
        /// Node Id
        node_id: PublicKey,
        /// Section to which the message needs to be sent to. (NB: this is the section of the node id).
        section: XorName,
    },
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferCmd {
    ///
    PropagateTransfer(TransferAgreementProof),
    ///
    ValidateSectionPayout(SignedTransfer),
    ///
    RegisterSectionPayout(TransferAgreementProof),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeDataCmd {
    /// Replicate a given chunk at another Adult
    ReplicateChunk {
        /// New holders's name.
        new_holder: XorName,
        /// Address of the blob to be replicated.
        address: BlobAddress,
        /// Current holders.
        current_holders: BTreeSet<XorName>,
    },
    /// Elder-to-Adult cmd.
    Blob(BlobWrite),
}

// -------------- Node Events --------------

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeEvent {
    /// Replication completed event, emitted by a node, received by elders.
    ReplicationCompleted {
        ///
        chunk: BlobAddress,
        /// The Elder's accumulated signature
        /// over the chunk address. This is sent back
        /// to them so that any uninformed Elder knows
        /// that this is all good.
        proof: Signature,
    },
    ///
    SectionPayoutValidated(TransferValidated),
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeQuery {
    ///
    Data(NodeDataQuery),
    ///
    Rewards(NodeRewardQuery),
    ///
    Transfers(NodeTransferQuery),
}

/// Reward query that is sent between sections.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeRewardQuery {
    /// Sent by the new section to the
    /// old section after node relocation.
    GetWalletId {
        /// The id of the node
        /// in the old section.
        old_node_id: XorName,
        /// The id of the node
        /// in the new section.
        new_node_id: XorName,
    },
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferQuery {
    ///
    GetSectionActorHistory(PublicKey),
    /// Replicas starting up
    /// need to query for events of
    /// the existing Replicas.
    GetReplicaEvents(PublicKey),
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeDataQuery {
    /// Acquire the chunk from current holders for replication.
    GetChunk {
        /// New Holder's name.
        new_holder: XorName,
        /// Address of the blob to be replicated.
        address: BlobAddress,
        /// Details of the section that authorised the replication.
        /// (This is the accumulated sig over the `ReplicateChunk` cmd.)
        section_authority: MsgSender,
        /// Current holders.
        current_holders: BTreeSet<XorName>,
    },
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeQueryResponse {
    ///
    Data(NodeDataQueryResponse),
    ///
    Rewards(NodeRewardQueryResponse),
    ///
    Transfers(NodeTransferQueryResponse),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeRewardQueryResponse {
    /// Returns the wallet address
    /// together with the new node id,
    /// that followed with the original query.
    GetWalletId(Result<(PublicKey, XorName)>),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferQueryResponse {
    /// Returns the history of the section actor.
    GetSectionActorHistory(Result<Vec<ReplicaEvent>>),
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
    GetChunk(Result<Blob>),
    /// Adult to Adult Get
    GetChunks(Result<Vec<Blob>>),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeCmdError {
    ///
    Data(NodeDataError),
    ///
    Rewards(NodeRewardError),
    ///
    Transfers(NodeTransferError),
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeDataError {
    ///
    ChunkReplication {
        ///
        address: BlobAddress,
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

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeRewardError {
    ///
    RewardClaiming {
        ///
        wallet: PublicKey,
        ///
        error: Error,
    },
    ///
    RewardPayoutInitiation {
        ///
        id: DebitId,
        ///
        wallet: PublicKey,
        ///
        error: Error,
    },
    ///
    RewardPayoutFinalisation {
        ///
        id: DebitId,
        ///
        wallet: PublicKey,
        ///
        error: Error,
    },
}

impl NodeCmd {
    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> Address {
        use Address::*;
        use NodeCmd::*;
        use NodeDataCmd::*;
        use NodeTransferCmd::*;
        match self {
            System(NodeSystemCmd::RegisterWallet { section, .. }) => Section(*section),
            System(NodeSystemCmd::StorageFull { section, .. }) => Section(*section),
            Data(cmd) => match cmd {
                ReplicateChunk { new_holder, .. } => Node(*new_holder),
                Blob(_write) => Node(XorName::default()), // todo: fix this!
            },
            Transfers(cmd) => match cmd {
                ValidateSectionPayout(signed_debit) => Section(signed_debit.sender().into()),
                RegisterSectionPayout(transfer_agreement) => {
                    Section(transfer_agreement.sender().into())
                }
                PropagateTransfer(transfer_agreement) => {
                    Section(transfer_agreement.recipient().into())
                }
            },
        }
    }
}

impl NodeEvent {
    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> Address {
        use Address::*;
        use NodeEvent::*;
        match self {
            ReplicationCompleted { chunk, .. } => Section(*chunk.name()),
            SectionPayoutValidated(event) => Section(event.sender().into()),
        }
    }
}

impl NodeQuery {
    /// Returns the address of the destination for the query.
    pub fn dst_address(&self) -> Address {
        use Address::*;
        use NodeDataQuery::*;
        use NodeQuery::*;
        use NodeRewardQuery::*;
        use NodeTransferQuery::*;
        match self {
            Data(data_query) => match data_query {
                GetChunk {
                    current_holders, ..
                } => Node(*current_holders.iter().next().unwrap_or(&XorName::random())),
            },
            Transfers(transfer_query) => match transfer_query {
                GetReplicaEvents(section_key) => Section((*section_key).into()),
                GetSectionActorHistory(section_key) => Section((*section_key).into()),
            },
            Rewards(GetWalletId { old_node_id, .. }) => Section(*old_node_id),
        }
    }
}
