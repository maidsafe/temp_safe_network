// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{MembershipState, NodeState, SectionAuth};
use crate::node::{
    api::cmds::Cmd,
    core::{bootstrap::JoiningAsRelocated, Node},
    Event, Result,
};
use crate::types::log_markers::LogMarker;

// Relocation
impl Node {
    pub(crate) async fn handle_relocate(
        &self,
        relocate_proof: SectionAuth<NodeState>,
    ) -> Result<Option<Cmd>> {
        let (dst_xorname, dst_section_key, new_age) =
            if let MembershipState::Relocated(ref relocate_details) = relocate_proof.value.state {
                (
                    relocate_details.dst,
                    relocate_details.dst_section_key,
                    relocate_details.age,
                )
            } else {
                debug!(
                    "Ignoring Relocate msg containing invalid NodeState: {:?}",
                    relocate_proof.state
                );
                return Ok(None);
            };

        let node = self.info.read().await.clone();
        if dst_xorname != node.name() {
            // This `Relocate` message is not for us - it's most likely a duplicate of a previous
            // message that we already handled.
            return Ok(None);
        }

        debug!(
            "Received Relocate message to join the section at {}",
            dst_xorname
        );

        match *self.relocate_state.read().await {
            Some(_) => {
                trace!("Ignore Relocate - relocation already in progress");
                return Ok(None);
            }
            None => {
                trace!("{}", LogMarker::RelocateStart);
                self.send_event(Event::RelocationStarted {
                    previous_name: node.name(),
                })
                .await;
            }
        }

        // Create a new instance of JoiningAsRelocated to start the relocation
        // flow. This same instance will handle responses till relocation is complete.
        let genesis_key = *self.network_knowledge.genesis_key();

        let bootstrap_addrs = if let Ok(sap) = self.network_knowledge.section_by_name(&dst_xorname)
        {
            sap.addresses()
        } else {
            self.network_knowledge
                .authority_provider()
                .await
                .addresses()
        };
        let (joining_as_relocated, cmd) = JoiningAsRelocated::start(
            node,
            genesis_key,
            relocate_proof,
            bootstrap_addrs,
            dst_xorname,
            dst_section_key,
            new_age,
        )?;

        *self.relocate_state.write().await = Some(Box::new(joining_as_relocated));

        Ok(Some(cmd))
    }
}
