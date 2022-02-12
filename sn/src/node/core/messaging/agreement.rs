// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    system::{DkgSessionId, JoinResponse, SectionAuth, SystemMsg},
    DstLocation, WireMsg,
};
use crate::node::{
    api::cmds::Cmd,
    core::{Node, Proposal},
    dkg::DkgSessionIdUtils,
    messages::WireMsgUtils,
    network_knowledge::{ElderCandidates, NodeState, SectionKeyShare},
    Result,
};
use crate::types::{log_markers::LogMarker, Peer};

use xor_name::XorName;

impl Node {
    /// Send proposal to all our elders.
    pub(crate) async fn propose(&self, proposal: Proposal) -> Result<Vec<Cmd>> {
        let elders = self
            .network_knowledge
            .authority_provider()
            .await
            .elders_vec();
        self.send_proposal(elders, proposal).await
    }

    /// Send `proposal` to `recipients`.
    pub(crate) async fn send_proposal(
        &self,
        recipients: Vec<Peer>,
        proposal: Proposal,
    ) -> Result<Vec<Cmd>> {
        let section_key = self.network_knowledge.section_key().await;

        let key_share = self
            .section_keys_provider
            .key_share(&section_key)
            .await
            .map_err(|err| {
                trace!("Can't propose {:?}: {:?}", proposal, err);
                err
            })?;

        self.send_proposal_with(recipients, proposal, &key_share)
            .await
    }

    /// Send `proposal` to `recipients` signing it with the provided key share.
    pub(crate) async fn send_proposal_with(
        &self,
        recipients: Vec<Peer>,
        proposal: Proposal,
        key_share: &SectionKeyShare,
    ) -> Result<Vec<Cmd>> {
        trace!(
            "Propose {:?}, key_share: {:?}, aggregators: {:?}",
            proposal,
            key_share,
            recipients,
        );

        let sig_share = proposal.sign_with_key_share(
            key_share.public_key_set.clone(),
            key_share.index,
            &key_share.secret_key_share,
        )?;

        // Broadcast the proposal to the rest of the section elders.
        let node_msg = SystemMsg::Propose {
            proposal: proposal.clone().into_msg(),
            sig_share: sig_share.clone(),
        };
        // Name of the section_pk may not matches the section prefix.
        // Carry out a substitution to prevent the dst_location becomes other section.
        let section_key = self.network_knowledge.section_key().await;
        let wire_msg = WireMsg::single_src(
            &self.info.read().await.clone(),
            DstLocation::Section {
                name: self.network_knowledge.prefix().await.name(),
                section_pk: section_key,
            },
            node_msg,
            section_key,
        )?;

        let msg_id = wire_msg.msg_id();

        let mut cmds = vec![];
        let our_name = self.info.read().await.name();
        // handle ourselves if we should
        for peer in recipients.clone() {
            if peer.name() == our_name {
                cmds.extend(
                    self.handle_proposal(msg_id, proposal.clone(), sig_share.clone(), peer)
                        .await?,
                )
            }
        }

        // remove ourself from recipients
        let recipients = recipients
            .into_iter()
            .filter(|peer| peer.name() != our_name)
            .collect();

        cmds.extend(
            self.send_messages_to_all_nodes_or_directly_handle_for_accumulation(
                recipients, wire_msg,
            )
            .await?,
        );

        Ok(cmds)
    }

    // Send `NodeApproval` to a joining node which makes it a section member
    pub(crate) async fn send_node_approval(&self, node_state: SectionAuth<NodeState>) -> Vec<Cmd> {
        let peer = node_state.peer().clone();
        let prefix = self.network_knowledge.prefix().await;
        info!("Our section with {:?} has approved peer {}.", prefix, peer,);

        let node_msg = SystemMsg::JoinResponse(Box::new(JoinResponse::Approval {
            genesis_key: *self.network_knowledge.genesis_key(),
            section_auth: self
                .network_knowledge
                .section_signed_authority_provider()
                .await
                .into_authed_msg(),
            node_state: node_state.into_authed_msg(),
            section_chain: self.network_knowledge.section_chain().await,
        }));

        let dst_section_pk = self.network_knowledge.section_key().await;
        trace!("{}", LogMarker::SendNodeApproval);
        match self
            .send_direct_msg(peer.clone(), node_msg, dst_section_pk)
            .await
        {
            Ok(cmd) => vec![cmd],
            Err(err) => {
                error!("Failed to send join approval to node {}: {:?}", peer, err);
                vec![]
            }
        }
    }

    /// Send a `DkgStart` message to the provided set of candidates
    pub(crate) async fn send_dkg_start(
        &self,
        elder_candidates: ElderCandidates,
    ) -> Result<Vec<Cmd>> {
        let src_prefix = elder_candidates.prefix();
        let generation = self.network_knowledge.chain_len().await;
        let session_id = DkgSessionId::new(&elder_candidates, generation);

        // Send DKG start to all candidates
        let recipients: Vec<_> = elder_candidates.elders().cloned().collect();

        trace!(
            "Send DkgStart for {:?} with {:?} to {:?}",
            elder_candidates,
            session_id,
            recipients
        );

        let node_msg = SystemMsg::DkgStart {
            session_id,
            prefix: elder_candidates.prefix(),
            elders: elder_candidates
                .elders()
                .map(|peer| (peer.name(), peer.addr()))
                .collect(),
        };
        let section_pk = self.network_knowledge.section_key().await;
        self.send_msg_for_dst_accumulation(
            src_prefix.name(),
            DstLocation::Section {
                name: src_prefix.name(),
                section_pk,
            },
            node_msg,
            recipients,
        )
        .await
    }

    async fn send_msg_for_dst_accumulation(
        &self,
        src: XorName,
        dst: DstLocation,
        node_msg: SystemMsg,
        recipients: Vec<Peer>,
    ) -> Result<Vec<Cmd>> {
        let section_key = self.network_knowledge.section_key().await;

        let key_share = self
            .section_keys_provider
            .key_share(&section_key)
            .await
            .map_err(|err| {
                trace!(
                    "Can't create message {:?} for accumulation at dst {:?}: {:?}",
                    node_msg,
                    dst,
                    err
                );
                err
            })?;

        let wire_msg = WireMsg::for_dst_accumulation(&key_share, src, dst, node_msg, section_key)?;

        trace!(
            "Send {:?} for accumulation at dst to {:?}",
            wire_msg,
            recipients
        );

        Ok(self
            .send_messages_to_all_nodes_or_directly_handle_for_accumulation(recipients, wire_msg)
            .await?)
    }
}
