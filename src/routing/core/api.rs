// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    delivery_group, enduser_registry::EndUserRegistry, split_barrier::SplitBarrier, Comm, Core,
    SignatureAggregator, KEY_CACHE_SIZE, RESOURCE_PROOF_DATA_SIZE, RESOURCE_PROOF_DIFFICULTY,
};
use crate::dbs::UsedSpace;
use crate::messaging::{
    node::{Network, NodeState, Peer, Proposal, Section},
    EndUser, MessageId, SectionAuthorityProvider, SocketId, WireMsg,
};
use crate::routing::{
    dkg::{DkgVoter, ProposalAggregator},
    error::Result,
    network::NetworkUtils,
    node::Node,
    peer::PeerUtils,
    routing_api::command::Command,
    section::{NodeStateUtils, SectionKeyShare, SectionKeysProvider, SectionUtils},
    Error, Event,
};
use resource_proof::ResourceProof;
use secured_linked_list::SecuredLinkedList;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use xor_name::XorName;

impl Core {
    // Creates `Core` for the first node in the network
    pub(crate) fn first_node(
        comm: Comm,
        mut node: Node,
        event_tx: mpsc::Sender<Event>,
        used_space: UsedSpace,
        root_storage_dir: PathBuf,
    ) -> Result<Self> {
        // make sure the Node has the correct local addr as Comm
        node.addr = comm.our_connection_info();

        let (section, section_key_share) = Section::first_node(node.peer())?;
        Self::new(
            comm,
            node,
            section,
            Some(section_key_share),
            event_tx,
            used_space,
            root_storage_dir,
        )
    }

    pub(crate) async fn relocated(&self, mut new_node: Node, new_section: Section) -> Result<Self> {
        let section_keys_provider = SectionKeysProvider::new(KEY_CACHE_SIZE, None);

        // make sure the new Node has the correct local addr as Comm
        let comm = self.comm.clone();
        new_node.addr = comm.our_connection_info();

        let network = Network::new();
        // TODO: to keep our knowledge of the network and avoid unnecessary AE msgs:
        // - clone self.network instead,
        // - remove the SAP of our new section from the cloned network
        // - and add current section's SAP to the cloned network

        Ok(Self {
            comm,
            node: new_node,
            section: new_section,
            network,
            section_keys_provider,
            proposal_aggregator: ProposalAggregator::default(),
            split_barrier: SplitBarrier::new(),
            message_aggregator: Arc::new(RwLock::new(SignatureAggregator::default())),
            dkg_voter: DkgVoter::default(),
            relocate_state: None,
            event_tx: self.event_tx.clone(),
            joins_allowed: true,
            resource_proof: ResourceProof::new(RESOURCE_PROOF_DATA_SIZE, RESOURCE_PROOF_DIFFICULTY),
            end_users: EndUserRegistry::new(),
            register_storage: self.register_storage.clone(),
            root_storage_dir: self.root_storage_dir.clone(),
            used_space: self.used_space.clone(),
            capacity: self.capacity.clone(),
            chunk_storage: self.chunk_storage.clone(),
            liveness: self.liveness.clone(),
        })
    }

    pub(crate) fn get_enduser_by_addr(&self, sender: &SocketAddr) -> Option<EndUser> {
        self.end_users.get_enduser_by_addr(sender)
    }

    pub(crate) fn get_socket_addr(&self, id: SocketId) -> Option<SocketAddr> {
        self.end_users.get_socket_addr(id)
    }

    pub(crate) fn try_add_enduser(&self, sender: SocketAddr) -> Result<EndUser> {
        let section_prefix = self.section.prefix();
        self.end_users.try_add(sender, section_prefix)
    }

    pub(crate) fn node(&self) -> &Node {
        &self.node
    }

    pub(crate) fn section(&self) -> &Section {
        &self.section
    }

    pub(crate) fn section_chain(&self) -> &SecuredLinkedList {
        self.section.chain()
    }

    pub(crate) fn network(&self) -> &Network {
        &self.network
    }

    /// Is this node an elder?
    pub(crate) fn is_elder(&self) -> bool {
        self.section.is_elder(&self.node.name())
    }

    pub(crate) fn is_not_elder(&self) -> bool {
        !self.is_elder()
    }

    /// Returns connection info of this node.
    pub(crate) fn our_connection_info(&self) -> SocketAddr {
        self.comm.our_connection_info()
    }

    /// Tries to sign with the secret corresponding to the provided BLS public key
    pub(crate) fn sign_with_section_key_share(
        &self,
        data: &[u8],
        public_key: &bls::PublicKey,
    ) -> Result<(usize, bls::SignatureShare)> {
        self.section_keys_provider.sign_with(data, public_key)
    }

    /// Returns the current BLS public key set
    pub(crate) fn public_key_set(&self) -> Result<bls::PublicKeySet> {
        Ok(self
            .section_keys_provider
            .key_share()?
            .public_key_set
            .clone())
    }

    /// Returns the info about the section matching the name.
    pub(crate) fn matching_section(&self, name: &XorName) -> Result<SectionAuthorityProvider> {
        if self.section.prefix().matches(name) {
            Ok(self.section.authority_provider().clone())
        } else {
            self.network.section_by_name(name)
        }
    }

    /// Returns our index in the current BLS group if this node is a member of one, or
    /// `Error::MissingSecretKeyShare` otherwise.
    pub(crate) fn our_index(&self) -> Result<usize> {
        Ok(self.section_keys_provider.key_share()?.index)
    }

    /// Returns our key share in the current BLS group if this node is a member of one, or
    /// `Error::MissingSecretKeyShare` otherwise.
    pub(crate) fn key_share(&self) -> Result<&SectionKeyShare> {
        self.section_keys_provider.key_share()
    }

    pub(crate) async fn send_event(&self, event: Event) {
        // Note: cloning the sender to avoid mutable access. Should have negligible cost.
        if self.event_tx.clone().send(event).await.is_err() {
            error!("Event receiver has been closed");
        }
    }

    // ----------------------------------------------------------------------------------------
    //   ---------------------------------- Mut ------------------------------------------
    // ----------------------------------------------------------------------------------------

    pub(crate) fn handle_timeout(&mut self, token: u64) -> Result<Vec<Command>> {
        self.dkg_voter
            .handle_timeout(&self.node, token, *self.section_chain().last_key())
    }

    // Send message over the network.
    pub(crate) fn relay_message(&self, mut wire_msg: WireMsg) -> Result<Command> {
        let dst_location = wire_msg.dst_location();
        let (targets, dg_size) = delivery_group::delivery_targets(
            dst_location,
            &self.node.name(),
            &self.section,
            &self.network,
        )?;

        trace!(
            "relay {:?} to first {:?} of {:?} (Section PK: {:?})",
            wire_msg,
            dg_size,
            targets,
            wire_msg.src_section_pk(),
        );

        let target_name = dst_location.name().ok_or(Error::CannotRoute)?;

        // To avoid loop: if destination is to Node, targets are multiple, self is an elder,
        //     self section prefix matches the destination name, then don't carry out a relay.
        if self.is_elder()
            && targets.len() > 1
            && dst_location.is_to_node()
            && self.section.prefix().matches(&target_name)
        {
            return Ok(Command::SendMessageDeliveryGroup {
                recipients: Vec::new(),
                delivery_group_size: 0,
                wire_msg,
            });
        }

        let dst_pk = self.section_key_by_name(&target_name);
        wire_msg.set_dst_section_pk(dst_pk);

        let command = Command::SendMessageDeliveryGroup {
            recipients: targets
                .into_iter()
                .map(|peer| (peer.name, peer.addr))
                .collect(),
            delivery_group_size: dg_size,
            wire_msg,
        };

        Ok(command)
    }

    // Setting the JoinsAllowed triggers a round Proposal::SetJoinsAllowed to update the flag.
    pub(crate) fn set_joins_allowed(&self, joins_allowed: bool) -> Result<Vec<Command>> {
        let mut commands = Vec::new();
        if self.is_elder() && joins_allowed != self.joins_allowed {
            let active_members: Vec<XorName> = self
                .section
                .active_members()
                .map(|peer| *peer.name())
                .collect();
            let msg_id = MessageId::from_content(&active_members)?;
            commands.extend(self.propose(Proposal::JoinsAllowed((msg_id, joins_allowed)))?);
        }
        Ok(commands)
    }

    // Generate a new section info based on the current set of members and if it differs from the
    // current elders, trigger a DKG.
    pub(crate) fn promote_and_demote_elders(&mut self) -> Result<Vec<Command>> {
        let mut commands = vec![];

        for info in self.section.promote_and_demote_elders(&self.node.name()) {
            commands.extend(self.send_dkg_start(info)?);
        }

        Ok(commands)
    }

    pub(crate) async fn make_online_proposal(
        &self,
        peer: Peer,
        previous_name: Option<XorName>,
        dst_key: Option<bls::PublicKey>,
    ) -> Result<Vec<Command>> {
        self.propose(Proposal::Online {
            node_state: NodeState::joined(peer, previous_name),
            dst_key,
        })
    }
}
