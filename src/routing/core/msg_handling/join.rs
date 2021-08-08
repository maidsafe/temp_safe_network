// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::messaging::{
    node::{
        JoinAsRelocatedRequest, JoinAsRelocatedResponse, JoinRejectionReason, JoinRequest,
        JoinResponse, NodeMsg, Peer,
    },
    WireMsg,
};
use crate::routing::{
    error::Result,
    peer::PeerUtils,
    relocation::RelocatePayloadUtils,
    routing_api::command::Command,
    section::{SectionLogic, SectionPeersLogic},
    FIRST_SECTION_MAX_AGE, FIRST_SECTION_MIN_AGE, MIN_ADULT_AGE,
};
use bls::PublicKey as BlsPublicKey;

// Message handling
impl Core {
    pub(crate) async fn handle_join_request(
        &self,
        peer: Peer,
        join_request: JoinRequest,
    ) -> Result<Vec<Command>> {
        debug!("Received {:?} from {}", join_request, peer);

        let section_key_matches = join_request.section_key == self.section.last_key().await;

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

        let prefix = self.section.prefix().await;
        if !section_key_matches || !prefix.matches(peer.name()) {
            debug!(
                "JoinRequest from {} - name doesn't match our prefix {:?}.",
                peer, prefix,
            );

            let redirect_sap = self.matching_section(peer.name()).await?;

            let node_msg = NodeMsg::JoinResponse(Box::new(JoinResponse::Retry(redirect_sap)));
            trace!("Sending {:?} to {}", node_msg, peer);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.last_key().await,
                )
                .await?,
            ]);
        }

        if self.section.members().is_joined(peer.name()).await {
            debug!(
                "Ignoring JoinRequest from {} - already member of our section.",
                peer
            );
            return Ok(vec![]);
        }

        if !self.joins_allowed.clone().await {
            debug!(
                "Rejecting JoinRequest from {} - joins currently not allowed.",
                peer,
            );
            let node_msg = NodeMsg::JoinResponse(Box::new(JoinResponse::Rejected(
                JoinRejectionReason::JoinsDisallowed,
            )));

            trace!("Sending {:?} to {}", node_msg, peer);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.last_key().await,
                )
                .await?,
            ]);
        }

        // Start as Adult as long as passed resource signed.
        let mut age = MIN_ADULT_AGE;

        // During the first section, node shall use ranged age to avoid too many nodes got
        // relocated at the same time. After the first section got split, later on nodes shall
        // only start with age of MIN_ADULT_AGE
        if self.section.prefix().await.is_empty() {
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
            let node_msg = NodeMsg::JoinResponse(Box::new(JoinResponse::Retry(
                self.section.authority_provider().await.clone(),
            )));
            trace!("New node after section split must join with age of MIN_ADULT_AGE. Sending {:?} to {}", node_msg, peer);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.last_key().await,
                )
                .await?,
            ]);
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
            // Do reachability check only for the initial join request
            let cmd = if self.comm.is_reachable(peer.addr()).await.is_err() {
                let node_msg = NodeMsg::JoinResponse(Box::new(JoinResponse::Rejected(
                    JoinRejectionReason::NodeNotReachable(*peer.addr()),
                )));

                trace!("Sending {:?} to {}", node_msg, peer);
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.last_key().await,
                )
                .await?
            } else {
                // It's reachable, let's then send the proof challenge
                self.send_resource_proof_challenge(&peer).await?
            };

            return Ok(vec![cmd]);
        }

        Ok(vec![Command::ProposeOnline {
            peer,
            previous_name: None,
            dst_key: None,
        }])
    }

    pub(crate) async fn handle_join_as_relocated_request(
        &self,
        peer: Peer,
        join_request: JoinAsRelocatedRequest,
        known_keys: &[BlsPublicKey],
    ) -> Result<Vec<Command>> {
        debug!("Received {:?} from {}", join_request, peer);
        let relocate_payload = if let Some(relocate_payload) = join_request.relocate_payload {
            relocate_payload
        } else {
            // Do reachability check
            let node_msg = if self.comm.is_reachable(peer.addr()).await.is_err() {
                NodeMsg::JoinAsRelocatedResponse(Box::new(
                    JoinAsRelocatedResponse::NodeNotReachable(*peer.addr()),
                ))
            } else {
                NodeMsg::JoinAsRelocatedResponse(Box::new(JoinAsRelocatedResponse::Retry(
                    self.section.authority_provider().await.clone(),
                )))
            };

            trace!("Sending {:?} to {}", node_msg, peer);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.last_key().await,
                )
                .await?,
            ]);
        };

        let prefix = self.section.prefix().await;
        if !prefix.matches(peer.name()) || join_request.section_key != self.section.last_key().await
        {
            debug!(
                "JoinAsRelocatedRequest from {} - name doesn't match our prefix {:?}.",
                peer, prefix,
            );

            let node_msg = NodeMsg::JoinAsRelocatedResponse(Box::new(
                JoinAsRelocatedResponse::Retry(self.section.authority_provider().await.clone()),
            ));
            trace!("Sending {:?} to {}", node_msg, peer);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.last_key().await,
                )
                .await?,
            ]);
        }

        if self.section.members().is_joined(peer.name()).await {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - already member of our section.",
                peer
            );
            return Ok(vec![]);
        }

        if !relocate_payload.verify_identity(peer.name()) {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - invalid signature.",
                peer
            );
            return Ok(vec![]);
        }

        let details = relocate_payload.relocate_details()?;

        if !prefix.matches(&details.dst) {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - destination {} doesn't match \
                         our prefix {:?}.",
                peer, details.dst, prefix,
            );
            return Ok(vec![]);
        }

        // Requires the node name matches the age.
        let age = details.age;
        if age != peer.age() {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - relocation age ({}) doesn't match peer's age ({}).",
                peer, age, peer.age(),
            );
            return Ok(vec![]);
        }

        // Check for signatures and trust of the relocate_payload msg
        let serialised_relocate_details =
            WireMsg::serialize_msg_payload(&NodeMsg::Relocate(details.clone()))?;

        let payload_section_signed = &relocate_payload.section_signed;
        let is_signautre_valid = payload_section_signed.section_pk.verify(
            &payload_section_signed.sig.signature,
            serialised_relocate_details,
        );
        let is_key_unknown = !known_keys
            .iter()
            .any(|key| *key == payload_section_signed.section_pk);

        if !is_signautre_valid || is_key_unknown {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - invalid signature or untrusted src.",
                peer
            );
            return Ok(vec![]);
        }

        let previous_name = Some(details.pub_id);
        let dst_key = Some(details.dst_key);

        if self.section.members().is_relocated(&details.pub_id).await {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - original node {:?} already relocated to us.",
                peer, previous_name
            );
            return Ok(vec![]);
        }

        Ok(vec![Command::ProposeOnline {
            peer,
            previous_name,
            dst_key,
        }])
    }
}
