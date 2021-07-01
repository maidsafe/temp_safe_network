// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{delivery_group, enduser_registry::SocketId, Core};
use crate::messaging::{
    node::{DstInfo, Network, NodeMsg, NodeState, Peer, Proposal, Section},
    EndUser, Itinerary, MessageId, MessageType, SectionAuthorityProvider, SrcLocation, WireMsg,
};
use crate::routing::{
    dkg::commands::DkgCommands,
    error::Result,
    messages::WireMsgUtils,
    network::NetworkUtils,
    node::Node,
    peer::PeerUtils,
    routing_api::command::Command,
    section::{NodeStateUtils, SectionAuthorityProviderUtils, SectionUtils},
    Error, Event,
};
use secured_linked_list::SecuredLinkedList;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use xor_name::{Prefix, XorName};

impl Core {
    // Creates `Core` for the first node in the network
    pub fn first_node(node: Node, event_tx: mpsc::Sender<Event>) -> Result<Self> {
        let (section, section_key_share) = Section::first_node(node.peer())?;
        Ok(Self::new(node, section, Some(section_key_share), event_tx))
    }

    pub fn get_enduser_by_addr(&self, sender: &SocketAddr) -> Option<&EndUser> {
        self.end_users.get_enduser_by_addr(sender)
    }

    pub fn get_socket_addr(&self, id: SocketId) -> Option<&SocketAddr> {
        self.end_users.get_socket_addr(id)
    }

    pub fn try_add_enduser(&mut self, sender: SocketAddr) -> Result<EndUser> {
        let section_prefix = self.section.prefix();
        self.end_users.try_add(sender, section_prefix)
    }

    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn section(&self) -> &Section {
        &self.section
    }

    pub fn section_chain(&self) -> &SecuredLinkedList {
        self.section.chain()
    }

    pub fn network(&self) -> &Network {
        &self.network
    }

    /// Is this node an elder?
    pub fn is_elder(&self) -> bool {
        self.section.is_elder(&self.node.name())
    }

    pub fn is_not_elder(&self) -> bool {
        !self.is_elder()
    }

    /// Tries to sign with the secret corresponding to the provided BLS public key
    pub fn sign_with_section_key_share(
        &self,
        data: &[u8],
        public_key: &bls::PublicKey,
    ) -> Result<(usize, bls::SignatureShare)> {
        self.section_keys_provider.sign_with(data, public_key)
    }

    /// Returns the current BLS public key set
    pub fn public_key_set(&self) -> Result<bls::PublicKeySet> {
        Ok(self
            .section_keys_provider
            .key_share()?
            .public_key_set
            .clone())
    }

    /// Returns the latest known public key of the section with `prefix`.
    pub fn section_key(&self, prefix: &Prefix) -> Option<bls::PublicKey> {
        if prefix == self.section.prefix() || prefix.is_extension_of(self.section.prefix()) {
            Some(*self.section.chain().last_key())
        } else {
            self.network.key_by_prefix(prefix).or_else(|| {
                if self.is_elder() {
                    // We are elder - the first key is the genesis key
                    Some(*self.section.chain().root_key())
                } else {
                    // We are not elder - the chain might be truncated so the first key is not
                    // necessarily the genesis key.
                    None
                }
            })
        }
    }

    /// Returns the info about the section matching the name.
    pub fn matching_section(&self, name: &XorName) -> Result<SectionAuthorityProvider> {
        if self.section.prefix().matches(name) {
            Ok(self.section.authority_provider().clone())
        } else {
            self.network.section_by_name(name)
        }
    }

    /// Returns our index in the current BLS group if this node is a member of one, or
    /// `Error::MissingSecretKeyShare` otherwise.
    pub fn our_index(&self) -> Result<usize> {
        Ok(self.section_keys_provider.key_share()?.index)
    }

    pub async fn send_event(&self, event: Event) {
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
            .handle_timeout(&self.node.keypair, token)
            .into_commands(&self.node, *self.section_chain().last_key())
    }

    pub async fn add_to_filter(&mut self, wire_msg: &WireMsg) -> bool {
        self.msg_filter.add_to_filter(wire_msg.msg_id()).await
    }

    // Send message over the network.
    pub async fn relay_message(&self, mut wire_msg: WireMsg) -> Result<Option<Command>> {
        let dst_location = wire_msg.dst_location();
        let (presumed_targets, dg_size) = delivery_group::delivery_targets(
            dst_location,
            &self.node.name(),
            &self.section,
            &self.network,
        )?;

        let mut targets = vec![];

        for peer in presumed_targets {
            if self
                .msg_filter
                .filter_outgoing(&wire_msg, peer.name())
                .await
                .is_new()
            {
                let _ = targets.push((*peer.name(), *peer.addr()));
            }
        }

        if targets.is_empty() {
            return Ok(None);
        }

        trace!(
            "relay {:?} to first {:?} of {:?} (Section PK: {:?})",
            wire_msg,
            dg_size,
            targets,
            wire_msg.src_section_pk(),
        );

        let target_name = dst_location.name().ok_or(Error::CannotRoute)?;
        let dst_pk = self.section_key_by_name(&target_name);
        wire_msg.set_dst_section_pk(dst_pk);

        let command = Command::SendMessage {
            recipients: targets,
            delivery_group_size: dg_size,
            wire_msg,
        };

        Ok(Some(command))
    }

    /*
    TODO: it seems we don't need this, when the user calls to send
    a message it's simply relayed to the corresponding destinaion with Command::RelayMessage

        pub async fn send_user_message(&self, wire_msg: WireMsg) -> Result<Vec<Command>> {
            let dst_name = if let Some(name) = wire_msg.dst_location().name() {
                name
            } else {
                trace!(
                    "Not sending user message {:?} -> {:?}: direct dst not supported",
                    wire_msg.msg_kind(),
                    wire_msg.dst_location()
                );
                return Err(Error::InvalidDstLocation);
            };
            let dst_section_pk = self.section_key_by_name(&dst_name);

            /*
            unimplemented!();
            TODO: aggregation as a top level msg flag is being removed, we need
            to double check if the logic below is needed at all, or it can be simply
            removed since each msg type handler will ta<ke care of this

            // TODO: don't require this serialize or perhaps even variant altogether?
            let variant = NodeMsg::UserMessage(content.serialize()?.to_vec());

            // If the msg is to be aggregated at dst, we don't vote among our peers,
            // we simply send the msg as our vote to the dst.
            let msg = if itinerary.aggregate_at_dst() {
                NodeMsg::for_dst_accumulation(
                    self.section_keys_provider.key_share()?,
                    itinerary.src.name(),
                    itinerary.dst,
                    variant,
                    self.section.chain().clone(),
                )?
            } else if itinerary.aggregate_at_src() {
                let proposal = self.create_aggregate_at_src_proposal(itinerary.dst, variant, None)?;
                return self.propose(proposal);
            } else {
             NodeMsg::single_src(
                    &self.node,
                    dst,
                    variant,
                    self.section.authority_provider().section_key(),
                )?
            };
            */
            let mut commands = vec![];

            // TODO: consider removing this, we are getting duplicate msgs by it
            /*
            if itinerary
                .dst
                .contains(&self.node.name(), self.section.prefix())
            {
                commands.push(Command::HandleMessage {
                    sender: Some(self.node.addr),
                    message: msg.clone(),
                    dst_info: DstInfo {
                        dst: dst_name,
                        dst_section_pk,
                    },
                });
            }
            */

            commands.extend(self.relay_message(wire_msg).await?);

            Ok(commands)
        }
    */
    // Setting the JoinsAllowed triggers a round Proposal::SetJoinsAllowed to update the flag.
    pub fn set_joins_allowed(&self, joins_allowed: bool) -> Result<Vec<Command>> {
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

    pub async fn make_online_proposal(
        &self,
        peer: Peer,
        previous_name: Option<XorName>,
        dst_key: Option<bls::PublicKey>,
    ) -> Result<Vec<Command>> {
        self.propose(Proposal::Online {
            node_state: NodeState::joined(peer),
            previous_name,
            dst_key,
        })
    }
}
