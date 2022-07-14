// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd, relocation::ChurnId, Event, MembershipEvent, Node, Proposal, Result,
};
use sn_consensus::Decision;
use sn_interface::{
    messaging::system::{
        JoinResponse, KeyedSig, MembershipState, NodeState as NodeStateMsg, SectionAuth, SystemMsg,
    },
    network_knowledge::{
        NodeState, SapCandidate, SectionAuthUtils, SectionAuthorityProvider, MIN_ADULT_AGE,
    },
    types::log_markers::LogMarker,
};
use std::collections::BTreeSet;

// Agreement
impl Node {
    // Send `NodeApproval` to a joining node which makes it a section member
    pub(crate) fn send_node_approval(&self, decision: Decision<NodeStateMsg>) -> Vec<Cmd> {
        let peers = Vec::from_iter(
            decision
                .proposals
                .keys()
                .filter(|n| n.state == MembershipState::Joined)
                .map(|n| n.peer()),
        );
        let prefix = self.network_knowledge.prefix();
        info!("Section {prefix:?} has approved new peers {peers:?}.");

        let node_msg = SystemMsg::JoinResponse(Box::new(JoinResponse::Approval {
            genesis_key: *self.network_knowledge.genesis_key(),
            section_auth: self
                .network_knowledge
                .section_signed_authority_provider()
                .into_authed_msg(),
            section_chain: self.network_knowledge.section_chain(),
            decision,
        }));

        let sap = self.network_knowledge.authority_provider();
        let dst_section_pk = sap.section_key();
        let section_name = sap.prefix().name();

        trace!("{}", LogMarker::SendNodeApproval);
        match self.send_direct_msg_to_nodes(peers.clone(), node_msg, section_name, dst_section_pk) {
            Ok(cmd) => vec![cmd],
            Err(err) => {
                error!("Failed to send join approval to new peers {peers:?}: {err:?}");
                vec![]
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_general_agreements(
        &mut self,
        proposal: Proposal,
        sig: KeyedSig,
    ) -> Result<Vec<Cmd>> {
        debug!("{:?} {:?}", LogMarker::ProposalAgreed, proposal);
        match proposal {
            Proposal::Offline(node_state) => self.handle_offline_agreement(node_state, sig),
            Proposal::SectionInfo(sap) => self.handle_section_info_agreement(sap, sig).await,
            Proposal::NewElders(_) => {
                error!("Elders agreement should be handled in a separate blocking fashion");
                Ok(vec![])
            }
            Proposal::JoinsAllowed(joins_allowed) => {
                self.joins_allowed = joins_allowed;
                Ok(vec![])
            }
        }
    }

    pub(crate) async fn handle_online_agreement(
        &mut self,
        decision: Decision<NodeStateMsg>,
    ) -> Result<Vec<Cmd>> {
        debug!("{}", LogMarker::AgreementOfOnline);
        let mut cmds = vec![];

        cmds.extend(self.send_node_approval(decision.clone()));
        let joining_nodes = Vec::from_iter(
            decision
                .proposals
                .clone()
                .into_iter()
                .filter(|(n, _)| n.state == MembershipState::Joined),
        );

        for (new_info, signature) in joining_nodes.iter().cloned() {
            if let Some(old_info) = self
                .network_knowledge
                .is_either_member_or_archived(&new_info.name)
            {
                // We would approve and relocate it only if half its age is at least MIN_ADULT_AGE
                let new_age = old_info.age() / 2;
                if new_age >= MIN_ADULT_AGE {
                    cmds.extend(self.relocate_rejoining_peer(old_info.value, new_age)?);

                    continue;
                }
            }

            let sig = KeyedSig {
                public_key: self.network_knowledge.section_key(),
                signature,
            };

            let new_info = SectionAuth {
                value: new_info.into_state(),
                sig,
            };

            if !self.network_knowledge.update_member(new_info.clone()) {
                info!("ignore Online: {}", new_info.peer());
                return Ok(vec![]);
            }

            self.add_new_adult_to_trackers(new_info.name());

            info!("handle Online: {}", new_info.peer());

            // still used for testing
            self.send_event(Event::Membership(MembershipEvent::MemberJoined {
                name: new_info.name(),
                previous_name: new_info.previous_name(),
                age: new_info.age(),
            }))
            .await;
        }

        self.log_section_stats();

        // Do not disable node joins in first section.
        let our_prefix = self.network_knowledge.prefix();
        if !our_prefix.is_empty() {
            // ..otherwise, switch off joins_allowed on a node joining.
            // TODO: fix racing issues here? https://github.com/maidsafe/safe_network/issues/890
            self.joins_allowed = false;
        }

        if let Some((_, sig)) = joining_nodes.iter().max_by_key(|(_, sig)| sig) {
            let churn_id = ChurnId(sig.to_bytes().to_vec());
            let excluded_from_relocation =
                BTreeSet::from_iter(joining_nodes.iter().map(|(n, _)| n.name));

            cmds.extend(self.relocate_peers(churn_id, excluded_from_relocation)?);
        }

        let result = self.promote_and_demote_elders_except(&BTreeSet::default())?;

        if result.is_empty() {
            // Send AE-Update to our section
            cmds.extend(self.send_ae_update_to_our_section());
        }

        cmds.extend(result);

        info!("cmds in queue for Accepting node {:?}", cmds);

        self.print_network_stats();

        Ok(cmds)
    }

    #[instrument(skip(self))]
    fn handle_offline_agreement(
        &mut self,
        node_state: NodeState,
        sig: KeyedSig,
    ) -> Result<Vec<Cmd>> {
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
    ) -> Result<Vec<Cmd>> {
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
        let dkg_sessions = self.promote_and_demote_elders(&BTreeSet::new())?;

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
            self.propose_handover_consensus(SapCandidate::SectionSplit(
                sap1.to_owned(),
                sap2.to_owned(),
            ))
        } else {
            debug!("Waiting for more split handover candidates");
            Ok(vec![])
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
        let old_chain = self.section_chain().clone();

        let prefix = signed_section_auth.prefix();
        trace!("{}: for {:?}", LogMarker::NewSignedSap, prefix);

        info!("New SAP agreed for:{}", *signed_section_auth);

        let our_name = self.info().name();

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
                match self.network_knowledge.update_knowledge_if_valid(
                    signed_section_auth.clone(),
                    &proof_chain,
                    None,
                    &our_name,
                    &self.section_keys_provider,
                ) {
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
