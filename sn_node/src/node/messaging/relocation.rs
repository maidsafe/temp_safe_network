// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd,
    messaging::{Peers, relocation},
    relocated::JoiningAsRelocated,
    relocation::{get_nodes_to_relocate, ChurnId, find_nodes_to_relocate},
    MyNode, Result,
};

use sn_interface::{
    elder_count,
    messaging::system::{NodeMsg, SectionSigned},
    network_knowledge::{MembershipState, NodeState, node_state::RelocationInfo},
    types::{keys::ed25519, log_markers::LogMarker},
};

use ed25519_dalek::Signer;
use std::collections::BTreeSet;
use xor_name::XorName;

// Relocation
impl MyNode {
    pub(crate) fn relocate_peers(
        &mut self,
        churn_id: ChurnId,
        excluded: BTreeSet<XorName>,
    ) -> Result<Vec<Cmd>> {
        // Do not carry out relocations in the first section
        // TODO: consider avoiding relocations in first 16 sections instead.
        if self.network_knowledge.prefix().is_empty() {
            return Ok(vec![]);
        }
        debug!("Try to find relocate peers, excluded {excluded:?}");
        // Do not carry out relocation when there is not enough elder nodes.
        if self.network_knowledge.section_auth().elder_count() < elder_count() {
            debug!(
                "Not enough elders current {:?} vs. expected {:?}",
                self.network_knowledge.section_auth().elder_count(),
                elder_count()
            );
            return Ok(vec![]);
        }

        let mut cmds = vec![];
        for (node_state, relocation_details) in
            find_nodes_to_relocate(&self.network_knowledge, &churn_id, excluded)
        {
            debug!(
                "Relocating {:?} to {} (on churn of {churn_id})",
                relocation_details.peer(),
                relocation_details.dst,
            );

            let relocated_node_state = node_state.relocate(relocation_details);
            cmds.extend(self.propose_membership_change(relocated_node_state));
        }

        Ok(cmds)
    }

    pub(crate) fn handle_relocate(
        &mut self,
        proof: SectionSigned<NodeState>,
    ) -> Result<Option<Cmd>> {
        trace!("Handle relocate {:?}", proof);
        // should be unreachable, but a sanity check
        let serialized_proof = bincode::serialize(&proof.value)?;
        if !proof.sig.verify(&serialized_proof) {
            return Err(super::Error::InvalidSignature);
        }
        if node.name() != proof.peer().name() {
            // not for us, drop it
            return Ok(vec![]);
        }

        let dst_section =
            if let MembershipState::Relocated(ref relocation_dst) = relocate_proof.state() {
                relocate_details.0
            } else {
                debug!(
                    "Ignoring Relocate msg containing invalid NodeState: {:?}",
                    relocate_proof.state()
                );
                return Ok(None);
            };
        
        trace!("{}", LogMarker::RelocateStart);
        debug!("Received Relocate message to join the section at {dst_xorname}");

        let original_info = self.info();

        let dst_sap = self.network_knowledge.closest_signed_sap(&dst_section)?;
        let new_keypair = ed25519::gen_keypair(&dst_sap.prefix().range_inclusive(), original_info.age().saturating_add(1));
        let new_name = ed25519::name(&new_keypair.public);

        let info = RelocationInfo::new(proof, new_name);
        let serialized_info = bincode::serialize(&info)?;
        // we verify that this new name was actually created by the old name
        let node_sig = original_info.keypair.sign(&serialized_info);

        // we try shift to the new section
        self.set_new_section(dst_sap, new_keypair)?;

        trace!("Previous name: {:?} shifted to dst section {} as {new_name}. Now trying to join..", original_info.name(), dst_sap.prefix());

        Ok(MyNode::try_join_section(self.context(), Some(SignedRelocationInfo::new(info, node_sig, original_info.keypair.public))))
    }

}
