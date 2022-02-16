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
        sig: KeyedSig,
    ) -> Result<Vec<Cmd>> {
        let equal_or_extension = section_auth.prefix() == self.network_knowledge.prefix().await
            || section_auth
                .prefix()
                .is_extension_of(&self.network_knowledge.prefix().await);

        if equal_or_extension {
            // Our section or sub-section
            debug!(
                "Updating section info for our prefix: {:?}",
                section_auth.prefix()
            );

            let signed_section_auth = SectionAuth::new(section_auth, sig.clone());
            let saps_candidates = self
                .network_knowledge
                .promote_and_demote_elders(&self.info.read().await.name(), &BTreeSet::new())
                .await;

            if !saps_candidates.contains(&signed_section_auth.elder_candidates()) {
                // SectionInfo out of date, ignore.
                return Ok(vec![]);
            }

            // Send the `NewElders` proposal to all of the to-be-Elders so it's aggregated by them.
            let proposal = Proposal::NewElders(signed_section_auth);
            let proposal_recipients = saps_candidates
                .iter()
                .flat_map(|info| info.elders())
                .cloned()
                .collect();

            let section_key = self.network_knowledge.section_key().await;
            let key_share = self
                .section_keys_provider
                .key_share(&section_key)
                .await
                .map_err(|err| {
                    trace!("Can't propose {:?}: {:?}", proposal, err);
                    err
                })?;

            self.send_proposal(proposal_recipients, proposal, &key_share)
                .await
        } else {
            // Other section. We shouln't be receiving or updating a SAP for
            // a remote section here, that is done with a AE msg response.
            debug!(
                "Ignoring Proposal::SectionInfo since prefix doesn't match ours: {:?}",
                section_auth
            );
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
