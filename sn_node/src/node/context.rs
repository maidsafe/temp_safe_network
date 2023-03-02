// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{flow_ctrl::fault_detection::FaultsCmd, DataStorage, Membership};

use sn_comms::Comm;
use sn_fault_detection::IssueType;
use sn_interface::network_knowledge::{
    MyNodeInfo, NetworkKnowledge, RelocationState, SectionKeysProvider,
};

use bls::PublicKey;
use ed25519_dalek::Keypair;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use xor_name::XorName;

/// Snapshot of node's state
#[derive(custom_debug::Debug, Clone)]
pub struct NodeContext {
    pub(crate) is_elder: bool,
    pub(crate) data_storage: DataStorage,
    pub(crate) name: XorName,
    pub(crate) info: MyNodeInfo,
    pub(crate) keypair: Arc<Keypair>,
    pub(crate) reward_key: PublicKey,
    pub(crate) store_cost: sn_dbc::Token,
    pub(crate) network_knowledge: NetworkKnowledge,
    pub(crate) section_keys_provider: SectionKeysProvider,
    #[debug(skip)]
    pub(crate) comm: Comm,
    #[debug(skip)]
    pub(crate) membership: Option<Membership>,
    pub(crate) joins_allowed: bool,
    pub(crate) joins_allowed_until_split: bool,
    #[debug(skip)]
    pub(crate) fault_cmds_sender: Sender<FaultsCmd>,
    pub(crate) relocation_state: RelocationState,
}

impl NodeContext {
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
}
