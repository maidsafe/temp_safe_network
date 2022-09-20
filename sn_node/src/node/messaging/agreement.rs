// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{flow_ctrl::cmds::Cmd, Node, Proposal, Result};
use sn_interface::{
    messaging::system::{KeyedSig, SectionAuth},
    network_knowledge::{NodeState, SapCandidate, SectionAuthUtils, SectionAuthorityProvider},
    types::log_markers::LogMarker,
};
use std::collections::BTreeSet;

// Agreement
impl Node {
    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_general_agreements(
        &mut self,
        proposal: Proposal,
        sig: KeyedSig,
    ) -> Result<Option<Cmd>> {
        debug!("{:?} {:?}", LogMarker::ProposalAgreed, proposal);
        match proposal {
            Proposal::VoteNodeOffline(node_state) => {
                Ok(self.handle_offline_agreement(node_state, sig))
            }
            Proposal::SectionInfo(sap) => self.handle_section_info_agreement(sap, sig).await,
            Proposal::NewElders(_) => {
                error!("Elders agreement should be handled in a separate blocking fashion");
                Ok(None)
            }
            Proposal::JoinsAllowed(joins_allowed) => {
                self.joins_allowed = joins_allowed;
                Ok(None)
            }
        }
    }

    #[instrument(skip(self))]
    fn handle_offline_agreement(&mut self, node_state: NodeState, sig: KeyedSig) -> Option<Cmd> {
        info!(
            "Agreement - proposing membership change with node offline: {}",
            node_state.peer()
        );
        self.propose_membership_change(node_state.to_msg())
    }

    #[instrument(skip(self), level = "trace")]
    async fn handle_section_info_agreement(
        &mut self,
        section_auth: SectionAuthorityProvider,
        sig: KeyedSig,
    ) -> Result<Option<Cmd>> {
        // check if section matches our prefix
        let equal_prefix = section_auth.prefix() == self.network_knowledge.prefix();
        let is_extension_prefix = section_auth
            .prefix()
            .is_extension_of(&self.network_knowledge.prefix());
        if !equal_prefix && !is_extension_prefix {
            // Other section. We shouln't be receiving or updating a SAP for
            // a remote section here, that is done with a AE msg response.
            debug!(
                "Ignoring Proposal::SectionInfo since prefix doesn't match ours: {:?}",
                section_auth
            );
            return Ok(None);
        }
        debug!(
            "Updating section info for our prefix: {:?}",
            section_auth.prefix()
        );

        // check if SAP is already in our network knowledge
        let signed_section_auth = SectionAuth::new(section_auth, sig.clone());
        // TODO: on dkg-failure, we may have tried to re-start DKG with some
        //       elders excluded, this check here uses the empty set for the
        //       excluded_candidates which would prevent a dkg-retry from
        //       succeeding.
        let dkg_sessions = self.promote_and_demote_elders(&BTreeSet::new());

        let agreeing_elders = BTreeSet::from_iter(signed_section_auth.names());
        if dkg_sessions
            .iter()
            .all(|session| !session.elder_names().eq(agreeing_elders.iter().copied()))
        {
            warn!("SectionInfo out of date, ignore");
            return Ok(None);
        };

        // handle regular elder handover (1 to 1)
        // trigger handover consensus among elders
        if equal_prefix {
            debug!(
                "Propose elder handover to: {:?}",
                signed_section_auth.prefix()
            );
            return self
                .propose_handover_consensus(SapCandidate::ElderHandover(signed_section_auth));
        }

        // manage pending split SAP candidates
        // NB TODO temporary while we wait for Membership generations and possibly double DKG
        let chosen_candidates = self
            .split_barrier
            .process(
                &self.network_knowledge.prefix(),
                signed_section_auth.clone(),
                sig.clone(),
            )
            .await;

        // handle section split (1 to 2)
        if let [(sap1, _sig1), (sap2, _sig2)] = chosen_candidates.as_slice() {
            debug!(
                "Propose section split handover to: {:?} {:?}",
                sap1.prefix(),
                sap2.prefix()
            );
            self.propose_handover_consensus(SapCandidate::SectionSplit(sap1.clone(), sap2.clone()))
        } else {
            debug!("Waiting for more split handover candidates");
            Ok(None)
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_new_elders_agreement(
        &mut self,
        signed_section_auth: SectionAuth<SectionAuthorityProvider>,
        key_sig: KeyedSig,
    ) -> Result<Vec<Cmd>> {
        trace!("{}", LogMarker::HandlingNewEldersAgreement);
        let snapshot = self.state_snapshot();
        let mut old_dag = self.our_section_dag();
        let last_key = old_dag.last_key()?;

        let prefix = signed_section_auth.prefix();
        trace!("{}: for {:?}", LogMarker::NewSignedSap, prefix);

        info!("New SAP agreed for:{}", *signed_section_auth);

        // Let's update our network knowledge, including our
        // section SAP and chain if the new SAP's prefix matches our name
        // We need to generate the proof chain to connect our current chain to new SAP.
        match old_dag.insert(
            &last_key,
            signed_section_auth.section_key(),
            key_sig.signature,
        ) {
            Err(err) => error!(
                "Failed to generate proof chain for a newly received SAP: {:?}",
                err
            ),
            Ok(()) => {
                match self.update_network_knowledge(signed_section_auth.clone(), &old_dag, None) {
                    Err(err) => {
                        error!("Error updating our network knowledge for {prefix:?}: {err:?}")
                    }
                    Ok(true) => {
                        info!("Updated our network knowledge for {:?}", prefix);
                        info!("Writing updated knowledge to disk");
                        self.write_section_tree().await
                    }
                    _ => {}
                }
            }
        }

        info!(
            "Prefixes we know about: {:?}",
            self.network_knowledge.section_tree()
        );

        self.update_on_elder_change(&snapshot).await
    }
}
