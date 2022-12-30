// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{flow_ctrl::cmds::Cmd, MyNode, Result, SectionStateVote};
use sn_interface::{
    messaging::system::{SectionSig, SectionSigned},
    network_knowledge::{
        NodeState, SapCandidate, SectionAuthUtils, SectionAuthorityProvider, SectionTreeUpdate,
    },
    types::log_markers::LogMarker,
};
use std::collections::BTreeSet;

// Agreement
impl MyNode {
    #[instrument(skip(self), level = "trace")]
    pub(crate) fn handle_section_decision_agreement(
        &mut self,
        proposal: SectionStateVote,
        sig: SectionSig,
    ) -> Result<Vec<Cmd>> {
        debug!("{:?} {:?}", LogMarker::ProposalAgreed, proposal);
        let mut cmds = Vec::new();
        match proposal {
            SectionStateVote::NodeIsOffline(node_state) => {
                cmds.extend(self.handle_offline_agreement(node_state, sig));
            }
            SectionStateVote::JoinsAllowed(joins_allowed) => {
                info!("Section reached agreement to set joins_allowed to: {joins_allowed:?}");
                self.joins_allowed = joins_allowed;
            }
        }
        Ok(cmds)
    }

    #[instrument(skip(self))]
    fn handle_offline_agreement(&mut self, node_state: NodeState, sig: SectionSig) -> Option<Cmd> {
        info!(
            "Agreement - proposing membership change with node offline: {}",
            node_state.peer()
        );
        self.propose_membership_change(node_state)
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) fn handle_request_handover_agreement(
        &mut self,
        sap: SectionAuthorityProvider,
        sig: SectionSig,
    ) -> Result<Vec<Cmd>> {
        // check if section matches our prefix
        let equal_prefix = sap.prefix() == self.network_knowledge.prefix();
        let is_extension_prefix = sap
            .prefix()
            .is_extension_of(&self.network_knowledge.prefix());
        if !equal_prefix && !is_extension_prefix {
            // Other section. We shouln't be receiving or updating a SAP for
            // a remote section here, that is done with a AE msg response.
            debug!(
                "Ignoring handover request since prefix doesn't match ours: {:?}",
                sap
            );
            return Ok(vec![]);
        }
        debug!("Handling section info with prefix: {:?}", sap.prefix());

        // check if at the given memberhip gen, the elders candidates are matching
        let membership_gen = sap.membership_gen();
        let signed_sap = SectionSigned::new(sap, sig);
        let dkg_sessions_info = self.best_elder_candidates_at_gen(membership_gen);

        let elder_candidates = BTreeSet::from_iter(signed_sap.names());
        if dkg_sessions_info
            .iter()
            .all(|session| !session.elder_names().eq(elder_candidates.iter().copied()))
        {
            error!("Elder candidates don't match best elder candidates at given gen in received section agreement, ignoring it.");
            return Ok(vec![]);
        };

        // handle regular elder handover (1 to 1)
        // trigger handover consensus among elders
        if equal_prefix {
            debug!("Propose elder handover to: {:?}", signed_sap.prefix());
            return self.propose_handover_consensus(SapCandidate::ElderHandover(signed_sap));
        }

        // add to pending split SAP candidates
        // those are stored in a mapping from Generation to BTreeSet so the order in the set is deterministic
        let section_candidates_for_gen = self
            .pending_split_sections
            .entry(membership_gen)
            .and_modify(|curr| {
                let _ = curr.insert(signed_sap.clone());
            })
            .or_insert_with(|| BTreeSet::from([signed_sap]));

        // if we have reached 2 split SAP candidates for this generation
        // handle section split (1 to 2)
        if let [sap1, sap2] = section_candidates_for_gen
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .as_slice()
        {
            debug!(
                "Propose section split handover to: {:?} {:?}",
                sap1.prefix(),
                sap2.prefix()
            );
            self.propose_handover_consensus(SapCandidate::SectionSplit(sap1.clone(), sap2.clone()))
        } else {
            debug!("Waiting for more split handover candidates");
            Ok(vec![])
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_new_sections_agreement(
        &mut self,
        sap1: SectionSigned<SectionAuthorityProvider>,
        sig1: SectionSig,
        sap2: SectionSigned<SectionAuthorityProvider>,
        sig2: SectionSig,
    ) -> Result<Vec<Cmd>> {
        if sap1.members().any(|m| m.name() == self.name()) {
            self.update_us(sap1, sig1, sap2, sig2).await
        } else if sap2.members().any(|m| m.name() == self.name()) {
            self.update_us(sap2, sig2, sap1, sig1).await
        } else {
            // Should not be possible..
            error!(
                "Error handling sections agreement, we are not a member in either section {}, {}",
                sap1.prefix(),
                sap2.prefix()
            );
            Ok(vec![])
        }
    }

    async fn update_us(
        &mut self,
        our_sap: SectionSigned<SectionAuthorityProvider>,
        sig_over_us: SectionSig,
        their_sap: SectionSigned<SectionAuthorityProvider>,
        sig_over_them: SectionSig,
    ) -> Result<Vec<Cmd>> {
        trace!("{}", LogMarker::HandlingNewSectionsAgreement);
        let context = self.context();

        // First we snapshot the section chain
        let mut parent_section_chain = self.section_chain();
        let parent_key = parent_section_chain.last_key()?;

        // Then we apply the add of our own new section
        let cmds = self
            .handle_new_elders_agreement(our_sap, sig_over_us)
            .await?;

        // Finally we update our network knowledge with our sibling section SAP.
        // We use the parent proof chain to connect our current chain to sibling SAP.
        parent_section_chain.verify_and_insert(
            &parent_key,
            their_sap.section_key(),
            sig_over_them.signature,
        )?;
        let their_prefix = their_sap.prefix();
        let section_tree_update = SectionTreeUpdate::new(their_sap, parent_section_chain);
        let updated = self.network_knowledge.update_knowledge_if_valid(
            section_tree_update,
            None,
            &context.name,
        )?;
        if updated {
            info!("Updated our network knowledge for {:?}", their_prefix);
            info!("Writing updated knowledge to disk");
            MyNode::write_section_tree(&context);
        }
        Ok(cmds)
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_new_elders_agreement(
        &mut self,
        signed_sap: SectionSigned<SectionAuthorityProvider>,
        section_sig: SectionSig,
    ) -> Result<Vec<Cmd>> {
        trace!("{}", LogMarker::HandlingNewEldersAgreement);
        let context = self.context();
        let mut section_chain = self.section_chain();
        let last_key = section_chain.last_key()?;

        let prefix = signed_sap.prefix();
        trace!("{}: for {:?}", LogMarker::NewSignedSap, prefix);

        info!("New SAP agreed for:{}", *signed_sap);

        // Let's update our network knowledge, including our
        // section SAP and chain if the new SAP's prefix matches our name
        // We need to generate the proof chain to connect our current chain to new SAP.
        section_chain.verify_and_insert(
            &last_key,
            signed_sap.section_key(),
            section_sig.signature,
        )?;
        let update = SectionTreeUpdate::new(signed_sap, section_chain);
        let name = self.context().name;
        let updated = self
            .network_knowledge
            .update_knowledge_if_valid(update, None, &name)?;

        if updated {
            info!("Updated our network knowledge for {:?}", prefix);
            info!("Writing updated knowledge to disk");
            MyNode::write_section_tree(&context);

            info!(
                "Prefixes we know about: {:?}",
                self.network_knowledge.section_tree()
            );

            self.update_on_elder_change(&context).await
        } else {
            Ok(vec![])
        }
    }
}
