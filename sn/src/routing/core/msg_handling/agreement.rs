// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::{cmp, collections::BTreeSet};

use crate::messaging::{
    system::{KeyedSig, MembershipState, NodeState, Proposal, SectionAuth},
    SectionAuthorityProvider,
};
use crate::routing::{
    dkg::SectionAuthUtils,
    error::Result,
    log_markers::LogMarker,
    network_knowledge::{ElderCandidatesUtils, NodeStateUtils},
    routing_api::command::Command,
    Event, SectionAuthorityProviderUtils, MIN_AGE,
};

use super::Core;

// Agreement
impl Core {
    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_non_elder_agreement(
        &self,
        proposal: Proposal,
        sig: KeyedSig,
    ) -> Result<Vec<Command>> {
        debug!("handle agreement on {:?}", proposal);
        match proposal {
            Proposal::Online { node_state, .. } => {
                self.handle_online_agreement(node_state, sig).await
            }
            Proposal::Offline(node_state) => self.handle_offline_agreement(node_state, sig).await,
            Proposal::SectionInfo(section_auth) => {
                self.handle_section_info_agreement(section_auth, sig).await
            }
            Proposal::OurElders(_) => {
                error!("Elders agreement should be handled in a separate blocking fashion");
                Ok(vec![])
            }
            Proposal::JoinsAllowed(joins_allowed) => {
                *self.joins_allowed.write().await = joins_allowed;
                Ok(vec![])
            }
        }
    }

    async fn handle_online_agreement(
        &self,
        new_info: NodeState,
        sig: KeyedSig,
    ) -> Result<Vec<Command>> {
        debug!("{}", LogMarker::AgreementOfOnline);
        let mut commands = vec![];
        if let Some(old_info) = self
            .network_knowledge
            .members()
            .get_section_signed(&new_info.name)
        {
            // This node is rejoin with same name.

            if old_info.state != MembershipState::Left {
                debug!(
                    "Ignoring Online node {} - {:?} not Left.",
                    new_info.name, old_info.state,
                );

                return Ok(commands);
            }

            let new_age = cmp::max(MIN_AGE, old_info.age() / 2);

            if new_age > MIN_AGE {
                // TODO: consider handling the relocation inside the bootstrap phase, to avoid
                // having to send this `NodeApproval`.
                commands.push(self.send_node_approval(old_info.clone()).await?);

                let peer = new_info.to_peer();
                commands.extend(self.relocate_rejoining_peer(&peer, new_age).await?);

                return Ok(commands);
            }
        }

        let new_info = SectionAuth {
            value: new_info,
            sig,
        };

        if !self.network_knowledge.update_member(new_info.clone()).await {
            info!("ignore Online: {} at {}", new_info.name, new_info.addr);
            return Ok(vec![]);
        }

        info!("handle Online: {} at {}", new_info.name, new_info.addr);

        self.send_event(Event::MemberJoined {
            name: new_info.name,
            previous_name: new_info.previous_name,
            age: new_info.age(),
        })
        .await;

        commands.extend(
            self.relocate_peers(&new_info.name, &new_info.sig.signature)
                .await?,
        );

        let result = self.promote_and_demote_elders().await?;
        if result.is_empty() {
            commands.extend(self.send_ae_update_to_adults().await?);
        }

        commands.extend(result);
        commands.push(self.send_node_approval(new_info).await?);

        self.print_network_stats().await;

        Ok(commands)
    }

    async fn handle_offline_agreement(
        &self,
        node_state: NodeState,
        sig: KeyedSig,
    ) -> Result<Vec<Command>> {
        let mut commands = vec![];
        let age = node_state.age();
        let signature = sig.signature.clone();

        if !self
            .network_knowledge
            .update_member(SectionAuth {
                value: node_state,
                sig,
            })
            .await
        {
            info!("ignore Offline: {} at {}", node_state.name, node_state.addr);
            return Ok(commands);
        }

        info!("handle Offline: {} at {}", node_state.name, node_state.addr);

        commands.extend(self.relocate_peers(&node_state.name, &signature).await?);

        let result = self.promote_and_demote_elders().await?;
        if result.is_empty() {
            commands.extend(self.send_ae_update_to_adults().await?);
        }

        commands.extend(result);

        self.send_event(Event::MemberLeft {
            name: node_state.name,
            age,
        })
        .await;

        Ok(commands)
    }

    #[instrument(skip(self), level = "trace")]
    async fn handle_section_info_agreement(
        &self,
        section_auth: SectionAuthorityProvider,
        sig: KeyedSig,
    ) -> Result<Vec<Command>> {
        let equal_or_extension = section_auth.prefix() == self.network_knowledge.prefix().await
            || section_auth
                .prefix()
                .is_extension_of(&self.network_knowledge.prefix().await);

        if equal_or_extension {
            debug!(
                "Updating section info for our prefix: {:?}",
                section_auth.prefix
            );
            // Our section or sub-section
            let signed_section_auth = SectionAuth::new(section_auth, sig.clone());
            let infos = self
                .network_knowledge
                .promote_and_demote_elders(&self.node.read().await.name(), &BTreeSet::new())
                .await;
            if !infos.contains(&signed_section_auth.elder_candidates()) {
                // SectionInfo out of date, ignore.
                return Ok(vec![]);
            }

            // Send a `AE Update` message to all the to-be-promoted members so they have the full
            // section and network data.
            let mut ae_update_recipients = vec![];

            let mut peers = vec![];

            for elder_candidate in infos.clone() {
                peers.extend(elder_candidate.peers())
            }

            for peer in peers {
                if !self.network_knowledge.is_elder(&peer.name()).await {
                    ae_update_recipients.push((peer.name(), peer.addr()));
                }
            }

            let mut commands = vec![];
            if !ae_update_recipients.is_empty() {
                let node_msg = self.generate_ae_update(sig.public_key, true).await?;
                let cmd = self
                    .send_direct_message_to_nodes(
                        ae_update_recipients,
                        node_msg,
                        self.network_knowledge.prefix().await.name(),
                        sig.public_key,
                    )
                    .await?;

                commands.push(cmd);
            }

            // Send the `OurElder` proposal to all of the to-be-elders so it's aggregated by them.
            let our_elders_recipients: Vec<_> =
                infos.clone().iter().flat_map(|info| info.peers()).collect();
            commands.extend(
                self.send_proposal(
                    our_elders_recipients,
                    Proposal::OurElders(signed_section_auth),
                )
                .await?,
            );

            Ok(commands)
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
    pub(crate) async fn handle_our_elders_agreement(
        &self,
        signed_section_auth: SectionAuth<SectionAuthorityProvider>,
        key_sig: KeyedSig,
    ) -> Result<Vec<Command>> {
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
            let prefix = signed_sap.prefix;
            info!("New SAP agreed for {:?}: {:?}", prefix, signed_sap);

            // If we have the key share for new SAP key we can switch to this new SAP
            let switch_to_new_sap = self.is_not_elder().await
                || self
                    .section_keys_provider
                    .key_share(&signed_sap.section_key())
                    .await
                    .is_ok();

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
                        signed_sap,
                        &proof_chain,
                        None,
                        &self.node.read().await.name(),
                        switch_to_new_sap,
                    )
                    .await
                {
                    Err(err) => error!(
                        "Error updating our network knowledge for {:?}: {:?}",
                        prefix, err
                    ),
                    Ok(true) => {
                        info!("Updated our network knowledge for {:?}", prefix);
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

        self.update_self_for_new_node_state_and_fire_events(snapshot)
            .await
    }
}
