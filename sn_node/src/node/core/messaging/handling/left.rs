// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    api::cmds::Cmd,
    core::{Node, Result},
};

use bls_dkg::PublicKeySet;
use sn_consensus::Decision;
use sn_interface::{messaging::system::NodeState, types::log_markers::LogMarker};

impl Node {
    pub(crate) async fn handle_node_left(
        &self,
        node_state: NodeState,
        section_key_set: &PublicKeySet,
        decision: &Decision<NodeState>,
    ) -> Result<Vec<Cmd>> {
        info!(
            "{}: {} at {}",
            LogMarker::AcceptedNodeAsOffline,
            node_state.name,
            node_state.addr
        );

        let mut cmds = vec![];

        // If this is an Offline agreement where the new node state is Relocated,
        // we then need to send the Relocate msg to the peer attaching the signed NodeState
        // containing the relocation details.
        if node_state.clone().into_state().is_relocated() {
            cmds.push(
                self.send_relocate(
                    node_state.peer(),
                    node_state,
                    section_key_set.clone(),
                    decision.clone(),
                )
                .await?,
            );
        }

        *self.joins_allowed.write().await = true;

        Ok(cmds)
    }
}
