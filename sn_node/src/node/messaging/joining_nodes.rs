// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    core::NodeContext, flow_ctrl::cmds::Cmd, messaging::Recipients, Error, MyNode, Result,
};

use qp2p::SendStream;
use sn_interface::{
    messaging::{
        system::{JoinResponse, NodeMsg},
        MsgId,
    },
    network_knowledge::{NodeState, RelocationProof, MIN_ADULT_AGE},
    types::{log_markers::LogMarker, NodeId, Participant},
};

// Message handling
impl MyNode {
    pub(crate) fn handle_join(
        node: &mut MyNode,
        context: &NodeContext,
        node_id: NodeId,
        correlation_id: MsgId,
        relocation: Option<RelocationProof>,
        send_stream: Option<SendStream>,
    ) -> Result<Vec<Cmd>> {
        trace!("Handling join from {node_id:?}");

        // Ignore a join request if we are not elder.
        if !context.is_elder {
            warn!("Join request received to our section, but I am not an elder...");
            // Note: We don't bounce this message because the current bounce-resend
            // mechanism wouldn't preserve the original SocketAddr which is needed for
            // properly handling this message.
            // This is OK because in the worst case the join request just timeouts and the
            // joining node sends it again.
            return Ok(vec![]);
        }
        let our_prefix = context.network_knowledge.prefix();
        if !our_prefix.matches(&node_id.name()) {
            debug!("Unreachable path; {node_id} name doesn't match our prefix. Should be covered by AE. Dropping the msg.");
            return Ok(vec![]);
        }

        let previous_name = if let Some(proof) = relocation {
            // Relocation ->
            // Verify that we know the src key..
            let src_key = proof.signed_by();
            if !context
                .network_knowledge
                .verify_section_key_is_known(src_key)
            {
                warn!("Node {} is trying to join with signature by unknown source section key {src_key:?}. Message is dropped.", node_id.name());
                return Ok(vec![]);
            }

            // Verify the signatures..
            proof.verify()?;

            // Verify the age..
            MyNode::verify_relocated_age(&node_id, &proof)?;

            // NB: Relocated nodes that try to join, are accepted even if joins are disallowed.
            Some(proof.previous_name())
        } else {
            // New node ->
            if !MyNode::is_infant_node(&node_id) {
                debug!("Unreachable path; {node_id} age is invalid: {}. This should be a hard coded value in join logic. Dropping the msg.", node_id.age());
                return Ok(vec![]);
            }

            if !context.joins_allowed {
                trace!("Rejecting join request from {node_id} - joins currently not allowed.");
                let msg = NodeMsg::JoinResponse(JoinResponse::JoinsDisallowed);
                trace!("{}", LogMarker::SendJoinRejected);
                trace!("Sending {msg:?} to {node_id}");

                // Send it over response stream if we have one
                if let Some(stream) = send_stream {
                    return Ok(vec![Cmd::send_node_response(
                        msg,
                        correlation_id,
                        node_id,
                        stream,
                    )]);
                }

                return Ok(vec![Cmd::send_msg(
                    msg,
                    Recipients::Single(Participant::from_node(node_id)),
                )]);
            }

            None
        };

        let mut cmds = vec![];

        // Let the joiner know we are considering.
        if let Some(send_stream) = send_stream {
            cmds.push(Cmd::send_node_response(
                NodeMsg::JoinResponse(JoinResponse::UnderConsideration),
                correlation_id,
                node_id,
                send_stream,
            ));
        }

        // We propose membership
        let node_state = NodeState::joined(node_id, previous_name);

        if let Some(cmd) = node.propose_membership_change(node_state) {
            cmds.push(cmd);
        }
        Ok(cmds)
    }

    pub(crate) fn is_infant_node(node_id: &NodeId) -> bool {
        // Age should be MIN_ADULT_AGE for joining infant.
        node_id.age() == MIN_ADULT_AGE
    }

    pub(crate) fn verify_relocated_age(node_id: &NodeId, proof: &RelocationProof) -> Result<()> {
        let name = node_id.name();
        let new_age = node_id.age();
        let previous_age = proof.previous_age();

        // We require node new age to be one more than the previous age.
        if new_age != previous_age.saturating_add(1) {
            info!(
                "Invalid relocation from {name} - node new age ({new_age}) should be one more than node's previous age ({previous_age}), or same if {}.", u8::MAX
            );
            return Err(Error::InvalidRelocationDetails);
        }

        Ok(())
    }
}
