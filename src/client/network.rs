// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{
    client::{DataCmd as NodeDataCmd, DataQuery as NodeDataQuery, Error, Result},
    EndUser,
};
use serde::{Deserialize, Serialize};
use sn_data_types::{
    Blob, BlobAddress, Credit, DebitId, PublicKey, ReplicaEvent, Signature, SignatureShare,
    SignedCredit, SignedTransferShare, TransferAgreementProof, TransferValidated, WalletInfo,
};
use std::collections::BTreeSet;
use xor_name::XorName;

use super::{BlobRead, BlobWrite};

// -------------- Node Cmds --------------

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeCmd {
    /// Metadata is handled by Elders
    Metadata { cmd: NodeDataCmd, origin: EndUser },
    /// Chunks are handled by Adults
    Chunks { cmd: BlobWrite, origin: EndUser },
    /// Transfers are handled by Elders
    Transfers(NodeTransferCmd),
    /// Cmds related to the running of a node.
    System(NodeSystemCmd),
}

/// Cmds related to the running of a node.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemCmd {
    /// When threshold Elders have been reached
    /// in genesis section, they all propose genesis.
    ProposeGenesis {
        /// The genesis credit.
        credit: Credit,
        /// An individual Elder's sig share.
        sig: SignatureShare,
    },
    /// When proposal has been agreed
    /// in genesis section, they all accumulate genesis.
    AccumulateGenesis {
        /// The genesis credit.
        signed_credit: SignedCredit,
        /// An individual Elder's sig share.
        sig: SignatureShare,
    },
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
    /// Replicate a given chunk at another Adult
    ReplicateChunk {
        /// New holders's name.
        new_holder: XorName,
        /// Address of the blob to be replicated.
        address: BlobAddress,
        /// Current holders.
        current_holders: BTreeSet<XorName>,
    },
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferCmd {
    ///
    PropagateTransfer(TransferAgreementProof),
    ///
    ValidateSectionPayout(SignedTransferShare),
    ///
    RegisterSectionPayout(TransferAgreementProof),
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
    ///
    SectionPayoutRegistered { from: PublicKey, to: PublicKey },
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeQuery {
    /// Metadata is handled by Elders
    Metadata {
        query: NodeDataQuery,
        origin: EndUser,
    },
    /// Chunks are handled by Adults
    Chunks { query: BlobRead, origin: EndUser },
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
    /// Sent by the new section to the
    /// old section after node relocation.
    GetNodeWalletId {
        /// The id of the node
        /// in the old section.
        old_node_id: XorName,
        /// The id of the node
        /// in the new section.
        new_node_id: XorName,
    },
    /// A new Section Actor share (i.e. a new Elder) needs to query
    /// its peer Elders for the replicas' public key set
    /// and the history of events of the section wallet.
    GetSectionWalletHistory,
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferQuery {
    /// On Elder change, all Elders need to query
    /// network for the new wallet's replicas' public key set
    /// and the history of events of the wallet (which will be empty at that point..).
    /// A second pk may be optinally passed for sibling section setup
    SetupNewSectionWallets((PublicKey, Option<PublicKey>)),
    /// Replicas starting up
    /// need to query for events of
    /// the existing Replicas. (Sent to the other Elders).
    GetReplicaEvents,
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemQuery {
    /// Acquire the chunk from current holders for replication.
    GetChunk {
        /// New Holder's name.
        new_holder: XorName,
        /// Address of the blob to be replicated.
        address: BlobAddress,
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
    GetNodeWalletId(Result<(PublicKey, XorName)>),
    /// Returns the history of the section wallet.
    GetSectionWalletHistory(WalletInfo),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferQueryResponse {
    /// On Elder change, all Elders neet to query
    /// network for the new wallet's replicas' public key set
    /// and the history of events of the wallet (which will be empty at that point..).
    SetupNewSectionWallets{
        our_wallet: Result<WalletInfo>,
        sibling_key: Option<PublicKey>
    },
    // /// Returns the history of the section actor.
    // GetSectionWalletInfo(Result<Vec<ReplicaEvent>>),
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
    /// The error of registration of a section payout.
    SectionPayoutRegistration(Error),
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
