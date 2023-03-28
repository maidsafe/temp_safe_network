// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd,
    membership::{self, Generation, Membership},
    messaging::Recipients,
    MyNode, NodeContext, Result,
};

use sn_consensus::mvba::{
    bundle::Bundle,
    bundle::Outgoing::{Direct, Gossip},
    Decision,
};
use sn_interface::{
    messaging::system::{JoinResponse, NodeMsg},
    network_knowledge::{
        node_state::{NodeState, RelocationTrigger},
        MembershipState,
    },
    types::{log_markers::LogMarker, NodeId, Participant},
};

use std::{collections::BTreeSet, vec};

// Message handling
impl MyNode {
    pub(crate) fn propose_membership_change(&mut self, node_state: NodeState) -> Option<Cmd> {
        info!(
            "Proposing membership change: {} - {:?}",
            node_state.name(),
            node_state.state()
        );

        let context = &self.context();
        let prefix = self.network_knowledge.prefix();
        if let Some(membership) = self.membership.as_mut() {
            let outgoings = match membership.propose(node_state, &prefix) {
                Ok(vote) => vote,
                Err(e) => {
                    warn!("Membership - failed to propose change: {e:?}");
                    return None;
                }
            };

            let mut bundles = Vec::new();
            for outgoing in outgoings {
                let bundle = match outgoing {
                    Gossip(bundle) => bundle,
                    Direct(_node_id, bundle) => bundle,
                };

                bundles.push(bundle);
            }

            Some(MyNode::send_to_elders(
                context,
                NodeMsg::MembershipVotes(bundles),
            ))
        } else {
            error!("Membership - Failed to propose membership change, no membership instance");
            None
        }
    }

    /// Get our latest vote if any at this generation, and get cmds to resend to all elders
    /// (which should in turn trigger them to resend their votes)
    #[instrument(skip_all)]
    pub(crate) fn membership_gossip_votes(context: &NodeContext) -> Option<Cmd> {
        if let Some(membership) = &context.membership {
            trace!("{}", LogMarker::GossippingMembershipVotes);
            if let Some(ae_outgoings) = membership.anti_entropy(membership.generation()) {
                let bundle = match ae_outgoings {
                    Gossip(bundle) => bundle,
                    Direct(_node_id, bundle) => bundle,
                };

                let cmd = MyNode::send_to_elders(context, NodeMsg::MembershipVotes(vec![bundle]));
                return Some(cmd);
            }
        }

        None
    }

    pub(crate) fn handle_membership_votes(
        &mut self,
        node_id: NodeId,
        bundles: Vec<Bundle<NodeState>>,
    ) -> Result<Vec<Cmd>> {
        trace!(
            "{:?} {bundles:?} from {node_id}",
            LogMarker::MembershipVotesBeingHandled
        );

        let context = &self.context();
        let prefix = context.network_knowledge.prefix();

        let mut cmds = vec![];

        for signed_vote in bundles {
            if let Some(membership) = self.membership.as_mut() {
                let (outgoings, decision) = match membership
                    .handle_signed_vote(signed_vote, &prefix)
                {
                    Ok(result) => result,
                    Err(membership::Error::RequestAntiEntropy) => {
                        debug!("Membership - We are behind the voter, requesting AE");
                        // We hit an error while processing this vote, perhaps we are missing information.
                        // We'll send a membership AE request to see if they can help us catch up.
                        debug!("{:?}", LogMarker::MembershipSendingAeUpdateRequest);
                        let msg = NodeMsg::MembershipAE(membership.generation());
                        cmds.push(Cmd::send_msg(
                            msg,
                            Recipients::Single(Participant::from_node(node_id)),
                        ));
                        // return the vec w/ the AE cmd there so as not to loop and generate AE for
                        // any subsequent commands
                        return Ok(cmds);
                    }
                    Err(e) => {
                        error!("Membership - error while processing vote {e:?}, dropping this and all votes in this batch thereafter");
                        break;
                    }
                };

                let mut bundles = vec![];
                for outgoing in outgoings {
                    match outgoing {
                        Gossip(bundle) => {
                            bundles.push(bundle);
                        }
                        Direct(_, bundle) => {
                            bundles.push(bundle);
                        }
                    }
                }

                cmds.push(MyNode::send_to_elders(
                    context,
                    NodeMsg::MembershipVotes(bundles),
                ));

                if let Some(decision) = decision {
                    info!(
                        "{node_id} decided for membership proposal {:?}",
                        decision.proposal
                    );

                    cmds.push(Cmd::HandleMembershipDecision(decision));
                }
            } else {
                error!(
                    "Attempted to handle membership vote when we don't yet have a membership instance"
                );
            };
        }

        Ok(cmds)
    }

    pub(crate) fn handle_membership_anti_entropy_request(
        membership_context: &Option<Membership>,
        node_id: NodeId,
        gen: Generation,
    ) -> Option<Cmd> {
        debug!(
            "{:?} membership anti-entropy request for gen {gen:?} from {node_id}",
            LogMarker::MembershipAeRequestReceived,
        );

        if let Some(membership) = membership_context {
            match membership.anti_entropy(gen) {
                Some(catchup_votes) => {
                    trace!("Sending catchup votes to {node_id:?}");
                    let bundle = match catchup_votes {
                        Gossip(bundle) => bundle,
                        Direct(_node_id, bundle) => bundle,
                    };
                    Some(Cmd::send_msg(
                        NodeMsg::MembershipVotes(vec![bundle]),
                        Recipients::Single(Participant::from_node(node_id)),
                    ))
                }
                None => None,
            }
        } else {
            error!(
                "Attempted to handle membership anti-entropy when we don't yet have a membership instance"
            );
            None
        }
    }

    pub(crate) async fn handle_membership_decision(
        &mut self,
        gen: u64,
        decision: Decision<NodeState>,
    ) -> Result<Vec<Cmd>> {
        info!("{}", LogMarker::AgreementOfMembership);
        let mut cmds = vec![];
        let node_state = decision.proposal.clone();

        match self
            .network_knowledge
            .try_update_member(gen, decision.clone())
        {
            Err(_err) => {
                error!("Ignored decision {decision:?} as we are lagging");
                cmds.push(Self::generate_probe_msg(&self.context())?);
                return Ok(cmds);
            }
            Ok(updated) => {
                if !updated {
                    debug!("decision {decision:?} didn't update the members");
                }
            }
        }

        let mut excluded_from_relocation = BTreeSet::new();
        if node_state.state() == MembershipState::Joined {
            cmds.extend(self.handle_node_joined(decision.clone()).await);
            cmds.push(self.send_node_approvals(decision.clone()));

            let _res = excluded_from_relocation.insert(node_state.name());
        } else {
            cmds.extend(self.handle_node_left(decision.clone()).into_iter());
        }

        // Do not disable node joins in first section.
        if !self.is_startup_joining_allowed() {
            // ..otherwise, switch off joins_allowed on a node joining.
            // TODO: fix racing issues here? https://github.com/maidsafe/safe_network/issues/890
            self.joins_allowed = false;
        }

        //let node_state = decision.proposal;
        let relocation_trigger = RelocationTrigger::new(decision);

        cmds.extend(self.try_relocate_nodes(relocation_trigger, excluded_from_relocation)?);

        cmds.extend(self.trigger_dkg()?);

        cmds.extend(self.send_ae_update_to_our_section()?);

        self.fault_detection_retain_only(
            self.network_knowledge
                .adults()
                .iter()
                .map(|node_id| node_id.name())
                .collect(),
            self.network_knowledge
                .elders()
                .iter()
                .map(|node_id| node_id.name())
                .collect(),
        )
        .await;

        if node_state.state() != MembershipState::Joined {
            self.joins_allowed = true;
        }

        let net_increase = node_state.state() == MembershipState::Joined;

        // We do this check on every net node join.
        // It is a cheap check and any actual cleanup won't happen back to back,
        // due to requirement of `has_reached_min_capacity() == true` before doing it.
        if net_increase {
            self.data_storage
                .try_retain_data_of(self.network_knowledge.prefix());
            // if we are _still_ at min capacity, then it's time to allow joins until split
            if self.data_storage.has_reached_min_capacity() {
                self.joins_allowed = true;
                self.joins_allowed_until_split = true;
            }
        }

        // Once we've grown the section, we do not need to allow more nodes in.
        // (Unless we've triggered the storage critical fail safe to grow until split.)
        if net_increase && !self.is_startup_joining_allowed() && !self.joins_allowed_until_split {
            self.joins_allowed = false;
        }

        self.log_section_stats();
        self.log_network_stats();

        MyNode::update_comm_target_list(
            &self.comm,
            &self.network_knowledge.archived_members(),
            self.network_knowledge().members(),
        );

        // lets check that we have the correct data now we're changing membership
        cmds.push(MyNode::ask_for_any_new_data_from_whole_section(&self.context()).await);

        Ok(cmds)
    }

    pub(crate) fn is_startup_joining_allowed(&self) -> bool {
        const TEMP_SECTION_LIMIT: usize = 20;

        let is_first_section = self.network_knowledge.prefix().is_empty();
        let members_count = self.network_knowledge.members().len();

        if cfg!(feature = "limit-network-size") {
            is_first_section && members_count <= TEMP_SECTION_LIMIT
        } else {
            is_first_section
        }
    }

    async fn handle_node_joined(&mut self, decision: Decision<NodeState>) -> Vec<Cmd> {
        let node_state = decision.proposal;
        self.add_new_adult_to_trackers(node_state.name()).await;

        info!("handle Online: {:?}", node_state);

        vec![]
    }

    // Send `NodeApproval` to a joining node which makes it a section member
    pub(crate) fn send_node_approvals(&self, decision: Decision<NodeState>) -> Cmd {
        let mut nodes = BTreeSet::new();
        if decision.proposal.state() == MembershipState::Joined {
            let _res = nodes.insert(*decision.proposal.node_id());
        }
        let prefix = self.network_knowledge.prefix();
        info!("Section {prefix:?} has approved new nodes {nodes:?}.");

        let msg = NodeMsg::JoinResponse(JoinResponse::Approved(decision));

        trace!("{}", LogMarker::SendNodeApproval);
        Cmd::send_msg(msg, Recipients::Multiple(nodes))
    }

    pub(crate) fn handle_node_left(&mut self, decision: Decision<NodeState>) -> Option<Cmd> {
        let node_state = decision.proposal.clone();
        info!(
            "{}: {}",
            LogMarker::AcceptedNodeAsOffline,
            node_state.node_id()
        );

        // If this is an Offline agreement where the new node state is Relocated,
        // we then need to send the Relocate msg to the node attaching the signed NodeState
        // containing the relocation details.
        if node_state.is_relocated() {
            let node_id = *node_state.node_id();
            info!("Notify relocation to node {node_id:?}");

            let msg = NodeMsg::CompleteRelocation(decision);
            Some(Cmd::send_msg(
                msg,
                Recipients::Single(Participant::from_node(node_id)),
            ))
        } else {
            None
        }
    }
}
