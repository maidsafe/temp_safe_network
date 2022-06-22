// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    api::cmds::Cmd,
    core::{relocation::ChurnId, Node, Proposal},
    Event, MembershipEvent, Result,
};

use bls_dkg::PublicKeySet;
use sn_consensus::{Decision, Generation};
use sn_interface::{
    messaging::system::{KeyedSig, MembershipState, NodeState as NodeStateMsg, SectionAuth},
    network_knowledge::{
        NodeState, SapCandidate, SectionAuthUtils, SectionAuthorityProvider, MIN_ADULT_AGE,
    },
    types::log_markers::LogMarker,
};

use std::collections::BTreeSet;

// Agreement
impl Node {
    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_general_agreements(
        &self,
        proposal: Proposal,
        sig: KeyedSig,
    ) -> Result<Vec<Cmd>> {
        debug!("{:?} {:?}", LogMarker::ProposalAgreed, proposal);
        match proposal {
            Proposal::Offline(node_state) => self.handle_offline_agreement(node_state, sig).await,
            Proposal::SectionInfo { sap, generation } => {
                self.handle_section_info_agreement(sap, sig, generation)
                    .await
            }
            Proposal::NewElders(_) => {
                error!("Elders agreement should be handled in a separate blocking fashion");
                Ok(vec![])
            }
            Proposal::JoinsAllowed(joins_allowed) => {
                *self.joins_allowed.write().await = joins_allowed;
                Ok(vec![])
            }
        }
    }

    pub(crate) async fn handle_membership_decision(
        &self,
        section_key_set: PublicKeySet,
        decision: Decision<NodeStateMsg>,
    ) -> Result<Vec<Cmd>> {
        let mut cmds = vec![];

        if !self
            .network_knowledge
            .update_members(&section_key_set, decision.clone())
            .await
        {
            info!("Skipping membership decision");
            return Ok(cmds);
        }

        for node_state in decision.proposals() {
            if node_state.state == MembershipState::Joined {
                // Node has been accepted into the section
                cmds.extend(self.handle_node_joined(node_state).await?)
            } else {
                // Node was removed from the section
                cmds.extend(
                    self.handle_node_left(node_state, &section_key_set, &decision)
                        .await?,
                )
            }
        }

        // first things first, inform the node it can join us
        cmds.extend(self.send_node_approvals(decision.clone()).await);

        let churn_id = ChurnId(decision.proposals_sig.to_bytes().to_vec());
        let excluded_from_relocation = BTreeSet::from_iter(
            decision
                .proposals()
                .into_iter()
                .filter(|n| n.state == MembershipState::Joined)
                .map(|n| n.name),
        );

        cmds.extend(
            self.relocate_peers(churn_id, excluded_from_relocation)
                .await?,
        );

        let promote_demote_cmds = self
            .promote_and_demote_elders_except(&BTreeSet::default())
            .await?;

        if promote_demote_cmds.is_empty() {
            // Send AE-Update to our section
            cmds.extend(self.send_ae_update_to_our_section().await);
        }

        cmds.extend(promote_demote_cmds);

        self.liveness_retain_only(
            self.network_knowledge
                .adults()
                .await
                .iter()
                .map(|peer| peer.name())
                .collect(),
        )
        .await?;

        info!("cmds in queue for membership churn {:?}", cmds);

        Ok(cmds)
    }

    async fn handle_node_joined(&self, new_info: NodeStateMsg) -> Result<Vec<Cmd>> {
        debug!(
            "{}, {} at {}",
            LogMarker::AgreementOfOnline,
            new_info.name,
            new_info.addr
        );

        let mut cmds = vec![];

        if let Some(old_info) = self.network_knowledge.is_archived(&new_info.name).await {
            // This node is rejoin with same name.
            let new_age = MIN_ADULT_AGE.max(old_info.age() / 2);
            cmds.extend(self.relocate_rejoining_peer(old_info, new_age).await?);
            return Ok(cmds);
        }

        self.add_new_adult_to_trackers(new_info.name).await;

        // Do not disable node joins in first section.
        if self.network_knowledge.prefix().await.bit_count() > 0 {
            // ..otherwise, switch off joins_allowed on a node joining.
            // TODO: fix racing issues here? https://github.com/maidsafe/safe_network/issues/890
            *self.joins_allowed.write().await = false;
        }

        info!("cmds in queue for Accepting node {:?}", cmds);

        self.log_section_stats().await;
        self.print_network_stats().await;

        // still used for testing
        self.send_event(Event::Membership(MembershipEvent::MemberJoined {
            name: new_info.name,
            previous_name: new_info.previous_name,
            age: new_info.age(),
        }))
        .await;

        Ok(cmds)
    }

    #[instrument(skip(self))]
    async fn handle_offline_agreement(
        &self,
        node_state: NodeState,
        sig: KeyedSig,
    ) -> Result<Vec<Cmd>> {
        info!(
            "Agreement - proposing membership change with node offline: {} at {}",
            node_state.name(),
            node_state.addr()
        );

        self.propose_membership_change(node_state.to_msg()).await
    }

    #[instrument(skip(self), level = "trace")]
    async fn handle_section_info_agreement(
        &self,
        section_auth: SectionAuthorityProvider,
        sig: KeyedSig,
        generation: Generation,
    ) -> Result<Vec<Cmd>> {
        // check if section matches our prefix
        let equal_prefix = section_auth.prefix() == self.network_knowledge.prefix().await;
        let is_extension_prefix = section_auth
            .prefix()
            .is_extension_of(&self.network_knowledge.prefix().await);
        if !equal_prefix && !is_extension_prefix {
            // Other section. We shouln't be receiving or updating a SAP for
            // a remote section here, that is done with a AE msg response.
            debug!(
                "Ignoring Proposal::SectionInfo since prefix doesn't match ours: {:?}",
                section_auth
            );
            return Ok(vec![]);
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
        let dkg_sessions = self.promote_and_demote_elders(&BTreeSet::new()).await?;

        let agreeing_elders = BTreeSet::from_iter(signed_section_auth.names());
        if dkg_sessions
            .iter()
            .all(|session| !session.elder_names().eq(agreeing_elders.iter().copied()))
        {
            warn!("SectionInfo out of date, ignore");
            return Ok(vec![]);
        };

        // handle regular elder handover (1 to 1)
        // trigger handover consensus among elders
        if equal_prefix {
            debug!(
                "Propose elder handover to: {:?}",
                signed_section_auth.prefix()
            );
            return self
                .propose_handover_consensus(SapCandidate::ElderHandover(signed_section_auth))
                .await;
        }

        // manage pending split SAP candidates
        // NB TODO temporary while we wait for Membership generations and possibly double DKG
        let chosen_candidates = self
            .split_barrier
            .write()
            .await
            .process(
                &self.network_knowledge.prefix().await,
                signed_section_auth.clone(),
                sig.clone(),
                generation,
            )
            .await;

        // handle section split (1 to 2)
        if let [(sap1, _sig1), (sap2, _sig2)] = chosen_candidates.as_slice() {
            debug!(
                "Propose section split handover to: {:?} {:?}",
                sap1.prefix(),
                sap2.prefix()
            );
            self.propose_handover_consensus(SapCandidate::SectionSplit(
                sap1.to_owned(),
                sap2.to_owned(),
            ))
            .await
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
        let snapshot = self.state_snapshot().await;
        let old_chain = self.section_chain().await.clone();

        let prefix = signed_section_auth.prefix();
        trace!("{}: for {:?}", LogMarker::NewSignedSap, prefix);

        info!("New SAP agreed for:{}", *signed_section_auth);

        let our_name = self.info.read().await.name();

        // Let's update our network knowledge, including our
        // section SAP and chain if the new SAP's prefix matches our name
        // We need to generate the proof chain to connect our current chain to new SAP.
        let mut proof_chain = old_chain.clone();
        match proof_chain.insert(
            old_chain.last_key(),
            signed_section_auth.section_key(),
            key_sig.signature,
        ) {
            Err(err) => error!(
                "Failed to generate proof chain for a newly received SAP: {:?}",
                err
            ),
            Ok(()) => {
                match self
                    .network_knowledge
                    .update_knowledge_if_valid(
                        signed_section_auth.clone(),
                        &proof_chain,
                        BTreeSet::new(),
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
                }
            }
        }

        info!(
            "Prefixes we know about: {:?}",
            self.network_knowledge.prefix_map()
        );

        self.update_self_for_new_node_state(snapshot).await
    }
}
