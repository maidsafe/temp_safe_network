// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    dkg::DkgVoter, flow_ctrl::fault_detection::FaultsCmd, DataStorage, DkgSessionInfo, Membership,
};

use ed25519_dalek::Keypair;
use sn_comms::Comm;
use sn_dbc::Token;
use sn_fault_detection::IssueType;
use sn_interface::{
    network_knowledge::{MyNodeInfo, NetworkKnowledge, RelocationState, SectionKeysProvider},
    types::{
        fees::{SpendPriority, SpendQSnapshot},
        keys::ed25519::Digest256,
    },
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::mpsc::Sender;
use xor_name::XorName;

/// Snapshot of node's state
#[derive(custom_debug::Debug, Clone)]
pub struct NodeContext {
    pub(crate) is_elder: bool,
    pub(crate) data_storage: DataStorage,
    pub(crate) name: XorName,
    pub(crate) info: MyNodeInfo,
    #[debug(skip)]
    pub(crate) keypair: Arc<Keypair>,
    #[debug(skip)]
    pub(crate) reward_secret_key: Arc<bls::SecretKey>,
    pub(crate) network_knowledge: NetworkKnowledge,
    pub(crate) section_keys_provider: SectionKeysProvider,
    #[debug(skip)]
    pub(crate) comm: Comm,
    #[debug(skip)]
    pub(crate) membership: Option<Membership>,
    #[debug(skip)]
    pub(crate) dkg_voter: DkgVoter,
    #[debug(skip)]
    pub(crate) dkg_sessions_info: HashMap<Digest256, DkgSessionInfo>,
    pub(crate) joins_allowed: bool,
    pub(crate) joins_allowed_until_split: bool,
    #[debug(skip)]
    pub(crate) fault_cmds_sender: Sender<FaultsCmd>,
    pub(crate) relocation_state: RelocationState,
    #[debug(skip)]
    pub(super) spend_q_snapshot: SpendQSnapshot,
}

impl NodeContext {
    /************ Public API methods ********************/

    /// The socket address of our node.
    pub fn socket_addr(&self) -> SocketAddr {
        self.comm.socket_addr()
    }

    /// The name of our node.
    pub fn name(&self) -> XorName {
        self.name
    }

    /// Wether the node is an Elder in its section.
    pub fn is_elder(&self) -> bool {
        self.is_elder
    }

    /// Current node's network knowledge.
    pub fn network_knowledge(&self) -> &NetworkKnowledge {
        &self.network_knowledge
    }

    /************ END OF Public API methods **************/

    /// Log an issue in dysfunction
    /// Spawns a process to send this incase the channel may be full, we don't hold up
    /// processing around this (as this can be called during dkg eg)
    pub(crate) fn track_node_issue(&self, name: XorName, issue: IssueType) {
        debug!("Logging issue {issue:?} in dysfunction for {name}");
        let dysf_sender = self.fault_cmds_sender.clone();
        // TODO: do we need to kill the node if we fail tracking dysf?
        let _handle = tokio::spawn(async move {
            if let Err(error) = dysf_sender.send(FaultsCmd::TrackIssue(name, issue)).await {
                // Log the issue, and error. We need to be wary of actually hitting this.
                warn!("Could not send FaultsCmd through dysfunctional_cmds_tx: {error}");
            }
        });
    }

    /// Calculate current fee for payments or storing data.
    pub(crate) fn current_fee(&self, priority: &SpendPriority) -> Token {
        let spend_q_stats = self.spend_q_snapshot.stats();
        Token::from_nano(spend_q_stats.derive_fee(priority))
    }

    pub(crate) fn validate_fee(&self, fee_paid: Token) -> (bool, Token) {
        let spend_q_stats = self.spend_q_snapshot.stats();
        let (valid, lowest) = spend_q_stats.validate_fee(fee_paid.as_nano());
        (valid, Token::from_nano(lowest))
    }
}

pub(super) fn op_cost(network_knowledge: &NetworkKnowledge, data_storage: &DataStorage) -> Token {
    use sn_interface::{messaging::data::DataCmd, op_cost::required_tokens};
    let bytes = std::mem::size_of::<DataCmd>();
    let prefix_len = network_knowledge.prefix().bit_count();
    let num_storage_nodes = network_knowledge.members().len() as u8;
    let percent_filled = data_storage.used_space_ratio();
    required_tokens(bytes, prefix_len, num_storage_nodes, percent_filled)
}
