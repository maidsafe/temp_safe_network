// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{KeyedSig, SectionAuth};
use crate::node::{
    api::cmds::Cmd,
    core::{Node, Proposal},
    dkg::SectionAuthUtils,
    network_knowledge::SectionAuthorityProvider,
    Result,
};
use crate::types::log_markers::LogMarker;

use std::collections::BTreeSet;

// Agreement
impl Node {
    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_section_info_agreement(
        &self,
        section_auth: SectionAuthorityProvider,
        membership_gen: u64,
        sig: KeyedSig,
    ) -> Result<Vec<Cmd>> {
        // check if section matches our prefix
        let matches_prefix = section_auth.prefix() == self.network_knowledge.prefix().await;
        let is_extension_prefix = section_auth.prefix().is_extension_of(&self.network_knowledge.prefix().await;
        if !matches_prefix && !is_extension_prefix {
            // Other section. We shouln't be receiving or updating a SAP for
            // a remote section here, that is done with a AE msg response.
            debug!(
                "Ignoring Proposal::SectionInfo since prefix doesn't match ours: {:?}",
                section_auth
            );
            return Ok(vec![])
        }
        debug!(
            "Updating section info for our prefix: {:?}",
            section_auth.prefix()
        );

        // check if SAP is already in our network knowledge
        let signed_section_auth = SectionAuth::new(section_auth, sig.clone());
        let saps_candidates = self
            .network_knowledge
            .promote_and_demote_elders(&self.info.read().await.name(), &BTreeSet::new())
            .await;
        if !saps_candidates.contains(&signed_section_auth.elder_candidates()) {
            // SectionInfo out of date, ignore.
            return Ok(vec![]);
        }

        // handle regular elder handover (1 to 1)
        // trigger handover consensus among elders
        if matches_prefix {
            debug!("Propose elder handover to: {:?}", section_auth.prefix());
            return self.propose_handover_sap(SapCandidates::ElderHandover(section_auth)).await
        }

        // handle section split (1 to 2)
        // TODO check for race conditions on pending_split_sap_candidates
        // Do we ever need to clear it? when DKG fails on one side maybe?
        // Don't like to keep states here, is there a better way?
        self.pending_split_sap_candidates.write().await.insert(section_auth);
        if let [sap1, sap2] = self.pending_split_sap_candidates.read().await.as_slice() {
            debug!("Propose section split handover to: {:?} {:?}", sap1.prefix(), sap2.prefix());
            self.propose_handover_sap(SapCandidates::SectionSplit(sap1, sap2)).await
        } else {
            debug!("Waiting for more split handover candidates");
            Ok(vec![])
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_new_elders_agreement(
        &self,
        signed_section_auth: SectionAuth<SectionAuthorityProvider>,
        key_sig: KeyedSig,
    ) -> Result<Vec<Cmd>> {
        trace!("{}", LogMarker::HandlingNewEldersAgreement);
        let updates = self.split_barrier.write().await.process(
            &self.network_knowledge.prefix().await,
            signed_section_auth.clone(),
            key_sig,
        );

        if updates.is_empty() {
            return Ok(vec![]);
        }

        let snapshot = self.state_snapshot().await;
        let old_chain = self.section_chain().await.clone();

        for (signed_sap, key_sig) in updates {
            let prefix = signed_sap.prefix();
            trace!("{}: for {:?}", LogMarker::NewSignedSap, prefix);

            info!("New SAP agreed for:{}", *signed_sap);

            let our_name = self.info.read().await.name();

            // Let's update our network knowledge, including our
            // section SAP and chain if the new SAP's prefix matches our name
            // We need to generate the proof chain to connect our current chain to new SAP.
            let mut proof_chain = old_chain.clone();
            match proof_chain.insert(
                old_chain.last_key(),
                signed_sap.section_key(),
                key_sig.signature,
            ) {
                Err(err) => error!("Failed to generate proof chain for new SAP: {:?}", err),
                Ok(()) => match self
                    .network_knowledge
                    .update_knowledge_if_valid(
                        signed_sap.clone(),
                        &proof_chain,
                        None,
                        &our_name,
                        &self.section_keys_provider,
                    )
                    .await
                {
                    Err(err) => error!(
                        "Error updating our network knowledge for {:?}: {:?}",
                        prefix, err
                    ),
                    Ok(true) => {
                        info!("Updated our network knowledge for {:?}", prefix);
                        info!("Writing updated knowledge to disk");
                        self.write_prefix_map().await
                    }
                    _ => {}
                },
            }
        }

        info!(
            "Prefixes we know about: {:?}",
            self.network_knowledge.prefix_map()
        );

        self.log_section_stats().await;
        self.update_self_for_new_node_state(snapshot).await
    }
}
