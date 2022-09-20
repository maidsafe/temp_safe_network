// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    comm::Comm,
    node::{
        core::DkgSessionInfo,
        flow_ctrl::cmds::Cmd,
        messaging::{OutgoingMsg, Peers},
        Error, Event, MembershipEvent, Node, Proposal as CoreProposal, Result, MIN_LEVEL_WHEN_FULL,
    },
    storage::Error as StorageError,
};

use bytes::Bytes;
use std::collections::BTreeSet;

#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::{
    messaging::{
        data::StorageLevel,
        signature_aggregator::Error as AggregatorError,
        system::{
            // SectionAuth is gonna cause issue
            JoinResponse,
            NodeCmd,
            NodeEvent,
            NodeMsg,
            NodeQuery,
            Proposal as ProposalMsg,
        },
        MsgId, NodeMsgAuthority, SectionAuth,
    },
    network_knowledge::NetworkKnowledge,
    types::{log_markers::LogMarker, Keypair, Peer, PublicKey},
};
use xor_name::XorName;

impl Node {
    /// Send a (`NodeMsg`) message to all Elders in our section
    pub(crate) fn send_msg_to_our_elders(&self, msg: NodeMsg) -> Cmd {
        let sap = self.network_knowledge.section_auth();
        let recipients = sap.elders_set();
        self.send_system_msg(msg, Peers::Multiple(recipients))
    }

    /// Send a (`NodeMsg`) message to a node
    pub(crate) fn send_system_msg(&self, msg: NodeMsg, recipients: Peers) -> Cmd {
        self.trace_system_msg(
            msg,
            recipients,
            #[cfg(feature = "traceroute")]
            Traceroute(vec![]),
        )
    }

    /// Send a (`NodeMsg`) message to a node
    pub(crate) fn trace_system_msg(
        &self,
        msg: NodeMsg,
        recipients: Peers,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Cmd {
        trace!("{}: {:?}", LogMarker::SendToNodes, msg);
        Cmd::send_traced_msg(
            OutgoingMsg::Node(msg),
            recipients,
            #[cfg(feature = "traceroute")]
            traceroute,
        )
    }

    /// Aggregation of system messages
    /// Returns an updated NodeMsgAuthority if
    /// msg was aggregated, or same as input if not
    /// of type [`NodeMsgAuthority::BlsShare`], else [`None`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn aggregate_system_msg(
        &mut self,
        msg_id: MsgId,
        msg_authority: NodeMsgAuthority,
        payload: Bytes,
    ) -> Option<NodeMsgAuthority> {
        // We assume to be aggregated if it contains a BLS Share sig as authority.
        match self.aggregate_msg(msg_authority, payload) {
            Ok(msg_authority) => msg_authority,
            Err(Error::InvalidSignatureShare) => {
                warn!(
                    "Invalid signature on received system message, dropping the message: {:?}",
                    msg_id
                );
                None
            }
            Err(err) => {
                trace!("aggregate_system_msg got error {:?}", err);
                None
            }
        }
    }

    // Handler for data messages which have successfully
    // passed all signature checks and msg verifications
    pub(crate) async fn handle_valid_system_msg(
        &mut self,
        msg_id: MsgId,
        msg_authority: NodeMsgAuthority,
        msg: NodeMsg,
        sender: Peer,
        comm: &Comm,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Result<Vec<Cmd>> {
        trace!("{:?}", LogMarker::NodeMsgToBeHandled);

        #[cfg(feature = "traceroute")]
        {
            if !traceroute.0.is_empty() {
                info!(
                    "Handling NodeMsg {}:{:?} with trace \n{:?}",
                    msg, msg_id, traceroute
                );
            }
        }

        match msg {
            NodeMsg::Relocate(node_state) => {
                trace!("Handling msg: Relocate from {}: {:?}", sender, msg_id);
                Ok(self
                    .handle_relocate(node_state)
                    .await?
                    .into_iter()
                    .collect())
            }
            NodeMsg::JoinAsRelocatedResponse(join_response) => {
                trace!("Handling msg: JoinAsRelocatedResponse from {}", sender);
                if let Some(ref mut joining_as_relocated) = self.relocate_state {
                    if let Some(cmd) =
                        joining_as_relocated.handle_join_response(*join_response, sender.addr())?
                    {
                        return Ok(vec![cmd]);
                    }
                } else {
                    error!(
                        "No relocation in progress upon receiving {:?}",
                        join_response
                    );
                }

                Ok(vec![])
            }
            NodeMsg::AntiEntropy {
                section_tree_update,
                kind,
            } => {
                trace!("Handling msg: AE from {sender}: {msg_id:?}");
                self.handle_anti_entropy_msg(
                    section_tree_update,
                    kind,
                    sender,
                    #[cfg(feature = "traceroute")]
                    traceroute,
                )
                .await
            }
            // Respond to a probe msg
            // We always respond to probe msgs if we're an elder as health checks use this to see if a node is alive
            // and repsonsive, as well as being a method of keeping nodes up to date.
            NodeMsg::AntiEntropyProbe(section_key) => {
                let mut cmds = vec![];
                if !self.is_elder() {
                    // early return here as we do not get health checks as adults,
                    // normal AE rules should have applied
                    return Ok(cmds);
                }

                trace!("Received Probe message from {}: {:?}", sender, msg_id);
                let mut recipients = BTreeSet::new();
                let _existed = recipients.insert(sender);
                cmds.push(self.send_ae_update_to_nodes(recipients, section_key));
                Ok(cmds)
            }
            // The AcceptedOnlineShare for relocation will be received here.
            NodeMsg::JoinResponse(join_response) => {
                match *join_response {
                    JoinResponse::Approved {
                        section_tree_update,
                        ..
                    } => {
                        info!(
                            "Relocation: Aggregating received ApprovalShare from {:?}",
                            sender
                        );
                        info!("Relocation: Successfully aggregated ApprovalShares for joining the network");

                        if let Some(ref mut joining_as_relocated) = self.relocate_state {
                            let new_node = joining_as_relocated.node.clone();
                            let new_name = new_node.name();
                            let previous_name = self.info().name();
                            let new_keypair = new_node.keypair.clone();

                            info!(
                                "Relocation: switching from {:?} to {:?}",
                                previous_name, new_name
                            );

                            let recipients = section_tree_update.section_auth.elders.clone();

                            let section_tree = self.network_knowledge.section_tree().clone();
                            let new_network_knowledge =
                                NetworkKnowledge::new(section_tree, section_tree_update)?;

                            // TODO: confirm whether carry out the switch immediately here
                            //       or still using the cmd pattern.
                            //       As the sending of the JoinRequest as notification
                            //       may require the `node` to be switched to new already.

                            self.relocate(new_keypair.clone(), new_network_knowledge)?;

                            trace!(
                                "Relocation: Sending aggregated JoinRequest to {:?}",
                                recipients
                            );

                            self.send_event(Event::Membership(MembershipEvent::Relocated {
                                previous_name,
                                new_keypair,
                            }))
                            .await;

                            trace!("{}", LogMarker::RelocateEnd);
                        } else {
                            warn!("Relocation:  self.relocate_state is not in Progress");
                        }

                        Ok(vec![])
                    }
                    _ => {
                        debug!("Relocation: Ignoring unexpected join response message: {join_response:?}");
                        Ok(vec![])
                    }
                }
            }
            NodeMsg::HandoverVotes(votes) => self.handle_handover_msg(sender, votes).await,
            NodeMsg::HandoverAE(gen) => Ok(self
                .handle_handover_anti_entropy(sender, gen)
                .into_iter()
                .collect()),
            NodeMsg::JoinRequest(join_request) => {
                trace!("Handling msg {:?}: JoinRequest from {}", msg_id, sender);
                self.handle_join_request(sender, join_request, comm)
                    .await
                    .map(|c| c.into_iter().collect())
            }
            NodeMsg::JoinAsRelocatedRequest(join_request) => {
                trace!("Handling msg: JoinAsRelocatedRequest from {}", sender);
                if self.is_not_elder()
                    && join_request.section_key == self.network_knowledge.section_key()
                {
                    return Ok(vec![]);
                }
                Ok(self
                    .handle_join_as_relocated_request(sender, *join_request, comm)
                    .await
                    .into_iter()
                    .collect())
            }
            NodeMsg::MembershipVotes(votes) => {
                let mut cmds = vec![];
                cmds.extend(self.handle_membership_votes(sender, votes)?);
                Ok(cmds)
            }
            NodeMsg::MembershipAE(gen) => Ok(self
                .handle_membership_anti_entropy(sender, gen)
                .into_iter()
                .collect()),
            NodeMsg::Propose {
                proposal,
                sig_share,
            } => {
                if self.is_not_elder() {
                    trace!("Adult handling a Propose msg from {}: {:?}", sender, msg_id);
                }

                trace!(
                    "Handling proposal msg: {proposal:?} from {}: {:?}",
                    sender,
                    msg_id
                );

                // lets convert our message into a usable proposal for core
                let core_proposal = match proposal {
                    ProposalMsg::VoteNodeOffline(node_state) => {
                        CoreProposal::VoteNodeOffline(node_state.into_state())
                    }
                    ProposalMsg::SectionInfo(sap) => CoreProposal::SectionInfo(sap.into_state()),
                    ProposalMsg::NewElders(sap) => CoreProposal::NewElders(sap.into_authed_state()),
                    ProposalMsg::JoinsAllowed(allowed) => CoreProposal::JoinsAllowed(allowed),
                };

                Node::handle_proposal(
                    msg_id,
                    core_proposal,
                    sig_share,
                    sender,
                    &self.network_knowledge,
                    &mut self.proposal_aggregator,
                )
            }
            NodeMsg::DkgStart(session_id) => {
                trace!(
                    "Handling msg: DkgStart s{} {:?}: {} elders from {}",
                    session_id.sh(),
                    session_id.prefix,
                    session_id.elders.len(),
                    sender
                );
                self.log_dkg_session(&sender.name());
                if let NodeMsgAuthority::Section(authority) = msg_authority {
                    let session_info = DkgSessionInfo {
                        session_id: session_id.clone(),
                        authority,
                    };
                    let _existing = self
                        .dkg_sessions_info
                        .insert(session_id.hash(), session_info);
                }
                self.handle_dkg_start(session_id)
            }
            NodeMsg::DkgEphemeralPubKey {
                session_id,
                section_auth,
                pub_key,
                sig,
            } => {
                trace!(
                    "{} s{} from {}",
                    LogMarker::DkgHandleEphemeralPubKey,
                    session_id.sh(),
                    sender
                );
                self.handle_dkg_ephemeral_pubkey(&session_id, section_auth, pub_key, sig, sender)
            }
            NodeMsg::DkgVotes {
                session_id,
                pub_keys,
                votes,
            } => {
                trace!(
                    "{} s{} from {}: {:?}",
                    LogMarker::DkgVotesHandling,
                    session_id.sh(),
                    sender,
                    votes
                );
                self.log_dkg_session(&sender.name());
                self.handle_dkg_votes(&session_id, pub_keys, votes, sender)
            }
            NodeMsg::DkgAE(session_id) => {
                trace!("Handling msg: DkgAE s{} from {}", session_id.sh(), sender);
                self.handle_dkg_anti_entropy(session_id, sender)
            }
            NodeMsg::NodeCmd(NodeCmd::RecordStorageLevel { node_id, level, .. }) => {
                let changed = self.set_storage_level(&node_id, level);
                if changed && level.value() == MIN_LEVEL_WHEN_FULL {
                    // ..then we accept a new node in place of the full node
                    self.joins_allowed = true;
                }
                Ok(vec![])
            }
            NodeMsg::NodeCmd(NodeCmd::ReceiveMetadata { metadata }) => {
                info!("Processing received MetadataExchange packet: {:?}", msg_id);
                self.set_adult_levels(metadata);
                Ok(vec![])
            }
            NodeMsg::NodeEvent(NodeEvent::CouldNotStoreData {
                node_id,
                data,
                full,
            }) => {
                info!(
                    "Processing CouldNotStoreData event with MsgId: {:?}",
                    msg_id
                );
                if self.is_not_elder() {
                    error!("Received unexpected message while Adult");
                    return Ok(vec![]);
                }

                if full {
                    let changed =
                        self.set_storage_level(&node_id, StorageLevel::from(StorageLevel::MAX)?);
                    if changed {
                        // ..then we accept a new node in place of the full node
                        self.joins_allowed = true;
                    }
                }

                let targets = self.target_data_holders(data.name());

                Ok(vec![self.replicate_data(
                    data,
                    targets,
                    #[cfg(feature = "traceroute")]
                    traceroute,
                )])
            }
            NodeMsg::NodeCmd(NodeCmd::ReplicateData(data_collection)) => {
                info!("ReplicateData MsgId: {:?}", msg_id);

                if self.is_elder() {
                    error!("Received unexpected message while Elder");
                    return Ok(vec![]);
                }

                let mut cmds = vec![];

                let section_pk = PublicKey::Bls(self.network_knowledge.section_key());
                let node_keypair = Keypair::Ed25519(self.keypair.clone());

                for data in data_collection {
                    // We are an adult here, so just store away!
                    // This may return a DatabaseFull error... but we should have reported storage increase
                    // well before this
                    match self
                        .data_storage
                        .store(&data, section_pk, node_keypair.clone())
                        .await
                    {
                        Ok(level_report) => {
                            info!("Storage level report: {:?}", level_report);
                            cmds.extend(self.record_storage_level_if_any(
                                level_report,
                                #[cfg(feature = "traceroute")]
                                traceroute.clone(),
                            )?);

                            #[cfg(feature = "traceroute")]
                            info!("End of message flow. Trace: {:?}", traceroute);
                        }
                        Err(StorageError::NotEnoughSpace) => {
                            // storage full
                            error!("Not enough space to store more data");
                            let node_id = PublicKey::from(self.keypair.public);
                            let msg = NodeMsg::NodeEvent(NodeEvent::CouldNotStoreData {
                                node_id,
                                data,
                                full: true,
                            });

                            cmds.push(self.send_msg_to_our_elders(msg))
                        }
                        Err(error) => {
                            // the rest seem to be non-problematic errors.. (?)
                            error!("Problem storing data, but it was ignored: {error}");
                        }
                    }
                }

                Ok(cmds)
            }
            NodeMsg::NodeCmd(NodeCmd::SendAnyMissingRelevantData(known_data_addresses)) => {
                info!(
                    "{:?} MsgId: {:?}",
                    LogMarker::RequestForAnyMissingData,
                    msg_id
                );
                Ok(self
                    .get_missing_data_for_node(sender, known_data_addresses)
                    .await
                    .into_iter()
                    .collect())
            }
            NodeMsg::NodeQuery(NodeQuery::Data {
                query,
                auth,
                operation_id,
            }) => {
                // A request from EndUser - via elders - for locally stored data
                debug!(
                    "Handle NodeQuery with msg_id {:?}, operation_id {}",
                    msg_id, operation_id
                );
                // There is no point in verifying a sig from a sender A or B here.
                // Send back response to the sending elder
                Ok(vec![
                    self.handle_data_query_at_adult(
                        operation_id,
                        &query,
                        auth,
                        sender,
                        #[cfg(feature = "traceroute")]
                        traceroute,
                    )
                    .await,
                ])
            }
            NodeMsg::NodeQueryResponse {
                response,
                operation_id,
            } => {
                debug!(
                    "{:?}: op_id {}, sender: {sender} origin msg_id: {msg_id:?}",
                    LogMarker::ChunkQueryResponseReceviedFromAdult,
                    operation_id
                );

                match msg_authority {
                    NodeMsgAuthority::Node(auth) => {
                        let sending_nodes_pk = PublicKey::from(auth.into_inner().node_ed_pk);
                        Ok(self
                            .handle_data_query_response_at_elder(
                                operation_id,
                                response,
                                sending_nodes_pk,
                                #[cfg(feature = "traceroute")]
                                traceroute,
                            )
                            .await
                            .into_iter()
                            .collect())
                    }
                    _ => Err(Error::InvalidQueryResponseAuthority),
                }
            }
        }
    }

    fn record_storage_level_if_any(
        &self,
        level: Option<StorageLevel>,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Result<Vec<Cmd>> {
        let mut cmds = vec![];
        if let Some(level) = level {
            info!("Storage has now passed {} % used.", 10 * level.value());
            let node_id = PublicKey::from(self.keypair.public);
            let node_xorname = XorName::from(node_id);

            // we ask the section to record the new level reached
            let msg = NodeMsg::NodeCmd(NodeCmd::RecordStorageLevel {
                section: node_xorname,
                node_id,
                level,
            });

            let dst = Peers::Multiple(self.network_knowledge.elders());

            cmds.push(Cmd::send_traced_msg(
                OutgoingMsg::Node(msg),
                dst,
                #[cfg(feature = "traceroute")]
                traceroute,
            ));
        }

        Ok(cmds)
    }

    // Converts the provided NodeMsgAuthority to be a `Section` message
    // authority on successful accumulation.
    fn aggregate_msg(
        &mut self,
        msg_authority: NodeMsgAuthority,
        payload: Bytes,
    ) -> Result<Option<NodeMsgAuthority>> {
        let bls_share_auth = if let NodeMsgAuthority::BlsShare(bls_share_auth) = msg_authority {
            bls_share_auth
        } else {
            return Ok(Some(msg_authority));
        };

        match SectionAuth::try_authorize(
            &mut self.message_aggregator,
            bls_share_auth.into_inner(),
            &payload,
        ) {
            Ok(section_auth) => {
                info!("Successfully aggregated message");
                Ok(Some(NodeMsgAuthority::Section(section_auth)))
            }
            Err(AggregatorError::NotEnoughShares) => {
                info!("Not enough shares to aggregate received message");
                Ok(None)
            }
            Err(err) => {
                error!("Error accumulating message at dst: {:?}", err);
                Err(Error::InvalidSignatureShare)
            }
        }
    }
}
