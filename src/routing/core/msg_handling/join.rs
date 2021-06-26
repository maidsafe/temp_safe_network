// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::messaging::node::{
    JoinAsRelocatedRequest, JoinAsRelocatedResponse, JoinRejectionReason, JoinRequest,
    JoinResponse, Peer, Variant,
};
use crate::routing::{
    core::WireMsgUtils,
    error::Result,
    peer::PeerUtils,
    relocation::{RelocatePayloadUtils, SignedRelocateDetailsUtils},
    routing_api::command::Command,
    section::{
        SectionPeersUtils, SectionUtils, FIRST_SECTION_MAX_AGE, FIRST_SECTION_MIN_AGE,
        MIN_ADULT_AGE,
    },
};
use bls::PublicKey as BlsPublicKey;

// Message handling
impl Core {
    pub(crate) fn handle_join_request(
        &mut self,
        peer: Peer,
        join_request: JoinRequest,
    ) -> Result<Vec<Command>> {
        debug!("Received {:?} from {}", join_request, peer);

        let section_key_matches = join_request.section_key == *self.section.chain().last_key();

        // Ignore `JoinRequest` if we are not elder unless the join request
        // is outdated in which case we reply with `BootstrapResponse::Join`
        // with the up-to-date info (see `handle_join_request`).
        if self.is_not_elder() && section_key_matches {
            // Note: We don't bounce this message because the current bounce-resend
            // mechanism wouldn't preserve the original SocketAddr which is needed for
            // properly handling this message.
            // This is OK because in the worst case the join request just timeouts and the
            // joining node sends it again.
            return Ok(vec![]);
        }

        if !section_key_matches || !self.section.prefix().matches(peer.name()) {
            debug!(
                "JoinRequest from {} - name doesn't match our prefix {:?}.",
                peer,
                self.section.prefix()
            );

            let redirect_sap = self.matching_section(peer.name())?;

            let variant = Variant::JoinResponse(Box::new(JoinResponse::Retry(redirect_sap)));
            trace!("Sending {:?} to {}", variant, peer);
            return Ok(vec![self.send_direct_message(
                (*peer.name(), *peer.addr()),
                variant,
                *self.section.chain().last_key(),
            )?]);
        }

        if self.section.members().is_joined(peer.name()) {
            debug!(
                "Ignoring JoinRequest from {} - already member of our section.",
                peer
            );
            return Ok(vec![]);
        }

        if !self.joins_allowed {
            debug!(
                "Rejecting JoinRequest from {} - joins currently not allowed.",
                peer,
            );
            let variant = Variant::JoinResponse(Box::new(JoinResponse::Rejected(
                JoinRejectionReason::JoinsDisallowed,
            )));

            trace!("Sending {:?} to {}", variant, peer);
            return Ok(vec![self.send_direct_message(
                (*peer.name(), *peer.addr()),
                variant,
                *self.section.chain().last_key(),
            )?]);
        }

        // Start as Adult as long as passed resource signed.
        let mut age = MIN_ADULT_AGE;

        // During the first section, node shall use ranged age to avoid too many nodes got
        // relocated at the same time. After the first section got split, later on nodes shall
        // only start with age of MIN_ADULT_AGE
        if self.section.prefix().is_empty() {
            if peer.age() < FIRST_SECTION_MIN_AGE || peer.age() > FIRST_SECTION_MAX_AGE {
                debug!(
                    "Ignoring JoinRequest from {} - first-section node having wrong age {:?}",
                    peer,
                    peer.age(),
                );
                return Ok(vec![]);
            } else {
                age = peer.age();
            }
        } else if peer.age() != MIN_ADULT_AGE {
            // After section split, new node has to join with age of MIN_ADULT_AGE.
            let variant = Variant::JoinResponse(Box::new(JoinResponse::Retry(
                self.section.authority_provider().clone(),
            )));
            trace!("New node after section split must join with age of MIN_ADULT_AGE. Sending {:?} to {}", variant, peer);
            return Ok(vec![self.send_direct_message(
                (*peer.name(), *peer.addr()),
                variant,
                *self.section.chain().last_key(),
            )?]);
        }

        // Requires the node name matches the age.
        if age != peer.age() {
            debug!(
                "Ignoring JoinRequest from {} - required age {:?} not presented.",
                peer, age,
            );
            return Ok(vec![]);
        }

        // Require resource signed if joining as a new node.
        if let Some(response) = join_request.resource_proof_response {
            if !self.validate_resource_proof_response(peer.name(), response) {
                debug!(
                    "Ignoring JoinRequest from {} - invalid resource signed response",
                    peer
                );
                return Ok(vec![]);
            }
        } else {
            return Ok(vec![self.send_resource_proof_challenge(&peer)?]);
        }

        Ok(vec![Command::ProposeOnline {
            peer,
            previous_name: None,
            dst_key: None,
        }])
    }

    pub(crate) fn handle_join_as_relocated_request(
        &mut self,
        peer: Peer,
        join_request: JoinAsRelocatedRequest,
        known_keys: &[BlsPublicKey],
    ) -> Result<Vec<Command>> {
        debug!("Received {:?} from {}", join_request, peer);
        let payload = if let Some(payload) = join_request.relocate_payload {
            payload
        } else {
            let variant = Variant::JoinAsRelocatedResponse(Box::new(
                JoinAsRelocatedResponse::Retry(self.section.authority_provider().clone()),
            ));

            trace!("Sending {:?} to {}", variant, peer);
            return Ok(vec![self.send_direct_message(
                (*peer.name(), *peer.addr()),
                variant,
                *self.section.chain().last_key(),
            )?]);
        };

        if !self.section.prefix().matches(peer.name())
            || join_request.section_key != *self.section.chain().last_key()
        {
            debug!(
                "JoinAsRelocatedRequest from {} - name doesn't match our prefix {:?}.",
                peer,
                self.section.prefix()
            );

            let variant = Variant::JoinAsRelocatedResponse(Box::new(
                JoinAsRelocatedResponse::Retry(self.section.authority_provider().clone()),
            ));
            trace!("Sending {:?} to {}", variant, peer);
            return Ok(vec![self.send_direct_message(
                (*peer.name(), *peer.addr()),
                variant,
                *self.section.chain().last_key(),
            )?]);
        }

        if self.section.members().is_joined(peer.name()) {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - already member of our section.",
                peer
            );
            return Ok(vec![]);
        }

        if !payload.verify_identity(peer.name()) {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - invalid signature.",
                peer
            );
            return Ok(vec![]);
        }

        let details = payload.relocate_details()?;

        if !self.section.prefix().matches(&details.dst) {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - destination {} doesn't match \
                         our prefix {:?}.",
                peer,
                details.dst,
                self.section.prefix()
            );
            return Ok(vec![]);
        }

        // Check for signatures and trust of the payload msg
        let payload_msg = payload.details.signed_msg();
        if payload_msg.check_signature().is_err()
            || !payload_msg.verify_src_section_chain(&known_keys)
        {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - invalid signature or untrusted src.",
                peer
            );
            return Ok(vec![]);
        }

        // Requires the node name matches the age.
        let age = details.age;
        if age != peer.age() {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - required age {:?} not presented.",
                peer, age,
            );
            return Ok(vec![]);
        }

        let previous_name = Some(details.pub_id);
        let dst_key = Some(details.dst_key);

        Ok(vec![Command::ProposeOnline {
            peer,
            previous_name,
            dst_key,
        }])
    }
}
